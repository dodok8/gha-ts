use std::path::Path;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use anyhow::Result;
use colored::Colorize;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::cache::Cache;
use crate::generator::TypeGenerator;
use crate::parser;

const DEBOUNCE_DURATION: Duration = Duration::from_millis(300);

pub async fn watch_directory(dir: &Path) -> Result<()> {
    println!(
        "{} Watching {} for changes...",
        "👀".green(),
        dir.display()
    );
    println!("{}", "Press Ctrl+C to stop".dimmed());

    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(dir, RecursiveMode::Recursive)?;

    let mut last_event: Option<Instant> = None;

    for res in rx {
        match res {
            Ok(event) => {
                // Debounce
                if let Some(last) = last_event {
                    if last.elapsed() < DEBOUNCE_DURATION {
                        continue;
                    }
                }
                last_event = Some(Instant::now());

                if should_process_event(&event) {
                    if let Err(e) = handle_event(&event).await {
                        eprintln!("{} Error handling event: {}", "❌".red(), e);
                    }
                }
            }
            Err(e) => {
                eprintln!("{} Watch error: {}", "❌".red(), e);
            }
        }
    }

    Ok(())
}

fn should_process_event(event: &Event) -> bool {
    // Only process create and modify events
    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) => {}
        _ => return false,
    }

    // Only process .ts and .tsx files
    for path in &event.paths {
        if let Some(ext) = path.extension() {
            if ext == "ts" || ext == "tsx" {
                // Exclude generated files and node_modules
                let path_str = path.to_string_lossy();
                if !path_str.contains("node_modules")
                    && !path_str.contains("generated")
                    && !path_str.contains(".gaji-cache")
                {
                    return true;
                }
            }
        }
    }

    false
}

async fn handle_event(event: &Event) -> Result<()> {
    for path in &event.paths {
        println!(
            "{} {} changed",
            "📝".cyan(),
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.display().to_string())
        );

        // Analyze the file
        let action_refs = parser::analyze_file(path).await?;

        if action_refs.is_empty() {
            println!("{}", "   No action references found".dimmed());
            continue;
        }

        println!(
            "{} Found {} action reference(s)",
            "🔍".cyan(),
            action_refs.len()
        );

        // Generate types
        let cache = Cache::load_or_create()?;
        let generator = TypeGenerator::new(cache, std::path::PathBuf::from("generated"));

        let new_refs: std::collections::HashSet<String> = action_refs
            .into_iter()
            .filter(|r| !generator_has_type(r))
            .collect();

        if new_refs.is_empty() {
            println!("{}", "   All types already generated".dimmed());
            continue;
        }

        println!(
            "{} Generating types for {} new action(s)...",
            "⏳".yellow(),
            new_refs.len()
        );

        for action_ref in &new_refs {
            println!("   {} {}...", "⏳".yellow(), action_ref);
        }

        match generator.generate_types_for_refs(&new_refs).await {
            Ok(files) => {
                for file in files {
                    println!(
                        "   {} Generated {}",
                        "✅".green(),
                        file.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default()
                    );
                }
            }
            Err(e) => {
                eprintln!("   {} Failed to generate types: {}", "❌".red(), e);
            }
        }
    }

    Ok(())
}

fn generator_has_type(action_ref: &str) -> bool {
    let filename = crate::generator::action_ref_to_filename(action_ref);
    let path = std::path::Path::new("generated").join(&filename);
    path.exists()
}
