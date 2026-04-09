//! Backlog store port traits.
//!
//! Defines the abstract boundaries for reading and writing backlog entries,
//! lock files, and the backlog index. Production adapters live in [`ecc_infra`];
//! test doubles live in [`ecc_test_support`].

use ecc_domain::backlog::entry::{BacklogEntry, BacklogError};
use ecc_domain::backlog::lock::LockFile;
use std::path::Path;

/// Port for loading and persisting individual backlog entries.
pub trait BacklogEntryStore: Send + Sync {
    /// Load all entries from `backlog_dir`.
    fn load_entries(&self, backlog_dir: &Path) -> Result<Vec<BacklogEntry>, BacklogError>;

    /// Load a single entry by `id` from `backlog_dir`.
    fn load_entry(&self, backlog_dir: &Path, id: &str) -> Result<BacklogEntry, BacklogError>;

    /// Persist an entry with the given markdown `body` to `backlog_dir`.
    fn save_entry(
        &self,
        backlog_dir: &Path,
        entry: &BacklogEntry,
        body: &str,
    ) -> Result<(), BacklogError>;

    /// Compute the next available entry ID (max existing + 1).
    fn next_id(&self, backlog_dir: &Path) -> Result<String, BacklogError>;
}

/// Port for managing session lock files on backlog entries.
pub trait BacklogLockStore: Send + Sync {
    /// Load the lock for `id`, returning `None` if absent.
    fn load_lock(&self, backlog_dir: &Path, id: &str) -> Result<Option<LockFile>, BacklogError>;

    /// Persist a lock for `id`.
    fn save_lock(&self, backlog_dir: &Path, id: &str, lock: &LockFile) -> Result<(), BacklogError>;

    /// Remove the lock for `id`.
    fn remove_lock(&self, backlog_dir: &Path, id: &str) -> Result<(), BacklogError>;

    /// List all (id, lock) pairs in `backlog_dir`.
    fn list_locks(&self, backlog_dir: &Path) -> Result<Vec<(String, LockFile)>, BacklogError>;
}

/// Port for reading and writing the backlog index file.
pub trait BacklogIndexStore: Send + Sync {
    /// Write `content` as the backlog index at `backlog_dir`.
    fn write_index(&self, backlog_dir: &Path, content: &str) -> Result<(), BacklogError>;

    /// Read the backlog index from `backlog_dir`, returning `None` if absent.
    fn read_index(&self, backlog_dir: &Path) -> Result<Option<String>, BacklogError>;
}
