use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use gaji::builder::WorkflowBuilder;
use gaji::cache::Cache;
use gaji::cli::{Cli, Commands};
use gaji::config::Config;
use gaji::generator::TypeGenerator;
use gaji::parser;
use gaji::watcher;

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
        Commands::Dev { dir, watch } => {
            cmd_dev(&dir, watch).await?;
        }
        Commands::Build {
            input,
            output,
            dry_run,
        } => {
            cmd_build(&input, &output, dry_run).await?;
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
    println!("{} Initializing gaji project...\n", "🚀".green());

    // Create directories
    tokio::fs::create_dir_all("workflows").await?;
    tokio::fs::create_dir_all("generated").await?;
    tokio::fs::create_dir_all(".github/workflows").await?;

    println!("{} Created project directories", "✓".green());

    // Create config file
    let config = Config::default();
    config.save()?;
    println!("{} Created .gaji.toml", "✓".green());

    println!("\n{} Project initialized!\n", "✨".green());
    println!("Next steps:");
    println!("  1. Create workflow files in workflows/");
    println!("  2. Run: gaji dev");
    println!("  3. Run: gaji build");

    Ok(())
}

async fn cmd_dev(dir: &str, watch: bool) -> Result<()> {
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

    if watch {
        // Start watching
        watcher::watch_directory(&path).await?;
    } else {
        println!("{} Done. Run with --watch to keep watching.", "✓".green());
    }

    Ok(())
}

async fn cmd_build(input: &str, output: &str, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("{} Dry run: previewing workflows...\n", "🔨".cyan());
    } else {
        println!("{} Building workflows...\n", "🔨".cyan());
    }

    let builder = WorkflowBuilder::new(PathBuf::from(input), PathBuf::from(output), dry_run);

    let built = builder.build_all().await?;

    if built.is_empty() {
        println!("{} No workflows built", "⚠️".yellow());
    } else {
        println!("\n{} Built {} workflow(s)", "✅".green(), built.len());
    }

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
                println!("{} Generated {}", "✅".green(), file.display());
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
