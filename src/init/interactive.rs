use std::path::Path;

use anyhow::Result;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect};

use super::migration;
use super::{
    detect_project_state, init_existing_project, init_new_project, InitOptions, ProjectState,
};

pub async fn interactive_init(root: &Path) -> Result<()> {
    println!("{} gaji Interactive Setup\n", "🚀".green());

    let project_state = detect_project_state(root)?;

    let mut selected_workflows: Option<Vec<std::path::PathBuf>> = None;

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
        }
    }

    let create_example = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Create example workflow (workflows/ci.ts)?")
        .default(true)
        .interact()?;

    // Execute migration if requested
    if let Some(wfs) = &selected_workflows {
        migration::migrate_workflows(root, wfs).await?;
    }

    // Proceed with init
    let options = InitOptions {
        force: false,
        skip_examples: !create_example,
        migrate: false,
        interactive: false,
    };

    match project_state {
        ProjectState::Empty => init_new_project(root, &options).await?,
        ProjectState::ExistingProject | ProjectState::HasWorkflows => {
            init_existing_project(root, &options).await?
        }
    }

    Ok(())
}
