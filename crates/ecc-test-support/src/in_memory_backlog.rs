use ecc_domain::backlog::entry::{BacklogEntry, BacklogError, replace_frontmatter_status};
use ecc_domain::backlog::lock::LockFile;
use ecc_ports::backlog::{BacklogEntryStore, BacklogIndexStore, BacklogLockStore};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// In-memory implementation of all three backlog port traits for testing.
#[derive(Debug, Clone)]
pub struct InMemoryBacklogRepository {
    entries: Arc<Mutex<Vec<BacklogEntry>>>,
    locks: Arc<Mutex<HashMap<String, LockFile>>>,
    index: Arc<Mutex<Option<String>>>,
    raw_contents: Arc<Mutex<HashMap<String, String>>>,
}

impl InMemoryBacklogRepository {
    /// Create a new empty repository.
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
            locks: Arc::new(Mutex::new(HashMap::new())),
            index: Arc::new(Mutex::new(None)),
            raw_contents: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Seed raw file content for entry `id`.
    pub fn with_raw_content(self, id: &str, content: &str) -> Self {
        self.raw_contents
            .lock()
            .unwrap()
            .insert(id.to_string(), content.to_string());
        self
    }

    /// Seed an entry into the repository.
    pub fn with_entry(self, entry: BacklogEntry) -> Self {
        self.entries.lock().unwrap().push(entry);
        self
    }

    /// Seed a lock into the repository.
    pub fn with_lock(self, id: impl Into<String>, lock: LockFile) -> Self {
        self.locks.lock().unwrap().insert(id.into(), lock);
        self
    }

    /// Seed an index into the repository.
    pub fn with_index(self, content: impl Into<String>) -> Self {
        *self.index.lock().unwrap() = Some(content.into());
        self
    }
}

impl Default for InMemoryBacklogRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl BacklogEntryStore for InMemoryBacklogRepository {
    fn load_entries(&self, _backlog_dir: &Path) -> Result<Vec<BacklogEntry>, BacklogError> {
        Ok(self.entries.lock().unwrap().clone())
    }

    fn load_entry(&self, _backlog_dir: &Path, id: &str) -> Result<BacklogEntry, BacklogError> {
        self.entries
            .lock()
            .unwrap()
            .iter()
            .find(|e| e.id == id)
            .cloned()
            .ok_or_else(|| BacklogError::Io {
                path: id.to_string(),
                message: "entry not found".into(),
            })
    }

    fn save_entry(
        &self,
        _backlog_dir: &Path,
        entry: &BacklogEntry,
        _body: &str,
    ) -> Result<(), BacklogError> {
        let mut entries = self.entries.lock().unwrap();
        if let Some(existing) = entries.iter_mut().find(|e| e.id == entry.id) {
            *existing = entry.clone();
        } else {
            entries.push(entry.clone());
        }
        Ok(())
    }

    fn next_id(&self, _backlog_dir: &Path) -> Result<String, BacklogError> {
        let entries = self.entries.lock().unwrap();
        let max_num = entries
            .iter()
            .filter_map(|e| e.id.strip_prefix("BL-").and_then(|n| n.parse::<u32>().ok()))
            .max()
            .unwrap_or(0);
        Ok(format!("BL-{:03}", max_num + 1))
    }

    fn update_entry_status(
        &self,
        _backlog_dir: &Path,
        id: &str,
        new_status: &str,
    ) -> Result<(), BacklogError> {
        let mut raw_contents = self.raw_contents.lock().unwrap();
        let content = raw_contents
            .get(id)
            .cloned()
            .ok_or_else(|| BacklogError::Io {
                path: id.to_string(),
                message: "entry not found".into(),
            })?;
        let updated = replace_frontmatter_status(&content, new_status)?;
        raw_contents.insert(id.to_string(), updated);
        Ok(())
    }

    fn read_entry_content(&self, _backlog_dir: &Path, id: &str) -> Result<String, BacklogError> {
        self.raw_contents
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| BacklogError::Io {
                path: id.to_string(),
                message: "entry not found".into(),
            })
    }
}

impl BacklogLockStore for InMemoryBacklogRepository {
    fn load_lock(&self, _backlog_dir: &Path, id: &str) -> Result<Option<LockFile>, BacklogError> {
        Ok(self.locks.lock().unwrap().get(id).cloned())
    }

    fn save_lock(
        &self,
        _backlog_dir: &Path,
        id: &str,
        lock: &LockFile,
    ) -> Result<(), BacklogError> {
        self.locks
            .lock()
            .unwrap()
            .insert(id.to_string(), lock.clone());
        Ok(())
    }

    fn remove_lock(&self, _backlog_dir: &Path, id: &str) -> Result<(), BacklogError> {
        self.locks.lock().unwrap().remove(id);
        Ok(())
    }

    fn list_locks(&self, _backlog_dir: &Path) -> Result<Vec<(String, LockFile)>, BacklogError> {
        Ok(self
            .locks
            .lock()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect())
    }
}

impl BacklogIndexStore for InMemoryBacklogRepository {
    fn write_index(&self, _backlog_dir: &Path, content: &str) -> Result<(), BacklogError> {
        *self.index.lock().unwrap() = Some(content.to_string());
        Ok(())
    }

    fn read_index(&self, _backlog_dir: &Path) -> Result<Option<String>, BacklogError> {
        Ok(self.index.lock().unwrap().clone())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use ecc_domain::backlog::entry::BacklogStatus;
    use std::path::Path;

    fn make_entry(id: &str, num: u32) -> BacklogEntry {
        BacklogEntry {
            id: id.to_string(),
            title: format!("Entry {num}"),
            status: BacklogStatus::Open,
            created: "2026-04-07".into(),
            tier: None,
            scope: None,
            target: None,
            target_command: None,
            tags: vec![],
        }
    }

    fn make_lock() -> LockFile {
        LockFile::new("my-worktree".into(), "2026-04-07T10:00:00Z".into()).unwrap()
    }

    #[test]
    fn implements_trait() {
        let repo = InMemoryBacklogRepository::new();
        // verify all three trait objects can be constructed
        let _: &dyn BacklogEntryStore = &repo;
        let _: &dyn BacklogLockStore = &repo;
        let _: &dyn BacklogIndexStore = &repo;
    }

    #[test]
    fn load_entries_returns_seeded() {
        let entry1 = make_entry("BL-001", 1);
        let entry2 = make_entry("BL-002", 2);
        let repo = InMemoryBacklogRepository::new()
            .with_entry(entry1.clone())
            .with_entry(entry2.clone());

        let dir = Path::new("/backlog");
        let entries = repo.load_entries(dir).unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().any(|e| e.id == "BL-001"));
        assert!(entries.iter().any(|e| e.id == "BL-002"));
    }

    #[test]
    fn next_id_sequential() {
        let repo = InMemoryBacklogRepository::new()
            .with_entry(make_entry("BL-001", 1))
            .with_entry(make_entry("BL-005", 5))
            .with_entry(make_entry("BL-003", 3));

        let dir = Path::new("/backlog");
        let next = repo.next_id(dir).unwrap();
        assert_eq!(next, "BL-006");
    }

    #[test]
    fn lock_crud() {
        let lock = make_lock();
        let repo = InMemoryBacklogRepository::new();
        let dir = Path::new("/backlog");

        // Initially no lock
        assert!(repo.load_lock(dir, "BL-001").unwrap().is_none());

        // Save lock
        repo.save_lock(dir, "BL-001", &lock).unwrap();
        let loaded = repo.load_lock(dir, "BL-001").unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().worktree_name, "my-worktree");

        // List locks
        let locks = repo.list_locks(dir).unwrap();
        assert_eq!(locks.len(), 1);

        // Remove lock
        repo.remove_lock(dir, "BL-001").unwrap();
        assert!(repo.load_lock(dir, "BL-001").unwrap().is_none());
    }

    #[test]
    fn index_roundtrip() {
        let repo = InMemoryBacklogRepository::new();
        let dir = Path::new("/backlog");

        // Initially no index
        assert!(repo.read_index(dir).unwrap().is_none());

        // Write then read
        let content = "# Backlog Index\n- BL-001: Test entry";
        repo.write_index(dir, content).unwrap();
        let read = repo.read_index(dir).unwrap();
        assert_eq!(read.as_deref(), Some(content));
    }

    // PC-020: update_entry_status roundtrip
    #[test]
    fn update_entry_status_roundtrip() {
        let raw = "---\nid: BL-042\nstatus: open\ncreated: 2026-01-01\n---\n\n# Body";
        let repo = InMemoryBacklogRepository::new().with_raw_content("BL-042", raw);
        let dir = Path::new("/backlog");

        repo.update_entry_status(dir, "BL-042", "in-progress")
            .unwrap();

        let updated = repo.read_entry_content(dir, "BL-042").unwrap();
        assert!(updated.contains("status: in-progress"));
        assert!(!updated.contains("status: open"));
    }
}
