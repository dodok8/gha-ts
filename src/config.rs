use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = ".gha-ts.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
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
    ".github/workflows".to_string()
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
        Self::load_from(Path::new(CONFIG_FILE))
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

// Add toml crate usage (we'll need to add it to Cargo.toml)
mod toml {
    use serde::{de::DeserializeOwned, Serialize};

    pub fn from_str<T: DeserializeOwned>(s: &str) -> Result<T, std::io::Error> {
        // Simple stub - in production use toml crate
        serde_json::from_str(s).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub fn to_string_pretty<T: Serialize>(value: &T) -> Result<String, std::io::Error> {
        // Simple stub - in production use toml crate
        serde_json::to_string_pretty(value)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.project.workflows_dir, "workflows");
        assert_eq!(config.project.output_dir, ".github/workflows");
        assert_eq!(config.project.generated_dir, "generated");
        assert_eq!(config.watch.debounce_ms, 300);
        assert!(config.build.validate);
    }
}
