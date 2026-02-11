use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use gha_ts::cli::{Cli, Commands};
use gha_ts::builder::WorkflowBuilder;
use gha_ts::cache::Cache;
use gha_ts::config::Config;
use gha_ts::generator::TypeGenerator;
use gha_ts::parser;
use gha_ts::watcher;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            force,
            skip_examples,
            migrate,
            interactive,
        } => {
            cmd_init(force, skip_examples, migrate, interactive).await?;
        }
        Commands::Dev { dir } => {
            cmd_dev(&dir).await?;
        }
        Commands::Build { input, output } => {
            cmd_build(&input, &output).await?;
        }
        Commands::Watch { dir } => {
            cmd_watch(&dir).await?;
        }
        Commands::Add { action } => {
            cmd_add(&action).await?;
        }
        Commands::Clean { cache } => {
            cmd_clean(cache).await?;
        }
    }

    Ok(())
}

async fn cmd_init(
    _force: bool,
    _skip_examples: bool,
    _migrate: bool,
    _interactive: bool,
) -> Result<()> {
    println!("{} Initializing gha-ts project...\n", "🚀".green());

    // Create directories
    tokio::fs::create_dir_all("workflows").await?;
    tokio::fs::create_dir_all("generated").await?;
    tokio::fs::create_dir_all(".github/workflows").await?;

    println!("{} Created project directories", "✓".green());

    // Create config file
    let config = Config::default();
    config.save()?;
    println!("{} Created .gha-ts.toml", "✓".green());

    println!("\n{} Project initialized!\n", "✨".green());
    println!("Next steps:");
    println!("  1. Create workflow files in workflows/");
    println!("  2. Run: gha-ts dev");
    println!("  3. Run: gha-ts build");

    Ok(())
}

async fn cmd_dev(dir: &str) -> Result<()> {
    println!("{} Starting development mode...\n", "🚀".green());

    // Initial scan
    let path = PathBuf::from(dir);
    if path.exists() {
        let results = parser::analyze_directory(&path).await?;

        let mut all_refs = std::collections::HashSet::new();
        for refs in results.values() {
            all_refs.extend(refs.clone());
        }

        if !all_refs.is_empty() {
            println!(
                "{} Found {} action reference(s), generating types...",
                "🔍".cyan(),
                all_refs.len()
            );

            let cache = Cache::load_or_create()?;
            let generator = TypeGenerator::new(cache, PathBuf::from("generated"));
            generator.generate_types_for_refs(&all_refs).await?;

            println!("{} Types generated!\n", "✨".green());
        }
    }

    // Start watching
    watcher::watch_directory(&path).await?;

    Ok(())
}

async fn cmd_build(input: &str, output: &str) -> Result<()> {
    println!("{} Building workflows...\n", "🔨".cyan());

    let builder = WorkflowBuilder::new(PathBuf::from(input), PathBuf::from(output));

    let built = builder.build_all().await?;

    if built.is_empty() {
        println!("{} No workflows built", "⚠️".yellow());
    } else {
        println!(
            "\n{} Built {} workflow(s)",
            "✅".green(),
            built.len()
        );
    }

    Ok(())
}

async fn cmd_watch(dir: &str) -> Result<()> {
    let path = PathBuf::from(dir);
    watcher::watch_directory(&path).await?;
    Ok(())
}

async fn cmd_add(action: &str) -> Result<()> {
    println!("{} Adding action: {}\n", "📦".cyan(), action);

    let cache = Cache::load_or_create()?;
    let generator = TypeGenerator::new(cache, PathBuf::from("generated"));

    let mut refs = std::collections::HashSet::new();
    refs.insert(action.to_string());

    match generator.generate_types_for_refs(&refs).await {
        Ok(files) => {
            for file in files {
                println!(
                    "{} Generated {}",
                    "✅".green(),
                    file.display()
                );
            }
        }
        Err(e) => {
            eprintln!("{} Failed to generate types: {}", "❌".red(), e);
            return Err(e);
        }
    }

    Ok(())
}

async fn cmd_clean(clean_cache: bool) -> Result<()> {
    println!("{} Cleaning generated files...\n", "🧹".cyan());

    // Remove generated directory
    if PathBuf::from("generated").exists() {
        tokio::fs::remove_dir_all("generated").await?;
        println!("{} Removed generated/", "✓".green());
    }

    // Optionally clean cache
    if clean_cache {
        let cache = Cache::load_or_create()?;
        cache.clear()?;
        println!("{} Cleared cache", "✓".green());
    }

    println!("\n{} Clean complete!", "✨".green());

    Ok(())
}
