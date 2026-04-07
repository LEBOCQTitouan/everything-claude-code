use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use ecc_ports::cache_store::{CacheEntry, CacheError, CacheStore};

/// In-memory test double for [`CacheStore`].
pub struct InMemoryCacheStore {
    entries: Mutex<HashMap<String, CacheEntry>>,
}

impl InMemoryCacheStore {
    /// Create an empty store.
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryCacheStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CacheStore for InMemoryCacheStore {
    fn check(&self, key: &str) -> Result<Option<CacheEntry>, CacheError> {
        let guard = self.entries.lock().expect("lock poisoned");
        let Some(entry) = guard.get(key) else {
            return Ok(None);
        };
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if now >= entry.created_at + entry.ttl_secs {
            return Ok(None);
        }
        Ok(Some(entry.clone()))
    }

    fn write(
        &self,
        key: &str,
        value: &str,
        ttl_secs: u64,
        content_hash: &str,
    ) -> Result<(), CacheError> {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let entry = CacheEntry {
            value: value.to_owned(),
            created_at,
            ttl_secs,
            content_hash: content_hash.to_owned(),
        };
        let mut guard = self.entries.lock().expect("lock poisoned");
        guard.insert(key.to_owned(), entry);
        Ok(())
    }

    fn clear(&self) -> Result<(), CacheError> {
        let mut guard = self.entries.lock().expect("lock poisoned");
        guard.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_cache_round_trip() {
        let store = InMemoryCacheStore::new();
        store.write("test", "value", 3600, "hash").unwrap();
        let entry = store.check("test").unwrap().unwrap();
        assert_eq!(entry.value, "value");
    }

    #[test]
    fn cache_miss_returns_none() {
        let store = InMemoryCacheStore::new();
        let result = store.check("missing").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn cache_clear_removes_all_entries() {
        let store = InMemoryCacheStore::new();
        store.write("k1", "v1", 3600, "h1").unwrap();
        store.write("k2", "v2", 3600, "h2").unwrap();
        store.clear().unwrap();
        assert!(store.check("k1").unwrap().is_none());
        assert!(store.check("k2").unwrap().is_none());
    }

    #[test]
    fn expired_entry_returns_none() {
        let store = InMemoryCacheStore::new();
        // ttl_secs = 0 means entry expires immediately
        store.write("expired", "val", 0, "hash").unwrap();
        let result = store.check("expired").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn content_hash_is_stored() {
        let store = InMemoryCacheStore::new();
        store.write("key", "val", 3600, "myhash").unwrap();
        let entry = store.check("key").unwrap().unwrap();
        assert_eq!(entry.content_hash, "myhash");
    }
}
