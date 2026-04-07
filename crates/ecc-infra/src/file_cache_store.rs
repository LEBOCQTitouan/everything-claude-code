//! FileCacheStore — disk-backed cache adapter implementing `CacheStore`.
//!
//! Stores cache entries as JSON files under `<cache_dir>/<sanitized_key>.json`.
//! Key sanitization: replace non-alphanumeric characters with `_`.
//! Writes are atomic: write to a tempfile then rename.

use ecc_ports::cache_store::{CacheError, CacheEntry, CacheStore};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Disk-backed cache store.
pub struct FileCacheStore {
    cache_dir: PathBuf,
}

impl FileCacheStore {
    /// Create a new store backed by `cache_dir`.
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }
}

/// Local JSON-serializable mirror of `CacheEntry`.
#[derive(serde::Serialize, serde::Deserialize)]
struct CacheEntryJson {
    value: String,
    created_at: u64,
    ttl_secs: u64,
    content_hash: String,
}

impl From<CacheEntryJson> for CacheEntry {
    fn from(j: CacheEntryJson) -> Self {
        CacheEntry {
            value: j.value,
            created_at: j.created_at,
            ttl_secs: j.ttl_secs,
            content_hash: j.content_hash,
        }
    }
}

/// Replace non-alphanumeric characters in a cache key with `_`.
fn sanitize_key(key: &str) -> String {
    key.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl CacheStore for FileCacheStore {
    fn check(&self, key: &str) -> Result<Option<CacheEntry>, CacheError> {
        let path = self.cache_dir.join(format!("{}.json", sanitize_key(key)));
        if !path.exists() {
            return Ok(None);
        }
        let contents =
            std::fs::read_to_string(&path).map_err(|e| CacheError::Io(e.to_string()))?;
        let entry: CacheEntryJson =
            serde_json::from_str(&contents).map_err(|e| CacheError::Parse(e.to_string()))?;

        // Check TTL expiry.
        let expiry = entry.created_at.saturating_add(entry.ttl_secs);
        if now_secs() >= expiry {
            return Ok(None);
        }

        Ok(Some(entry.into()))
    }

    fn write(
        &self,
        key: &str,
        value: &str,
        ttl_secs: u64,
        content_hash: &str,
    ) -> Result<(), CacheError> {
        std::fs::create_dir_all(&self.cache_dir)
            .map_err(|e| CacheError::Io(e.to_string()))?;

        let entry = CacheEntryJson {
            value: value.to_owned(),
            created_at: now_secs(),
            ttl_secs,
            content_hash: content_hash.to_owned(),
        };
        let serialized =
            serde_json::to_string(&entry).map_err(|e| CacheError::Io(e.to_string()))?;

        // Atomic write: write to tempfile then rename.
        let final_path = self.cache_dir.join(format!("{}.json", sanitize_key(key)));
        let tmp_path = self.cache_dir.join(format!(".{}.json.tmp", sanitize_key(key)));
        std::fs::write(&tmp_path, &serialized).map_err(|e| CacheError::Io(e.to_string()))?;
        std::fs::rename(&tmp_path, &final_path).map_err(|e| CacheError::Io(e.to_string()))?;

        Ok(())
    }

    fn clear(&self) -> Result<(), CacheError> {
        if !self.cache_dir.exists() {
            return Ok(());
        }
        let entries =
            std::fs::read_dir(&self.cache_dir).map_err(|e| CacheError::Io(e.to_string()))?;
        for entry in entries {
            let entry = entry.map_err(|e| CacheError::Io(e.to_string()))?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                std::fs::remove_file(&path).map_err(|e| CacheError::Io(e.to_string()))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn write_and_check_round_trip() {
        let tmp = TempDir::new().unwrap();
        let store = FileCacheStore::new(tmp.path().to_path_buf());

        store.write("my-key", "audit findings", 3600, "hash123").unwrap();

        let result = store.check("my-key").unwrap();
        assert!(result.is_some(), "expected Some(entry) after write");
        let entry = result.unwrap();
        assert_eq!(entry.value, "audit findings");
        assert_eq!(entry.content_hash, "hash123");
        assert_eq!(entry.ttl_secs, 3600);
        // created_at should be close to now
        let delta = now_secs().saturating_sub(entry.created_at);
        assert!(delta < 5, "created_at should be within 5s of now");
    }

    #[test]
    fn expired_entry_returns_none() {
        let tmp = TempDir::new().unwrap();
        let store = FileCacheStore::new(tmp.path().to_path_buf());

        // Write entry that expired long ago: created_at = 1, ttl = 1 => expired at t=2
        let key = "expired-key";
        let sanitized = sanitize_key(key);
        let path = tmp.path().join(format!("{sanitized}.json"));
        let json = serde_json::json!({
            "value": "old data",
            "created_at": 1u64,
            "ttl_secs": 1u64,
            "content_hash": "oldhash"
        });
        std::fs::write(&path, serde_json::to_string(&json).unwrap()).unwrap();

        let result = store.check(key).unwrap();
        assert!(result.is_none(), "expired entry should return None");
    }

    #[test]
    fn clear_removes_all_entries() {
        let tmp = TempDir::new().unwrap();
        let store = FileCacheStore::new(tmp.path().to_path_buf());

        store.write("key1", "val1", 3600, "h1").unwrap();
        store.write("key2", "val2", 3600, "h2").unwrap();

        store.clear().unwrap();

        assert!(store.check("key1").unwrap().is_none(), "key1 should be gone after clear");
        assert!(store.check("key2").unwrap().is_none(), "key2 should be gone after clear");
    }

    #[test]
    fn check_nonexistent_returns_none() {
        let tmp = TempDir::new().unwrap();
        let store = FileCacheStore::new(tmp.path().to_path_buf());

        let result = store.check("never-written").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn cache_hash_invalidation_returns_none_on_mismatch() {
        let tmp = TempDir::new().unwrap();
        let store = FileCacheStore::new(tmp.path().to_path_buf());

        store.write("my-key", "audit findings", 3600, "abc123").unwrap();

        let result = store.check_with_hash("my-key", "different_hash").unwrap();
        assert!(result.is_none(), "expected None when hash does not match");
    }

    #[test]
    fn cache_hash_match_returns_entry() {
        let tmp = TempDir::new().unwrap();
        let store = FileCacheStore::new(tmp.path().to_path_buf());

        store.write("my-key", "audit findings", 3600, "abc123").unwrap();

        let result = store.check_with_hash("my-key", "abc123").unwrap();
        assert!(result.is_some(), "expected Some(entry) when hash matches");
        let entry = result.unwrap();
        assert_eq!(entry.value, "audit findings");
        assert_eq!(entry.content_hash, "abc123");
    }

    #[test]
    fn write_failure_returns_error() {
        // Create a regular file where the cache_dir should be — create_dir_all will fail.
        let tmp = TempDir::new().unwrap();
        let blocker = tmp.path().join("blocker");
        std::fs::write(&blocker, b"not a dir").unwrap();
        let store = FileCacheStore::new(blocker.clone());

        let result = store.write("key", "value", 60, "hash");
        assert!(
            matches!(result, Err(CacheError::Io(_))),
            "expected Io error when cache_dir is a file, got: {result:?}"
        );
    }
}
