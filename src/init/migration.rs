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

/// Generate a step call in the TypeScript output.
fn generate_step(ts: &mut String, step: &serde_yaml::Value, actions: &[String]) {
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
        if let Some(name) = step.get("name").and_then(|v| v.as_str()) {
            ts.push_str(&format!("        name: \"{}\",\n", escape_js_string(name)));
        }
        if run.contains('\n') {
            ts.push_str(&format!("        run: `{}`", run.replace('`', "\\`")));
        } else {
            ts.push_str(&format!("        run: \"{}\"", escape_js_string(run)));
        }
        ts.push_str(",\n");

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

/// Convert action ref to a camelCase variable name.
/// "actions/checkout@v4" -> "checkout"
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
                format!("`{}`", s.replace('`', "\\`"))
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
        assert_eq!(action_to_var_name("actions/checkout@v4"), "checkout");
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
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
      - name: Test
        run: npm test
"#,
        )
        .unwrap();
        let actions = extract_actions_from_yaml(&yaml);
        assert_eq!(
            actions,
            vec!["actions/checkout@v4", "actions/setup-node@v4"]
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
      - uses: actions/checkout@v4
      - name: Test
        run: npm test
"#;
        let ts = generate_typescript_from_yaml(yaml_content, "ci").unwrap();

        assert!(ts.contains("import { getAction, Job, Workflow }"));
        assert!(ts.contains(r#"getAction("actions/checkout@v4")"#));
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
}
