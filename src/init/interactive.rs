use std::path::Path;

use anyhow::Result;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect};

use super::migration;
use super::{detect_project_state, InitOptions, ProjectState};
use crate::config::{Config, ProjectConfig};

pub struct InteractiveConfig {
    pub workflows_dir: String,
    pub output_dir: String,
    pub generated_dir: String,
    pub create_example: bool,
    pub update_package_json: bool,
    pub update_tsconfig: bool,
}

pub async fn interactive_init(root: &Path) -> Result<()> {
    println!("{} gaji Interactive Setup\n", "🚀".green());

    let project_state = detect_project_state(root)?;

    let mut selected_workflows: Option<Vec<std::path::PathBuf>> = None;
    let mut selected_actions: Option<Vec<std::path::PathBuf>> = None;

    match &project_state {
        ProjectState::Empty => {
            println!("  Detected: Empty directory\n");
        }
        ProjectState::ExistingProject => {
            println!("  Detected: Existing project\n");
        }
        ProjectState::HasWorkflows => {
            println!("  Detected: Existing GitHub Actions workflows\n");

            let workflows = migration::discover_workflows(root)?;
            if !workflows.is_empty() {
                println!("Found {} workflow(s):", workflows.len());
                for wf in &workflows {
                    println!("  - {}", wf.display());
                }
                println!();

                let should_migrate = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Migrate existing workflows to TypeScript?")
                    .default(false)
                    .interact()?;

                if should_migrate {
                    if workflows.len() > 1 {
                        let names: Vec<String> = workflows
                            .iter()
                            .map(|p| {
                                p.file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string()
                            })
                            .collect();

                        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
                            .with_prompt("Select workflows to migrate")
                            .items(&names)
                            .defaults(&vec![true; names.len()])
                            .interact()?;

                        selected_workflows =
                            Some(selections.iter().map(|&i| workflows[i].clone()).collect());
                    } else {
                        selected_workflows = Some(workflows);
                    }
                }
            }

            let actions = migration::discover_actions(root)?;
            if !actions.is_empty() {
                println!("Found {} action(s):", actions.len());
                for action_path in &actions {
                    let action_id = action_path
                        .parent()
                        .and_then(|p| p.file_name())
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown");
                    println!("  - {}", action_id);
                }
                println!();

                let should_migrate_actions = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Migrate existing actions to TypeScript?")
                    .default(false)
                    .interact()?;

                if should_migrate_actions {
                    if actions.len() > 1 {
                        let names: Vec<String> = actions
                            .iter()
                            .map(|p| {
                                p.parent()
                                    .and_then(|d| d.file_name())
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string()
                            })
                            .collect();

                        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
                            .with_prompt("Select actions to migrate")
                            .items(&names)
                            .defaults(&vec![true; names.len()])
                            .interact()?;

                        selected_actions =
                            Some(selections.iter().map(|&i| actions[i].clone()).collect());
                    } else {
                        selected_actions = Some(actions);
                    }
                }
            }
        }
    }

    // Collect interactive configuration
    println!("{} Configuration Options\n", "⚙️".cyan());

    let workflows_dir: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("TypeScript workflows directory")
        .default("workflows".to_string())
        .interact_text()?;

    let output_dir: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("YAML output directory")
        .default(".github".to_string())
        .interact_text()?;

    let generated_dir: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Generated types directory")
        .default("generated".to_string())
        .interact_text()?;

    let create_example = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Create example workflow?")
        .default(true)
        .interact()?;

    let has_package_json = root.join("package.json").exists();
    let update_package_json = if has_package_json {
        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Update package.json with gaji scripts?")
            .default(true)
            .interact()?
    } else {
        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Create package.json with gaji scripts?")
            .default(false)
            .interact()?
    };

    let has_tsconfig = root.join("tsconfig.json").exists();
    let update_tsconfig = if has_tsconfig {
        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Update tsconfig.json for TypeScript workflows?")
            .default(false)
            .interact()?
    } else if update_package_json {
        // Only ask about creating tsconfig if we're also creating/updating package.json
        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Create tsconfig.json for TypeScript workflows?")
            .default(true)
            .interact()?
    } else {
        false
    };

    let interactive_config = InteractiveConfig {
        workflows_dir: workflows_dir.clone(),
        output_dir: output_dir.clone(),
        generated_dir: generated_dir.clone(),
        create_example,
        update_package_json,
        update_tsconfig,
    };

    println!();

    // Execute migration if requested
    if let Some(wfs) = &selected_workflows {
        migration::migrate_workflows(root, wfs).await?;
    }

    // Execute action migration if requested
    if let Some(acts) = &selected_actions {
        migration::migrate_actions(root, acts).await?;
    }

    // Create custom config with user-specified directories
    let config = Config {
        project: ProjectConfig {
            workflows_dir: interactive_config.workflows_dir.clone(),
            output_dir: interactive_config.output_dir.clone(),
            generated_dir: interactive_config.generated_dir.clone(),
        },
        ..Default::default()
    };

    // Save the custom config
    let config_path = root.join(".gaji.toml");
    config.save_to(&config_path)?;
    println!("{} Created .gaji.toml with custom directories", "✓".green());

    // Create directories based on custom config
    tokio::fs::create_dir_all(root.join(&interactive_config.workflows_dir)).await?;
    tokio::fs::create_dir_all(root.join(&interactive_config.generated_dir)).await?;
    tokio::fs::create_dir_all(root.join(&interactive_config.output_dir).join("workflows")).await?;
    println!("{} Created project directories", "✓".green());

    // Handle package.json if requested
    if interactive_config.update_package_json {
        if !root.join("package.json").exists() {
            // Create new package.json
            let package_json = serde_json::json!({
                "name": root.file_name().unwrap_or_default().to_string_lossy(),
                "version": "0.0.0",
                "private": true,
                "scripts": {
                    "gha:dev": "gaji dev",
                    "gha:build": "gaji build"
                },
                "devDependencies": {
                    "tsx": "^4.0.0",
                    "typescript": "^5.0.0"
                }
            });
            tokio::fs::write(
                root.join("package.json"),
                serde_json::to_string_pretty(&package_json)? + "\n",
            )
            .await?;
            println!("{} Created package.json", "✓".green());
        } else {
            // Update existing package.json
            super::update_package_json(root).await?;
        }
    }

    // Handle tsconfig.json if requested (separately from package.json)
    if interactive_config.update_tsconfig {
        super::handle_tsconfig(
            root,
            &InitOptions {
                force: false,
                skip_examples: false,
                migrate: false,
                interactive: true,
            },
        )
        .await?;
    }

    // Handle gitignore
    super::ensure_gitignore(root, project_state != ProjectState::Empty).await?;

    // Create example workflow if requested
    if interactive_config.create_example {
        let example_path = root.join(&interactive_config.workflows_dir).join("ci.ts");
        if !example_path.exists() {
            tokio::fs::write(&example_path, super::templates::EXAMPLE_WORKFLOW_TEMPLATE).await?;
            println!(
                "{} Created example workflow ({}/ci.ts)",
                "✓".green(),
                interactive_config.workflows_dir
            );
        } else {
            println!(
                "{} Example workflow already exists, skipping",
                "⏭️ ".dimmed()
            );
        }
    }

    // Try to generate types
    super::try_generate_initial_types(root).await;

    println!("\n{} Project initialized!\n", "✨".green());
    super::print_next_steps();

    Ok(())
}
