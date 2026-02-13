use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = ".gaji.toml";
const LOCAL_CONFIG_FILE: &str = ".gaji.local.toml";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    #[serde(default = "default_workflows_dir")]
    pub workflows_dir: String,

    #[serde(default = "default_output_dir")]
    pub output_dir: String,

    #[serde(default = "default_generated_dir")]
    pub generated_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchConfig {
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,

    #[serde(default)]
    pub ignored_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    #[serde(default = "default_true")]
    pub validate: bool,

    #[serde(default = "default_true")]
    pub format: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

fn default_true() -> bool {
    true
}

impl Config {
    pub fn load() -> Result<Self> {
        Self::load_with_local(Path::new(CONFIG_FILE), Path::new(LOCAL_CONFIG_FILE))
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

    pub fn save(&self) -> Result<()> {
        self.save_to(Path::new(CONFIG_FILE))
    }

    pub fn save_to(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
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
}
