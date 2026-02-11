use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::fetcher::ActionMetadata;

const CACHE_FILE: &str = ".gha-ts-cache.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub action_ref: String,
    pub content_hash: String,
    pub generated_at: u64,
    pub metadata: ActionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheData {
    pub version: u32,
    pub entries: HashMap<String, CacheEntry>,
}

#[derive(Debug, Clone)]
pub struct Cache {
    data: CacheData,
    cache_file: PathBuf,
}

impl Cache {
    pub fn load_or_create() -> Result<Self> {
        let cache_file = PathBuf::from(CACHE_FILE);

        let data = if cache_file.exists() {
            let content = std::fs::read_to_string(&cache_file)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            CacheData {
                version: 1,
                ..Default::default()
            }
        };

        Ok(Self { data, cache_file })
    }

    pub fn get(&self, action_ref: &str) -> Option<ActionMetadata> {
        self.data
            .entries
            .get(action_ref)
            .map(|entry| entry.metadata.clone())
    }

    pub fn set(
        &self,
        action_ref: &str,
        metadata: &ActionMetadata,
        yaml_content: &str,
    ) -> Result<()> {
        let mut data = self.data.clone();

        let content_hash = calculate_hash(yaml_content);
        let generated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        data.entries.insert(
            action_ref.to_string(),
            CacheEntry {
                action_ref: action_ref.to_string(),
                content_hash,
                generated_at,
                metadata: metadata.clone(),
            },
        );

        let json = serde_json::to_string_pretty(&data)?;
        std::fs::write(&self.cache_file, json)?;

        Ok(())
    }

    pub fn should_regenerate(&self, action_ref: &str, new_hash: &str) -> bool {
        match self.data.entries.get(action_ref) {
            Some(entry) => entry.content_hash != new_hash,
            None => true,
        }
    }

    pub fn clear(&self) -> Result<()> {
        if self.cache_file.exists() {
            std::fs::remove_file(&self.cache_file)?;
        }
        Ok(())
    }

    pub fn remove(&self, action_ref: &str) -> Result<()> {
        let mut data = self.data.clone();
        data.entries.remove(action_ref);

        let json = serde_json::to_string_pretty(&data)?;
        std::fs::write(&self.cache_file, json)?;

        Ok(())
    }

    pub fn list(&self) -> Vec<&str> {
        self.data.entries.keys().map(|s| s.as_str()).collect()
    }

    pub fn is_expired(&self, action_ref: &str, max_age_days: u64) -> bool {
        let max_age_secs = max_age_days * 24 * 60 * 60;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        match self.data.entries.get(action_ref) {
            Some(entry) => now - entry.generated_at > max_age_secs,
            None => true,
        }
    }
}

fn calculate_hash(content: &str) -> String {
    // Simple hash implementation
    // In production, use sha256
    let mut hash: u64 = 0;
    for byte in content.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
    }
    format!("{:016x}", hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_hash() {
        let hash1 = calculate_hash("test content");
        let hash2 = calculate_hash("test content");
        let hash3 = calculate_hash("different content");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_cache_data_default() {
        let data = CacheData::default();
        assert_eq!(data.version, 0);
        assert!(data.entries.is_empty());
    }
}
