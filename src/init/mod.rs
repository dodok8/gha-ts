pub mod interactive;
pub mod migration;
pub mod templates;

use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::Path;

use anyhow::{Context, Result};
use colored::Colorize;
use tokio::fs;

use crate::cache::Cache;
use crate::config::Config;
use crate::generator::TypeGenerator;
use crate::parser;

/// CLI options passed from clap.
pub struct InitOptions {
    pub force: bool,
    pub skip_examples: bool,
    pub migrate: bool,
    pub interactive: bool,
}

/// Detected state of the target project directory.
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectState {
    /// No recognizable project files.
    Empty,
    /// Has project files (package.json, Cargo.toml, .git, etc.) but no YAML workflows.
    ExistingProject,
    /// Has .github/workflows/*.yml or *.yaml files.
    HasWorkflows,
}

/// Detect the current project state based on filesystem contents.
pub fn detect_project_state(root: &Path) -> Result<ProjectState> {
    let has_workflows = root
        .join(".github/workflows")
        .read_dir()
        .ok()
        .map(|entries| {
            entries.filter_map(|e| e.ok()).any(|e| {
                let ext = e.path().extension().map(|s| s.to_owned());
                ext.as_deref() == Some(OsStr::new("yml"))
                    || ext.as_deref() == Some(OsStr::new("yaml"))
            })
        })
        .unwrap_or(false);

    let has_actions = root
        .join(".github/actions")
        .read_dir()
        .ok()
        .map(|entries| {
            entries.filter_map(|e| e.ok()).any(|e| {
                e.path().is_dir()
                    && (e.path().join("action.yml").exists()
                        || e.path().join("action.yaml").exists())
            })
        })
        .unwrap_or(false);

    if has_workflows || has_actions {
        return Ok(ProjectState::HasWorkflows);
    }

    // Check for common project markers (language-agnostic)
    let project_markers = [
        "package.json",
        "Cargo.toml",
        "go.mod",
        "pyproject.toml",
        "Makefile",
        "CMakeLists.txt",
        "pom.xml",
        "build.gradle",
        ".git",
    ];

    let has_project_files = project_markers
        .iter()
        .any(|marker| root.join(marker).exists());

    if has_project_files {
        Ok(ProjectState::ExistingProject)
    } else {
        Ok(ProjectState::Empty)
    }
}

/// Main entry point for the init command.
pub async fn init_project(root: &Path, options: InitOptions) -> Result<()> {
    println!("{} Initializing gaji project...\n", "🚀".green());

    if options.interactive {
        return interactive::interactive_init(root).await;
    }

    let project_state = detect_project_state(root)?;

    match project_state {
        ProjectState::Empty => init_new_project(root, &options).await?,
        ProjectState::ExistingProject => init_existing_project(root, &options).await?,
        ProjectState::HasWorkflows => init_with_workflows(root, &options).await?,
    }

    Ok(())
}

/// Empty directory: create gaji project structure.
pub(crate) async fn init_new_project(root: &Path, options: &InitOptions) -> Result<()> {
    println!("{} Creating new project structure...\n", "📦".cyan());

    create_directories(root).await?;
    ensure_gitignore(root, false).await?;
    create_config(root)?;

    if !options.skip_examples {
        create_example_workflow(root).await?;
        try_generate_initial_types(root).await;
    }

    println!("\n{} Project initialized!\n", "✨".green());
    print_next_steps();

    Ok(())
}

/// Existing project: add gaji on top without disturbing existing setup.
pub(crate) async fn init_existing_project(root: &Path, options: &InitOptions) -> Result<()> {
    println!("{} Adding gaji to existing project...\n", "📦".cyan());

    create_directories(root).await?;

    // Only touch package.json / tsconfig.json if the project already has them
    if root.join("package.json").exists() {
        update_package_json(root).await?;
        handle_tsconfig(root, options).await?;
    }

    ensure_gitignore(root, true).await?;
    create_config(root)?;

    if !options.skip_examples {
        let example_path = root.join("workflows/ci.ts");
        if !example_path.exists() {
            create_example_workflow(root).await?;
        } else {
            println!(
                "{} workflows/ci.ts already exists, skipping example",
                "⏭️ ".dimmed()
            );
        }
    }

    try_generate_initial_types(root).await;

    println!("\n{} gaji added to your project!\n", "✨".green());
    print_next_steps();

    Ok(())
}

/// Has .github/workflows/*.yml or .github/actions/: offer migration, then proceed like existing project.
async fn init_with_workflows(root: &Path, options: &InitOptions) -> Result<()> {
    println!(
        "{} Adding gaji to project with existing workflows...\n",
        "📦".cyan()
    );

    let existing_workflows = migration::discover_workflows(root)?;
    if !existing_workflows.is_empty() {
        println!("Found {} existing workflow(s):", existing_workflows.len());
        for workflow in &existing_workflows {
            println!("  - {}", workflow.display());
        }
        println!();
    }

    let existing_actions = migration::discover_actions(root)?;
    if !existing_actions.is_empty() {
        println!("Found {} existing action(s):", existing_actions.len());
        for action in &existing_actions {
            let action_id = action
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            println!("  - {}", action_id);
        }
        println!();
    }

    if options.migrate {
        if !existing_workflows.is_empty() {
            migration::migrate_workflows(root, &existing_workflows).await?;
        }
        if !existing_actions.is_empty() {
            migration::migrate_actions(root, &existing_actions).await?;
        }
    } else if !existing_workflows.is_empty() || !existing_actions.is_empty() {
        println!(
            "{} Tip: Run with --migrate to convert existing YAML workflows and actions to TypeScript",
            "💡".yellow()
        );
        println!("   gaji init --migrate\n");
    }

    init_existing_project(root, options).await?;

    Ok(())
}

// -- Shared helpers --

async fn create_directories(root: &Path) -> Result<()> {
    fs::create_dir_all(root.join("workflows")).await?;
    fs::create_dir_all(root.join("generated")).await?;
    fs::create_dir_all(root.join(".github/workflows")).await?;
    println!("{} Created project directories", "✓".green());
    Ok(())
}

pub(crate) async fn update_package_json(root: &Path) -> Result<()> {
    let path = root.join("package.json");
    let content = fs::read_to_string(&path)
        .await
        .context("Failed to read package.json")?;
    let mut package: serde_json::Value =
        serde_json::from_str(&content).context("Failed to parse package.json")?;

    // Ensure "scripts" object exists
    if package.get("scripts").is_none() {
        package["scripts"] = serde_json::json!({});
    }

    // Merge scripts (do NOT overwrite existing keys)
    if let Some(scripts) = package["scripts"].as_object_mut() {
        for &(key, value) in templates::GAJI_SCRIPTS {
            scripts
                .entry(key.to_string())
                .or_insert(serde_json::Value::String(value.to_string()));
        }
    }

    // Ensure "devDependencies" object exists
    if package.get("devDependencies").is_none() {
        package["devDependencies"] = serde_json::json!({});
    }

    // Merge devDependencies (do NOT overwrite existing versions)
    if let Some(dev_deps) = package["devDependencies"].as_object_mut() {
        for &(key, value) in templates::GAJI_DEV_DEPS {
            dev_deps
                .entry(key.to_string())
                .or_insert(serde_json::Value::String(value.to_string()));
        }
    }

    let formatted = serde_json::to_string_pretty(&package)?;
    fs::write(&path, formatted + "\n").await?;
    println!("{} Updated package.json", "✓".green());

    Ok(())
}

pub(crate) async fn handle_tsconfig(root: &Path, options: &InitOptions) -> Result<()> {
    let path = root.join("tsconfig.json");

    if path.exists() {
        if options.force {
            let backup_path = root.join("tsconfig.json.backup");
            fs::copy(&path, &backup_path).await?;
            println!(
                "{} Backed up tsconfig.json to tsconfig.json.backup",
                "✓".green()
            );

            fs::write(&path, templates::TSCONFIG_TEMPLATE).await?;
            println!("{} Created tsconfig.json", "✓".green());
        } else {
            println!(
                "{} tsconfig.json already exists (use --force to overwrite)",
                "⚠️ ".yellow()
            );
            println!("   Consider adding to your compilerOptions:");
            println!("   \"include\": [\"workflows/**/*\"]");
        }
    } else {
        fs::write(&path, templates::TSCONFIG_TEMPLATE).await?;
        println!("{} Created tsconfig.json", "✓".green());
    }

    Ok(())
}

pub(crate) async fn ensure_gitignore(root: &Path, may_exist: bool) -> Result<()> {
    let path = root.join(".gitignore");

    if may_exist && path.exists() {
        let content = fs::read_to_string(&path).await?;

        if content.contains("# gaji generated files") {
            println!("{} .gitignore already has gaji entries", "⏭️ ".dimmed());
            return Ok(());
        }

        let mut new_content = content;
        if !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push_str(templates::GITIGNORE_SECTION);
        fs::write(&path, new_content).await?;
        println!("{} Updated .gitignore", "✓".green());
    } else {
        // Create new - trim leading newline from section
        fs::write(&path, templates::GITIGNORE_SECTION.trim_start()).await?;
        println!("{} Created .gitignore", "✓".green());
    }

    Ok(())
}

async fn create_example_workflow(root: &Path) -> Result<()> {
    let path = root.join("workflows/ci.ts");
    fs::write(&path, templates::EXAMPLE_WORKFLOW_TEMPLATE).await?;
    println!("{} Created example workflow (workflows/ci.ts)", "✓".green());
    Ok(())
}

fn create_config(root: &Path) -> Result<()> {
    let config_path = root.join(".gaji.toml");
    if config_path.exists() {
        println!("{} .gaji.toml already exists", "⏭️ ".dimmed());
        return Ok(());
    }
    let config = Config::default();
    config.save_to(&config_path)?;
    println!("{} Created .gaji.toml", "✓".green());
    Ok(())
}

pub(crate) async fn try_generate_initial_types(root: &Path) {
    match generate_initial_types(root).await {
        Ok(()) => {}
        Err(e) => {
            eprintln!("{} Could not generate initial types: {}", "⚠️ ".yellow(), e);
            eprintln!("   Run 'gaji dev' later to generate types.");
        }
    }
}

async fn generate_initial_types(root: &Path) -> Result<()> {
    let workflows_path = root.join("workflows");
    if !workflows_path.exists() {
        return Ok(());
    }

    println!("\n{} Analyzing workflow files...", "🔍".cyan());

    let results = parser::analyze_directory(&workflows_path).await?;

    let mut all_refs = HashSet::new();
    for refs in results.values() {
        all_refs.extend(refs.clone());
    }

    if all_refs.is_empty() {
        return Ok(());
    }

    println!(
        "{} Generating types for {} action(s)...",
        "📦".cyan(),
        all_refs.len()
    );

    let config = Config::load()?;
    let token = config.resolve_token();
    let api_url = config.resolve_api_url();
    let cache = Cache::load_or_create()?;
    let generator = TypeGenerator::new(cache, root.join("generated"), token, api_url);
    generator.generate_types_for_refs(&all_refs).await?;

    println!("{} Types generated!", "✨".green());

    Ok(())
}

pub(crate) fn print_next_steps() {
    println!("Next steps:");
    println!("  1. Edit workflows/*.ts");
    println!("  2. Run: gaji dev");
    println!("  3. Run: gaji build");
    println!();
    println!(
        "{} For private repos or GitHub Enterprise, create .gaji.local.toml:",
        "💡".yellow()
    );
    println!("   [github]");
    println!("   token = \"ghp_your_token_here\"");
    println!("   # api_url = \"https://github.example.com\"  # for GitHub Enterprise");
    println!();
    println!("Learn more: https://github.com/dodok8/gaji");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_empty_project() {
        let temp = tempfile::TempDir::new().unwrap();
        let state = detect_project_state(temp.path()).unwrap();
        assert_eq!(state, ProjectState::Empty);
    }

    #[test]
    fn test_detect_existing_project_with_package_json() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::write(temp.path().join("package.json"), "{}").unwrap();
        let state = detect_project_state(temp.path()).unwrap();
        assert_eq!(state, ProjectState::ExistingProject);
    }

    #[test]
    fn test_detect_existing_project_with_cargo_toml() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::write(temp.path().join("Cargo.toml"), "[package]").unwrap();
        let state = detect_project_state(temp.path()).unwrap();
        assert_eq!(state, ProjectState::ExistingProject);
    }

    #[test]
    fn test_detect_existing_project_with_go_mod() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::write(temp.path().join("go.mod"), "module example").unwrap();
        let state = detect_project_state(temp.path()).unwrap();
        assert_eq!(state, ProjectState::ExistingProject);
    }

    #[test]
    fn test_detect_existing_project_with_git() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir(temp.path().join(".git")).unwrap();
        let state = detect_project_state(temp.path()).unwrap();
        assert_eq!(state, ProjectState::ExistingProject);
    }

    #[test]
    fn test_detect_has_workflows() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join(".github/workflows")).unwrap();
        std::fs::write(temp.path().join(".github/workflows/ci.yml"), "name: CI").unwrap();
        let state = detect_project_state(temp.path()).unwrap();
        assert_eq!(state, ProjectState::HasWorkflows);
    }

    #[test]
    fn test_detect_has_workflows_takes_precedence() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::write(temp.path().join("Cargo.toml"), "[package]").unwrap();
        std::fs::create_dir_all(temp.path().join(".github/workflows")).unwrap();
        std::fs::write(temp.path().join(".github/workflows/ci.yml"), "name: CI").unwrap();
        let state = detect_project_state(temp.path()).unwrap();
        assert_eq!(state, ProjectState::HasWorkflows);
    }

    #[tokio::test]
    async fn test_create_directories() {
        let temp = tempfile::TempDir::new().unwrap();
        create_directories(temp.path()).await.unwrap();
        assert!(temp.path().join("workflows").is_dir());
        assert!(temp.path().join("generated").is_dir());
        assert!(temp.path().join(".github/workflows").is_dir());
    }

    #[tokio::test]
    async fn test_update_package_json_preserves_existing() {
        let temp = tempfile::TempDir::new().unwrap();
        let existing = serde_json::json!({
            "name": "my-app",
            "scripts": {
                "test": "jest",
                "gha:dev": "custom-command"
            }
        });
        std::fs::write(
            temp.path().join("package.json"),
            serde_json::to_string_pretty(&existing).unwrap(),
        )
        .unwrap();

        update_package_json(temp.path()).await.unwrap();

        let content = std::fs::read_to_string(temp.path().join("package.json")).unwrap();
        let pkg: serde_json::Value = serde_json::from_str(&content).unwrap();

        // Existing script should NOT be overwritten
        assert_eq!(pkg["scripts"]["gha:dev"], "custom-command");
        // New scripts should be added
        assert_eq!(pkg["scripts"]["gha:build"], "gaji build");
        // Existing scripts should be preserved
        assert_eq!(pkg["scripts"]["test"], "jest");
        // devDependencies should be added
        assert_eq!(pkg["devDependencies"]["tsx"], "^4.0.0");
    }

    #[tokio::test]
    async fn test_ensure_gitignore_create_new() {
        let temp = tempfile::TempDir::new().unwrap();
        ensure_gitignore(temp.path(), false).await.unwrap();
        let content = std::fs::read_to_string(temp.path().join(".gitignore")).unwrap();
        assert!(content.contains("# gaji generated files"));
        assert!(content.contains("generated/"));
        assert!(content.contains(".gaji-cache.json"));
    }

    #[tokio::test]
    async fn test_ensure_gitignore_append() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::write(temp.path().join(".gitignore"), "node_modules/\n").unwrap();

        ensure_gitignore(temp.path(), true).await.unwrap();

        let content = std::fs::read_to_string(temp.path().join(".gitignore")).unwrap();
        assert!(content.starts_with("node_modules/"));
        assert!(content.contains("# gaji generated files"));
    }

    #[tokio::test]
    async fn test_ensure_gitignore_idempotent() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::write(
            temp.path().join(".gitignore"),
            "node_modules/\n# gaji generated files\ngenerated/\n",
        )
        .unwrap();

        ensure_gitignore(temp.path(), true).await.unwrap();

        let content = std::fs::read_to_string(temp.path().join(".gitignore")).unwrap();
        assert_eq!(content.matches("# gaji generated files").count(), 1);
    }

    #[tokio::test]
    async fn test_create_example_workflow() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join("workflows")).unwrap();
        create_example_workflow(temp.path()).await.unwrap();
        let content = std::fs::read_to_string(temp.path().join("workflows/ci.ts")).unwrap();
        assert!(content.contains("getAction"));
        assert!(content.contains("actions/checkout@v5"));
        assert!(content.contains("workflow.build"));
    }

    #[tokio::test]
    async fn test_init_new_project_is_minimal() {
        let temp = tempfile::TempDir::new().unwrap();
        let options = InitOptions {
            force: false,
            skip_examples: true,
            migrate: false,
            interactive: false,
        };

        init_new_project(temp.path(), &options).await.unwrap();

        // Core gaji files should exist
        assert!(temp.path().join("workflows").is_dir());
        assert!(temp.path().join("generated").is_dir());
        assert!(temp.path().join(".github/workflows").is_dir());
        assert!(temp.path().join(".gaji.toml").is_file());
        assert!(temp.path().join(".gitignore").is_file());
        // Should NOT create Node.js-specific files
        assert!(!temp.path().join("package.json").exists());
        assert!(!temp.path().join("tsconfig.json").exists());
    }

    #[tokio::test]
    async fn test_init_existing_project_with_package_json() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("package.json"),
            r#"{"name": "my-app", "scripts": {}}"#,
        )
        .unwrap();

        let options = InitOptions {
            force: false,
            skip_examples: true,
            migrate: false,
            interactive: false,
        };

        init_existing_project(temp.path(), &options).await.unwrap();

        // package.json should be updated with gaji scripts
        let content = std::fs::read_to_string(temp.path().join("package.json")).unwrap();
        let pkg: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(pkg["scripts"]["gha:build"], "gaji build");
        // tsconfig.json should be created alongside package.json
        assert!(temp.path().join("tsconfig.json").is_file());
    }

    #[tokio::test]
    async fn test_init_existing_project_without_package_json() {
        let temp = tempfile::TempDir::new().unwrap();
        // A Rust project — no package.json
        std::fs::write(temp.path().join("Cargo.toml"), "[package]").unwrap();

        let options = InitOptions {
            force: false,
            skip_examples: true,
            migrate: false,
            interactive: false,
        };

        init_existing_project(temp.path(), &options).await.unwrap();

        // Should NOT create Node.js-specific files
        assert!(!temp.path().join("package.json").exists());
        assert!(!temp.path().join("tsconfig.json").exists());
        // Gaji files should exist
        assert!(temp.path().join(".gaji.toml").is_file());
        assert!(temp.path().join("workflows").is_dir());
    }

    #[test]
    fn test_create_config() {
        let temp = tempfile::TempDir::new().unwrap();
        create_config(temp.path()).unwrap();
        assert!(temp.path().join(".gaji.toml").is_file());
        let content = std::fs::read_to_string(temp.path().join(".gaji.toml")).unwrap();
        assert!(content.contains("workflows_dir"));
    }
}
