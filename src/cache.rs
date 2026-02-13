use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::fetcher::ActionMetadata;

const CACHE_FILE: &str = ".gaji-cache.json";

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

    #[test]
    fn test_should_regenerate_missing_entry() {
        let data = CacheData {
            version: 1,
            entries: HashMap::new(),
        };
        let cache = Cache {
            data,
            cache_file: PathBuf::from(".test-cache.json"),
        };
        assert!(cache.should_regenerate("actions/checkout@v4", "somehash"));
    }

    #[test]
    fn test_should_regenerate_same_hash() {
        let hash = calculate_hash("content");
        let mut entries = HashMap::new();
        entries.insert(
            "actions/checkout@v4".to_string(),
            CacheEntry {
                action_ref: "actions/checkout@v4".to_string(),
                content_hash: hash.clone(),
                generated_at: 0,
                metadata: ActionMetadata {
                    name: "Checkout".to_string(),
                    description: None,
                    inputs: None,
                    outputs: None,
                    runs: None,
                },
            },
        );
        let cache = Cache {
            data: CacheData {
                version: 1,
                entries,
            },
            cache_file: PathBuf::from(".test-cache.json"),
        };
        assert!(!cache.should_regenerate("actions/checkout@v4", &hash));
    }

    #[test]
    fn test_should_regenerate_different_hash() {
        let mut entries = HashMap::new();
        entries.insert(
            "actions/checkout@v4".to_string(),
            CacheEntry {
                action_ref: "actions/checkout@v4".to_string(),
                content_hash: "oldhash".to_string(),
                generated_at: 0,
                metadata: ActionMetadata {
                    name: "Checkout".to_string(),
                    description: None,
                    inputs: None,
                    outputs: None,
                    runs: None,
                },
            },
        );
        let cache = Cache {
            data: CacheData {
                version: 1,
                entries,
            },
            cache_file: PathBuf::from(".test-cache.json"),
        };
        assert!(cache.should_regenerate("actions/checkout@v4", "newhash"));
    }

    #[test]
    fn test_cache_save_and_load_roundtrip() {
        let dir = tempfile::TempDir::new().unwrap();
        let cache_file = dir.path().join("cache.json");

        let cache = Cache {
            data: CacheData {
                version: 1,
                entries: HashMap::new(),
            },
            cache_file: cache_file.clone(),
        };

        let metadata = ActionMetadata {
            name: "Test Action".to_string(),
            description: Some("A test".to_string()),
            inputs: None,
            outputs: None,
            runs: None,
        };
        cache
            .set("test/action@v1", &metadata, "yaml content")
            .unwrap();

        // Read back the file and verify
        let content = std::fs::read_to_string(&cache_file).unwrap();
        let loaded: CacheData = serde_json::from_str(&content).unwrap();
        assert!(loaded.entries.contains_key("test/action@v1"));
        assert_eq!(
            loaded.entries["test/action@v1"].metadata.name,
            "Test Action"
        );
    }

    #[test]
    fn test_is_expired_missing_entry() {
        let cache = Cache {
            data: CacheData {
                version: 1,
                entries: HashMap::new(),
            },
            cache_file: PathBuf::from(".test-cache.json"),
        };
        assert!(cache.is_expired("nonexistent@v1", 30));
    }

    #[test]
    fn test_is_expired_fresh_entry() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut entries = HashMap::new();
        entries.insert(
            "actions/checkout@v4".to_string(),
            CacheEntry {
                action_ref: "actions/checkout@v4".to_string(),
                content_hash: "hash".to_string(),
                generated_at: now,
                metadata: ActionMetadata {
                    name: "Checkout".to_string(),
                    description: None,
                    inputs: None,
                    outputs: None,
                    runs: None,
                },
            },
        );
        let cache = Cache {
            data: CacheData {
                version: 1,
                entries,
            },
            cache_file: PathBuf::from(".test-cache.json"),
        };
        assert!(!cache.is_expired("actions/checkout@v4", 30));
    }

    #[test]
    fn test_is_expired_old_entry() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // 31 days ago
        let old_time = now - (31 * 24 * 60 * 60);
        let mut entries = HashMap::new();
        entries.insert(
            "actions/checkout@v4".to_string(),
            CacheEntry {
                action_ref: "actions/checkout@v4".to_string(),
                content_hash: "hash".to_string(),
                generated_at: old_time,
                metadata: ActionMetadata {
                    name: "Checkout".to_string(),
                    description: None,
                    inputs: None,
                    outputs: None,
                    runs: None,
                },
            },
        );
        let cache = Cache {
            data: CacheData {
                version: 1,
                entries,
            },
            cache_file: PathBuf::from(".test-cache.json"),
        };
        assert!(cache.is_expired("actions/checkout@v4", 30));
    }
}
