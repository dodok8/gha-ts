use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use colored::Colorize;
use tokio::fs;

use crate::executor;

pub struct WorkflowBuilder {
    input_dir: PathBuf,
    output_dir: PathBuf,
    dry_run: bool,
}

impl WorkflowBuilder {
    pub fn new(input_dir: PathBuf, output_dir: PathBuf, dry_run: bool) -> Self {
        Self {
            input_dir,
            output_dir,
            dry_run,
        }
    }

    pub async fn build_all(&self) -> Result<Vec<PathBuf>> {
        // Ensure output directory exists (skip in dry-run mode)
        if !self.dry_run {
            fs::create_dir_all(&self.output_dir).await?;
        }

        // Find all workflow files
        let workflow_files = self.find_workflow_files().await?;

        if workflow_files.is_empty() {
            println!(
                "{} No workflow files found in {}",
                "⚠️".yellow(),
                self.input_dir.display()
            );
            return Ok(Vec::new());
        }

        let mut built_files = Vec::new();

        for file in workflow_files {
            match self.build_workflow(&file).await {
                Ok(output_paths) => {
                    built_files.extend(output_paths);
                }
                Err(e) => {
                    eprintln!(
                        "{} Failed to build {}: {}",
                        "❌".red(),
                        file.display(),
                        e
                    );
                }
            }
        }

        Ok(built_files)
    }

    async fn find_workflow_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let mut entries = fs::read_dir(&self.input_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "ts" && !path.to_string_lossy().contains(".d.ts") {
                    files.push(path);
                }
            }
        }

        Ok(files)
    }

    /// Build a single workflow file. Returns multiple output paths since one
    /// file can define multiple workflows/actions via multiple .build() calls.
    pub async fn build_workflow(&self, workflow_path: &Path) -> Result<Vec<PathBuf>> {
        println!(
            "{} Building {}...",
            "🔨".cyan(),
            workflow_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        );

        // Try QuickJS execution first if generated/index.js exists
        // Look relative to CWD (project root), not relative to input_dir
        let runtime_js_path = PathBuf::from("generated/index.js");

        let build_outputs = if runtime_js_path.exists() {
            match executor::execute_workflow(workflow_path, &runtime_js_path) {
                Ok(outputs) if !outputs.is_empty() => outputs,
                Ok(_) => {
                    // QuickJS succeeded but no build() calls found, try fallback
                    eprintln!(
                        "   {} QuickJS: no build() calls found, trying npx tsx fallback...",
                        "⚠️".yellow()
                    );
                    let json = execute_workflow_npx(workflow_path)?;
                    vec![executor::BuildOutput {
                        id: workflow_path
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                        json,
                        output_type: "workflow".to_string(),
                    }]
                }
                Err(e) => {
                    eprintln!(
                        "   {} QuickJS failed ({}), trying npx tsx fallback...",
                        "⚠️".yellow(),
                        e
                    );
                    let json = execute_workflow_npx(workflow_path)?;
                    vec![executor::BuildOutput {
                        id: workflow_path
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                        json,
                        output_type: "workflow".to_string(),
                    }]
                }
            }
        } else {
            // No runtime JS, use npx tsx directly
            let json = execute_workflow_npx(workflow_path)?;
            vec![executor::BuildOutput {
                id: workflow_path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                json,
                output_type: "workflow".to_string(),
            }]
        };

        let mut output_paths = Vec::new();

        for build_output in &build_outputs {
            let yaml_content = json_to_yaml(&build_output.json)?;

            if build_output.output_type == "workflow" {
                validate_workflow_yaml(&yaml_content)?;
            }

            if self.dry_run {
                // Print YAML to stdout without writing files
                println!(
                    "--- {} ({}) ---",
                    build_output.id,
                    build_output.output_type
                );
                print!("{}", yaml_content);
                continue;
            }

            // Determine output directory based on type
            let out_dir = if build_output.output_type == "action" {
                let action_dir = self.output_dir.parent().unwrap_or(Path::new(".")).join("actions").join(&build_output.id);
                fs::create_dir_all(&action_dir).await?;
                action_dir
            } else {
                self.output_dir.clone()
            };

            // Determine output filename
            let output_path = if build_output.output_type == "action" {
                out_dir.join("action.yml")
            } else {
                out_dir.join(format!("{}.yml", build_output.id))
            };

            // Check if content changed
            if should_write_file(&output_path, &yaml_content).await? {
                let final_content = format!(
                    "# Auto-generated by gaji\n# Do not edit manually - Edit {} instead\n# Generated at: {}\n\n{}",
                    workflow_path.display(),
                    timestamp_now(),
                    yaml_content
                );

                fs::write(&output_path, final_content).await?;
                println!(
                    "   {} Wrote {}",
                    "✅".green(),
                    output_path.display()
                );
            } else {
                println!(
                    "   {} {} (unchanged)",
                    "⏭️".dimmed(),
                    output_path.display()
                );
            }

            // Handle node shell file copying
            copy_node_shell_files(&build_output.json, workflow_path, &out_dir).await?;

            output_paths.push(output_path);
        }

        Ok(output_paths)
    }
}

/// Execute a workflow file using npx tsx (fallback strategy)
fn execute_workflow_npx(workflow_path: &Path) -> Result<String> {
    let output = Command::new("npx")
        .args(["tsx", workflow_path.to_str().unwrap()])
        .output();

    match output {
        Ok(output) if output.status.success() => Ok(String::from_utf8(output.stdout)?),
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!(
                "Failed to execute workflow:\n{}",
                stderr
            ))
        }
        Err(_) => {
            // Try ts-node as fallback
            let output = Command::new("npx")
                .args(["ts-node", workflow_path.to_str().unwrap()])
                .output()
                .context("Neither tsx nor ts-node is available")?;

            if output.status.success() {
                Ok(String::from_utf8(output.stdout)?)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(anyhow::anyhow!(
                    "Failed to execute workflow:\n{}",
                    stderr
                ))
            }
        }
    }
}

fn json_to_yaml(json_str: &str) -> Result<String> {
    let json_value: serde_json::Value =
        serde_json::from_str(json_str).context("Invalid JSON output from workflow")?;

    let yaml_str = serde_yaml::to_string(&json_value)?;

    Ok(yaml_str)
}

fn validate_workflow_yaml(yaml: &str) -> Result<()> {
    let value: serde_yaml::Value =
        serde_yaml::from_str(yaml).context("Invalid YAML syntax")?;

    let mapping = value
        .as_mapping()
        .ok_or_else(|| anyhow::anyhow!("Workflow must be a YAML mapping"))?;

    // Check for required 'on' field
    if !mapping.contains_key(serde_yaml::Value::String("on".to_string())) {
        return Err(anyhow::anyhow!("Workflow missing required 'on' field"));
    }

    // Check for required 'jobs' field
    if !mapping.contains_key(serde_yaml::Value::String("jobs".to_string())) {
        return Err(anyhow::anyhow!("Workflow missing required 'jobs' field"));
    }

    Ok(())
}

async fn should_write_file(path: &Path, new_content: &str) -> Result<bool> {
    if !path.exists() {
        return Ok(true);
    }

    let old_content = fs::read_to_string(path).await?;

    // Compare without the header (first 4 lines are comments)
    let old_lines: Vec<&str> = old_content.lines().skip(4).collect();
    let old_stripped = old_lines.join("\n");

    Ok(old_stripped.trim() != new_content.trim())
}

/// If a workflow uses `shell: node` with a JS file path in `run`,
/// copy that file to the output directory.
async fn copy_node_shell_files(
    json_str: &str,
    workflow_path: &Path,
    output_dir: &Path,
) -> Result<()> {
    let json_value: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return Ok(()),
    };

    let workflow_dir = workflow_path.parent().unwrap_or(Path::new("."));

    // Look for steps with shell: node and a run field pointing to a JS file
    if let Some(jobs) = json_value.get("jobs").and_then(|j| j.as_object()) {
        for (_job_id, job) in jobs {
            if let Some(steps) = job.get("steps").and_then(|s| s.as_array()) {
                for step in steps {
                    let shell = step.get("shell").and_then(|s| s.as_str()).unwrap_or("");
                    let run = step.get("run").and_then(|s| s.as_str()).unwrap_or("");

                    if shell.contains("node") && (run.ends_with(".js") || run.ends_with(".mjs")) {
                        let source_path = workflow_dir.join(run);
                        if source_path.exists() {
                            let dest_path = output_dir.join(run);
                            if let Some(parent) = dest_path.parent() {
                                fs::create_dir_all(parent).await?;
                            }
                            fs::copy(&source_path, &dest_path).await?;
                            println!(
                                "   {} Copied {} -> {}",
                                "📋".cyan(),
                                source_path.display(),
                                dest_path.display()
                            );
                        }
                    }
                }
            }
        }
    }

    // Also check composite action steps
    if let Some(runs) = json_value.get("runs") {
        if let Some(steps) = runs.get("steps").and_then(|s| s.as_array()) {
            for step in steps {
                let shell = step.get("shell").and_then(|s| s.as_str()).unwrap_or("");
                let run = step.get("run").and_then(|s| s.as_str()).unwrap_or("");

                if shell.contains("node") && (run.ends_with(".js") || run.ends_with(".mjs")) {
                    let source_path = workflow_dir.join(run);
                    if source_path.exists() {
                        let dest_path = output_dir.join(run);
                        if let Some(parent) = dest_path.parent() {
                            fs::create_dir_all(parent).await?;
                        }
                        fs::copy(&source_path, &dest_path).await?;
                        println!(
                            "   {} Copied {} -> {}",
                            "📋".cyan(),
                            source_path.display(),
                            dest_path.display()
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

fn timestamp_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // Simple ISO-like timestamp
    let secs_per_day = 86400;
    let days = now / secs_per_day;
    let remaining = now % secs_per_day;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;

    // Days since epoch to date (simplified)
    let mut y = 1970;
    let mut d = days as i64;
    loop {
        let days_in_year = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
            366
        } else {
            365
        };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        y += 1;
    }
    let is_leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let days_in_months = if is_leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m = 0;
    for (i, &dim) in days_in_months.iter().enumerate() {
        if d < dim {
            m = i + 1;
            break;
        }
        d -= dim;
    }

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y,
        m,
        d + 1,
        hours,
        minutes,
        seconds
    )
}

pub async fn ensure_workflows_dir() -> Result<PathBuf> {
    let dir = PathBuf::from(".github/workflows");
    fs::create_dir_all(&dir).await?;
    Ok(dir)
}
