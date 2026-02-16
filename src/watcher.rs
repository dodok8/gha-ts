use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use anyhow::Result;
use colored::Colorize;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::cache::Cache;
use crate::config::Config as GajiConfig;
use crate::generator::TypeGenerator;
use crate::parser;

const DEBOUNCE_DURATION: Duration = Duration::from_millis(300);

pub async fn watch_paths(paths: &[PathBuf]) -> Result<()> {
    let display: Vec<String> = paths.iter().map(|p| p.display().to_string()).collect();
    println!(
        "{} Watching {} for changes...",
        "👀".green(),
        display.join(", ")
    );
    println!("{}", "Press Ctrl+C to stop".dimmed());

    let gaji_config = GajiConfig::load()?;
    let ignored_patterns = gaji_config.watch.ignored_patterns.clone();

    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    // Track individually watched files for event filtering
    let mut watched_files: HashSet<PathBuf> = HashSet::new();
    let mut watched_parents: HashSet<PathBuf> = HashSet::new();

    for path in paths {
        if path.is_dir() {
            watcher.watch(path, RecursiveMode::Recursive)?;
        } else if path.is_file() {
            let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());
            watched_files.insert(canonical);
            if let Some(parent) = path.parent() {
                let parent_buf = parent.to_path_buf();
                // Only watch parent if not already covered by a directory watch
                if !paths
                    .iter()
                    .any(|p| p.is_dir() && parent_buf.starts_with(p))
                    && watched_parents.insert(parent_buf.clone())
                {
                    watcher.watch(&parent_buf, RecursiveMode::NonRecursive)?;
                }
            }
        }
    }

    let has_file_filter = !watched_files.is_empty();
    let mut last_event: Option<Instant> = None;

    for res in rx {
        match res {
            Ok(event) => {
                if let Some(last) = last_event {
                    if last.elapsed() < DEBOUNCE_DURATION {
                        continue;
                    }
                }
                last_event = Some(Instant::now());

                if should_process_event(&event, &ignored_patterns, has_file_filter, &watched_files)
                {
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

fn should_process_event(
    event: &Event,
    ignored_patterns: &[String],
    has_file_filter: bool,
    watched_files: &HashSet<PathBuf>,
) -> bool {
    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) => {}
        _ => return false,
    }

    for path in &event.paths {
        if let Some(ext) = path.extension() {
            if ext == "ts" || ext == "tsx" {
                let path_str = path.to_string_lossy();
                let is_ignored = ignored_patterns
                    .iter()
                    .any(|pattern| path_str.contains(pattern));

                if is_ignored {
                    continue;
                }

                // If we have specific file filters, check that this file
                // either matches a watched file or comes from a directory watch
                if has_file_filter {
                    let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());
                    if watched_files.contains(&canonical) {
                        return true;
                    }
                    // If the file is not in watched_files, it might still come
                    // from a recursively watched directory — allow it through
                    // (the watcher only sends events for watched paths)
                }

                return true;
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

    fn make_event(kind: EventKind, paths: Vec<PathBuf>) -> Event {
        Event {
            kind,
            paths,
            attrs: Default::default(),
        }
    }

    #[test]
    fn test_should_process_ts_create() {
        let ignored = vec![];
        let no_files = HashSet::new();
        let event = make_event(
            EventKind::Create(CreateKind::File),
            vec![PathBuf::from("/project/workflows/ci.ts")],
        );
        assert!(should_process_event(&event, &ignored, false, &no_files));
    }

    #[test]
    fn test_should_process_tsx_modify() {
        let ignored = vec![];
        let no_files = HashSet::new();
        let event = make_event(
            EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Content)),
            vec![PathBuf::from("/project/workflows/component.tsx")],
        );
        assert!(should_process_event(&event, &ignored, false, &no_files));
    }

    #[test]
    fn test_should_ignore_non_ts() {
        let ignored = vec![];
        let no_files = HashSet::new();
        let event = make_event(
            EventKind::Create(CreateKind::File),
            vec![PathBuf::from("/project/src/main.rs")],
        );
        assert!(!should_process_event(&event, &ignored, false, &no_files));

        let event_json = make_event(
            EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Content)),
            vec![PathBuf::from("/project/config.json")],
        );
        assert!(!should_process_event(
            &event_json,
            &ignored,
            false,
            &no_files
        ));
    }

    #[test]
    fn test_should_ignore_node_modules() {
        let ignored = vec!["node_modules".to_string()];
        let no_files = HashSet::new();
        let event = make_event(
            EventKind::Create(CreateKind::File),
            vec![PathBuf::from("/project/node_modules/pkg/index.ts")],
        );
        assert!(!should_process_event(&event, &ignored, false, &no_files));
    }

    #[test]
    fn test_should_ignore_generated() {
        let ignored = vec!["generated".to_string()];
        let no_files = HashSet::new();
        let event = make_event(
            EventKind::Create(CreateKind::File),
            vec![PathBuf::from("/project/generated/types.ts")],
        );
        assert!(!should_process_event(&event, &ignored, false, &no_files));
    }

    #[test]
    fn test_should_ignore_delete_event() {
        let ignored = vec![];
        let no_files = HashSet::new();
        let event = make_event(
            EventKind::Remove(RemoveKind::File),
            vec![PathBuf::from("/project/workflows/ci.ts")],
        );
        assert!(!should_process_event(&event, &ignored, false, &no_files));
    }

    #[test]
    fn test_should_ignore_custom_pattern() {
        let ignored = vec!["dist".to_string(), ".cache".to_string()];
        let no_files = HashSet::new();

        let event_dist = make_event(
            EventKind::Create(CreateKind::File),
            vec![PathBuf::from("/project/dist/bundle.ts")],
        );
        assert!(!should_process_event(
            &event_dist,
            &ignored,
            false,
            &no_files
        ));

        let event_cache = make_event(
            EventKind::Create(CreateKind::File),
            vec![PathBuf::from("/project/.cache/types.ts")],
        );
        assert!(!should_process_event(
            &event_cache,
            &ignored,
            false,
            &no_files
        ));
    }

    #[test]
    fn test_default_ignored_patterns() {
        let default_ignored = vec![
            "node_modules".to_string(),
            ".git".to_string(),
            "generated".to_string(),
        ];
        let no_files = HashSet::new();

        let event_node = make_event(
            EventKind::Create(CreateKind::File),
            vec![PathBuf::from("/project/node_modules/pkg/index.ts")],
        );
        assert!(!should_process_event(
            &event_node,
            &default_ignored,
            false,
            &no_files
        ));

        let event_git = make_event(
            EventKind::Create(CreateKind::File),
            vec![PathBuf::from("/project/.git/hooks/pre-commit.ts")],
        );
        assert!(!should_process_event(
            &event_git,
            &default_ignored,
            false,
            &no_files
        ));

        let event_generated = make_event(
            EventKind::Create(CreateKind::File),
            vec![PathBuf::from("/project/generated/types.ts")],
        );
        assert!(!should_process_event(
            &event_generated,
            &default_ignored,
            false,
            &no_files
        ));
    }
}
