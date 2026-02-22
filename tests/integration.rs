/// Strip `export` keywords and `import` lines from JS source.
/// Simplified version of the internal `remove_imports` for testing.
fn strip_module_syntax(source: &str) -> String {
    let mut result = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("import ") || trimmed.starts_with("import{") {
            continue;
        }
        if trimmed.starts_with("export ") {
            if trimmed.starts_with("export default ") {
                result.push(trimmed.trim_start_matches("export default ").to_string());
            } else if trimmed.starts_with("export {") || trimmed.starts_with("export type ") {
                continue;
            } else {
                result.push(trimmed.replacen("export ", "", 1));
            }
            continue;
        }
        result.push(line.to_string());
    }
    result.join("\n")
}

/// Test the builder + executor pipeline: generate runtime JS,
/// create a workflow script, execute via QuickJS,
/// convert JSON output to YAML, and validate the result.
#[test]
fn test_executor_to_yaml_pipeline() {
    use gaji::executor;

    // Build runtime + workflow as a single script
    let runtime_js = format!(
        r#"function getAction(ref) {{
    return function(config) {{
        if (config === undefined) config = {{}};
        var step = {{ uses: ref }};
        if (config.name !== undefined) step.name = config.name;
        if (config.with !== undefined) step.with = config.with;
        return step;
    }};
}}
{}"#,
        gaji::generator::templates::JOB_WORKFLOW_RUNTIME_TEMPLATE
    );

    let workflow_js = r#"
var checkout = getAction("actions/checkout@v5");

new Workflow({
    name: "Integration Test",
    on: { push: { branches: ["main"] } },
}).jobs(function(j) { return j
    .add("build",
        new Job("ubuntu-latest")
            .steps(function(s) { return s
                .add(checkout({ name: "Checkout" }))
                .add({ name: "Test", run: "npm test" })
            })
    )
}).build("integration-test");
"#;

    // Strip export/import for QuickJS script mode
    let runtime_stripped = strip_module_syntax(&runtime_js);
    let bundled = format!("{}\n\n{}", runtime_stripped, workflow_js);

    let outputs = executor::execute_js(&bundled).unwrap();
    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0].id, "integration-test");
    assert_eq!(outputs[0].output_type, "workflow");

    // Convert to YAML
    let json_value: serde_json::Value = serde_json::from_str(&outputs[0].json).unwrap();
    let yaml_str = serde_yaml::to_string(&json_value).unwrap();

    // Validate the YAML structure
    assert!(yaml_str.contains("name: Integration Test"));
    assert!(yaml_str.contains("runs-on: ubuntu-latest"));
    assert!(yaml_str.contains("uses: actions/checkout@v5"));
    assert!(yaml_str.contains("run: npm test"));

    // Parse back as YAML and verify required fields
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(&yaml_str).unwrap();
    let mapping = yaml_value.as_mapping().unwrap();
    assert!(mapping.contains_key(serde_yaml::Value::String("on".to_string())));
    assert!(mapping.contains_key(serde_yaml::Value::String("jobs".to_string())));
}

/// Test that multiple workflow.build() calls produce multiple outputs.
#[test]
fn test_multiple_workflow_builds() {
    use gaji::executor;

    let runtime_js = format!(
        "function getAction(ref) {{ return function(config) {{ return {{ uses: ref }}; }}; }}\n{}",
        gaji::generator::templates::JOB_WORKFLOW_RUNTIME_TEMPLATE
    );

    let runtime_stripped = strip_module_syntax(&runtime_js);

    let workflow_js = r#"
new Workflow({ name: "WF1", on: { push: {} } })
    .jobs(function(j) { return j
        .add("job1", new Job("ubuntu-latest").steps(function(s) { return s.add({ name: "Step1", run: "echo 1" }) }))
    })
    .build("workflow-1");

new Workflow({ name: "WF2", on: { pull_request: {} } })
    .jobs(function(j) { return j
        .add("job2", new Job("ubuntu-latest").steps(function(s) { return s.add({ name: "Step2", run: "echo 2" }) }))
    })
    .build("workflow-2");
"#;

    let bundled = format!("{}\n\n{}", runtime_stripped, workflow_js);

    let outputs = executor::execute_js(&bundled).unwrap();
    assert_eq!(outputs.len(), 2);
    assert_eq!(outputs[0].id, "workflow-1");
    assert_eq!(outputs[1].id, "workflow-2");

    // Verify each output is valid JSON with correct names
    let json1: serde_json::Value = serde_json::from_str(&outputs[0].json).unwrap();
    let json2: serde_json::Value = serde_json::from_str(&outputs[1].json).unwrap();
    assert_eq!(json1["name"], "WF1");
    assert_eq!(json2["name"], "WF2");

    // Both should produce valid YAML
    for output in &outputs {
        let json_val: serde_json::Value = serde_json::from_str(&output.json).unwrap();
        let yaml = serde_yaml::to_string(&json_val).unwrap();
        let parsed: serde_yaml::Value = serde_yaml::from_str(&yaml).unwrap();
        assert!(parsed.as_mapping().is_some());
    }
}

/// Test Job inheritance: class inheritance to create reusable job templates.
#[test]
fn test_composite_job_inheritance() {
    use gaji::executor;

    let runtime_js = format!(
        "function getAction(ref) {{ return function(config) {{ if (config === undefined) config = {{}}; var step = {{ uses: ref }}; if (config.name !== undefined) step.name = config.name; return step; }}; }}\n{}",
        gaji::generator::templates::JOB_WORKFLOW_RUNTIME_TEMPLATE
    );

    let runtime_stripped = strip_module_syntax(&runtime_js);

    // Simulate TypeScript compiled output: class DeployJob extends Job
    let workflow_js = r#"
var checkout = getAction("actions/checkout@v5");

class DeployJob extends Job {
    constructor(environment, config) {
        if (config === undefined) config = {};
        config.env = { ENVIRONMENT: environment };
        super("ubuntu-latest", config);
        this.steps(function(s) { return s
            .add(checkout({ name: "Checkout" }))
            .add({ name: "Deploy", run: "npm run deploy:" + environment })
        });
    }
}

new Workflow({
    name: "Deploy",
    on: { push: { tags: ["v*"] } },
}).jobs(function(j) { return j
    .add("deploy-staging", new DeployJob("staging"))
    .add("deploy-production", new DeployJob("production", { needs: ["deploy-staging"] }))
}).build("deploy");
"#;

    let bundled = format!("{}\n\n{}", runtime_stripped, workflow_js);

    let outputs = executor::execute_js(&bundled).unwrap();
    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0].id, "deploy");
    assert_eq!(outputs[0].output_type, "workflow");

    let json_value: serde_json::Value = serde_json::from_str(&outputs[0].json).unwrap();

    // Verify staging job
    let staging = &json_value["jobs"]["deploy-staging"];
    assert_eq!(staging["runs-on"], "ubuntu-latest");
    assert_eq!(staging["env"]["ENVIRONMENT"], "staging");
    assert_eq!(staging["steps"][0]["uses"], "actions/checkout@v5");
    assert_eq!(staging["steps"][1]["run"], "npm run deploy:staging");

    // Verify production job with needs
    let production = &json_value["jobs"]["deploy-production"];
    assert_eq!(production["runs-on"], "ubuntu-latest");
    assert_eq!(production["env"]["ENVIRONMENT"], "production");
    assert_eq!(production["needs"][0], "deploy-staging");

    // Convert to YAML and validate
    let yaml_str = serde_yaml::to_string(&json_value).unwrap();
    assert!(yaml_str.contains("name: Deploy"));
    assert!(yaml_str.contains("deploy-staging"));
    assert!(yaml_str.contains("deploy-production"));
}

/// Test Action (composite) migration roundtrip: TypeScript -> QuickJS -> JSON -> YAML.
#[test]
fn test_composite_action_migration_roundtrip() {
    use gaji::executor;

    let runtime_js = format!(
        r#"function getAction(ref) {{
    return function(config) {{
        if (config === undefined) config = {{}};
        var step = {{ uses: ref }};
        if (config.name !== undefined) step.name = config.name;
        if (config.with !== undefined) step.with = config.with;
        return step;
    }};
}}
{}"#,
        gaji::generator::templates::JOB_WORKFLOW_RUNTIME_TEMPLATE
    );

    let runtime_stripped = strip_module_syntax(&runtime_js);

    // Simulate migrated Action TypeScript (what generate_composite_action_ts would produce)
    let action_js = r#"
var checkout = getAction("actions/checkout@v5");

new Action({
    name: "Setup Environment",
    description: "Sets up the build environment",
    inputs: {
        "node-version": {
            description: "Node.js version to use",
            required: false,
            default: "20",
        },
    },
})
    .steps(function(s) { return s
        .add(checkout({ name: "Checkout" }))
        .add({ name: "Install deps", run: "npm ci", shell: "bash" })
        .add({ name: "Lint", run: "npm run lint", shell: "bash" })
    })
    .build("setup-env");
"#;

    let bundled = format!("{}\n\n{}", runtime_stripped, action_js);
    let outputs = executor::execute_js(&bundled).unwrap();

    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0].id, "setup-env");
    assert_eq!(outputs[0].output_type, "action");

    let json: serde_json::Value = serde_json::from_str(&outputs[0].json).unwrap();
    assert_eq!(json["name"], "Setup Environment");
    assert_eq!(json["description"], "Sets up the build environment");
    assert_eq!(json["runs"]["using"], "composite");
    assert_eq!(json["runs"]["steps"][0]["uses"], "actions/checkout@v5");
    assert_eq!(json["runs"]["steps"][1]["shell"], "bash");
    assert_eq!(json["runs"]["steps"][1]["run"], "npm ci");

    // Convert to YAML and verify it's valid action.yml
    let yaml_str = serde_yaml::to_string(&json).unwrap();
    assert!(yaml_str.contains("using: composite"));
    assert!(yaml_str.contains("shell: bash"));
}

/// Test NodeAction migration roundtrip: TypeScript -> QuickJS -> JSON -> YAML.
#[test]
fn test_javascript_action_migration_roundtrip() {
    use gaji::executor;

    let runtime_js = format!(
        "function getAction(ref) {{ return function(config) {{ return {{ uses: ref }}; }}; }}\n{}",
        gaji::generator::templates::JOB_WORKFLOW_RUNTIME_TEMPLATE
    );

    let runtime_stripped = strip_module_syntax(&runtime_js);

    // Simulate migrated NodeAction TypeScript
    let action_js = r#"
var action = new NodeAction(
    {
        name: "My Node Action",
        description: "A test Node.js action",
        inputs: {
            token: {
                description: "GitHub token",
                required: true,
            },
        },
    },
    {
        using: "node20",
        main: "dist/index.js",
        post: "dist/cleanup.js",
    },
);

action.build("my-node-action");
"#;

    let bundled = format!("{}\n\n{}", runtime_stripped, action_js);
    let outputs = executor::execute_js(&bundled).unwrap();

    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0].id, "my-node-action");
    assert_eq!(outputs[0].output_type, "action");

    let json: serde_json::Value = serde_json::from_str(&outputs[0].json).unwrap();
    assert_eq!(json["name"], "My Node Action");
    assert_eq!(json["runs"]["using"], "node20");
    assert_eq!(json["runs"]["main"], "dist/index.js");
    assert_eq!(json["runs"]["post"], "dist/cleanup.js");

    let yaml_str = serde_yaml::to_string(&json).unwrap();
    assert!(yaml_str.contains("using: node20"));
    assert!(yaml_str.contains("main: dist/index.js"));
}

/// Test DockerAction migration roundtrip: TypeScript -> QuickJS -> JSON -> YAML.
#[test]
fn test_docker_action_migration_roundtrip() {
    use gaji::executor;

    let runtime_js = format!(
        "function getAction(ref) {{ return function(config) {{ return {{ uses: ref }}; }}; }}\n{}",
        gaji::generator::templates::JOB_WORKFLOW_RUNTIME_TEMPLATE
    );

    let runtime_stripped = strip_module_syntax(&runtime_js);

    let action_js = r#"
var action = new DockerAction(
    {
        name: "My Docker Action",
        description: "A test Docker action",
        inputs: {
            tag: {
                description: "Docker image tag",
                required: true,
            },
        },
    },
    {
        using: "docker",
        image: "Dockerfile",
        entrypoint: "entrypoint.sh",
        args: ["--tag", "${{ inputs.tag }}"],
        env: {
            REGISTRY: "ghcr.io",
        },
    },
);

action.build("my-docker-action");
"#;

    let bundled = format!("{}\n\n{}", runtime_stripped, action_js);
    let outputs = executor::execute_js(&bundled).unwrap();

    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0].id, "my-docker-action");
    assert_eq!(outputs[0].output_type, "action");

    let json: serde_json::Value = serde_json::from_str(&outputs[0].json).unwrap();
    assert_eq!(json["name"], "My Docker Action");
    assert_eq!(json["runs"]["using"], "docker");
    assert_eq!(json["runs"]["image"], "Dockerfile");
    assert_eq!(json["runs"]["entrypoint"], "entrypoint.sh");
    assert_eq!(json["runs"]["args"][0], "--tag");
    assert_eq!(json["runs"]["env"]["REGISTRY"], "ghcr.io");

    let yaml_str = serde_yaml::to_string(&json).unwrap();
    assert!(yaml_str.contains("using: docker"));
    assert!(yaml_str.contains("image: Dockerfile"));
    assert!(yaml_str.contains("entrypoint: entrypoint.sh"));
}

/// Test that action step outputs are populated with correct expression strings
/// when `id` is provided, and stripped from serialized JSON/YAML.
#[test]
fn test_action_step_outputs_with_id() {
    use gaji::executor;

    let runtime_js = format!(
        r#"var __action_outputs = {{
    'actions/checkout@v5': ['commit', 'ref'],
}};
{}
{}"#,
        gaji::generator::templates::GET_ACTION_RUNTIME_TEMPLATE,
        gaji::generator::templates::JOB_WORKFLOW_RUNTIME_TEMPLATE,
    );

    let runtime_stripped = strip_module_syntax(&runtime_js);

    let workflow_js = r#"
var checkout = getAction("actions/checkout@v5");

new Workflow({
    name: "Output Test",
    on: { push: {} },
}).jobs(function(j) { return j
    .add("build",
        new Job("ubuntu-latest")
            .steps(function(s) { return s
                .add(checkout({ id: "my-checkout" }))
                .add(function(output) { return { name: "Use output", run: "echo " + output["my-checkout"].ref } })
            })
    )
}).build("output-test");
"#;

    let bundled = format!("{}\n\n{}", runtime_stripped, workflow_js);
    let outputs = executor::execute_js(&bundled).unwrap();

    assert_eq!(outputs.len(), 1);
    let json: serde_json::Value = serde_json::from_str(&outputs[0].json).unwrap();

    // Verify outputs and toJSON are NOT in the serialized step
    let steps = json["jobs"]["build"]["steps"].as_array().unwrap();
    assert!(steps[0].get("outputs").is_none());
    assert!(steps[0].get("toJSON").is_none());
    assert_eq!(steps[0]["id"], "my-checkout");
    assert_eq!(steps[0]["uses"], "actions/checkout@v5");

    // Verify the output expression was used in the second step's run command
    assert_eq!(steps[1]["run"], "echo ${{ steps.my-checkout.outputs.ref }}");

    // Verify YAML is clean
    let yaml_str = serde_yaml::to_string(&json).unwrap();
    assert!(yaml_str.contains("steps.my-checkout.outputs.ref"));
    assert!(!yaml_str.contains("toJSON"));
}

/// Test that action step outputs are empty when no `id` is provided.
#[test]
fn test_action_step_outputs_without_id() {
    use gaji::executor;

    let runtime_js = format!(
        r#"var __action_outputs = {{
    'actions/checkout@v5': ['commit', 'ref'],
}};
{}
{}"#,
        gaji::generator::templates::GET_ACTION_RUNTIME_TEMPLATE,
        gaji::generator::templates::JOB_WORKFLOW_RUNTIME_TEMPLATE,
    );

    let runtime_stripped = strip_module_syntax(&runtime_js);

    // Without id, outputs should be empty, and accessing a key gives undefined
    let workflow_js = r#"
var checkout = getAction("actions/checkout@v5");
var step = checkout({});
var hasRef = step.outputs.ref !== undefined;

new Workflow({
    name: "No ID Test",
    on: { push: {} },
}).jobs(function(j) { return j
    .add("build",
        new Job("ubuntu-latest")
            .steps(function(s) { return s
                .add(step)
                .add({ name: "Check", run: "hasRef=" + hasRef })
            })
    )
}).build("no-id-test");
"#;

    let bundled = format!("{}\n\n{}", runtime_stripped, workflow_js);
    let outputs = executor::execute_js(&bundled).unwrap();

    assert_eq!(outputs.len(), 1);
    let json: serde_json::Value = serde_json::from_str(&outputs[0].json).unwrap();

    let steps = json["jobs"]["build"]["steps"].as_array().unwrap();

    // Step should not have outputs or toJSON in serialized form
    assert!(steps[0].get("outputs").is_none());
    assert!(steps[0].get("toJSON").is_none());

    // hasRef should be false since no id was provided
    assert_eq!(steps[1]["run"], "hasRef=false");
}

/// Test job-to-job output passing via `jobOutputs`:
/// - Build job defines outputs from step outputs
/// - `jobOutputs` creates `${{ needs.<id>.outputs.<key> }}` expressions
/// - Deploy job uses those expressions in a run command
/// - Serialized YAML has correct outputs on the build job
#[test]
fn test_job_outputs_passing() {
    use gaji::executor;

    let runtime_js = format!(
        r#"var __action_outputs = {{
    'actions/checkout@v5': ['commit', 'ref'],
}};
{}
{}"#,
        gaji::generator::templates::GET_ACTION_RUNTIME_TEMPLATE,
        gaji::generator::templates::JOB_WORKFLOW_RUNTIME_TEMPLATE,
    );

    let runtime_stripped = strip_module_syntax(&runtime_js);

    let workflow_js = r#"
var checkout = getAction("actions/checkout@v5");

new Workflow({
    name: "Job Outputs Test",
    on: { push: {} },
}).jobs(function(j) { return j
    .add("build",
        new Job("ubuntu-latest")
            .steps(function(s) { return s
                .add(checkout({ id: "my-checkout" }))
            })
            .outputs(function(output) { return { ref: output["my-checkout"].ref, sha: output["my-checkout"].commit } })
    )
    .add("deploy", function(output) {
        return new Job("ubuntu-latest", { needs: ["build"] })
            .steps(function(s) { return s
                .add({ name: "Use output", run: "echo " + output.build.ref + " " + output.build.sha })
            })
    })
}).build("job-outputs-test");
"#;

    let bundled = format!("{}\n\n{}", runtime_stripped, workflow_js);
    let outputs = executor::execute_js(&bundled).unwrap();

    assert_eq!(outputs.len(), 1);
    let json: serde_json::Value = serde_json::from_str(&outputs[0].json).unwrap();

    // Verify build job has outputs with step expressions
    let build_outputs = &json["jobs"]["build"]["outputs"];
    assert_eq!(build_outputs["ref"], "${{ steps.my-checkout.outputs.ref }}");
    assert_eq!(
        build_outputs["sha"],
        "${{ steps.my-checkout.outputs.commit }}"
    );

    // Verify deploy job uses needs expressions
    let deploy_steps = json["jobs"]["deploy"]["steps"].as_array().unwrap();
    assert_eq!(
        deploy_steps[0]["run"],
        "echo ${{ needs.build.outputs.ref }} ${{ needs.build.outputs.sha }}"
    );

    // Verify deploy job has needs
    assert_eq!(json["jobs"]["deploy"]["needs"][0], "build");

    // Verify YAML output is clean
    let yaml_str = serde_yaml::to_string(&json).unwrap();
    assert!(yaml_str.contains("needs.build.outputs.ref"));
    assert!(yaml_str.contains("steps.my-checkout.outputs.ref"));
}

/// Test job-to-job output passing with manually defined string outputs
/// (not derived from action step outputs).
#[test]
fn test_job_outputs_passing_manual_values() {
    use gaji::executor;

    let runtime_js = format!(
        r#"var __action_outputs = {{}};
{}
{}"#,
        gaji::generator::templates::GET_ACTION_RUNTIME_TEMPLATE,
        gaji::generator::templates::JOB_WORKFLOW_RUNTIME_TEMPLATE,
    );

    let runtime_stripped = strip_module_syntax(&runtime_js);

    let workflow_js = r#"
new Workflow({
    name: "Manual Outputs Test",
    on: { push: { tags: ["v*"] } },
}).jobs(function(j) { return j
    .add("setup",
        new Job("ubuntu-latest")
            .steps(function(s) { return s
                .add({ id: "version", run: 'echo "value=1.2.3" >> "$GITHUB_OUTPUT"' })
                .add({ id: "hash", run: 'echo "value=$(git rev-parse --short HEAD)" >> "$GITHUB_OUTPUT"' })
            })
            .outputs({
                version: "${{ steps.version.outputs.value }}",
                commit_hash: "${{ steps.hash.outputs.value }}",
            })
    )
    .add("publish", function(output) {
        return new Job("ubuntu-latest", { needs: ["setup"] })
            .steps(function(s) { return s
                .add({
                    name: "Publish",
                    run: "publish --version " + output.setup.version + " --hash " + output.setup.commit_hash,
                })
            })
    })
}).build("manual-outputs-test");
"#;

    let bundled = format!("{}\n\n{}", runtime_stripped, workflow_js);
    let outputs = executor::execute_js(&bundled).unwrap();

    assert_eq!(outputs.len(), 1);
    let json: serde_json::Value = serde_json::from_str(&outputs[0].json).unwrap();

    // Verify setup job outputs contain step expressions
    let setup_outputs = &json["jobs"]["setup"]["outputs"];
    assert_eq!(
        setup_outputs["version"],
        "${{ steps.version.outputs.value }}"
    );
    assert_eq!(
        setup_outputs["commit_hash"],
        "${{ steps.hash.outputs.value }}"
    );

    // Verify publish job uses needs expressions
    let publish_steps = json["jobs"]["publish"]["steps"].as_array().unwrap();
    assert_eq!(
        publish_steps[0]["run"],
        "publish --version ${{ needs.setup.outputs.version }} --hash ${{ needs.setup.outputs.commit_hash }}"
    );

    // Verify YAML
    let yaml_str = serde_yaml::to_string(&json).unwrap();
    assert!(yaml_str.contains("needs.setup.outputs.version"));
    assert!(yaml_str.contains("steps.version.outputs.value"));
}

/// Test listing action references from a directory of workflow files.
#[tokio::test]
async fn test_list_action_refs_from_directory() {
    let dir = tempfile::TempDir::new().unwrap();
    let workflow_dir = dir.path().join("workflows");
    std::fs::create_dir_all(&workflow_dir).unwrap();

    std::fs::write(
        workflow_dir.join("ci.ts"),
        r#"
        const checkout = getAction("actions/checkout@v5");
        const setup = getAction("actions/setup-node@v4");
        "#,
    )
    .unwrap();

    std::fs::write(
        workflow_dir.join("deploy.ts"),
        r#"
        const checkout = getAction("actions/checkout@v5");
        const cache = getAction("actions/cache@v4");
        "#,
    )
    .unwrap();

    let results = gaji::parser::analyze_directory(&workflow_dir)
        .await
        .unwrap();

    // Invert to action -> files
    let mut action_to_files: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for (file_path, refs) in &results {
        for action_ref in refs {
            action_to_files
                .entry(action_ref.clone())
                .or_default()
                .push(file_path.file_name().unwrap().to_str().unwrap().to_string());
        }
    }
    for files in action_to_files.values_mut() {
        files.sort();
    }

    assert_eq!(action_to_files.len(), 3);
    assert_eq!(
        action_to_files["actions/checkout@v5"],
        vec!["ci.ts", "deploy.ts"]
    );
    assert_eq!(action_to_files["actions/setup-node@v4"], vec!["ci.ts"]);
    assert_eq!(action_to_files["actions/cache@v4"], vec!["deploy.ts"]);
}

/// Test that step callbacks receive previous step outputs via the `output` context.
#[test]
fn test_step_builder_callback_context() {
    use gaji::executor;

    let runtime_js = format!(
        r#"var __action_outputs = {{
    'actions/checkout@v5': ['commit', 'ref'],
}};
{}
{}"#,
        gaji::generator::templates::GET_ACTION_RUNTIME_TEMPLATE,
        gaji::generator::templates::JOB_WORKFLOW_RUNTIME_TEMPLATE,
    );

    let runtime_stripped = strip_module_syntax(&runtime_js);

    let workflow_js = r#"
var checkout = getAction("actions/checkout@v5");

new Workflow({
    name: "Step Callback Test",
    on: { push: {} },
}).jobs(function(j) { return j
    .add("build",
        new Job("ubuntu-latest")
            .steps(function(s) { return s
                .add(checkout({ id: "co" }))
                .add(function(output) { return { name: "Use ref", run: "echo " + output.co.ref } })
            })
    )
}).build("step-callback-test");
"#;

    let bundled = format!("{}\n\n{}", runtime_stripped, workflow_js);
    let outputs = executor::execute_js(&bundled).unwrap();

    assert_eq!(outputs.len(), 1);
    let json: serde_json::Value = serde_json::from_str(&outputs[0].json).unwrap();

    let steps = json["jobs"]["build"]["steps"].as_array().unwrap();
    assert_eq!(steps[0]["id"], "co");
    assert_eq!(steps[1]["run"], "echo ${{ steps.co.outputs.ref }}");
}

/// Test that the `outputs()` callback receives the step context accumulated by `steps()`.
#[test]
fn test_outputs_callback_context() {
    use gaji::executor;

    let runtime_js = format!(
        r#"var __action_outputs = {{
    'actions/checkout@v5': ['commit', 'ref'],
}};
{}
{}"#,
        gaji::generator::templates::GET_ACTION_RUNTIME_TEMPLATE,
        gaji::generator::templates::JOB_WORKFLOW_RUNTIME_TEMPLATE,
    );

    let runtime_stripped = strip_module_syntax(&runtime_js);

    let workflow_js = r#"
var checkout = getAction("actions/checkout@v5");

new Workflow({
    name: "Outputs Callback Test",
    on: { push: {} },
}).jobs(function(j) { return j
    .add("build",
        new Job("ubuntu-latest")
            .steps(function(s) { return s
                .add(checkout({ id: "co" }))
            })
            .outputs(function(output) { return { ref: output.co.ref } })
    )
}).build("outputs-callback-test");
"#;

    let bundled = format!("{}\n\n{}", runtime_stripped, workflow_js);
    let outputs = executor::execute_js(&bundled).unwrap();

    assert_eq!(outputs.len(), 1);
    let json: serde_json::Value = serde_json::from_str(&outputs[0].json).unwrap();

    let build_outputs = &json["jobs"]["build"]["outputs"];
    assert_eq!(build_outputs["ref"], "${{ steps.co.outputs.ref }}");
}

/// Test that job callbacks receive previous job outputs via the `output` context.
#[test]
fn test_job_builder_callback_context() {
    use gaji::executor;

    let runtime_js = format!(
        r#"var __action_outputs = {{
    'actions/checkout@v5': ['commit', 'ref'],
}};
{}
{}"#,
        gaji::generator::templates::GET_ACTION_RUNTIME_TEMPLATE,
        gaji::generator::templates::JOB_WORKFLOW_RUNTIME_TEMPLATE,
    );

    let runtime_stripped = strip_module_syntax(&runtime_js);

    let workflow_js = r#"
var checkout = getAction("actions/checkout@v5");

new Workflow({
    name: "Job Callback Test",
    on: { push: {} },
}).jobs(function(j) { return j
    .add("build",
        new Job("ubuntu-latest")
            .steps(function(s) { return s
                .add(checkout({ id: "co" }))
            })
            .outputs(function(output) { return { sha: output.co.commit } })
    )
    .add("deploy", function(output) {
        return new Job("ubuntu-latest", { needs: ["build"] })
            .steps(function(s) { return s
                .add({ name: "Deploy", run: "deploy --sha " + output.build.sha })
            })
    })
}).build("job-callback-test");
"#;

    let bundled = format!("{}\n\n{}", runtime_stripped, workflow_js);
    let outputs = executor::execute_js(&bundled).unwrap();

    assert_eq!(outputs.len(), 1);
    let json: serde_json::Value = serde_json::from_str(&outputs[0].json).unwrap();

    // Build job outputs should map to step expressions
    let build_outputs = &json["jobs"]["build"]["outputs"];
    assert_eq!(build_outputs["sha"], "${{ steps.co.outputs.commit }}");

    // Deploy job should use needs expressions from JobBuilder context
    let deploy_steps = json["jobs"]["deploy"]["steps"].as_array().unwrap();
    assert_eq!(
        deploy_steps[0]["run"],
        "deploy --sha ${{ needs.build.outputs.sha }}"
    );
}

/// Test WorkflowBuilder.build_all with an empty directory.
#[tokio::test]
async fn test_build_all_empty_directory() {
    let dir = tempfile::TempDir::new().unwrap();
    let input_dir = dir.path().join("workflows");
    let output_dir = dir.path().join("output");
    std::fs::create_dir_all(&input_dir).unwrap();

    let builder = gaji::builder::WorkflowBuilder::new(vec![input_dir], output_dir, false);
    let result = builder.build_all().await.unwrap();
    assert!(result.is_empty());
}

/// Test the full config TS → build pipeline: gaji.config.ts sets custom dirs,
/// build reads them and writes output accordingly.
#[test]
fn test_config_ts_pipeline() {
    let dir = tempfile::TempDir::new().unwrap();

    // Write a gaji.config.ts with custom paths
    std::fs::write(
        dir.path().join("gaji.config.ts"),
        r#"
export default defineConfig({
    workflows: "src/workflows",
    output: "out/.github",
    generated: "src/generated",
    build: {
        cacheTtlDays: 7,
    },
});
"#,
    )
    .unwrap();

    let config = gaji::config::Config::load_from_ts(&dir.path().join("gaji.config.ts")).unwrap();

    assert_eq!(config.project.workflows_dir, "src/workflows");
    assert_eq!(config.project.output_dir, "out/.github");
    assert_eq!(config.project.generated_dir, "src/generated");
    assert_eq!(config.build.cache_ttl_days, 7);

    // Verify paths resolve correctly
    assert_eq!(
        config.workflows_path(),
        std::path::PathBuf::from("src/workflows")
    );
    assert_eq!(
        config.output_path(),
        std::path::PathBuf::from("out/.github")
    );
    assert_eq!(
        config.generated_path(),
        std::path::PathBuf::from("src/generated")
    );
}
