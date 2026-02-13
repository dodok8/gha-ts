pub mod ast;
pub mod extractor;

use std::collections::HashSet;
use std::path::Path;

use anyhow::Result;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

use self::extractor::ActionRefExtractor;

pub struct TypeScriptParser {
    allocator: Allocator,
}

impl TypeScriptParser {
    pub fn new() -> Self {
        Self {
            allocator: Allocator::default(),
        }
    }

    pub fn extract_action_refs(&self, source: &str) -> Result<HashSet<String>> {
        let source_type = SourceType::from_path("file.ts").unwrap_or_default();
        let parser_return = Parser::new(&self.allocator, source, source_type).parse();

        if !parser_return.errors.is_empty() {
            let error_messages: Vec<String> =
                parser_return.errors.iter().map(|e| e.to_string()).collect();
            return Err(anyhow::anyhow!(
                "Parse errors: {}",
                error_messages.join(", ")
            ));
        }

        let mut extractor = ActionRefExtractor::new();
        extractor.visit_program(&parser_return.program);

        Ok(extractor.action_refs)
    }
}

impl Default for TypeScriptParser {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn analyze_file(path: &Path) -> Result<HashSet<String>> {
    let source = tokio::fs::read_to_string(path).await?;
    let parser = TypeScriptParser::new();
    parser.extract_action_refs(&source)
}

pub async fn analyze_directory(
    dir: &Path,
) -> Result<std::collections::HashMap<std::path::PathBuf, HashSet<String>>> {
    use std::collections::HashMap;

    let mut results: HashMap<std::path::PathBuf, HashSet<String>> = HashMap::new();

    let mut entries = tokio::fs::read_dir(dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        if path.is_dir() {
            let sub_results = Box::pin(analyze_directory(&path)).await?;
            results.extend(sub_results);
        } else if let Some(ext) = path.extension() {
            if ext == "ts" || ext == "tsx" {
                match analyze_file(&path).await {
                    Ok(refs) => {
                        if !refs.is_empty() {
                            results.insert(path, refs);
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                    }
                }
            }
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_call() {
        let parser = TypeScriptParser::new();
        let source = r#"const checkout = getAction("actions/checkout@v4")"#;
        let refs = parser.extract_action_refs(source).unwrap();
        assert!(refs.contains("actions/checkout@v4"));
    }

    #[test]
    fn test_multiple_calls() {
        let parser = TypeScriptParser::new();
        let source = r#"
            const checkout = getAction("actions/checkout@v4")
            const setupNode = getAction("actions/setup-node@v4")
        "#;
        let refs = parser.extract_action_refs(source).unwrap();
        assert!(refs.contains("actions/checkout@v4"));
        assert!(refs.contains("actions/setup-node@v4"));
        assert_eq!(refs.len(), 2);
    }

    #[test]
    fn test_nested_expressions() {
        let parser = TypeScriptParser::new();
        let source = r#"
            const workflow = new Workflow()
                .addJob(
                    new Job()
                        .addStep(getAction("actions/checkout@v4")({ name: "Checkout" }))
                )
        "#;
        let refs = parser.extract_action_refs(source).unwrap();
        assert!(refs.contains("actions/checkout@v4"));
    }

    #[test]
    fn test_array_expressions() {
        let parser = TypeScriptParser::new();
        let source = r#"
            const actions = [
                getAction("actions/checkout@v4"),
                getAction("actions/setup-node@v4")
            ]
        "#;
        let refs = parser.extract_action_refs(source).unwrap();
        assert!(refs.contains("actions/checkout@v4"));
        assert!(refs.contains("actions/setup-node@v4"));
    }

    #[test]
    fn test_object_property() {
        let parser = TypeScriptParser::new();
        let source = r#"
            const actions = {
                checkout: getAction("actions/checkout@v4"),
                node: getAction("actions/setup-node@v4")
            }
        "#;
        let refs = parser.extract_action_refs(source).unwrap();
        assert!(refs.contains("actions/checkout@v4"));
        assert!(refs.contains("actions/setup-node@v4"));
    }
}
