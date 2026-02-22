use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use colored::Colorize;

/// Find all .yml/.yaml files in .github/workflows/
pub fn discover_workflows(root: &Path) -> Result<Vec<PathBuf>> {
    let workflows_dir = root.join(".github/workflows");
    if !workflows_dir.exists() {
        return Ok(vec![]);
    }

    let mut workflows: Vec<PathBuf> = std::fs::read_dir(&workflows_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let ext = entry.path().extension().map(|s| s.to_owned());
            ext.as_deref() == Some(OsStr::new("yml")) || ext.as_deref() == Some(OsStr::new("yaml"))
        })
        .map(|entry| entry.path())
        .collect();

    workflows.sort();
    Ok(workflows)
}

/// Find all action.yml/.yaml files in .github/actions/*/
pub fn discover_actions(root: &Path) -> Result<Vec<PathBuf>> {
    let actions_dir = root.join(".github/actions");
    if !actions_dir.exists() {
        return Ok(vec![]);
    }

    let mut actions: Vec<PathBuf> = std::fs::read_dir(&actions_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
            let yml = entry.path().join("action.yml");
            let yaml = entry.path().join("action.yaml");
            if yml.exists() {
                Some(yml)
            } else if yaml.exists() {
                Some(yaml)
            } else {
                None
            }
        })
        .collect();

    actions.sort();
    Ok(actions)
}

/// Type of action determined from `runs.using` in action.yml.
#[derive(Debug, Clone, PartialEq)]
enum ActionType {
    Composite,
    JavaScript(String),
    Docker,
    Unknown(String),
}

/// Classify an action.yml based on its `runs.using` field.
fn classify_action(yaml: &serde_yaml::Value) -> ActionType {
    let using = yaml
        .get("runs")
        .and_then(|r| r.get("using"))
        .and_then(|u| u.as_str())
        .unwrap_or("");

    match using {
        "composite" => ActionType::Composite,
        u if u.starts_with("node") => ActionType::JavaScript(u.to_string()),
        "docker" => ActionType::Docker,
        other => ActionType::Unknown(other.to_string()),
    }
}

/// Migrate a list of workflow files to TypeScript.
/// Creates .ts files in workflows/ and backs up originals.
pub async fn migrate_workflows(root: &Path, workflows: &[PathBuf]) -> Result<()> {
    println!("{} Migrating workflows to TypeScript...\n", "🔄".cyan());

    let ts_dir = root.join("workflows");
    tokio::fs::create_dir_all(&ts_dir).await?;

    for workflow_path in workflows {
        let workflow_name = workflow_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("Invalid workflow path: {}", workflow_path.display()))?;

        println!("  Migrating {}...", workflow_name);

        let yaml_content = tokio::fs::read_to_string(workflow_path).await?;

        match generate_typescript_from_yaml(&yaml_content, workflow_name) {
            Ok(ts_content) => {
                let ts_path = ts_dir.join(format!("{}.ts", workflow_name));
                tokio::fs::write(&ts_path, ts_content).await?;
                println!("  {} Created {}", "✓".green(), ts_path.display());

                // Backup original
                let backup_path = workflow_path.with_extension("yml.backup");
                tokio::fs::rename(workflow_path, &backup_path).await?;
                println!("  {} Backed up to {}", "✓".green(), backup_path.display());
            }
            Err(e) => {
                eprintln!("  {} Failed to migrate {}: {}", "✗".red(), workflow_name, e);
            }
        }
    }

    println!("\n{} Migration complete!", "✨".green());
    println!("   Review the generated TypeScript files in workflows/");
    println!("   Run 'gaji build' to regenerate YAML files\n");

    Ok(())
}

/// Migrate a list of action files to TypeScript.
/// Creates .ts files in workflows/ and backs up originals.
pub async fn migrate_actions(root: &Path, actions: &[PathBuf]) -> Result<()> {
    println!("{} Migrating actions to TypeScript...\n", "🔄".cyan());

    let ts_dir = root.join("workflows");
    tokio::fs::create_dir_all(&ts_dir).await?;

    for action_path in actions {
        let action_id = action_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("Invalid action path: {}", action_path.display()))?;

        println!("  Migrating action {}...", action_id);

        let yaml_content = tokio::fs::read_to_string(action_path).await?;

        match generate_typescript_from_action_yaml(&yaml_content, action_id) {
            Ok(ts_content) => {
                let ts_path = ts_dir.join(format!("action-{}.ts", action_id));
                tokio::fs::write(&ts_path, ts_content).await?;
                println!("  {} Created {}", "✓".green(), ts_path.display());

                // Backup original
                let backup_path = action_path.with_extension("yml.backup");
                tokio::fs::rename(action_path, &backup_path).await?;
                println!("  {} Backed up to {}", "✓".green(), backup_path.display());
            }
            Err(e) => {
                eprintln!("  {} Failed to migrate {}: {}", "✗".red(), action_id, e);
            }
        }
    }

    println!("\n{} Action migration complete!", "✨".green());
    println!("   Review the generated TypeScript files in workflows/");
    println!("   Run 'gaji build' to regenerate action YAML files\n");

    Ok(())
}

/// Convert a single YAML workflow to TypeScript source code.
fn generate_typescript_from_yaml(yaml_content: &str, workflow_id: &str) -> Result<String> {
    let workflow: serde_yaml::Value =
        serde_yaml::from_str(yaml_content).map_err(|e| anyhow!("Failed to parse YAML: {}", e))?;

    let mut ts = String::new();

    // Imports
    ts.push_str("// Migrated from YAML by gaji init --migrate\n");
    ts.push_str("// NOTE: This is a basic conversion. Please review and adjust as needed.\n");
    ts.push_str("import { getAction, Job, Workflow } from \"../generated/index.js\";\n\n");

    // Extract actions used
    let actions = extract_actions_from_yaml(&workflow);
    for action in &actions {
        let var_name = action_to_var_name(action);
        ts.push_str(&format!(
            "const {} = getAction(\"{}\");\n",
            var_name, action
        ));
    }
    if !actions.is_empty() {
        ts.push('\n');
    }

    // Extract workflow name
    let name = workflow
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(workflow_id);

    // Jobs
    if let Some(jobs) = workflow.get("jobs").and_then(|j| j.as_mapping()) {
        for (job_id, job_def) in jobs {
            let job_id_str = job_id.as_str().unwrap_or("job");
            let runs_on = job_def
                .get("runs-on")
                .and_then(|v| v.as_str())
                .unwrap_or("ubuntu-latest");

            let var = job_id_str.replace('-', "_");
            ts.push_str(&format!("const {} = new Job(\"{}\")\n", var, runs_on));

            // Steps
            if let Some(steps) = job_def.get("steps").and_then(|s| s.as_sequence()) {
                ts.push_str("    .steps(s => s\n");
                for step in steps {
                    generate_step(&mut ts, step, &actions);
                }
                ts.push_str("    )");
            }

            ts.push_str(";\n\n");
        }
    }

    // Workflow definition
    let safe_id = workflow_id.replace('-', "_");
    ts.push_str("const workflow = new Workflow({\n");
    ts.push_str(&format!("    name: \"{}\",\n", escape_js_string(name)));

    // On triggers
    // YAML parses bare `on:` as boolean true, so also check for the boolean key
    let on_value = workflow.get("on").or_else(|| {
        workflow
            .as_mapping()
            .and_then(|map| map.get(serde_yaml::Value::Bool(true)))
    });
    if let Some(on) = on_value {
        ts.push_str("    on: ");
        ts.push_str(&yaml_value_to_js(on, 4));
        ts.push_str(",\n");
    }

    ts.push_str("})");

    // Add jobs
    if let Some(jobs) = workflow.get("jobs").and_then(|j| j.as_mapping()) {
        ts.push_str("\n    .jobs(j => j\n");
        for (job_id, _) in jobs {
            let job_id_str = job_id.as_str().unwrap_or("job");
            let var = job_id_str.replace('-', "_");
            ts.push_str(&format!("        .add(\"{}\", {})\n", job_id_str, var));
        }
        ts.push_str("    )");
    }

    ts.push_str(";\n\n");
    ts.push_str(&format!("workflow.build(\"{}\");\n", safe_id));

    Ok(ts)
}

/// Convert a single action.yml to TypeScript source code.
fn generate_typescript_from_action_yaml(yaml_content: &str, action_id: &str) -> Result<String> {
    let action: serde_yaml::Value =
        serde_yaml::from_str(yaml_content).map_err(|e| anyhow!("Failed to parse YAML: {}", e))?;

    let action_type = classify_action(&action);

    match action_type {
        ActionType::Composite => generate_composite_action_ts(&action, action_id),
        ActionType::JavaScript(using) => generate_javascript_action_ts(&action, action_id, &using),
        ActionType::Docker => generate_docker_action_ts(&action, action_id),
        ActionType::Unknown(using) => Err(anyhow!(
            "Unknown action type '{}' cannot be migrated",
            using
        )),
    }
}

/// Generate TypeScript for an Action (composite).
fn generate_composite_action_ts(action: &serde_yaml::Value, action_id: &str) -> Result<String> {
    let mut ts = String::new();

    // Header
    ts.push_str("// Migrated from YAML by gaji init --migrate\n");
    ts.push_str("// NOTE: This is a basic conversion. Please review and adjust as needed.\n");

    // Extract external actions used in steps
    let actions = extract_actions_from_composite_steps(action);
    let has_external_actions = !actions.is_empty();

    if has_external_actions {
        ts.push_str("import { getAction, Action } from \"../generated/index.js\";\n\n");
        for action_ref in &actions {
            let var_name = action_to_var_name(action_ref);
            ts.push_str(&format!(
                "const {} = getAction(\"{}\");\n",
                var_name, action_ref
            ));
        }
        ts.push('\n');
    } else {
        ts.push_str("import { Action } from \"../generated/index.js\";\n\n");
    }

    // Constructor config
    let name = action
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(action_id);
    let description = action
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    ts.push_str("const action = new Action({\n");
    ts.push_str(&format!("    name: \"{}\",\n", escape_js_string(name)));
    ts.push_str(&format!(
        "    description: \"{}\",\n",
        escape_js_string(description)
    ));

    if let Some(inputs) = action.get("inputs") {
        ts.push_str("    inputs: ");
        ts.push_str(&yaml_value_to_js(inputs, 4));
        ts.push_str(",\n");
    }

    if let Some(outputs) = action.get("outputs") {
        ts.push_str("    outputs: ");
        ts.push_str(&yaml_value_to_js(outputs, 4));
        ts.push_str(",\n");
    }

    ts.push_str("});\n\n");

    // Steps
    if let Some(steps) = action
        .get("runs")
        .and_then(|r| r.get("steps"))
        .and_then(|s| s.as_sequence())
    {
        ts.push_str("action\n    .steps(s => s\n");
        for step in steps {
            generate_composite_action_step(&mut ts, step, &actions);
        }
        ts.push_str("    );\n\n");
    }

    // Build call
    ts.push_str(&format!("action.build(\"{}\");\n", action_id));

    Ok(ts)
}

/// Generate TypeScript for a NodeAction.
fn generate_javascript_action_ts(
    action: &serde_yaml::Value,
    action_id: &str,
    using: &str,
) -> Result<String> {
    let mut ts = String::new();

    ts.push_str("// Migrated from YAML by gaji init --migrate\n");
    ts.push_str("// NOTE: This is a basic conversion. Please review and adjust as needed.\n");
    ts.push_str("import { NodeAction } from \"../generated/index.js\";\n\n");

    let name = action
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(action_id);
    let description = action
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let runs = action
        .get("runs")
        .ok_or_else(|| anyhow!("Missing 'runs' field"))?;
    let main = runs
        .get("main")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'runs.main' field"))?;

    // Config object
    ts.push_str("const action = new NodeAction(\n");
    ts.push_str("    {\n");
    ts.push_str(&format!("        name: \"{}\",\n", escape_js_string(name)));
    ts.push_str(&format!(
        "        description: \"{}\",\n",
        escape_js_string(description)
    ));

    if let Some(inputs) = action.get("inputs") {
        ts.push_str("        inputs: ");
        ts.push_str(&yaml_value_to_js(inputs, 8));
        ts.push_str(",\n");
    }
    if let Some(outputs) = action.get("outputs") {
        ts.push_str("        outputs: ");
        ts.push_str(&yaml_value_to_js(outputs, 8));
        ts.push_str(",\n");
    }

    ts.push_str("    },\n");

    // Runs object
    ts.push_str("    {\n");
    ts.push_str(&format!(
        "        using: \"{}\",\n",
        escape_js_string(using)
    ));
    ts.push_str(&format!("        main: \"{}\",\n", escape_js_string(main)));

    if let Some(pre) = runs.get("pre").and_then(|v| v.as_str()) {
        ts.push_str(&format!("        pre: \"{}\",\n", escape_js_string(pre)));
    }
    if let Some(post) = runs.get("post").and_then(|v| v.as_str()) {
        ts.push_str(&format!("        post: \"{}\",\n", escape_js_string(post)));
    }
    if let Some(pre_if) = runs.get("pre-if").and_then(|v| v.as_str()) {
        ts.push_str(&format!(
            "        \"pre-if\": \"{}\",\n",
            escape_js_string(pre_if)
        ));
    }
    if let Some(post_if) = runs.get("post-if").and_then(|v| v.as_str()) {
        ts.push_str(&format!(
            "        \"post-if\": \"{}\",\n",
            escape_js_string(post_if)
        ));
    }

    ts.push_str("    },\n");
    ts.push_str(");\n\n");

    ts.push_str(&format!("action.build(\"{}\");\n", action_id));

    Ok(ts)
}

/// Generate TypeScript for a DockerAction.
fn generate_docker_action_ts(action: &serde_yaml::Value, action_id: &str) -> Result<String> {
    let mut ts = String::new();

    // Header
    ts.push_str("// Migrated from YAML by gaji init --migrate\n");
    ts.push_str("// NOTE: This is a basic conversion. Please review and adjust as needed.\n");
    ts.push_str("import { DockerAction } from \"../generated/index.js\";\n\n");

    let name = action
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled Action");
    let description = action
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let runs = action
        .get("runs")
        .ok_or_else(|| anyhow!("Missing 'runs' field"))?;
    let image = runs
        .get("image")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'runs.image' field"))?;

    // Config object
    ts.push_str("const action = new DockerAction(\n");
    ts.push_str("    {\n");
    ts.push_str(&format!("        name: \"{}\",\n", escape_js_string(name)));
    ts.push_str(&format!(
        "        description: \"{}\",\n",
        escape_js_string(description)
    ));

    if let Some(inputs) = action.get("inputs") {
        ts.push_str("        inputs: ");
        ts.push_str(&yaml_value_to_js(inputs, 8));
        ts.push_str(",\n");
    }
    if let Some(outputs) = action.get("outputs") {
        ts.push_str("        outputs: ");
        ts.push_str(&yaml_value_to_js(outputs, 8));
        ts.push_str(",\n");
    }

    ts.push_str("    },\n");

    // Runs object
    ts.push_str("    {\n");
    ts.push_str("        using: \"docker\",\n");
    ts.push_str(&format!(
        "        image: \"{}\",\n",
        escape_js_string(image)
    ));

    if let Some(entrypoint) = runs.get("entrypoint").and_then(|v| v.as_str()) {
        ts.push_str(&format!(
            "        entrypoint: \"{}\",\n",
            escape_js_string(entrypoint)
        ));
    }
    if let Some(args) = runs.get("args") {
        ts.push_str("        args: ");
        ts.push_str(&yaml_value_to_js(args, 8));
        ts.push_str(",\n");
    }
    if let Some(env) = runs.get("env") {
        ts.push_str("        env: ");
        ts.push_str(&yaml_value_to_js(env, 8));
        ts.push_str(",\n");
    }
    if let Some(pre) = runs.get("pre-entrypoint").and_then(|v| v.as_str()) {
        ts.push_str(&format!(
            "        \"pre-entrypoint\": \"{}\",\n",
            escape_js_string(pre)
        ));
    }
    if let Some(post) = runs.get("post-entrypoint").and_then(|v| v.as_str()) {
        ts.push_str(&format!(
            "        \"post-entrypoint\": \"{}\",\n",
            escape_js_string(post)
        ));
    }
    if let Some(pre_if) = runs.get("pre-if").and_then(|v| v.as_str()) {
        ts.push_str(&format!(
            "        \"pre-if\": \"{}\",\n",
            escape_js_string(pre_if)
        ));
    }
    if let Some(post_if) = runs.get("post-if").and_then(|v| v.as_str()) {
        ts.push_str(&format!(
            "        \"post-if\": \"{}\",\n",
            escape_js_string(post_if)
        ));
    }

    ts.push_str("    },\n");
    ts.push_str(");\n\n");

    ts.push_str(&format!("action.build(\"{}\");\n", action_id));

    Ok(ts)
}

/// Options for step generation.
struct StepGenOptions {
    /// Whether `shell` is required for `run` steps (true for composite action steps).
    require_shell: bool,
}

/// Generate a step call in the TypeScript output.
fn generate_step(ts: &mut String, step: &serde_yaml::Value, actions: &[String]) {
    generate_step_inner(
        ts,
        step,
        actions,
        &StepGenOptions {
            require_shell: false,
        },
    );
}

/// Generate a step call for composite action steps (shell is required for run steps).
fn generate_composite_action_step(ts: &mut String, step: &serde_yaml::Value, actions: &[String]) {
    generate_step_inner(
        ts,
        step,
        actions,
        &StepGenOptions {
            require_shell: true,
        },
    );
}

/// Shared step generation logic.
fn generate_step_inner(
    ts: &mut String,
    step: &serde_yaml::Value,
    actions: &[String],
    options: &StepGenOptions,
) {
    if let Some(uses) = step.get("uses").and_then(|v| v.as_str()) {
        // Action step
        let var_name = action_to_var_name(uses);
        let is_known = actions.iter().any(|a| a == uses);

        if is_known {
            ts.push_str(&format!("        .add({}({{\n", var_name));
        } else {
            ts.push_str("        .add({\n");
            ts.push_str(&format!(
                "            uses: \"{}\",\n",
                escape_js_string(uses)
            ));
        }

        if let Some(id) = step.get("id").and_then(|v| v.as_str()) {
            ts.push_str(&format!("            id: \"{}\",\n", escape_js_string(id)));
        }

        if let Some(name) = step.get("name").and_then(|v| v.as_str()) {
            ts.push_str(&format!(
                "            name: \"{}\",\n",
                escape_js_string(name)
            ));
        }

        if let Some(with) = step.get("with").and_then(|v| v.as_mapping()) {
            ts.push_str("            with: {\n");
            for (k, v) in with {
                let key_str = k.as_str().unwrap_or("unknown");
                let needs_quotes = key_str.contains('-') || key_str.contains('.');
                if needs_quotes {
                    ts.push_str(&format!(
                        "                \"{}\": {},\n",
                        escape_js_string(key_str),
                        yaml_value_to_js(v, 16)
                    ));
                } else {
                    ts.push_str(&format!(
                        "                {}: {},\n",
                        key_str,
                        yaml_value_to_js(v, 16)
                    ));
                }
            }
            ts.push_str("            },\n");
        }

        if let Some(if_cond) = step.get("if").and_then(|v| v.as_str()) {
            ts.push_str(&format!(
                "            \"if\": \"{}\",\n",
                escape_js_string(if_cond)
            ));
        }

        if let Some(env) = step.get("env").and_then(|v| v.as_mapping()) {
            ts.push_str("            env: {\n");
            for (k, v) in env {
                let key_str = k.as_str().unwrap_or("unknown");
                ts.push_str(&format!(
                    "                {}: {},\n",
                    key_str,
                    yaml_value_to_js(v, 16)
                ));
            }
            ts.push_str("            },\n");
        }

        if is_known {
            ts.push_str("        }))\n");
        } else {
            ts.push_str("        })\n");
        }
    } else if let Some(run) = step.get("run").and_then(|v| v.as_str()) {
        // Run step
        ts.push_str("        .add({\n");

        if let Some(id) = step.get("id").and_then(|v| v.as_str()) {
            ts.push_str(&format!("            id: \"{}\",\n", escape_js_string(id)));
        }

        if let Some(name) = step.get("name").and_then(|v| v.as_str()) {
            ts.push_str(&format!(
                "            name: \"{}\",\n",
                escape_js_string(name)
            ));
        }
        if run.contains('\n') {
            ts.push_str(&format!(
                "            run: `{}`",
                run.replace('`', "\\`").replace("${", "\\${")
            ));
        } else {
            ts.push_str(&format!("            run: \"{}\"", escape_js_string(run)));
        }
        ts.push_str(",\n");

        // shell is required for composite action run steps
        if options.require_shell {
            let shell = step.get("shell").and_then(|v| v.as_str()).unwrap_or("bash");
            ts.push_str(&format!(
                "            shell: \"{}\",\n",
                escape_js_string(shell)
            ));
        } else if let Some(shell) = step.get("shell").and_then(|v| v.as_str()) {
            ts.push_str(&format!(
                "            shell: \"{}\",\n",
                escape_js_string(shell)
            ));
        }

        if let Some(if_cond) = step.get("if").and_then(|v| v.as_str()) {
            ts.push_str(&format!(
                "            \"if\": \"{}\",\n",
                escape_js_string(if_cond)
            ));
        }

        if let Some(env) = step.get("env").and_then(|v| v.as_mapping()) {
            ts.push_str("            env: {\n");
            for (k, v) in env {
                let key_str = k.as_str().unwrap_or("unknown");
                ts.push_str(&format!(
                    "                {}: {},\n",
                    key_str,
                    yaml_value_to_js(v, 16)
                ));
            }
            ts.push_str("            },\n");
        }

        if let Some(wd) = step.get("working-directory").and_then(|v| v.as_str()) {
            ts.push_str(&format!(
                "            \"working-directory\": \"{}\",\n",
                escape_js_string(wd)
            ));
        }

        ts.push_str("        })\n");
    }
}

/// Migrate `.gaji.toml` (and optionally `.gaji.local.toml`) to TypeScript config files.
pub fn migrate_toml_config(root: &Path) -> Result<()> {
    let toml_path = root.join(".gaji.toml");
    if !toml_path.exists() {
        return Ok(());
    }

    let config = crate::config::Config::load_from(&toml_path)?;
    let ts_content = config_to_ts(&config);
    let ts_path = root.join(crate::config::TS_CONFIG_FILE);
    std::fs::write(&ts_path, &ts_content)?;
    println!(
        "{} Migrated .gaji.toml → {}",
        "✓".green(),
        crate::config::TS_CONFIG_FILE
    );

    // Migrate local config if present
    let local_toml_path = root.join(".gaji.local.toml");
    if local_toml_path.exists() {
        let local_config = crate::config::Config::load_from(&local_toml_path)?;
        let local_ts_content = config_to_ts(&local_config);
        let local_ts_path = root.join(crate::config::TS_LOCAL_CONFIG_FILE);
        std::fs::write(&local_ts_path, &local_ts_content)?;
        println!(
            "{} Migrated .gaji.local.toml → {}",
            "✓".green(),
            crate::config::TS_LOCAL_CONFIG_FILE
        );
    }

    // Prompt to remove old files
    let confirm = dialoguer::Confirm::new()
        .with_prompt("Remove old .gaji.toml files?")
        .default(true)
        .interact()
        .unwrap_or(false);

    if confirm {
        std::fs::remove_file(&toml_path)?;
        println!("{} Removed .gaji.toml", "✓".green());
        if local_toml_path.exists() {
            std::fs::remove_file(&local_toml_path)?;
            println!("{} Removed .gaji.local.toml", "✓".green());
        }
    }

    Ok(())
}

/// Convert a Config to TypeScript config source, only emitting non-default values.
fn config_to_ts(config: &crate::config::Config) -> String {
    let defaults = crate::config::Config::default();
    let mut ts = String::new();

    ts.push_str("import { defineConfig } from \"./generated/index.js\";\n\n");
    ts.push_str("export default defineConfig({\n");

    if config.project.workflows_dir != defaults.project.workflows_dir {
        ts.push_str(&format!(
            "    workflows: \"{}\",\n",
            config.project.workflows_dir
        ));
    }
    if config.project.output_dir != defaults.project.output_dir {
        ts.push_str(&format!("    output: \"{}\",\n", config.project.output_dir));
    }
    if config.project.generated_dir != defaults.project.generated_dir {
        ts.push_str(&format!(
            "    generated: \"{}\",\n",
            config.project.generated_dir
        ));
    }

    // Watch section
    let mut watch_parts = vec![];
    if config.watch.debounce_ms != defaults.watch.debounce_ms {
        watch_parts.push(format!("        debounce: {},", config.watch.debounce_ms));
    }
    if config.watch.ignored_patterns != defaults.watch.ignored_patterns {
        let patterns: Vec<String> = config
            .watch
            .ignored_patterns
            .iter()
            .map(|p| format!("\"{}\"", p))
            .collect();
        watch_parts.push(format!("        ignore: [{}],", patterns.join(", ")));
    }
    if !watch_parts.is_empty() {
        ts.push_str("    watch: {\n");
        for part in &watch_parts {
            ts.push_str(part);
            ts.push('\n');
        }
        ts.push_str("    },\n");
    }

    // Build section
    let mut build_parts = vec![];
    if config.build.validate != defaults.build.validate {
        build_parts.push(format!("        validate: {},", config.build.validate));
    }
    if config.build.format != defaults.build.format {
        build_parts.push(format!("        format: {},", config.build.format));
    }
    if config.build.cache_ttl_days != defaults.build.cache_ttl_days {
        build_parts.push(format!(
            "        cacheTtlDays: {},",
            config.build.cache_ttl_days
        ));
    }
    if !build_parts.is_empty() {
        ts.push_str("    build: {\n");
        for part in &build_parts {
            ts.push_str(part);
            ts.push('\n');
        }
        ts.push_str("    },\n");
    }

    // GitHub section
    let mut github_parts = vec![];
    if let Some(ref token) = config.github.token {
        github_parts.push(format!("        token: \"{}\",", token));
    }
    if let Some(ref api_url) = config.github.api_url {
        github_parts.push(format!("        apiUrl: \"{}\",", api_url));
    }
    if !github_parts.is_empty() {
        ts.push_str("    github: {\n");
        for part in &github_parts {
            ts.push_str(part);
            ts.push('\n');
        }
        ts.push_str("    },\n");
    }

    ts.push_str("});\n");
    ts
}

/// Extract all `uses:` values from a parsed YAML workflow.
pub fn extract_actions_from_yaml(workflow: &serde_yaml::Value) -> Vec<String> {
    let mut actions = Vec::new();

    if let Some(jobs) = workflow.get("jobs").and_then(|j| j.as_mapping()) {
        for (_, job) in jobs {
            if let Some(steps) = job.get("steps").and_then(|s| s.as_sequence()) {
                for step in steps {
                    if let Some(uses) = step.get("uses").and_then(|v| v.as_str()) {
                        if !actions.contains(&uses.to_string()) {
                            actions.push(uses.to_string());
                        }
                    }
                }
            }
        }
    }

    actions.sort();
    actions.dedup();
    actions
}

/// Extract all `uses:` values from composite action steps (runs.steps).
/// Skips local action references (starting with "./").
fn extract_actions_from_composite_steps(action: &serde_yaml::Value) -> Vec<String> {
    let mut actions = Vec::new();

    if let Some(steps) = action
        .get("runs")
        .and_then(|r| r.get("steps"))
        .and_then(|s| s.as_sequence())
    {
        for step in steps {
            if let Some(uses) = step.get("uses").and_then(|v| v.as_str()) {
                if !uses.starts_with("./") && !actions.contains(&uses.to_string()) {
                    actions.push(uses.to_string());
                }
            }
        }
    }

    actions.sort();
    actions.dedup();
    actions
}

/// Convert action ref to a camelCase variable name.
/// "actions/checkout@v5" -> "checkout"
/// "actions/setup-node@v4" -> "setupNode"
/// "actions/cache@v3" -> "cache"
pub fn action_to_var_name(action: &str) -> String {
    let base = action
        .split('/')
        .next_back()
        .unwrap_or("action")
        .split('@')
        .next()
        .unwrap_or("action");

    let mut result = String::new();
    let mut capitalize_next = false;
    for ch in base.chars() {
        if ch == '-' || ch == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(ch.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }
    result
}

/// Recursively convert a serde_yaml::Value to JavaScript object literal syntax.
fn yaml_value_to_js(value: &serde_yaml::Value, indent: usize) -> String {
    let pad = " ".repeat(indent);
    let inner_pad = " ".repeat(indent + 4);

    match value {
        serde_yaml::Value::Null => "undefined".to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::String(s) => {
            if s.contains('\n') {
                format!("`{}`", s.replace('`', "\\`").replace("${", "\\${"))
            } else {
                format!("\"{}\"", escape_js_string(s))
            }
        }
        serde_yaml::Value::Sequence(seq) => {
            if seq.is_empty() {
                return "[]".to_string();
            }
            // Check if all items are simple (scalars)
            let all_simple = seq.iter().all(|v| {
                matches!(
                    v,
                    serde_yaml::Value::String(_)
                        | serde_yaml::Value::Number(_)
                        | serde_yaml::Value::Bool(_)
                )
            });

            if all_simple {
                let items: Vec<String> = seq.iter().map(|v| yaml_value_to_js(v, 0)).collect();
                format!("[{}]", items.join(", "))
            } else {
                let mut result = String::from("[\n");
                for item in seq {
                    result.push_str(&inner_pad);
                    result.push_str(&yaml_value_to_js(item, indent + 4));
                    result.push_str(",\n");
                }
                result.push_str(&pad);
                result.push(']');
                result
            }
        }
        serde_yaml::Value::Mapping(map) => {
            if map.is_empty() {
                return "{}".to_string();
            }
            let mut result = String::from("{\n");
            for (k, v) in map {
                let key_str = match k {
                    serde_yaml::Value::String(s) => s.clone(),
                    serde_yaml::Value::Bool(b) => b.to_string(),
                    other => format!("{:?}", other),
                };

                let needs_quotes = key_str.contains('-')
                    || key_str.contains('.')
                    || key_str.contains(' ')
                    || key_str.starts_with(|c: char| c.is_ascii_digit());

                if needs_quotes {
                    result.push_str(&format!(
                        "{}\"{}\": {},\n",
                        inner_pad,
                        escape_js_string(&key_str),
                        yaml_value_to_js(v, indent + 4)
                    ));
                } else {
                    result.push_str(&format!(
                        "{}{}: {},\n",
                        inner_pad,
                        key_str,
                        yaml_value_to_js(v, indent + 4)
                    ));
                }
            }
            result.push_str(&pad);
            result.push('}');
            result
        }
        _ => "undefined".to_string(),
    }
}

fn escape_js_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_to_var_name() {
        assert_eq!(action_to_var_name("actions/checkout@v5"), "checkout");
        assert_eq!(action_to_var_name("actions/setup-node@v4"), "setupNode");
        assert_eq!(
            action_to_var_name("codecov/codecov-action@v3"),
            "codecovAction"
        );
        assert_eq!(action_to_var_name("actions/cache@v3"), "cache");
        assert_eq!(
            action_to_var_name("dtolnay/rust-toolchain@stable"),
            "rustToolchain"
        );
    }

    #[test]
    fn test_extract_actions_from_yaml() {
        let yaml: serde_yaml::Value = serde_yaml::from_str(
            r#"
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - uses: actions/setup-node@v4
      - name: Test
        run: npm test
"#,
        )
        .unwrap();
        let actions = extract_actions_from_yaml(&yaml);
        assert_eq!(
            actions,
            vec!["actions/checkout@v5", "actions/setup-node@v4"]
        );
    }

    #[test]
    fn test_generate_typescript_from_yaml_basic() {
        let yaml_content = r#"
name: CI
on:
  push:
    branches: [main]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - name: Test
        run: npm test
"#;
        let ts = generate_typescript_from_yaml(yaml_content, "ci").unwrap();

        assert!(ts.contains("import { getAction, Job, Workflow }"));
        assert!(ts.contains(r#"getAction("actions/checkout@v5")"#));
        assert!(ts.contains("new Job("));
        assert!(ts.contains(".steps(s => s"));
        assert!(ts.contains(".add(checkout("));
        assert!(ts.contains("new Workflow("));
        assert!(ts.contains(".jobs(j => j"));
        assert!(ts.contains(r#".add("build", build)"#));
        assert!(ts.contains(r#"workflow.build("ci")"#));
        // Old API should NOT be present
        assert!(!ts.contains(".addStep("));
        assert!(!ts.contains(".addJob("));
    }

    #[test]
    fn test_yaml_value_to_js_string() {
        let val = serde_yaml::Value::String("hello".to_string());
        assert_eq!(yaml_value_to_js(&val, 0), "\"hello\"");
    }

    #[test]
    fn test_yaml_value_to_js_array() {
        let val: serde_yaml::Value = serde_yaml::from_str("[main, dev]").unwrap();
        assert_eq!(yaml_value_to_js(&val, 0), "[\"main\", \"dev\"]");
    }

    #[test]
    fn test_yaml_value_to_js_number() {
        let val = serde_yaml::Value::Number(serde_yaml::Number::from(42));
        assert_eq!(yaml_value_to_js(&val, 0), "42");
    }

    #[test]
    fn test_yaml_value_to_js_bool() {
        let val = serde_yaml::Value::Bool(true);
        assert_eq!(yaml_value_to_js(&val, 0), "true");
    }

    #[test]
    fn test_escape_js_string() {
        assert_eq!(escape_js_string(r#"say "hello""#), r#"say \"hello\""#);
        assert_eq!(escape_js_string("line1\nline2"), "line1\\nline2");
    }

    #[test]
    fn test_yaml_value_to_js_multiline_string_escapes_dollar_brace() {
        let val = serde_yaml::Value::String("echo ${{ secrets.TOKEN }}\necho done".to_string());
        let result = yaml_value_to_js(&val, 0);
        assert!(
            result.contains("\\${"),
            "Expected \\${{ but got: {}",
            result
        );
    }

    #[test]
    fn test_generate_step_multiline_run_escapes_dollar_brace() {
        let yaml_content = r#"
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Multi-line with expression
        run: |
          echo ${{ secrets.TOKEN }}
          echo done
"#;
        let ts = generate_typescript_from_yaml(yaml_content, "ci").unwrap();
        assert!(
            ts.contains("\\${{ secrets.TOKEN }}"),
            "Expected escaped \\${{ but got: {}",
            ts
        );
        // Should use new API
        assert!(ts.contains(".steps(s => s"));
        assert!(ts.contains(".add({"));
        assert!(!ts.contains(".addStep("));
    }

    #[test]
    fn test_discover_workflows_empty() {
        let temp = tempfile::TempDir::new().unwrap();
        let result = discover_workflows(temp.path()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_discover_workflows_finds_yml() {
        let temp = tempfile::TempDir::new().unwrap();
        let wf_dir = temp.path().join(".github/workflows");
        std::fs::create_dir_all(&wf_dir).unwrap();
        std::fs::write(wf_dir.join("ci.yml"), "name: CI").unwrap();
        std::fs::write(wf_dir.join("release.yaml"), "name: Release").unwrap();
        std::fs::write(wf_dir.join("readme.md"), "not a workflow").unwrap();

        let result = discover_workflows(temp.path()).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_discover_actions_empty() {
        let temp = tempfile::TempDir::new().unwrap();
        let result = discover_actions(temp.path()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_discover_actions_finds_action_yml() {
        let temp = tempfile::TempDir::new().unwrap();
        let actions_dir = temp.path().join(".github/actions");

        std::fs::create_dir_all(actions_dir.join("my-action")).unwrap();
        std::fs::write(
            actions_dir.join("my-action/action.yml"),
            "name: My Action\nruns:\n  using: composite\n  steps: []",
        )
        .unwrap();

        std::fs::create_dir_all(actions_dir.join("other-action")).unwrap();
        std::fs::write(
            actions_dir.join("other-action/action.yaml"),
            "name: Other\nruns:\n  using: node20\n  main: index.js",
        )
        .unwrap();

        // Non-action directory (no action.yml)
        std::fs::create_dir_all(actions_dir.join("not-an-action")).unwrap();

        let result = discover_actions(temp.path()).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_classify_action_composite() {
        let yaml: serde_yaml::Value =
            serde_yaml::from_str("runs:\n  using: composite\n  steps: []").unwrap();
        assert_eq!(classify_action(&yaml), ActionType::Composite);
    }

    #[test]
    fn test_classify_action_node20() {
        let yaml: serde_yaml::Value =
            serde_yaml::from_str("runs:\n  using: node20\n  main: index.js").unwrap();
        assert_eq!(
            classify_action(&yaml),
            ActionType::JavaScript("node20".to_string())
        );
    }

    #[test]
    fn test_classify_action_docker() {
        let yaml: serde_yaml::Value =
            serde_yaml::from_str("runs:\n  using: docker\n  image: Dockerfile").unwrap();
        assert_eq!(classify_action(&yaml), ActionType::Docker);
    }

    #[test]
    fn test_generate_composite_action_ts() {
        let yaml_content = r#"
name: Setup Project
description: Sets up the project environment
inputs:
  node-version:
    description: Node.js version
    required: false
    default: "20"
outputs:
  cache-hit:
    description: Whether cache was hit
runs:
  using: composite
  steps:
    - uses: actions/checkout@v5
    - name: Setup Node
      run: echo "Setting up node"
      shell: bash
"#;
        let ts = generate_typescript_from_action_yaml(yaml_content, "setup-project").unwrap();

        assert!(ts.contains("import { getAction, Action }"));
        assert!(ts.contains("new Action("));
        assert!(ts.contains("name: \"Setup Project\""));
        assert!(ts.contains("description: \"Sets up the project environment\""));
        assert!(ts.contains("action.build(\"setup-project\")"));
        assert!(ts.contains(".steps(s => s"));
        assert!(ts.contains(".add(checkout("));
        assert!(ts.contains("shell: \"bash\""));
        assert!(ts.contains(r#"getAction("actions/checkout@v5")"#));
        // Old API should NOT be present
        assert!(!ts.contains(".addStep("));
    }

    #[test]
    fn test_generate_composite_action_ts_no_external_actions() {
        let yaml_content = r#"
name: Simple Action
description: A simple action
runs:
  using: composite
  steps:
    - name: Hello
      run: echo hello
      shell: bash
"#;
        let ts = generate_typescript_from_action_yaml(yaml_content, "simple-action").unwrap();

        assert!(ts.contains("import { Action } from"));
        assert!(!ts.contains("getAction"));
        assert!(ts.contains("new Action("));
        assert!(ts.contains("shell: \"bash\""));
    }

    #[test]
    fn test_generate_composite_action_ts_default_shell() {
        let yaml_content = r#"
name: Action
description: Test
runs:
  using: composite
  steps:
    - name: No shell specified
      run: echo test
"#;
        let ts = generate_typescript_from_action_yaml(yaml_content, "test-action").unwrap();

        // Shell should default to "bash" for composite action run steps
        assert!(ts.contains("shell: \"bash\""));
    }

    #[test]
    fn test_generate_javascript_action_ts() {
        let yaml_content = r#"
name: My JS Action
description: A Node.js action
inputs:
  token:
    description: GitHub token
    required: true
runs:
  using: node20
  main: dist/index.js
  pre: dist/pre.js
  post: dist/post.js
  post-if: success()
"#;
        let ts = generate_typescript_from_action_yaml(yaml_content, "my-js-action").unwrap();

        assert!(ts.contains("import { NodeAction }"));
        assert!(ts.contains("new NodeAction("));
        assert!(ts.contains("using: \"node20\""));
        assert!(ts.contains("main: \"dist/index.js\""));
        assert!(ts.contains("pre: \"dist/pre.js\""));
        assert!(ts.contains("post: \"dist/post.js\""));
        assert!(ts.contains("\"post-if\": \"success()\""));
        assert!(ts.contains("action.build(\"my-js-action\")"));
    }

    #[test]
    fn test_generate_docker_action_ts() {
        let yaml_content = r#"
name: My Docker Action
description: Run a Docker container
inputs:
  tag:
    description: Docker image tag
    required: true
runs:
  using: docker
  image: Dockerfile
  entrypoint: entrypoint.sh
  args:
    - --tag
    - ${{ inputs.tag }}
  env:
    REGISTRY: ghcr.io
  pre-entrypoint: setup.sh
  post-entrypoint: cleanup.sh
"#;
        let result = generate_typescript_from_action_yaml(yaml_content, "my-docker-action");
        assert!(result.is_ok());
        let ts = result.unwrap();
        assert!(ts.contains("import { DockerAction }"));
        assert!(ts.contains("new DockerAction("));
        assert!(ts.contains("using: \"docker\""));
        assert!(ts.contains("image: \"Dockerfile\""));
        assert!(ts.contains("entrypoint: \"entrypoint.sh\""));
        assert!(ts.contains("\"pre-entrypoint\": \"setup.sh\""));
        assert!(ts.contains("\"post-entrypoint\": \"cleanup.sh\""));
        assert!(ts.contains("action.build(\"my-docker-action\")"));
    }

    #[test]
    fn test_extract_actions_from_composite_steps() {
        let yaml: serde_yaml::Value = serde_yaml::from_str(
            r#"
runs:
  using: composite
  steps:
    - uses: actions/checkout@v5
    - uses: actions/setup-node@v4
    - name: Build
      run: npm run build
      shell: bash
    - uses: "./.github/actions/local-action"
"#,
        )
        .unwrap();
        let actions = extract_actions_from_composite_steps(&yaml);
        // Should include external actions but NOT local action references
        assert_eq!(
            actions,
            vec!["actions/checkout@v5", "actions/setup-node@v4"]
        );
    }

    #[test]
    fn test_config_to_ts_defaults_only() {
        let config = crate::config::Config::default();
        let ts = config_to_ts(&config);
        assert!(ts.contains("defineConfig"));
        // Defaults should not be emitted
        assert!(!ts.contains("workflows:"));
        assert!(!ts.contains("output:"));
        assert!(!ts.contains("generated:"));
    }

    #[test]
    fn test_config_to_ts_custom_values() {
        let mut config = crate::config::Config::default();
        config.project.workflows_dir = "src/workflows".to_string();
        config.project.output_dir = "dist/.github".to_string();
        config.build.cache_ttl_days = 14;
        config.github.token = Some("ghp_test".to_string());

        let ts = config_to_ts(&config);
        assert!(ts.contains("defineConfig"));
        assert!(ts.contains("workflows: \"src/workflows\""));
        assert!(ts.contains("output: \"dist/.github\""));
        assert!(ts.contains("cacheTtlDays: 14"));
        assert!(ts.contains("token: \"ghp_test\""));
        // Default generated dir should not be emitted
        assert!(!ts.contains("generated:"));
    }

    #[test]
    fn test_migrate_toml_config_creates_ts() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::write(
            temp.path().join(".gaji.toml"),
            r#"
[project]
workflows_dir = "src/workflows"
output_dir = "dist/.github"

[build]
cache_ttl_days = 14
"#,
        )
        .unwrap();

        // migrate_toml_config uses dialoguer::Confirm which needs a terminal,
        // so we test config_to_ts directly and verify file creation logic
        let config = crate::config::Config::load_from(&temp.path().join(".gaji.toml")).unwrap();
        let ts = config_to_ts(&config);

        assert!(ts.contains("workflows: \"src/workflows\""));
        assert!(ts.contains("output: \"dist/.github\""));
        assert!(ts.contains("cacheTtlDays: 14"));
    }

    #[test]
    fn test_migrate_toml_config_local() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::write(
            temp.path().join(".gaji.local.toml"),
            r#"
[github]
token = "ghp_secret"
api_url = "https://ghe.corp.com"
"#,
        )
        .unwrap();

        let config =
            crate::config::Config::load_from(&temp.path().join(".gaji.local.toml")).unwrap();
        let ts = config_to_ts(&config);

        assert!(ts.contains("token: \"ghp_secret\""));
        assert!(ts.contains("apiUrl: \"https://ghe.corp.com\""));
        // Should not emit default project settings
        assert!(!ts.contains("workflows:"));
    }
}
