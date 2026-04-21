//! Port trait for audit result caching.

/// A cached audit entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheEntry {
    /// The cached value (audit findings as string).
    pub value: String,
    /// Unix timestamp when the entry was created.
    pub created_at: u64,
    /// Time-to-live in seconds.
    pub ttl_secs: u64,
    /// SHA-256 content hash of the domain directory at cache time.
    pub content_hash: String,
}

/// Errors from cache operations.
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(String),
    /// A parse error occurred.
    #[error("parse error: {0}")]
    Parse(String),
}

/// Port trait for audit cache operations.
///
/// # Pattern
///
/// Port \[Hexagonal Architecture\]
pub trait CacheStore: Send + Sync {
    /// Check if a valid (non-expired) cache entry exists for the key.
    fn check(&self, key: &str) -> Result<Option<CacheEntry>, CacheError>;
    /// Write a cache entry with TTL.
    fn write(
        &self,
        key: &str,
        value: &str,
        ttl_secs: u64,
        content_hash: &str,
    ) -> Result<(), CacheError>;
    /// Clear all cache entries.
    fn clear(&self) -> Result<(), CacheError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_entry_fields() {
        let entry = CacheEntry {
            value: "test".to_string(),
            created_at: 1000,
            ttl_secs: 3600,
            content_hash: "abc123".to_string(),
        };
        assert_eq!(entry.value, "test");
        assert_eq!(entry.ttl_secs, 3600);
    }
}
