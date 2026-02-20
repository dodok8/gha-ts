pub mod templates;
pub mod types;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::fs;

use crate::cache::Cache;
use crate::fetcher::GitHubFetcher;

use self::templates::{
    BASE_TYPES_TEMPLATE, CLASS_DECLARATIONS_TEMPLATE, GET_ACTION_FALLBACK_DECL_TEMPLATE,
    GET_ACTION_RUNTIME_TEMPLATE, JOB_WORKFLOW_RUNTIME_TEMPLATE,
};
use self::types::generate_type_definition;

pub struct TypeGenerator {
    fetcher: GitHubFetcher,
    output_dir: PathBuf,
}

impl TypeGenerator {
    pub fn new(
        cache: Cache,
        output_dir: PathBuf,
        token: Option<String>,
        api_url: Option<String>,
    ) -> Self {
        Self::with_cache_ttl(cache, output_dir, token, api_url, 30)
    }

    pub fn with_cache_ttl(
        cache: Cache,
        output_dir: PathBuf,
        token: Option<String>,
        api_url: Option<String>,
        cache_ttl_days: u64,
    ) -> Self {
        Self {
            fetcher: GitHubFetcher::new(cache, token, api_url, cache_ttl_days),
            output_dir,
        }
    }

    pub async fn generate_types_for_refs(
        &self,
        action_refs: &HashSet<String>,
    ) -> Result<Vec<PathBuf>> {
        fs::create_dir_all(&self.output_dir).await?;

        let mut generated_files = Vec::new();

        // Generate base types first
        let base_path = self.generate_base_types().await?;
        generated_files.push(base_path);

        let mut action_infos = Vec::new();

        let pb = ProgressBar::new(action_refs.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("   {spinner:.green} [{bar:30.cyan/dim}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("━━─"),
        );
        pb.set_message("fetching action metadata...");

        // Fetch all action metadata in parallel (max 10 concurrent requests)
        let fetch_results = self
            .fetcher
            .fetch_action_metadata_batch(action_refs, 10)
            .await;

        pb.set_message("generating types...");

        for (action_ref, result) in fetch_results {
            match result {
                Ok(metadata) => {
                    match self
                        .generate_type_from_metadata(&action_ref, &metadata)
                        .await
                    {
                        Ok((path, info)) => {
                            generated_files.push(path);
                            action_infos.push(info);
                        }
                        Err(e) => {
                            pb.suspend(|| {
                                eprintln!("Failed to generate types for {}: {}", action_ref, e);
                            });
                        }
                    }
                }
                Err(e) => {
                    pb.suspend(|| {
                        eprintln!("Failed to fetch metadata for {}: {}", action_ref, e);
                    });
                }
            }
            pb.inc(1);
        }

        pb.finish_and_clear();

        // Remove old index.ts if it exists (replaced by index.d.ts + index.js)
        let old_index_ts = self.output_dir.join("index.ts");
        if old_index_ts.exists() {
            let _ = fs::remove_file(&old_index_ts).await;
        }

        // Generate index.d.ts (type declarations) and index.js (runtime)
        self.generate_index_dts(&action_infos).await?;
        self.generate_index_js(&action_infos).await?;

        Ok(generated_files)
    }

    async fn generate_base_types(&self) -> Result<PathBuf> {
        let content = BASE_TYPES_TEMPLATE.to_string();

        let file_path = self.output_dir.join("base.d.ts");
        fs::write(&file_path, content).await?;

        Ok(file_path)
    }

    async fn generate_type_from_metadata(
        &self,
        action_ref: &str,
        metadata: &crate::fetcher::ActionMetadata,
    ) -> Result<(PathBuf, ActionTypeInfo)> {
        let type_def = generate_type_definition(action_ref, metadata);

        let interface_name = action_ref_to_interface_name(action_ref);
        let module_name = action_ref_to_module_name(action_ref);
        let mut output_names: Vec<String> = metadata
            .outputs
            .as_ref()
            .map(|outputs| outputs.keys().cloned().collect())
            .unwrap_or_default();
        output_names.sort();
        let has_outputs = !output_names.is_empty();

        let filename = action_ref_to_filename(action_ref);
        let file_path = self.output_dir.join(&filename);

        fs::write(&file_path, type_def).await?;

        Ok((
            file_path,
            ActionTypeInfo {
                action_ref: action_ref.to_string(),
                interface_name,
                module_name,
                has_outputs,
                output_names,
            },
        ))
    }

    /// Generate index.d.ts - type declarations only
    async fn generate_index_dts(&self, action_infos: &[ActionTypeInfo]) -> Result<()> {
        let mut sorted_infos = action_infos.to_vec();
        sorted_infos.sort_by(|left, right| left.action_ref.cmp(&right.action_ref));

        let mut content = String::new();
        content.push_str("// Auto-generated by gaji\n// Do not edit manually\n\n");

        // Type imports
        content.push_str("import type { JobStep, ActionStep, JobDefinition, WorkflowConfig, WorkflowDefinition, Step, Permissions, Service, Container, WorkflowTrigger, ScheduleTrigger, WorkflowDispatchInput, WorkflowOn, ActionInputDefinition, ActionOutputDefinition, NodeActionConfig, NodeActionRuns, DockerActionConfig, DockerActionRuns, JobOutputs, GajiConfig } from './base';\n");

        // Per-action type imports
        for info in &sorted_infos {
            if info.has_outputs {
                content.push_str(&format!(
                    "import type {{ {}Inputs, {}Outputs }} from './{}';",
                    info.interface_name, info.interface_name, info.module_name
                ));
            } else {
                content.push_str(&format!(
                    "import type {{ {}Inputs }} from './{}';",
                    info.interface_name, info.module_name
                ));
            }
            content.push('\n');
        }

        content.push('\n');

        // getAction overloads (declare — no body in .d.ts)
        for info in &sorted_infos {
            if info.has_outputs {
                // Actions WITH outputs: callable interface with two overloads
                // First overload requires id with Id generic → returns ActionStep<Outputs, Id>
                // Second overload has optional id → returns JobStep
                content.push_str(&format!(
                    r#"export declare function getAction(
    ref: '{}'
): {{
    <Id extends string>(config: {{ id: Id; name?: string; with?: {}Inputs; if?: string; env?: Record<string, string> }}): ActionStep<{}Outputs, Id>;
    (config?: {{ name?: string; with?: {}Inputs; id?: string; if?: string; env?: Record<string, string> }}): JobStep;
}};
"#,
                    info.action_ref,
                    info.interface_name,
                    info.interface_name,
                    info.interface_name
                ));
            } else {
                // Actions WITHOUT outputs: simple signature → JobStep
                content.push_str(&format!(
                    r#"export declare function getAction(
    ref: '{}'
): (config?: {{
    name?: string;
    with?: {}Inputs;
    id?: string;
    if?: string;
    env?: Record<string, string>;
}}) => JobStep;
"#,
                    info.action_ref, info.interface_name
                ));
            }
        }

        // Base getAction overload (from template)
        content.push_str(GET_ACTION_FALLBACK_DECL_TEMPLATE);

        // Class declarations (from template)
        content.push_str(CLASS_DECLARATIONS_TEMPLATE);

        // Type re-exports
        content.push_str("\nexport type { JobStep, ActionStep, Step, JobDefinition, Service, Container, Permissions, WorkflowTrigger, ScheduleTrigger, WorkflowDispatchInput, WorkflowOn, WorkflowConfig, WorkflowDefinition, ActionInputDefinition, ActionOutputDefinition, NodeActionConfig, NodeActionRuns, DockerActionConfig, DockerActionRuns, JobOutputs, GajiConfig } from './base';\n");
        for info in &sorted_infos {
            if info.has_outputs {
                content.push_str(&format!(
                    "export type {{ {}Inputs, {}Outputs }} from './{}';",
                    info.interface_name, info.interface_name, info.module_name
                ));
            } else {
                content.push_str(&format!(
                    "export type {{ {}Inputs }} from './{}';",
                    info.interface_name, info.module_name
                ));
            }
            content.push('\n');
        }

        let path = self.output_dir.join("index.d.ts");
        fs::write(path, content).await?;

        Ok(())
    }

    /// Generate index.js - runtime implementation
    async fn generate_index_js(&self, action_infos: &[ActionTypeInfo]) -> Result<()> {
        let mut sorted_infos = action_infos.to_vec();
        sorted_infos.sort_by(|left, right| left.action_ref.cmp(&right.action_ref));

        let mut content = String::new();
        content.push_str("// Auto-generated by gaji\n// Do not edit manually\n\n");

        // Action output registry (dynamic per project)
        content.push_str("var __action_outputs = {\n");
        for info in &sorted_infos {
            if !info.output_names.is_empty() {
                let names: Vec<String> = info
                    .output_names
                    .iter()
                    .map(|n| format!("'{}'", n))
                    .collect();
                content.push_str(&format!(
                    "    '{}': [{}],\n",
                    info.action_ref,
                    names.join(", ")
                ));
            }
        }
        content.push_str("};\n");

        // getAction runtime (from template)
        content.push_str(GET_ACTION_RUNTIME_TEMPLATE);

        // Job/Workflow/Action/WorkflowCall/ActionRef/NodeAction runtime classes (from template)
        content.push_str(JOB_WORKFLOW_RUNTIME_TEMPLATE);
        content.push('\n');

        let path = self.output_dir.join("index.js");
        fs::write(path, content).await?;

        Ok(())
    }
}

#[derive(Clone)]
struct ActionTypeInfo {
    action_ref: String,
    interface_name: String,
    module_name: String,
    has_outputs: bool,
    output_names: Vec<String>,
}

pub fn action_ref_to_filename(action_ref: &str) -> String {
    action_ref.replace(['/', '@', '.'], "-") + ".d.ts"
}

pub fn action_ref_to_interface_name(action_ref: &str) -> String {
    // "actions/checkout@v5" -> "ActionsCheckoutV5"
    action_ref
        .split(['/', '@', '-', '.'])
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

fn action_ref_to_module_name(action_ref: &str) -> String {
    action_ref_to_filename(action_ref)
        .trim_end_matches(".d.ts")
        .to_string()
}

pub async fn ensure_generated_dir(path: &Path) -> Result<PathBuf> {
    let dir = path.to_path_buf();
    fs::create_dir_all(&dir).await?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_ref_to_filename() {
        assert_eq!(
            action_ref_to_filename("actions/checkout@v5"),
            "actions-checkout-v5.d.ts"
        );
        assert_eq!(
            action_ref_to_filename("owner/repo/path@main"),
            "owner-repo-path-main.d.ts"
        );
    }

    #[test]
    fn test_action_ref_to_interface_name() {
        assert_eq!(
            action_ref_to_interface_name("actions/checkout@v5"),
            "ActionsCheckoutV5"
        );
        assert_eq!(
            action_ref_to_interface_name("actions/setup-node@v4"),
            "ActionsSetupNodeV4"
        );
    }
}
