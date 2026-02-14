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
                for step in steps {
                    generate_step(&mut ts, step, &actions);
                }
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
        for (job_id, _) in jobs {
            let job_id_str = job_id.as_str().unwrap_or("job");
            let var = job_id_str.replace('-', "_");
            ts.push_str(&format!(".addJob(\"{}\", {})", job_id_str, var));
        }
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
        ActionType::Docker => Err(anyhow!(
            "Docker actions are not yet supported for migration"
        )),
        ActionType::Unknown(using) => Err(anyhow!(
            "Unknown action type '{}' cannot be migrated",
            using
        )),
    }
}

/// Generate TypeScript for a CompositeAction.
fn generate_composite_action_ts(action: &serde_yaml::Value, action_id: &str) -> Result<String> {
    let mut ts = String::new();

    // Header
    ts.push_str("// Migrated from YAML by gaji init --migrate\n");
    ts.push_str("// NOTE: This is a basic conversion. Please review and adjust as needed.\n");

    // Extract external actions used in steps
    let actions = extract_actions_from_composite_steps(action);
    let has_external_actions = !actions.is_empty();

    if has_external_actions {
        ts.push_str("import { getAction, CompositeAction } from \"../generated/index.js\";\n\n");
        for action_ref in &actions {
            let var_name = action_to_var_name(action_ref);
            ts.push_str(&format!(
                "const {} = getAction(\"{}\");\n",
                var_name, action_ref
            ));
        }
        ts.push('\n');
    } else {
        ts.push_str("import { CompositeAction } from \"../generated/index.js\";\n\n");
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

    ts.push_str("const action = new CompositeAction({\n");
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
        ts.push_str("action\n");
        for step in steps {
            generate_composite_action_step(&mut ts, step, &actions);
        }
        ts.push_str(";\n\n");
    }

    // Build call
    ts.push_str(&format!("action.build(\"{}\");\n", action_id));

    Ok(ts)
}

/// Generate TypeScript for a JavaScriptAction.
fn generate_javascript_action_ts(
    action: &serde_yaml::Value,
    action_id: &str,
    using: &str,
) -> Result<String> {
    let mut ts = String::new();

    ts.push_str("// Migrated from YAML by gaji init --migrate\n");
    ts.push_str("// NOTE: This is a basic conversion. Please review and adjust as needed.\n");
    ts.push_str("import { JavaScriptAction } from \"../generated/index.js\";\n\n");

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
    ts.push_str("const action = new JavaScriptAction(\n");
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
            ts.push_str(&format!("    .addStep({}({{\n", var_name));
        } else {
            ts.push_str("    .addStep({\n");
            ts.push_str(&format!("        uses: \"{}\",\n", escape_js_string(uses)));
        }

        if let Some(id) = step.get("id").and_then(|v| v.as_str()) {
            ts.push_str(&format!("        id: \"{}\",\n", escape_js_string(id)));
        }

        if let Some(name) = step.get("name").and_then(|v| v.as_str()) {
            ts.push_str(&format!("        name: \"{}\",\n", escape_js_string(name)));
        }

        if let Some(with) = step.get("with").and_then(|v| v.as_mapping()) {
            ts.push_str("        with: {\n");
            for (k, v) in with {
                let key_str = k.as_str().unwrap_or("unknown");
                let needs_quotes = key_str.contains('-') || key_str.contains('.');
                if needs_quotes {
                    ts.push_str(&format!(
                        "            \"{}\": {},\n",
                        escape_js_string(key_str),
                        yaml_value_to_js(v, 12)
                    ));
                } else {
                    ts.push_str(&format!(
                        "            {}: {},\n",
                        key_str,
                        yaml_value_to_js(v, 12)
                    ));
                }
            }
            ts.push_str("        },\n");
        }

        if let Some(if_cond) = step.get("if").and_then(|v| v.as_str()) {
            ts.push_str(&format!(
                "        \"if\": \"{}\",\n",
                escape_js_string(if_cond)
            ));
        }

        if let Some(env) = step.get("env").and_then(|v| v.as_mapping()) {
            ts.push_str("        env: {\n");
            for (k, v) in env {
                let key_str = k.as_str().unwrap_or("unknown");
                ts.push_str(&format!(
                    "            {}: {},\n",
                    key_str,
                    yaml_value_to_js(v, 12)
                ));
            }
            ts.push_str("        },\n");
        }

        if is_known {
            ts.push_str("    }))\n");
        } else {
            ts.push_str("    })\n");
        }
    } else if let Some(run) = step.get("run").and_then(|v| v.as_str()) {
        // Run step
        ts.push_str("    .addStep({\n");

        if let Some(id) = step.get("id").and_then(|v| v.as_str()) {
            ts.push_str(&format!("        id: \"{}\",\n", escape_js_string(id)));
        }

        if let Some(name) = step.get("name").and_then(|v| v.as_str()) {
            ts.push_str(&format!("        name: \"{}\",\n", escape_js_string(name)));
        }
        if run.contains('\n') {
            ts.push_str(&format!(
                "        run: `{}`",
                run.replace('`', "\\`").replace("${", "\\${")
            ));
        } else {
            ts.push_str(&format!("        run: \"{}\"", escape_js_string(run)));
        }
        ts.push_str(",\n");

        // shell is required for composite action run steps
        if options.require_shell {
            let shell = step.get("shell").and_then(|v| v.as_str()).unwrap_or("bash");
            ts.push_str(&format!(
                "        shell: \"{}\",\n",
                escape_js_string(shell)
            ));
        } else if let Some(shell) = step.get("shell").and_then(|v| v.as_str()) {
            ts.push_str(&format!(
                "        shell: \"{}\",\n",
                escape_js_string(shell)
            ));
        }

        if let Some(if_cond) = step.get("if").and_then(|v| v.as_str()) {
            ts.push_str(&format!(
                "        \"if\": \"{}\",\n",
                escape_js_string(if_cond)
            ));
        }

        if let Some(env) = step.get("env").and_then(|v| v.as_mapping()) {
            ts.push_str("        env: {\n");
            for (k, v) in env {
                let key_str = k.as_str().unwrap_or("unknown");
                ts.push_str(&format!(
                    "            {}: {},\n",
                    key_str,
                    yaml_value_to_js(v, 12)
                ));
            }
            ts.push_str("        },\n");
        }

        if let Some(wd) = step.get("working-directory").and_then(|v| v.as_str()) {
            ts.push_str(&format!(
                "        \"working-directory\": \"{}\",\n",
                escape_js_string(wd)
            ));
        }

        ts.push_str("    })\n");
    }
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
        assert!(ts.contains("new Workflow("));
        assert!(ts.contains(r#"workflow.build("ci")"#));
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

        assert!(ts.contains("import { getAction, CompositeAction }"));
        assert!(ts.contains("new CompositeAction("));
        assert!(ts.contains("name: \"Setup Project\""));
        assert!(ts.contains("description: \"Sets up the project environment\""));
        assert!(ts.contains("action.build(\"setup-project\")"));
        assert!(ts.contains(".addStep("));
        assert!(ts.contains("shell: \"bash\""));
        assert!(ts.contains(r#"getAction("actions/checkout@v5")"#));
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

        assert!(ts.contains("import { CompositeAction } from"));
        assert!(!ts.contains("getAction"));
        assert!(ts.contains("new CompositeAction("));
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

        assert!(ts.contains("import { JavaScriptAction }"));
        assert!(ts.contains("new JavaScriptAction("));
        assert!(ts.contains("using: \"node20\""));
        assert!(ts.contains("main: \"dist/index.js\""));
        assert!(ts.contains("pre: \"dist/pre.js\""));
        assert!(ts.contains("post: \"dist/post.js\""));
        assert!(ts.contains("\"post-if\": \"success()\""));
        assert!(ts.contains("action.build(\"my-js-action\")"));
    }

    #[test]
    fn test_generate_docker_action_fails() {
        let yaml_content = "name: Docker\nruns:\n  using: docker\n  image: Dockerfile";
        let result = generate_typescript_from_action_yaml(yaml_content, "docker-action");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Docker"));
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
}
