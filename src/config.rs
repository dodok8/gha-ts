use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use anyhow::{Context, Result};
use rquickjs::{function::Func, Context as JsContext, Runtime as JsRuntime};
use serde::Deserialize;

pub const TS_CONFIG_FILE: &str = "gaji.config.ts";
pub const TS_LOCAL_CONFIG_FILE: &str = "gaji.config.local.ts";

const TOML_CONFIG_FILE: &str = ".gaji.toml";
const TOML_LOCAL_CONFIG_FILE: &str = ".gaji.local.toml";

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub project: ProjectConfig,

    #[serde(default)]
    pub watch: WatchConfig,

    #[serde(default)]
    pub build: BuildConfig,

    #[serde(default)]
    pub github: GitHubConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectConfig {
    #[serde(default = "default_workflows_dir")]
    pub workflows_dir: String,

    #[serde(default = "default_output_dir")]
    pub output_dir: String,

    #[serde(default = "default_generated_dir")]
    pub generated_dir: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WatchConfig {
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,

    #[serde(default)]
    pub ignored_patterns: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BuildConfig {
    #[serde(default = "default_true")]
    pub validate: bool,

    #[serde(default = "default_true")]
    pub format: bool,

    #[serde(default = "default_cache_ttl_days")]
    pub cache_ttl_days: u64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct GitHubConfig {
    pub token: Option<String>,
    pub api_url: Option<String>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            workflows_dir: default_workflows_dir(),
            output_dir: default_output_dir(),
            generated_dir: default_generated_dir(),
        }
    }
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            debounce_ms: default_debounce_ms(),
            ignored_patterns: vec![
                "node_modules".to_string(),
                ".git".to_string(),
                "generated".to_string(),
            ],
        }
    }
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            validate: true,
            format: true,
            cache_ttl_days: default_cache_ttl_days(),
        }
    }
}

fn default_workflows_dir() -> String {
    "workflows".to_string()
}

fn default_output_dir() -> String {
    ".github".to_string()
}

fn default_generated_dir() -> String {
    "generated".to_string()
}

fn default_debounce_ms() -> u64 {
    300
}

fn default_cache_ttl_days() -> u64 {
    30
}

fn default_true() -> bool {
    true
}

// -- TS config intermediate types (camelCase JSON) --

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct TsGajiConfig {
    workflows: Option<String>,
    output: Option<String>,
    generated: Option<String>,
    watch: Option<TsWatchConfig>,
    build: Option<TsBuildConfig>,
    github: Option<TsGitHubConfig>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct TsWatchConfig {
    debounce: Option<u64>,
    ignore: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct TsBuildConfig {
    validate: Option<bool>,
    format: Option<bool>,
    #[serde(rename = "cacheTtlDays")]
    cache_ttl_days: Option<u64>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct TsGitHubConfig {
    token: Option<String>,
    #[serde(rename = "apiUrl")]
    api_url: Option<String>,
}

impl From<TsGajiConfig> for Config {
    fn from(ts: TsGajiConfig) -> Self {
        let mut config = Config::default();

        if let Some(workflows) = ts.workflows {
            config.project.workflows_dir = workflows;
        }
        if let Some(output) = ts.output {
            config.project.output_dir = output;
        }
        if let Some(generated) = ts.generated {
            config.project.generated_dir = generated;
        }

        if let Some(watch) = ts.watch {
            if let Some(debounce) = watch.debounce {
                config.watch.debounce_ms = debounce;
            }
            if let Some(ignore) = watch.ignore {
                config.watch.ignored_patterns = ignore;
            }
        }

        if let Some(build) = ts.build {
            if let Some(validate) = build.validate {
                config.build.validate = validate;
            }
            if let Some(format) = build.format {
                config.build.format = format;
            }
            if let Some(ttl) = build.cache_ttl_days {
                config.build.cache_ttl_days = ttl;
            }
        }

        if let Some(github) = ts.github {
            config.github.token = github.token;
            config.github.api_url = github.api_url;
        }

        config
    }
}

/// Execute JavaScript in QuickJS and capture config JSON via `__gha_set_config`.
fn execute_config_js(code: &str) -> Result<String> {
    let result: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    {
        let rt = JsRuntime::new().context("Failed to create QuickJS runtime")?;
        let ctx = JsContext::full(&rt).context("Failed to create QuickJS context")?;

        let code_owned = code.to_string();

        ctx.with(|ctx| {
            let result_clone = result.clone();

            let set_config_fn = Func::from(move |json: String| {
                *result_clone.borrow_mut() = Some(json);
            });

            ctx.globals()
                .set("__gha_set_config", set_config_fn)
                .map_err(|e| anyhow::anyhow!("Failed to set __gha_set_config: {}", e))?;

            ctx.eval::<(), _>(code_owned.as_bytes())
                .map_err(|e| anyhow::anyhow!("QuickJS config evaluation error: {}", e))?;

            Ok::<_, anyhow::Error>(())
        })?;
    }

    let json = Rc::try_unwrap(result)
        .map_err(|_| anyhow::anyhow!("Failed to unwrap Rc"))?
        .into_inner()
        .context("Config script did not call __gha_set_config")?;

    Ok(json)
}

impl Config {
    /// Load config: try `gaji.config.ts` first, fall back to `.gaji.toml`.
    pub fn load() -> Result<Self> {
        let ts_path = Path::new(TS_CONFIG_FILE);
        if ts_path.exists() {
            let mut config = Self::load_from_ts(ts_path)?;

            // Merge local TS config if it exists
            let ts_local_path = Path::new(TS_LOCAL_CONFIG_FILE);
            if ts_local_path.exists() {
                let local = Self::load_from_ts(ts_local_path)?;
                config.merge_local(local);
            }

            return Ok(config);
        }

        // Fall back to TOML
        Self::load_with_local(
            Path::new(TOML_CONFIG_FILE),
            Path::new(TOML_LOCAL_CONFIG_FILE),
        )
    }

    /// Load config from a TypeScript file by stripping types, executing in QuickJS.
    pub fn load_from_ts(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Config::default());
        }

        let source = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Strip TypeScript types
        let js = crate::executor::strip_typescript(&source, &filename)?;

        // Remove import/export statements
        let js = crate::executor::remove_imports(&js);

        // Wrap: override defineConfig to identity, capture result
        let wrapped = format!(
            r#"function defineConfig(c) {{ return c; }}
var __config_result = {};
__gha_set_config(JSON.stringify(__config_result));"#,
            js.trim().trim_end_matches(';')
        );

        let json = execute_config_js(&wrapped)?;

        let ts_config: TsGajiConfig = serde_json::from_str(&json)
            .with_context(|| format!("Failed to parse config JSON from {}", path.display()))?;

        Ok(Config::from(ts_config))
    }

    pub fn load_from(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn load_with_local(config_path: &Path, local_path: &Path) -> Result<Self> {
        let mut config = Self::load_from(config_path)?;

        if local_path.exists() {
            let local_content = std::fs::read_to_string(local_path)?;
            let local: Config = toml::from_str(&local_content)?;
            config.merge_local(local);
        }

        Ok(config)
    }

    fn merge_local(&mut self, local: Config) {
        if local.github.token.is_some() {
            self.github.token = local.github.token;
        }
        if local.github.api_url.is_some() {
            self.github.api_url = local.github.api_url;
        }
    }

    /// Resolve the GitHub token with priority: env var > local config > public config
    pub fn resolve_token(&self) -> Option<String> {
        std::env::var("GITHUB_TOKEN")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| self.github.token.clone())
    }

    /// Resolve the GitHub API base URL for raw content.
    /// Returns None for default github.com (uses raw.githubusercontent.com).
    /// Returns Some(url) for GitHub Enterprise.
    pub fn resolve_api_url(&self) -> Option<String> {
        self.github.api_url.clone()
    }

    pub fn workflows_path(&self) -> PathBuf {
        PathBuf::from(&self.project.workflows_dir)
    }

    pub fn output_path(&self) -> PathBuf {
        PathBuf::from(&self.project.output_dir)
    }

    pub fn generated_path(&self) -> PathBuf {
        PathBuf::from(&self.project.generated_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.project.workflows_dir, "workflows");
        assert_eq!(config.project.output_dir, ".github");
        assert_eq!(config.project.generated_dir, "generated");
        assert_eq!(config.watch.debounce_ms, 300);
        assert!(config.build.validate);
        assert_eq!(config.build.cache_ttl_days, 30);
    }

    #[test]
    fn test_parse_full_toml() {
        let toml_str = r#"
[project]
workflows_dir = "custom_workflows"
output_dir = "custom_output"
generated_dir = "custom_generated"

[watch]
debounce_ms = 500
ignored_patterns = ["dist", "tmp"]

[build]
validate = false
format = false

[github]
token = "ghp_test123"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.project.workflows_dir, "custom_workflows");
        assert_eq!(config.project.output_dir, "custom_output");
        assert_eq!(config.project.generated_dir, "custom_generated");
        assert_eq!(config.watch.debounce_ms, 500);
        assert!(!config.build.validate);
        assert!(!config.build.format);
        assert_eq!(config.github.token, Some("ghp_test123".to_string()));
    }

    #[test]
    fn test_parse_partial_toml_uses_defaults() {
        let toml_str = r#"
[project]
workflows_dir = "my_workflows"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.project.workflows_dir, "my_workflows");
        // Other fields should use defaults
        assert_eq!(config.project.output_dir, ".github");
        assert_eq!(config.project.generated_dir, "generated");
        assert_eq!(config.watch.debounce_ms, 300);
        assert!(config.build.validate);
        assert!(config.github.token.is_none());
    }

    #[test]
    fn test_parse_github_config_with_api_url() {
        let toml_str = r#"
[github]
token = "ghp_enterprise123"
api_url = "https://github.example.com"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.github.token, Some("ghp_enterprise123".to_string()));
        assert_eq!(
            config.github.api_url,
            Some("https://github.example.com".to_string())
        );
    }

    #[test]
    fn test_merge_local_overrides_token() {
        let mut config = Config::default();
        config.github.token = Some("public_token".to_string());

        let mut local = Config::default();
        local.github.token = Some("local_secret_token".to_string());

        config.merge_local(local);
        assert_eq!(config.github.token, Some("local_secret_token".to_string()));
    }

    #[test]
    fn test_merge_local_does_not_clear_existing() {
        let mut config = Config::default();
        config.github.token = Some("existing_token".to_string());

        let local = Config::default(); // no token set
        config.merge_local(local);

        assert_eq!(config.github.token, Some("existing_token".to_string()));
    }

    #[test]
    fn test_merge_local_overrides_api_url() {
        let mut config = Config::default();

        let mut local = Config::default();
        local.github.api_url = Some("https://ghe.corp.com".to_string());

        config.merge_local(local);
        assert_eq!(
            config.github.api_url,
            Some("https://ghe.corp.com".to_string())
        );
    }

    #[test]
    fn test_load_with_local_file() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join(".gaji.toml");
        let local_path = dir.path().join(".gaji.local.toml");

        std::fs::write(
            &config_path,
            r#"
[project]
workflows_dir = "workflows"
"#,
        )
        .unwrap();

        std::fs::write(
            &local_path,
            r#"
[github]
token = "secret_from_local"
api_url = "https://ghe.internal.com"
"#,
        )
        .unwrap();

        let config = Config::load_with_local(&config_path, &local_path).unwrap();
        assert_eq!(config.project.workflows_dir, "workflows");
        assert_eq!(config.github.token, Some("secret_from_local".to_string()));
        assert_eq!(
            config.github.api_url,
            Some("https://ghe.internal.com".to_string())
        );
    }

    #[test]
    fn test_resolve_token_prefers_env_var() {
        let mut config = Config::default();
        config.github.token = Some("config_token".to_string());

        // When GITHUB_TOKEN env is set, it takes priority
        std::env::set_var("GITHUB_TOKEN", "env_token");
        assert_eq!(config.resolve_token(), Some("env_token".to_string()));
        std::env::remove_var("GITHUB_TOKEN");
    }

    #[test]
    fn test_resolve_token_falls_back_to_config() {
        // Ensure env var is not set
        std::env::remove_var("GITHUB_TOKEN");

        let mut config = Config::default();
        config.github.token = Some("config_token".to_string());

        assert_eq!(config.resolve_token(), Some("config_token".to_string()));
    }

    #[test]
    fn test_resolve_token_returns_none_when_empty() {
        std::env::remove_var("GITHUB_TOKEN");

        let config = Config::default();
        assert_eq!(config.resolve_token(), None);
    }

    #[test]
    fn test_load_from_ts_basic() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("gaji.config.ts");
        std::fs::write(
            &config_path,
            r#"
import { defineConfig } from "./generated/index.js";

export default defineConfig({
    workflows: "src/workflows",
    output: "dist/.github",
    generated: "src/generated",
    watch: {
        debounce: 500,
    },
    build: {
        cacheTtlDays: 14,
    },
});
"#,
        )
        .unwrap();

        let config = Config::load_from_ts(&config_path).unwrap();
        assert_eq!(config.project.workflows_dir, "src/workflows");
        assert_eq!(config.project.output_dir, "dist/.github");
        assert_eq!(config.project.generated_dir, "src/generated");
        assert_eq!(config.watch.debounce_ms, 500);
        assert_eq!(config.build.cache_ttl_days, 14);
    }

    #[test]
    fn test_load_from_ts_with_local() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("gaji.config.ts");
        let local_path = dir.path().join("gaji.config.local.ts");

        std::fs::write(
            &config_path,
            r#"
export default defineConfig({
    workflows: "workflows",
});
"#,
        )
        .unwrap();

        std::fs::write(
            &local_path,
            r#"
export default defineConfig({
    github: {
        token: "ghp_secret_local",
        apiUrl: "https://ghe.corp.com",
    },
});
"#,
        )
        .unwrap();

        let mut config = Config::load_from_ts(&config_path).unwrap();
        let local = Config::load_from_ts(&local_path).unwrap();
        config.merge_local(local);

        assert_eq!(config.project.workflows_dir, "workflows");
        assert_eq!(config.github.token, Some("ghp_secret_local".to_string()));
        assert_eq!(
            config.github.api_url,
            Some("https://ghe.corp.com".to_string())
        );
    }

    #[test]
    fn test_load_from_ts_empty_config() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("gaji.config.ts");
        std::fs::write(
            &config_path,
            r#"
export default defineConfig({});
"#,
        )
        .unwrap();

        let config = Config::load_from_ts(&config_path).unwrap();
        // Should use defaults
        assert_eq!(config.project.workflows_dir, "workflows");
        assert_eq!(config.project.output_dir, ".github");
        assert_eq!(config.project.generated_dir, "generated");
        assert_eq!(config.watch.debounce_ms, 300);
        assert!(config.build.validate);
    }

    #[test]
    fn test_load_from_ts_env_var_precedence() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("gaji.config.ts");
        std::fs::write(
            &config_path,
            r#"
export default defineConfig({
    github: {
        token: "ts_config_token",
    },
});
"#,
        )
        .unwrap();

        let config = Config::load_from_ts(&config_path).unwrap();
        assert_eq!(config.github.token, Some("ts_config_token".to_string()));

        // Set env var — should take precedence via resolve_token()
        std::env::set_var("GITHUB_TOKEN", "env_override_token");
        assert_eq!(
            config.resolve_token(),
            Some("env_override_token".to_string())
        );
        std::env::remove_var("GITHUB_TOKEN");

        // Without env var, falls back to TS config value
        assert_eq!(config.resolve_token(), Some("ts_config_token".to_string()));
    }
}
