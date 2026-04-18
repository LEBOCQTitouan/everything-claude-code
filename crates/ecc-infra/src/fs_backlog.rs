//! Filesystem adapter for backlog store ports.
//!
//! Implements [`BacklogEntryStore`], [`BacklogLockStore`], and [`BacklogIndexStore`]
//! via the [`FileSystem`] port, enabling real filesystem I/O and in-memory testing.

use ecc_domain::backlog::entry::{
    BacklogEntry, BacklogError, extract_id_from_filename, matches_backlog_filename,
    parse_frontmatter, replace_frontmatter_status,
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

    /// Locate a backlog entry file by ID prefix (e.g., `BL-042-*` or `BL-042.md`).
    fn find_entry_path(
        &self,
        backlog_dir: &Path,
        id: &str,
    ) -> Result<std::path::PathBuf, BacklogError> {
        let paths = self
            .fs
            .read_dir(backlog_dir)
            .map_err(|e| BacklogError::Io {
                path: backlog_dir.display().to_string(),
                message: e.to_string(),
            })?;
        for path in &paths {
            let filename = match path.file_name() {
                Some(name) => name.to_string_lossy().to_string(),
                None => continue,
            };
            let matched = match id.strip_prefix("BL-").and_then(|s| s.parse::<u32>().ok()) {
                Some(n) => matches_backlog_filename(&filename, n),
                None => filename.starts_with(&format!("{id}-")) || filename == format!("{id}.md"),
            };
            if matched {
                return Ok(path.clone());
            }
        }
        Err(BacklogError::Io {
            path: backlog_dir.display().to_string(),
            message: format!("entry {id} not found"),
        })
    }
}

impl BacklogEntryStore for FsBacklogRepository<'_> {
    fn load_entries(&self, backlog_dir: &Path) -> Result<Vec<BacklogEntry>, BacklogError> {
        let paths = self
            .fs
            .read_dir(backlog_dir)
            .map_err(|e| BacklogError::Io {
                path: backlog_dir.display().to_string(),
                message: e.to_string(),
            })?;

        let mut entries = Vec::new();
        for path in &paths {
            let filename = match path.file_name() {
                Some(name) => name.to_string_lossy().to_string(),
                None => continue,
            };
            if extract_id_from_filename(&filename).is_none() {
                continue;
            }
            let content = match self.fs.read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!("skipping {filename}: {e}");
                    continue;
                }
            };
            match parse_frontmatter(&content) {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    tracing::warn!("skipping {filename}: {e}");
                }
            }
        }
        Ok(entries)
    }

    fn load_entry(&self, backlog_dir: &Path, id: &str) -> Result<BacklogEntry, BacklogError> {
        let path = self.find_entry_path(backlog_dir, id)?;
        let content = self.fs.read_to_string(&path).map_err(|e| BacklogError::Io {
            path: path.display().to_string(),
            message: e.to_string(),
        })?;
        parse_frontmatter(&content)
    }

    fn save_entry(
        &self,
        backlog_dir: &Path,
        entry: &BacklogEntry,
        body: &str,
    ) -> Result<(), BacklogError> {
        let slug = entry.title.to_lowercase().replace(' ', "-");
        let filename = format!("{}-{slug}.md", entry.id);
        let path = backlog_dir.join(&filename);
        let yaml = serde_saphyr::to_string(entry)
            .map_err(|e| BacklogError::MalformedYaml(e.to_string()))?;
        let content = format!("---\n{yaml}---\n{body}");
        self.fs
            .write(&path, &content)
            .map_err(|e| BacklogError::Io {
                path: path.display().to_string(),
                message: e.to_string(),
            })
    }

    fn next_id(&self, backlog_dir: &Path) -> Result<String, BacklogError> {
        if !self.fs.is_dir(backlog_dir) {
            return Err(BacklogError::DirectoryNotFound(backlog_dir.to_path_buf()));
        }
        let paths = self
            .fs
            .read_dir(backlog_dir)
            .map_err(|e| BacklogError::Io {
                path: backlog_dir.display().to_string(),
                message: e.to_string(),
            })?;

        let max_id = paths
            .iter()
            .filter_map(|p| p.file_name())
            .filter_map(|name| extract_id_from_filename(&name.to_string_lossy()))
            .max()
            .unwrap_or(0);

        Ok(format!("BL-{:03}", max_id + 1))
    }

    fn update_entry_status(
        &self,
        backlog_dir: &Path,
        id: &str,
        new_status: &str,
    ) -> Result<(), BacklogError> {
        let path = self.find_entry_path(backlog_dir, id)?;
        let original = self.fs.read_to_string(&path).map_err(|e| BacklogError::Io {
            path: path.display().to_string(),
            message: e.to_string(),
        })?;
        let updated = replace_frontmatter_status(&original, new_status)?;
        if updated == original {
            return Ok(());
        }
        let tmp_path = path.with_extension("md.tmp");
        self.fs
            .write(&tmp_path, &updated)
            .map_err(|e| BacklogError::Io {
                path: tmp_path.display().to_string(),
                message: e.to_string(),
            })?;
        if let Err(e) = self.fs.rename(&tmp_path, &path) {
            let _ = self.fs.remove_file(&tmp_path);
            return Err(BacklogError::Io {
                path: path.display().to_string(),
                message: format!("failed to rename tmp file: {e}"),
            });
        }
        Ok(())
    }

    fn read_entry_content(&self, backlog_dir: &Path, id: &str) -> Result<String, BacklogError> {
        let path = self.find_entry_path(backlog_dir, id)?;
        self.fs.read_to_string(&path).map_err(|e| BacklogError::Io {
            path: path.display().to_string(),
            message: e.to_string(),
        })
    }
}

impl BacklogLockStore for FsBacklogRepository<'_> {
    fn load_lock(&self, backlog_dir: &Path, id: &str) -> Result<Option<LockFile>, BacklogError> {
        let lock_path = self.lock_path(backlog_dir, id);
        match self.fs.read_to_string(&lock_path) {
            Ok(content) => LockFile::parse(&content).map(Some),
            Err(ecc_ports::fs::FsError::NotFound(_)) => Ok(None),
            Err(e) => Err(BacklogError::Io {
                path: lock_path.display().to_string(),
                message: e.to_string(),
            }),
        }
    }

    fn save_lock(&self, backlog_dir: &Path, id: &str, lock: &LockFile) -> Result<(), BacklogError> {
        let locks_dir = backlog_dir.join(".locks");
        self.fs
            .create_dir_all(&locks_dir)
            .map_err(|e| BacklogError::Io {
                path: locks_dir.display().to_string(),
                message: e.to_string(),
            })?;
        let lock_path = self.lock_path(backlog_dir, id);
        self.fs
            .write(&lock_path, &lock.format())
            .map_err(|e| BacklogError::Io {
                path: lock_path.display().to_string(),
                message: e.to_string(),
            })
    }

    fn remove_lock(&self, backlog_dir: &Path, id: &str) -> Result<(), BacklogError> {
        let lock_path = self.lock_path(backlog_dir, id);
        match self.fs.remove_file(&lock_path) {
            Ok(()) => Ok(()),
            Err(ecc_ports::fs::FsError::NotFound(_)) => Ok(()),
            Err(e) => Err(BacklogError::Io {
                path: lock_path.display().to_string(),
                message: e.to_string(),
            }),
        }
    }

    fn list_locks(&self, backlog_dir: &Path) -> Result<Vec<(String, LockFile)>, BacklogError> {
        let locks_dir = backlog_dir.join(".locks");
        if !self.fs.is_dir(&locks_dir) {
            return Ok(vec![]);
        }
        let paths = self.fs.read_dir(&locks_dir).map_err(|e| BacklogError::Io {
            path: locks_dir.display().to_string(),
            message: e.to_string(),
        })?;

        let mut result = Vec::new();
        for path in &paths {
            let filename = match path.file_name() {
                Some(name) => name.to_string_lossy().to_string(),
                None => continue,
            };
            let id = match filename.strip_suffix(".lock") {
                Some(id) => id.to_owned(),
                None => continue,
            };
            let content = self.fs.read_to_string(path).map_err(|e| BacklogError::Io {
                path: path.display().to_string(),
                message: e.to_string(),
            })?;
            match LockFile::parse(&content) {
                Ok(lock) => result.push((id, lock)),
                Err(e) => tracing::warn!("skipping malformed lock {filename}: {e}"),
            }
        }
        Ok(result)
    }
}

impl BacklogIndexStore for FsBacklogRepository<'_> {
    fn write_index(&self, backlog_dir: &Path, content: &str) -> Result<(), BacklogError> {
        let index_path = backlog_dir.join("BACKLOG.md");
        let tmp_path = backlog_dir.join("BACKLOG.md.tmp");
        self.fs
            .write(&tmp_path, content)
            .map_err(|e| BacklogError::Io {
                path: tmp_path.display().to_string(),
                message: format!("failed to write temp file: {e}"),
            })?;
        if let Err(e) = self.fs.rename(&tmp_path, &index_path) {
            let _ = self.fs.remove_file(&tmp_path);
            return Err(BacklogError::Io {
                path: tmp_path.display().to_string(),
                message: format!("failed to rename temp file: {e}"),
            });
        }
        Ok(())
    }

    fn read_index(&self, backlog_dir: &Path) -> Result<Option<String>, BacklogError> {
        let index_path = backlog_dir.join("BACKLOG.md");
        match self.fs.read_to_string(&index_path) {
            Ok(content) => Ok(Some(content)),
            Err(ecc_ports::fs::FsError::NotFound(_)) => Ok(None),
            Err(e) => Err(BacklogError::Io {
                path: index_path.display().to_string(),
                message: e.to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::InMemoryFileSystem;

    const VALID_ENTRY: &str =
        "---\nid: BL-001\ntitle: First entry\nstatus: open\ncreated: 2026-04-07\n---\n# Body";
    const VALID_ENTRY_2: &str =
        "---\nid: BL-002\ntitle: Second entry\nstatus: open\ncreated: 2026-04-07\n---\n# Body 2";

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
        repo.save_lock(Path::new("/backlog"), "BL-042", &lock)
            .unwrap();
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
        let lock = repo.load_lock(Path::new("/backlog"), "BL-001").unwrap();
        assert!(lock.is_some());
        repo.remove_lock(Path::new("/backlog"), "BL-001").unwrap();
        let lock = repo.load_lock(Path::new("/backlog"), "BL-001").unwrap();
        assert!(lock.is_none());
    }

    // PC-021: update_entry_status performs atomic write (tmp+rename)
    #[test]
    fn update_entry_status_atomic_write() {
        let entry_content =
            "---\nid: BL-001\ntitle: First entry\nstatus: open\ncreated: 2026-04-07\n---\n# Body";
        let fs = InMemoryFileSystem::new()
            .with_dir("/backlog")
            .with_file("/backlog/BL-001-first-entry.md", entry_content);
        let repo = FsBacklogRepository::new(&fs);
        repo.update_entry_status(Path::new("/backlog"), "BL-001", "implemented")
            .unwrap();
        // The file should be updated
        let updated = fs
            .read_to_string(Path::new("/backlog/BL-001-first-entry.md"))
            .unwrap();
        assert!(updated.contains("status: implemented"));
        // No .tmp file should remain
        assert!(!fs.exists(Path::new("/backlog/BL-001-first-entry.md.tmp")));
    }

    // PC-022: update_entry_status preserves body character-for-character
    #[test]
    fn update_entry_status_preserves_body() {
        let body = "# Complex Body\n\nLine with special chars: &, <, >, \"quotes\", 'apostrophes'\n\n## Section\n\nMore content here.\n";
        let entry_content = format!(
            "---\nid: BL-042\ntitle: Complex entry\nstatus: open\ncreated: 2026-04-07\n---\n{body}"
        );
        let fs = InMemoryFileSystem::new()
            .with_dir("/backlog")
            .with_file("/backlog/BL-042-complex-entry.md", &entry_content);
        let repo = FsBacklogRepository::new(&fs);
        repo.update_entry_status(Path::new("/backlog"), "BL-042", "implemented")
            .unwrap();
        let updated = fs
            .read_to_string(Path::new("/backlog/BL-042-complex-entry.md"))
            .unwrap();
        // Status updated
        assert!(updated.contains("status: implemented"));
        // Body preserved exactly
        assert!(updated.ends_with(body), "body not preserved: {updated:?}");
    }

    #[test]
    fn write_index_atomic() {
        let fs = InMemoryFileSystem::new().with_dir("/backlog");
        let repo = FsBacklogRepository::new(&fs);
        repo.write_index(Path::new("/backlog"), "# Backlog Index\n")
            .unwrap();
        let content = repo.read_index(Path::new("/backlog")).unwrap();
        assert_eq!(content, Some("# Backlog Index\n".to_string()));
        // The temp file should not exist after successful write
        assert!(!fs.exists(Path::new("/backlog/BACKLOG.md.tmp")));
    }
}
