//! Filesystem adapter for backlog store ports.
//!
//! Implements [`BacklogEntryStore`], [`BacklogLockStore`], and [`BacklogIndexStore`]
//! via the [`FileSystem`] port, enabling real filesystem I/O and in-memory testing.

use ecc_domain::backlog::entry::{
    BacklogEntry, BacklogError, extract_id_from_filename, parse_frontmatter,
};
use ecc_domain::backlog::lock::LockFile;
use ecc_ports::backlog::{BacklogEntryStore, BacklogIndexStore, BacklogLockStore};
use ecc_ports::fs::FileSystem;
use std::path::Path;

/// Filesystem-backed backlog repository.
///
/// Wraps a [`FileSystem`] port reference; use [`ecc_test_support::InMemoryFileSystem`]
/// in tests and `OsFileSystem` in production.
pub struct FsBacklogRepository<'a> {
    fs: &'a dyn FileSystem,
}

impl<'a> FsBacklogRepository<'a> {
    /// Create a new repository backed by `fs`.
    pub fn new(fs: &'a dyn FileSystem) -> Self {
        Self { fs }
    }

    fn lock_path(&self, backlog_dir: &Path, id: &str) -> std::path::PathBuf {
        backlog_dir.join(".locks").join(format!("{id}.lock"))
    }
}

impl BacklogEntryStore for FsBacklogRepository<'_> {
    fn load_entries(&self, _backlog_dir: &Path) -> Result<Vec<BacklogEntry>, BacklogError> {
        todo!("implement load_entries")
    }

    fn load_entry(&self, _backlog_dir: &Path, _id: &str) -> Result<BacklogEntry, BacklogError> {
        todo!("implement load_entry")
    }

    fn save_entry(
        &self,
        _backlog_dir: &Path,
        _entry: &BacklogEntry,
        _body: &str,
    ) -> Result<(), BacklogError> {
        todo!("implement save_entry")
    }

    fn next_id(&self, _backlog_dir: &Path) -> Result<String, BacklogError> {
        todo!("implement next_id")
    }
}

impl BacklogLockStore for FsBacklogRepository<'_> {
    fn load_lock(&self, _backlog_dir: &Path, _id: &str) -> Result<Option<LockFile>, BacklogError> {
        todo!("implement load_lock")
    }

    fn save_lock(
        &self,
        _backlog_dir: &Path,
        _id: &str,
        _lock: &LockFile,
    ) -> Result<(), BacklogError> {
        todo!("implement save_lock")
    }

    fn remove_lock(&self, _backlog_dir: &Path, _id: &str) -> Result<(), BacklogError> {
        todo!("implement remove_lock")
    }

    fn list_locks(&self, _backlog_dir: &Path) -> Result<Vec<(String, LockFile)>, BacklogError> {
        todo!("implement list_locks")
    }
}

impl BacklogIndexStore for FsBacklogRepository<'_> {
    fn write_index(&self, _backlog_dir: &Path, _content: &str) -> Result<(), BacklogError> {
        todo!("implement write_index")
    }

    fn read_index(&self, _backlog_dir: &Path) -> Result<Option<String>, BacklogError> {
        todo!("implement read_index")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::InMemoryFileSystem;

    const VALID_ENTRY: &str = "---\nid: BL-001\ntitle: First entry\nstatus: open\ncreated: 2026-04-07\n---\n# Body";
    const VALID_ENTRY_2: &str = "---\nid: BL-002\ntitle: Second entry\nstatus: open\ncreated: 2026-04-07\n---\n# Body 2";

    #[test]
    fn load_entries_reads_bl_files() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/backlog")
            .with_file("/backlog/BL-001-first-entry.md", VALID_ENTRY)
            .with_file("/backlog/BL-002-second-entry.md", VALID_ENTRY_2);
        let repo = FsBacklogRepository::new(&fs);
        let entries = repo.load_entries(Path::new("/backlog")).unwrap();
        assert_eq!(entries.len(), 2);
        let ids: Vec<&str> = entries.iter().map(|e| e.id.as_str()).collect();
        assert!(ids.contains(&"BL-001"));
        assert!(ids.contains(&"BL-002"));
    }

    #[test]
    fn load_entries_skips_invalid() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/backlog")
            .with_file("/backlog/BL-001-valid.md", VALID_ENTRY)
            .with_file("/backlog/README.md", "# not a BL file")
            .with_file("/backlog/BL-002-malformed.md", "no frontmatter here");
        let repo = FsBacklogRepository::new(&fs);
        let entries = repo.load_entries(Path::new("/backlog")).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "BL-001");
    }

    #[test]
    fn next_id_computes_max_plus_one() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/backlog")
            .with_file("/backlog/BL-001-first.md", VALID_ENTRY)
            .with_file("/backlog/BL-007-seventh.md", VALID_ENTRY_2)
            .with_file("/backlog/README.md", "# index");
        let repo = FsBacklogRepository::new(&fs);
        let next = repo.next_id(Path::new("/backlog")).unwrap();
        assert_eq!(next, "BL-008");
    }

    #[test]
    fn load_lock_parses() {
        let lock_content = "my-worktree\n2026-04-07T14:10:39Z\n";
        let fs = InMemoryFileSystem::new()
            .with_dir("/backlog")
            .with_dir("/backlog/.locks")
            .with_file("/backlog/.locks/BL-001.lock", lock_content);
        let repo = FsBacklogRepository::new(&fs);
        let lock = repo.load_lock(Path::new("/backlog"), "BL-001").unwrap();
        assert!(lock.is_some());
        let lock = lock.unwrap();
        assert_eq!(lock.worktree_name, "my-worktree");
        assert_eq!(lock.timestamp, "2026-04-07T14:10:39Z");
    }

    #[test]
    fn lock_roundtrip() {
        let fs = InMemoryFileSystem::new().with_dir("/backlog");
        let repo = FsBacklogRepository::new(&fs);
        let lock = LockFile::new("my-worktree".into(), "2026-04-07T14:10:39Z".into()).unwrap();
        repo.save_lock(Path::new("/backlog"), "BL-042", &lock).unwrap();
        let loaded = repo.load_lock(Path::new("/backlog"), "BL-042").unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.worktree_name, "my-worktree");
        assert_eq!(loaded.timestamp, "2026-04-07T14:10:39Z");
    }

    #[test]
    fn remove_lock_deletes() {
        let lock_content = "my-worktree\n2026-04-07T14:10:39Z\n";
        let fs = InMemoryFileSystem::new()
            .with_dir("/backlog")
            .with_dir("/backlog/.locks")
            .with_file("/backlog/.locks/BL-001.lock", lock_content);
        let repo = FsBacklogRepository::new(&fs);
        // Verify it exists first
        let lock = repo.load_lock(Path::new("/backlog"), "BL-001").unwrap();
        assert!(lock.is_some());
        // Remove it
        repo.remove_lock(Path::new("/backlog"), "BL-001").unwrap();
        // Verify it's gone
        let lock = repo.load_lock(Path::new("/backlog"), "BL-001").unwrap();
        assert!(lock.is_none());
    }

    #[test]
    fn write_index_atomic() {
        let fs = InMemoryFileSystem::new().with_dir("/backlog");
        let repo = FsBacklogRepository::new(&fs);
        repo.write_index(Path::new("/backlog"), "# Backlog Index\n").unwrap();
        let content = repo.read_index(Path::new("/backlog")).unwrap();
        assert_eq!(content, Some("# Backlog Index\n".to_string()));
        // The temp file should not exist after successful write
        assert!(!fs.exists(Path::new("/backlog/BACKLOG.md.tmp")));
    }
}
