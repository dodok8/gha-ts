use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::fs;

use crate::config::Config as GajiConfig;
use crate::executor;

pub struct WorkflowBuilder {
    input_dir: PathBuf,
    output_dir: PathBuf,
    dry_run: bool,
    ignored_patterns: Vec<String>,
}

fn default_ignored_patterns() -> Vec<String> {
    vec![
        "node_modules".to_string(),
        ".git".to_string(),
        "generated".to_string(),
    ]
}

impl WorkflowBuilder {
    pub fn new(input_dir: PathBuf, output_dir: PathBuf, dry_run: bool) -> Self {
        let ignored_patterns = GajiConfig::load().map_or_else(
            |_| default_ignored_patterns(),
            |config| {
                if config.watch.ignored_patterns.is_empty() {
                    default_ignored_patterns()
                } else {
                    config.watch.ignored_patterns
                }
            },
        );

        Self {
            input_dir,
            output_dir,
            dry_run,
            ignored_patterns,
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

        let pb = ProgressBar::new(workflow_files.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("   {spinner:.green} [{bar:30.cyan/dim}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("━━─"),
        );

        for file in &workflow_files {
            let filename = file
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            pb.set_message(filename);
            match self.build_workflow(file).await {
                Ok(output_paths) => {
                    built_files.extend(output_paths);
                }
                Err(e) => {
                    pb.suspend(|| {
                        eprintln!("{} Failed to build {}: {}", "❌".red(), file.display(), e);
                    });
                }
            }
            pb.inc(1);
        }

        pb.finish_and_clear();

        Ok(built_files)
    }

    async fn find_workflow_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let mut entries = fs::read_dir(&self.input_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "ts" && !path.to_string_lossy().contains(".d.ts") {
                    // Check if path contains any ignored pattern
                    let path_str = path.to_string_lossy();
                    let is_ignored = self
                        .ignored_patterns
                        .iter()
                        .any(|pattern| path_str.contains(pattern));

                    if !is_ignored {
                        files.push(path);
                    }
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
                println!("--- {} ({}) ---", build_output.id, build_output.output_type);
                print!("{}", yaml_content);
                continue;
            }

            // Determine output directory based on type
            let out_dir = if build_output.output_type == "action" {
                let action_dir = self.output_dir.join("actions").join(&build_output.id);
                fs::create_dir_all(&action_dir).await?;
                action_dir
            } else {
                let workflows_dir = self.output_dir.join("workflows");
                fs::create_dir_all(&workflows_dir).await?;
                workflows_dir
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
                println!("   {} Wrote {}", "✅".green(), output_path.display());
            } else {
                println!("   {} {} (unchanged)", "⏭️".dimmed(), output_path.display());
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
            Err(anyhow::anyhow!("Failed to execute workflow:\n{}", stderr))
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
                Err(anyhow::anyhow!("Failed to execute workflow:\n{}", stderr))
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
    let value: serde_yaml::Value = serde_yaml::from_str(yaml).context("Invalid YAML syntax")?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // --- json_to_yaml tests ---

    #[test]
    fn test_json_to_yaml_simple() {
        let json = r#"{"name": "CI", "on": {"push": {}}, "jobs": {}}"#;
        let yaml = json_to_yaml(json).unwrap();
        assert!(yaml.contains("name: CI"));
        assert!(yaml.contains("on:"));
        assert!(yaml.contains("jobs:"));
    }

    #[test]
    fn test_json_to_yaml_nested() {
        let json = r#"{"name": "CI", "on": {"push": {"branches": ["main"]}}, "jobs": {"build": {"runs-on": "ubuntu-latest", "steps": [{"name": "Test", "run": "echo hello"}]}}}"#;
        let yaml = json_to_yaml(json).unwrap();
        assert!(yaml.contains("branches:"));
        assert!(yaml.contains("runs-on: ubuntu-latest"));
    }

    #[test]
    fn test_json_to_yaml_invalid_json() {
        let result = json_to_yaml("not valid json");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid JSON output"));
    }

    // --- validate_workflow_yaml tests ---

    #[test]
    fn test_validate_workflow_yaml_valid() {
        let yaml = "name: CI\non:\n  push: {}\njobs:\n  build:\n    runs-on: ubuntu-latest\n";
        assert!(validate_workflow_yaml(yaml).is_ok());
    }

    #[test]
    fn test_validate_workflow_yaml_missing_on() {
        let yaml = "name: CI\njobs:\n  build:\n    runs-on: ubuntu-latest\n";
        let result = validate_workflow_yaml(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("'on'"));
    }

    #[test]
    fn test_validate_workflow_yaml_missing_jobs() {
        let yaml = "name: CI\non:\n  push: {}\n";
        let result = validate_workflow_yaml(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("'jobs'"));
    }

    #[test]
    fn test_validate_workflow_yaml_not_mapping() {
        let yaml = "- item1\n- item2\n";
        let result = validate_workflow_yaml(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("mapping"));
    }

    #[test]
    fn test_validate_workflow_yaml_invalid_syntax() {
        let yaml = ":\n  :\n    : [[[";
        let result = validate_workflow_yaml(yaml);
        assert!(result.is_err());
    }

    // --- timestamp_now tests ---

    #[test]
    fn test_timestamp_now_format() {
        let ts = timestamp_now();
        // Should match ISO 8601 format: YYYY-MM-DDTHH:MM:SSZ
        assert!(
            regex_lite(ts.as_str()),
            "Timestamp '{}' does not match ISO 8601 format",
            ts
        );
        assert!(ts.ends_with('Z'));
        assert_eq!(ts.len(), 20);
    }

    /// Simple ISO 8601 format check without regex dependency
    fn regex_lite(s: &str) -> bool {
        let bytes = s.as_bytes();
        if bytes.len() != 20 {
            return false;
        }
        bytes[4] == b'-'
            && bytes[7] == b'-'
            && bytes[10] == b'T'
            && bytes[13] == b':'
            && bytes[16] == b':'
            && bytes[19] == b'Z'
            && bytes[0..4].iter().all(|b| b.is_ascii_digit())
            && bytes[5..7].iter().all(|b| b.is_ascii_digit())
            && bytes[8..10].iter().all(|b| b.is_ascii_digit())
            && bytes[11..13].iter().all(|b| b.is_ascii_digit())
            && bytes[14..16].iter().all(|b| b.is_ascii_digit())
            && bytes[17..19].iter().all(|b| b.is_ascii_digit())
    }

    // --- should_write_file tests ---

    #[tokio::test]
    async fn test_should_write_file_new_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent.yml");
        assert!(should_write_file(&path, "content").await.unwrap());
    }

    #[tokio::test]
    async fn test_should_write_file_unchanged() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.yml");
        // Write a file with 4 header lines + content
        let content = "# line1\n# line2\n# line3\n\nname: CI\non:\n  push: {}\n";
        tokio::fs::write(&path, content).await.unwrap();

        // The same content (without header) should return false
        let result = should_write_file(&path, "name: CI\non:\n  push: {}")
            .await
            .unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_should_write_file_changed() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.yml");
        let content = "# line1\n# line2\n# line3\n\nname: CI\non:\n  push: {}\n";
        tokio::fs::write(&path, content).await.unwrap();

        // Different content should return true
        let result = should_write_file(&path, "name: Updated\non:\n  push: {}\njobs: {}")
            .await
            .unwrap();
        assert!(result);
    }

    // --- find_workflow_files tests ---

    #[tokio::test]
    async fn test_find_workflow_files_filters_dts() {
        let dir = TempDir::new().unwrap();

        // Create various files
        tokio::fs::write(dir.path().join("ci.ts"), "// workflow")
            .await
            .unwrap();
        tokio::fs::write(dir.path().join("release.ts"), "// workflow")
            .await
            .unwrap();
        tokio::fs::write(dir.path().join("types.d.ts"), "// declarations")
            .await
            .unwrap();
        tokio::fs::write(dir.path().join("readme.md"), "# readme")
            .await
            .unwrap();
        tokio::fs::write(dir.path().join("config.json"), "{}")
            .await
            .unwrap();

        let builder =
            WorkflowBuilder::new(dir.path().to_path_buf(), dir.path().join("output"), false);
        let files = builder.find_workflow_files().await.unwrap();

        assert_eq!(files.len(), 2);
        let filenames: Vec<String> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(filenames.contains(&"ci.ts".to_string()));
        assert!(filenames.contains(&"release.ts".to_string()));
        assert!(!filenames.iter().any(|f| f.contains(".d.ts")));
    }

    #[tokio::test]
    async fn test_find_workflow_files_respects_ignored_patterns() {
        let dir = TempDir::new().unwrap();

        // Create a workflow file (normal)
        tokio::fs::write(dir.path().join("ci.ts"), "// workflow")
            .await
            .unwrap();

        // Create a workflow file with "node_modules" in the path name (simulating ignored pattern)
        // Note: ignored_patterns uses simple string contains matching
        tokio::fs::write(
            dir.path().join("node_modules_test.ts"),
            "// should be ignored",
        )
        .await
        .unwrap();

        let mut builder =
            WorkflowBuilder::new(dir.path().to_path_buf(), dir.path().join("output"), false);

        // Override ignored_patterns for testing
        builder.ignored_patterns = vec!["node_modules".to_string()];

        let files = builder.find_workflow_files().await.unwrap();

        // Should only include ci.ts, not node_modules_test.ts
        assert_eq!(files.len(), 1);
        let filenames: Vec<String> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(filenames.contains(&"ci.ts".to_string()));
        assert!(!filenames.contains(&"node_modules_test.ts".to_string()));
    }

    // --- build_all tests ---

    #[tokio::test]
    async fn test_build_all_empty_dir() {
        let dir = TempDir::new().unwrap();
        let builder =
            WorkflowBuilder::new(dir.path().to_path_buf(), dir.path().join("output"), false);
        let result = builder.build_all().await.unwrap();
        assert!(result.is_empty());
    }

    // --- copy_node_shell_files tests ---

    #[tokio::test]
    async fn test_copy_node_shell_files_no_node_steps() {
        let dir = TempDir::new().unwrap();
        let workflow_path = dir.path().join("test.ts");
        tokio::fs::write(&workflow_path, "").await.unwrap();

        let json = r#"{"jobs":{"build":{"steps":[{"name":"Test","run":"echo hello"}]}}}"#;
        let result = copy_node_shell_files(json, &workflow_path, dir.path()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_copy_node_shell_files_invalid_json() {
        let dir = TempDir::new().unwrap();
        let workflow_path = dir.path().join("test.ts");
        tokio::fs::write(&workflow_path, "").await.unwrap();

        let result = copy_node_shell_files("not json", &workflow_path, dir.path()).await;
        assert!(result.is_ok()); // Should silently succeed on invalid JSON
    }
}
