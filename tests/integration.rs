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
var checkout = getAction("actions/checkout@v4");
var build = new Job("ubuntu-latest")
    .addStep(checkout({ name: "Checkout" }))
    .addStep({ name: "Test", run: "npm test" });

var wf = new Workflow({
    name: "Integration Test",
    on: { push: { branches: ["main"] } },
}).addJob("build", build);

wf.build("integration-test");
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
    assert!(yaml_str.contains("uses: actions/checkout@v4"));
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
var job1 = new Job("ubuntu-latest").addStep({ name: "Step1", run: "echo 1" });
var job2 = new Job("ubuntu-latest").addStep({ name: "Step2", run: "echo 2" });

var wf1 = new Workflow({ name: "WF1", on: { push: {} } }).addJob("job1", job1);
var wf2 = new Workflow({ name: "WF2", on: { pull_request: {} } }).addJob("job2", job2);

wf1.build("workflow-1");
wf2.build("workflow-2");
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

/// Test CompositeJob: class inheritance to create reusable job templates.
#[test]
fn test_composite_job_inheritance() {
    use gaji::executor;

    let runtime_js = format!(
        "function getAction(ref) {{ return function(config) {{ if (config === undefined) config = {{}}; var step = {{ uses: ref }}; if (config.name !== undefined) step.name = config.name; return step; }}; }}\n{}",
        gaji::generator::templates::JOB_WORKFLOW_RUNTIME_TEMPLATE
    );

    let runtime_stripped = strip_module_syntax(&runtime_js);

    // Simulate TypeScript compiled output: class DeployJob extends CompositeJob
    let workflow_js = r#"
var checkout = getAction("actions/checkout@v4");

class DeployJob extends CompositeJob {
    constructor(environment) {
        super("ubuntu-latest");
        this.env({ ENVIRONMENT: environment })
            .addStep(checkout({ name: "Checkout" }))
            .addStep({ name: "Deploy", run: "npm run deploy:" + environment });
    }
}

var wf = new Workflow({
    name: "Deploy",
    on: { push: { tags: ["v*"] } },
})
    .addJob("deploy-staging", new DeployJob("staging"))
    .addJob("deploy-production", new DeployJob("production").needs(["deploy-staging"]));

wf.build("deploy");
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
    assert_eq!(staging["steps"][0]["uses"], "actions/checkout@v4");
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

/// Test WorkflowBuilder.build_all with an empty directory.
#[tokio::test]
async fn test_build_all_empty_directory() {
    let dir = tempfile::TempDir::new().unwrap();
    let input_dir = dir.path().join("workflows");
    let output_dir = dir.path().join("output");
    std::fs::create_dir_all(&input_dir).unwrap();

    let builder = gaji::builder::WorkflowBuilder::new(input_dir, output_dir, false);
    let result = builder.build_all().await.unwrap();
    assert!(result.is_empty());
}
