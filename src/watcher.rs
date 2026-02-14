use std::path::Path;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use colored::Colorize;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::cache::Cache;
use crate::config::Config as GajiConfig;
use crate::generator::TypeGenerator;
use crate::parser;

const DEBOUNCE_DURATION: Duration = Duration::from_millis(300);

/// Build a gitignore matcher from the project root.
/// This respects .gitignore files and also adds gaji-specific patterns.
fn build_gitignore(root: &Path) -> Gitignore {
    let mut builder = GitignoreBuilder::new(root);

    // Load .gitignore if it exists
    let gitignore_path = root.join(".gitignore");
    if gitignore_path.exists() {
        let _ = builder.add(&gitignore_path);
    }

    // Add gaji-specific patterns that should always be ignored
    // These are fallbacks in case .gitignore doesn't include them
    let _ = builder.add_line(None, "node_modules/");
    let _ = builder.add_line(None, "generated/");
    let _ = builder.add_line(None, ".gaji-cache.json");

    builder.build().unwrap_or_else(|_| {
        // If building fails, create an empty gitignore
        GitignoreBuilder::new(root).build().unwrap()
    })
}

pub async fn watch_directory(dir: &Path) -> Result<()> {
    println!("{} Watching {} for changes...", "👀".green(), dir.display());
    println!("{}", "Press Ctrl+C to stop".dimmed());

    // Build gitignore matcher from project root (current working directory)
    let root = std::env::current_dir().unwrap_or_else(|_| dir.to_path_buf());
    let gitignore = Arc::new(build_gitignore(&root));

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

                if should_process_event(&event, &gitignore) {
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

fn should_process_event(event: &Event, gitignore: &Gitignore) -> bool {
    // Only process create and modify events
    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) => {}
        _ => return false,
    }

    // Only process .ts and .tsx files that are not ignored
    for path in &event.paths {
        if let Some(ext) = path.extension() {
            if ext == "ts" || ext == "tsx" {
                // Check if path or any of its parents is ignored by .gitignore
                let is_dir = path.is_dir();
                if !gitignore
                    .matched_path_or_any_parents(path, is_dir)
                    .is_ignore()
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
        let gaji_config = GajiConfig::load()?;
        let token = gaji_config.resolve_token();
        let api_url = gaji_config.resolve_api_url();
        let cache = Cache::load_or_create()?;
        let generator = TypeGenerator::with_cache_ttl(
            cache,
            std::path::PathBuf::from("generated"),
            token,
            api_url,
            gaji_config.build.cache_ttl_days,
        );

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

#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::{CreateKind, ModifyKind, RemoveKind};
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn make_event(kind: EventKind, paths: Vec<PathBuf>) -> Event {
        Event {
            kind,
            paths,
            attrs: Default::default(),
        }
    }

    fn make_test_gitignore(root: &Path, patterns: &[&str]) -> Gitignore {
        let mut builder = GitignoreBuilder::new(root);
        for pattern in patterns {
            let _ = builder.add_line(None, pattern);
        }
        builder.build().unwrap()
    }

    #[test]
    fn test_should_process_ts_create() {
        let dir = TempDir::new().unwrap();
        let gitignore = make_test_gitignore(dir.path(), &[]);
        let event = make_event(
            EventKind::Create(CreateKind::File),
            vec![dir.path().join("workflows/ci.ts")],
        );
        assert!(should_process_event(&event, &gitignore));
    }

    #[test]
    fn test_should_process_tsx_modify() {
        let dir = TempDir::new().unwrap();
        let gitignore = make_test_gitignore(dir.path(), &[]);
        let event = make_event(
            EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Content)),
            vec![dir.path().join("workflows/component.tsx")],
        );
        assert!(should_process_event(&event, &gitignore));
    }

    #[test]
    fn test_should_ignore_non_ts() {
        let dir = TempDir::new().unwrap();
        let gitignore = make_test_gitignore(dir.path(), &[]);
        let event = make_event(
            EventKind::Create(CreateKind::File),
            vec![dir.path().join("src/main.rs")],
        );
        assert!(!should_process_event(&event, &gitignore));

        let event_json = make_event(
            EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Content)),
            vec![dir.path().join("config.json")],
        );
        assert!(!should_process_event(&event_json, &gitignore));
    }

    #[test]
    fn test_should_ignore_node_modules() {
        let dir = TempDir::new().unwrap();
        let gitignore = make_test_gitignore(dir.path(), &["node_modules/"]);
        let event = make_event(
            EventKind::Create(CreateKind::File),
            vec![dir.path().join("node_modules/pkg/index.ts")],
        );
        assert!(!should_process_event(&event, &gitignore));
    }

    #[test]
    fn test_should_ignore_generated() {
        let dir = TempDir::new().unwrap();
        let gitignore = make_test_gitignore(dir.path(), &["generated/"]);
        let event = make_event(
            EventKind::Create(CreateKind::File),
            vec![dir.path().join("generated/types.ts")],
        );
        assert!(!should_process_event(&event, &gitignore));
    }

    #[test]
    fn test_should_ignore_delete_event() {
        let dir = TempDir::new().unwrap();
        let gitignore = make_test_gitignore(dir.path(), &[]);
        let event = make_event(
            EventKind::Remove(RemoveKind::File),
            vec![dir.path().join("workflows/ci.ts")],
        );
        assert!(!should_process_event(&event, &gitignore));
    }

    #[test]
    fn test_should_ignore_custom_gitignore_pattern() {
        let dir = TempDir::new().unwrap();
        let gitignore = make_test_gitignore(dir.path(), &["dist/", "*.generated.ts"]);

        // dist/ should be ignored
        let event_dist = make_event(
            EventKind::Create(CreateKind::File),
            vec![dir.path().join("dist/bundle.ts")],
        );
        assert!(!should_process_event(&event_dist, &gitignore));

        // *.generated.ts should be ignored
        let event_generated = make_event(
            EventKind::Create(CreateKind::File),
            vec![dir.path().join("types.generated.ts")],
        );
        assert!(!should_process_event(&event_generated, &gitignore));
    }

    #[test]
    fn test_build_gitignore_with_defaults() {
        let dir = TempDir::new().unwrap();
        let gitignore = build_gitignore(dir.path());

        // Default patterns should be ignored even without .gitignore file
        assert!(gitignore
            .matched_path_or_any_parents(dir.path().join("node_modules/pkg/index.ts"), false)
            .is_ignore());
        assert!(gitignore
            .matched_path_or_any_parents(dir.path().join("generated/types.ts"), false)
            .is_ignore());
        assert!(gitignore
            .matched_path_or_any_parents(dir.path().join(".gaji-cache.json"), false)
            .is_ignore());
    }
}
