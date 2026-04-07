//! FileCacheStore — disk-backed cache adapter implementing `CachePort`.
//!
//! Stores cache entries as JSON files under `<cache_dir>/<sanitized_key>.json`.

use ecc_ports::cache_store::{CacheError, CacheEntry, CacheStore};
use std::path::PathBuf;

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

impl CacheStore for FileCacheStore {
    fn check(&self, _key: &str) -> Result<Option<CacheEntry>, CacheError> {
        unimplemented!("FileCacheStore::check not yet implemented")
    }

    fn write(
        &self,
        _key: &str,
        _value: &str,
        _ttl_secs: u64,
        _content_hash: &str,
    ) -> Result<(), CacheError> {
        unimplemented!("FileCacheStore::write not yet implemented")
    }

    fn clear(&self) -> Result<(), CacheError> {
        unimplemented!("FileCacheStore::clear not yet implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tempfile::TempDir;

    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

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
        // We do this by directly writing a JSON file with a past created_at.
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
    fn write_failure_returns_error() {
        // Use a path that does not exist and cannot be created (points to a file as dir)
        let tmp = TempDir::new().unwrap();
        // Create a regular file where the cache_dir should be
        let blocker = tmp.path().join("blocker");
        std::fs::write(&blocker, b"not a dir").unwrap();
        let store = FileCacheStore::new(blocker.clone());

        let result = store.write("key", "value", 60, "hash");
        assert!(
            matches!(result, Err(CacheError::Io(_))),
            "expected Io error when cache_dir is a file, got: {result:?}"
        );
    }

    /// Mirror of the key sanitization logic used in the implementation.
    fn sanitize_key(key: &str) -> String {
        key.chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect()
    }
}
