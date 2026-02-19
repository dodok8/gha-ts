use std::io;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use colored::Colorize;

use gaji::builder::WorkflowBuilder;
use gaji::cache::Cache;
use gaji::cli::{Cli, Commands};
use gaji::config::Config;
use gaji::generator::TypeGenerator;
use gaji::init::{self, InitOptions};
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
        Commands::Dev { input, watch } => {
            cmd_dev(&input, watch).await?;
        }
        Commands::Build {
            input,
            output,
            dry_run,
        } => {
            cmd_build(&input, output.as_deref(), dry_run).await?;
        }
        Commands::Add { action } => {
            cmd_add(&action).await?;
        }
        Commands::Clean { cache } => {
            cmd_clean(cache).await?;
        }
        Commands::Completions { shell } => {
            cmd_completions(&shell)?;
        }
    }

    Ok(())
}

async fn cmd_init(
    force: bool,
    skip_examples: bool,
    migrate: bool,
    interactive: bool,
) -> Result<()> {
    let root = std::env::current_dir()?;
    let options = InitOptions {
        force,
        skip_examples,
        migrate,
        interactive,
    };
    init::init_project(&root, options).await
}

async fn cmd_dev(inputs: &[String], watch: bool) -> Result<()> {
    println!("{} Starting development mode...\n", "🚀".green());

    let config = Config::load()?;
    let token = config.resolve_token();
    let api_url = config.resolve_api_url();

    // Initial scan
    let mut all_refs = std::collections::HashSet::new();
    let paths: Vec<PathBuf> = if inputs.is_empty() {
        vec![PathBuf::from(&config.project.workflows_dir)]
    } else {
        inputs.iter().map(PathBuf::from).collect()
    };

    for path in &paths {
        if !path.exists() {
            continue;
        }
        if path.is_dir() {
            let results = parser::analyze_directory(path).await?;
            for refs in results.values() {
                all_refs.extend(refs.clone());
            }
        } else if path.is_file() {
            match parser::analyze_file(path).await {
                Ok(refs) => all_refs.extend(refs),
                Err(e) => eprintln!("Warning: Failed to parse {}: {}", path.display(), e),
            }
        }
    }

    if !all_refs.is_empty() {
        println!(
            "{} Found {} action reference(s), generating types...",
            "🔍".cyan(),
            all_refs.len()
        );

        let gen_start = Instant::now();
        let cache = Cache::load_or_create()?;
        let generator = TypeGenerator::with_cache_ttl(
            cache,
            PathBuf::from("generated"),
            token,
            api_url,
            config.build.cache_ttl_days,
        );
        generator.generate_types_for_refs(&all_refs).await?;

        println!(
            "{} Types generated in {:.2}s!\n",
            "✨".green(),
            gen_start.elapsed().as_secs_f64()
        );
    }

    if watch {
        watcher::watch_paths(&paths).await?;
    } else {
        println!("{} Done. Run with --watch to keep watching.", "✓".green());
    }

    Ok(())
}

async fn cmd_build(inputs: &[String], output: Option<&str>, dry_run: bool) -> Result<()> {
    let start = Instant::now();

    if dry_run {
        println!("{} Dry run: previewing workflows...\n", "🔨".cyan());
    } else {
        println!("{} Building workflows...\n", "🔨".cyan());
    }

    let config = Config::load()?;

    let input_paths: Vec<PathBuf> = if inputs.is_empty() {
        vec![PathBuf::from(&config.project.workflows_dir)]
    } else {
        inputs.iter().map(PathBuf::from).collect()
    };

    let output_dir = output.unwrap_or(&config.project.output_dir);
    let builder = WorkflowBuilder::new(input_paths, PathBuf::from(output_dir), dry_run);

    let built = builder.build_all().await?;

    let elapsed = start.elapsed();
    if built.is_empty() {
        println!("{} No workflows built", "⚠️".yellow());
    } else {
        println!(
            "\n{} Built {} workflow(s) in {:.2}s",
            "✅".green(),
            built.len(),
            elapsed.as_secs_f64()
        );
    }

    Ok(())
}

async fn cmd_add(action: &str) -> Result<()> {
    let start = Instant::now();
    println!("{} Adding action: {}\n", "📦".cyan(), action);

    let config = Config::load()?;
    let token = config.resolve_token();
    let api_url = config.resolve_api_url();

    let cache = Cache::load_or_create()?;
    let generator = TypeGenerator::with_cache_ttl(
        cache,
        PathBuf::from("generated"),
        token,
        api_url,
        config.build.cache_ttl_days,
    );

    let mut refs = std::collections::HashSet::new();
    refs.insert(action.to_string());

    match generator.generate_types_for_refs(&refs).await {
        Ok(files) => {
            for file in files {
                println!("{} Generated {}", "✅".green(), file.display());
            }
            println!(
                "\n{} Done in {:.2}s",
                "✨".green(),
                start.elapsed().as_secs_f64()
            );
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

fn cmd_completions(shell_str: &str) -> Result<()> {
    let shell = shell_str.parse::<Shell>().map_err(|_| {
        anyhow::anyhow!(
            "Invalid shell: '{}'. Supported shells: bash, zsh, fish, powershell, elvish",
            shell_str
        )
    })?;

    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "gaji", &mut io::stdout());

    Ok(())
}
