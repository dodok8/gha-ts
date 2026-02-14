use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::cache::Cache;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionMetadata {
    pub name: String,
    pub description: Option<String>,
    pub inputs: Option<HashMap<String, ActionInput>>,
    pub outputs: Option<HashMap<String, ActionOutput>>,
    pub runs: Option<ActionRuns>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionInput {
    pub description: Option<String>,
    pub required: Option<bool>,
    pub default: Option<String>,
    #[serde(rename = "deprecationMessage")]
    pub deprecation_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionOutput {
    pub description: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRuns {
    pub using: String,
    pub main: Option<String>,
    pub pre: Option<String>,
    pub post: Option<String>,
    pub image: Option<String>,
    pub entrypoint: Option<String>,
    pub args: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct ActionRef {
    pub owner: String,
    pub repo: String,
    pub path: Option<String>,
    pub ref_: String,
}

impl ActionRef {
    pub fn parse(action_ref: &str) -> Result<Self> {
        // Parse formats like:
        // - actions/checkout@v5
        // - owner/repo@tag
        // - owner/repo/path@ref

        let parts: Vec<&str> = action_ref.splitn(2, '@').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid action reference format: {}. Expected format: owner/repo@ref",
                action_ref
            ));
        }

        let path_parts: Vec<&str> = parts[0].split('/').collect();
        if path_parts.len() < 2 {
            return Err(anyhow::anyhow!(
                "Invalid action reference: {}. Expected at least owner/repo",
                action_ref
            ));
        }

        let owner = path_parts[0].to_string();
        let repo = path_parts[1].to_string();
        let path = if path_parts.len() > 2 {
            Some(path_parts[2..].join("/"))
        } else {
            None
        };
        let ref_ = parts[1].to_string();

        Ok(Self {
            owner,
            repo,
            path,
            ref_,
        })
    }

    pub fn to_raw_url(&self) -> String {
        self.to_raw_url_with_base(None)
    }

    pub fn to_raw_url_yaml(&self) -> String {
        self.to_raw_url_yaml_with_base(None)
    }

    pub fn to_raw_url_with_base(&self, api_url: Option<&str>) -> String {
        let base_path = if let Some(path) = &self.path {
            format!("{}/{}/{}", self.owner, self.repo, path)
        } else {
            format!("{}/{}", self.owner, self.repo)
        };

        match api_url {
            Some(base) => {
                let base = base.trim_end_matches('/');
                format!(
                    "{}/api/v3/repos/{}/{}/contents/{}action.yml?ref={}",
                    base,
                    self.owner,
                    self.repo,
                    self.path
                        .as_ref()
                        .map(|p| format!("{}/", p))
                        .unwrap_or_default(),
                    self.ref_
                )
            }
            None => format!(
                "https://raw.githubusercontent.com/{}/{}/action.yml",
                base_path, self.ref_
            ),
        }
    }

    pub fn to_raw_url_yaml_with_base(&self, api_url: Option<&str>) -> String {
        let base_path = if let Some(path) = &self.path {
            format!("{}/{}/{}", self.owner, self.repo, path)
        } else {
            format!("{}/{}", self.owner, self.repo)
        };

        match api_url {
            Some(base) => {
                let base = base.trim_end_matches('/');
                format!(
                    "{}/api/v3/repos/{}/{}/contents/{}action.yaml?ref={}",
                    base,
                    self.owner,
                    self.repo,
                    self.path
                        .as_ref()
                        .map(|p| format!("{}/", p))
                        .unwrap_or_default(),
                    self.ref_
                )
            }
            None => format!(
                "https://raw.githubusercontent.com/{}/{}/action.yaml",
                base_path, self.ref_
            ),
        }
    }
}

pub struct GitHubFetcher {
    client: reqwest::Client,
    cache: Cache,
    token: Option<String>,
    api_url: Option<String>,
    cache_ttl_days: u64,
}

impl GitHubFetcher {
    pub fn new(
        cache: Cache,
        token: Option<String>,
        api_url: Option<String>,
        cache_ttl_days: u64,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("gaji")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            cache,
            token,
            api_url,
            cache_ttl_days,
        }
    }

    pub async fn fetch_action_metadata(&self, action_ref_str: &str) -> Result<ActionMetadata> {
        // Check cache first (with TTL expiration)
        if !self.cache.is_expired(action_ref_str, self.cache_ttl_days) {
            if let Some(cached) = self.cache.get(action_ref_str) {
                return Ok(cached);
            }
        }

        let action_ref = ActionRef::parse(action_ref_str)?;
        let yaml_content = self.fetch_action_yaml(&action_ref).await?;
        let metadata: ActionMetadata = serde_yaml::from_str(&yaml_content)
            .with_context(|| format!("Failed to parse action.yml for {}", action_ref_str))?;

        // Store in cache
        self.cache.set(action_ref_str, &metadata, &yaml_content)?;

        Ok(metadata)
    }

    async fn fetch_action_yaml(&self, action_ref: &ActionRef) -> Result<String> {
        let api_url = self.api_url.as_deref();

        // Try action.yml first
        let url = action_ref.to_raw_url_with_base(api_url);
        match self.fetch_with_retry(&url, api_url.is_some()).await {
            Ok(content) => Ok(content),
            Err(_) => {
                // Try action.yaml as fallback
                let url_yaml = action_ref.to_raw_url_yaml_with_base(api_url);
                self.fetch_with_retry(&url_yaml, api_url.is_some()).await
            }
        }
    }

    async fn fetch_with_retry(&self, url: &str, is_api: bool) -> Result<String> {
        let mut retries = 0;
        const MAX_RETRIES: u32 = 3;

        loop {
            let mut request = self.client.get(url);

            if let Some(token) = &self.token {
                request = request.header("Authorization", format!("token {}", token));
            }

            // For GitHub Enterprise API, request raw content
            if is_api {
                request = request.header("Accept", "application/vnd.github.raw+json");
            }

            match request.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        return response
                            .text()
                            .await
                            .with_context(|| format!("Failed to read response from {}", url));
                    } else if response.status() == reqwest::StatusCode::NOT_FOUND {
                        return Err(anyhow::anyhow!("Action not found: {}", url));
                    } else if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                        if retries < MAX_RETRIES {
                            retries += 1;
                            let delay = Duration::from_secs(2_u64.pow(retries));
                            eprintln!(
                                "Rate limited, retrying in {} seconds ({}/{})",
                                delay.as_secs(),
                                retries,
                                MAX_RETRIES
                            );
                            tokio::time::sleep(delay).await;
                            continue;
                        }
                        return Err(anyhow::anyhow!(
                            "Rate limited after {} retries",
                            MAX_RETRIES
                        ));
                    } else {
                        return Err(anyhow::anyhow!("HTTP error {}: {}", response.status(), url));
                    }
                }
                Err(e) => {
                    if retries < MAX_RETRIES {
                        retries += 1;
                        let delay = Duration::from_secs(2_u64.pow(retries));
                        eprintln!(
                            "Network error, retrying in {} seconds ({}/{}): {}",
                            delay.as_secs(),
                            retries,
                            MAX_RETRIES,
                            e
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    return Err(anyhow::anyhow!(
                        "Network error after {} retries: {}",
                        MAX_RETRIES,
                        e
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_action_ref() {
        let action_ref = ActionRef::parse("actions/checkout@v5").unwrap();
        assert_eq!(action_ref.owner, "actions");
        assert_eq!(action_ref.repo, "checkout");
        assert_eq!(action_ref.ref_, "v5");
        assert!(action_ref.path.is_none());
    }

    #[test]
    fn test_parse_action_ref_with_path() {
        let action_ref = ActionRef::parse("owner/repo/path/to/action@main").unwrap();
        assert_eq!(action_ref.owner, "owner");
        assert_eq!(action_ref.repo, "repo");
        assert_eq!(action_ref.path, Some("path/to/action".to_string()));
        assert_eq!(action_ref.ref_, "main");
    }

    #[test]
    fn test_raw_url_generation() {
        let action_ref = ActionRef::parse("actions/checkout@v5").unwrap();
        assert_eq!(
            action_ref.to_raw_url(),
            "https://raw.githubusercontent.com/actions/checkout/v5/action.yml"
        );
    }

    #[test]
    fn test_invalid_action_ref() {
        assert!(ActionRef::parse("invalid").is_err());
        assert!(ActionRef::parse("no-at-sign").is_err());
        assert!(ActionRef::parse("only/one@").is_ok()); // Empty ref is technically valid
    }

    #[test]
    fn test_invalid_action_ref_no_owner() {
        // Single segment before @ should fail (no owner/repo split)
        let result = ActionRef::parse("checkout@v4");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("at least owner/repo"));
    }

    #[test]
    fn test_raw_url_with_path() {
        let action_ref = ActionRef::parse("owner/repo/sub/path@main").unwrap();
        assert_eq!(
            action_ref.to_raw_url(),
            "https://raw.githubusercontent.com/owner/repo/sub/path/main/action.yml"
        );
        assert_eq!(
            action_ref.to_raw_url_yaml(),
            "https://raw.githubusercontent.com/owner/repo/sub/path/main/action.yaml"
        );
    }
}
