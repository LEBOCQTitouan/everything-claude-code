//! Port trait for the three-tier memory store.

use std::collections::HashMap;

use ecc_domain::memory::{MemoryEntry, MemoryId, MemoryStats, MemoryTier};

/// Errors returned by [`MemoryStore`] operations.
#[derive(Debug, thiserror::Error)]
pub enum MemoryStoreError {
    /// A requested entry was not found.
    #[error("memory entry not found: {0}")]
    NotFound(MemoryId),
    /// A database-level error (query failure, I/O, etc.).
    #[error("database error: {0}")]
    Database(String),
    /// The database is structurally corrupted.
    #[error("database corruption: {0}")]
    Corruption(String),
}

/// Port trait for persistent memory storage.
///
/// Production adapter: `SqliteMemoryStore` in `ecc-infra`.
/// Test double: `InMemoryMemoryStore` in `ecc-test-support`.
pub trait MemoryStore: Send + Sync {
    /// Insert a new entry; returns the assigned [`MemoryId`].
    fn insert(&self, entry: &MemoryEntry) -> Result<MemoryId, MemoryStoreError>;

    /// Retrieve an entry by id; returns [`MemoryStoreError::NotFound`] if absent.
    fn get(&self, id: MemoryId) -> Result<MemoryEntry, MemoryStoreError>;

    /// Overwrite an existing entry (matched by `entry.id`).
    fn update(&self, entry: &MemoryEntry) -> Result<(), MemoryStoreError>;

    /// Permanently delete an entry and its FTS index rows.
    fn delete(&self, id: MemoryId) -> Result<(), MemoryStoreError>;

    /// Full-text search; returns BM25-ranked results up to `limit`.
    fn search_fts(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>, MemoryStoreError>;

    /// List entries with optional tier and tag filters, scoped to an optional project.
    fn list_filtered(
        &self,
        tier: Option<MemoryTier>,
        tag: Option<&str>,
        project_id: Option<&str>,
    ) -> Result<Vec<MemoryEntry>, MemoryStoreError>;

    /// Return the `limit` most recently updated entries.
    fn list_recent(&self, limit: usize) -> Result<Vec<MemoryEntry>, MemoryStoreError>;

    /// Count entries grouped by tier.
    fn count_by_tier(&self) -> Result<HashMap<MemoryTier, usize>, MemoryStoreError>;

    /// Aggregate statistics (counts, size, age range).
    fn stats(&self) -> Result<MemoryStats, MemoryStoreError>;

    /// Find the entry whose `source_path` matches `path`, if any.
    fn get_by_source_path(&self, path: &str) -> Result<Option<MemoryEntry>, MemoryStoreError>;

    /// Delete entries marked stale and older than `days`; returns deleted entries.
    fn delete_stale_older_than(&self, days: u64) -> Result<Vec<MemoryEntry>, MemoryStoreError>;

    /// Atomically merge two entries: keep `keep_id` with `merged_content`, remove `remove_id`.
    fn merge_entries(
        &self,
        keep_id: MemoryId,
        remove_id: MemoryId,
        merged_content: &str,
    ) -> Result<(), MemoryStoreError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // PC-021: MemoryStore trait compiles with all required methods
    #[test]
    fn test_memory_store_error_not_found_display() {
        let err = MemoryStoreError::NotFound(MemoryId(42));
        assert!(err.to_string().contains("42"));
    }

    // PC-022: MemoryStoreError enum covers NotFound, Database, Corruption variants
    #[test]
    fn test_memory_store_error_database_display() {
        let err = MemoryStoreError::Database("connection failed".to_owned());
        assert!(err.to_string().contains("connection failed"));
    }

    #[test]
    fn test_memory_store_error_corruption_display() {
        let err = MemoryStoreError::Corruption("integrity check failed".to_owned());
        assert!(err.to_string().contains("integrity check failed"));
    }

    #[test]
    fn test_memory_store_error_not_found_variant() {
        let id = MemoryId(99);
        let err = MemoryStoreError::NotFound(id);
        assert!(matches!(err, MemoryStoreError::NotFound(_)));
    }

    #[test]
    fn test_memory_store_error_all_variants_are_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MemoryStoreError>();
    }
}
