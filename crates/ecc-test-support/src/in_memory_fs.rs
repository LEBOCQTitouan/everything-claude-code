use ecc_ports::fs::{FileSystem, FsError};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// In-memory filesystem for testing. Thread-safe via Mutex.
#[derive(Debug, Clone)]
pub struct InMemoryFileSystem {
    files: Arc<Mutex<BTreeMap<PathBuf, Vec<u8>>>>,
    dirs: Arc<Mutex<BTreeMap<PathBuf, ()>>>,
}

impl InMemoryFileSystem {
    pub fn new() -> Self {
        Self {
            files: Arc::new(Mutex::new(BTreeMap::new())),
            dirs: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    /// Pre-populate a file for test setup.
    pub fn with_file(self, path: &str, content: &str) -> Self {
        let p = PathBuf::from(path);
        if let Some(parent) = p.parent() {
            self.dirs.lock().unwrap().insert(parent.to_path_buf(), ());
        }
        self.files
            .lock()
            .unwrap()
            .insert(p, content.as_bytes().to_vec());
        self
    }

    /// Pre-populate a directory for test setup.
    pub fn with_dir(self, path: &str) -> Self {
        self.dirs.lock().unwrap().insert(PathBuf::from(path), ());
        self
    }
}

impl Default for InMemoryFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystem for InMemoryFileSystem {
    fn read_to_string(&self, path: &Path) -> Result<String, FsError> {
        let files = self.files.lock().unwrap();
        files
            .get(path)
            .map(|b| String::from_utf8_lossy(b).into_owned())
            .ok_or_else(|| FsError::NotFound(path.to_path_buf()))
    }

    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, FsError> {
        let files = self.files.lock().unwrap();
        files
            .get(path)
            .cloned()
            .ok_or_else(|| FsError::NotFound(path.to_path_buf()))
    }

    fn write(&self, path: &Path, content: &str) -> Result<(), FsError> {
        self.write_bytes(path, content.as_bytes())
    }

    fn write_bytes(&self, path: &Path, content: &[u8]) -> Result<(), FsError> {
        if let Some(parent) = path.parent() {
            self.dirs.lock().unwrap().insert(parent.to_path_buf(), ());
        }
        self.files
            .lock()
            .unwrap()
            .insert(path.to_path_buf(), content.to_vec());
        Ok(())
    }

    fn exists(&self, path: &Path) -> bool {
        let files = self.files.lock().unwrap();
        let dirs = self.dirs.lock().unwrap();
        files.contains_key(path) || dirs.contains_key(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.dirs.lock().unwrap().contains_key(path)
    }

    fn is_file(&self, path: &Path) -> bool {
        self.files.lock().unwrap().contains_key(path)
    }

    fn create_dir_all(&self, path: &Path) -> Result<(), FsError> {
        let mut dirs = self.dirs.lock().unwrap();
        let mut current = path.to_path_buf();
        loop {
            dirs.insert(current.clone(), ());
            match current.parent() {
                Some(p) if p != current => current = p.to_path_buf(),
                _ => break,
            }
        }
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> Result<(), FsError> {
        self.files
            .lock()
            .unwrap()
            .remove(path)
            .map(|_| ())
            .ok_or_else(|| FsError::NotFound(path.to_path_buf()))
    }

    fn remove_dir_all(&self, path: &Path) -> Result<(), FsError> {
        let prefix = path.to_path_buf();
        self.files
            .lock()
            .unwrap()
            .retain(|k, _| !k.starts_with(&prefix));
        self.dirs
            .lock()
            .unwrap()
            .retain(|k, _| !k.starts_with(&prefix));
        Ok(())
    }

    fn copy(&self, from: &Path, to: &Path) -> Result<(), FsError> {
        let content = self.read_bytes(from)?;
        self.write_bytes(to, &content)
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FsError> {
        let files = self.files.lock().unwrap();
        let dirs = self.dirs.lock().unwrap();
        let mut entries: Vec<PathBuf> = Vec::new();

        for key in files.keys() {
            if key.parent() == Some(path) {
                entries.push(key.clone());
            }
        }
        for key in dirs.keys() {
            if key.parent() == Some(path) && !entries.contains(key) {
                entries.push(key.clone());
            }
        }

        entries.sort();
        Ok(entries)
    }

    fn read_dir_recursive(&self, path: &Path) -> Result<Vec<PathBuf>, FsError> {
        let files = self.files.lock().unwrap();
        let dirs = self.dirs.lock().unwrap();
        let mut entries: Vec<PathBuf> = files
            .keys()
            .filter(|k| k.starts_with(path) && *k != path)
            .cloned()
            .collect();
        for dir in dirs.keys() {
            if dir.starts_with(path) && dir != path && !entries.contains(dir) {
                entries.push(dir.clone());
            }
        }
        entries.sort();
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_and_read() {
        let fs = InMemoryFileSystem::new();
        fs.write(Path::new("/tmp/test.txt"), "hello").unwrap();
        assert_eq!(
            fs.read_to_string(Path::new("/tmp/test.txt")).unwrap(),
            "hello"
        );
    }

    #[test]
    fn not_found() {
        let fs = InMemoryFileSystem::new();
        assert!(fs.read_to_string(Path::new("/nope")).is_err());
    }

    #[test]
    fn builder_pattern() {
        let fs = InMemoryFileSystem::new()
            .with_file("/a.txt", "content a")
            .with_file("/b.txt", "content b");
        assert!(fs.exists(Path::new("/a.txt")));
        assert!(fs.exists(Path::new("/b.txt")));
    }

    #[test]
    fn copy_file() {
        let fs = InMemoryFileSystem::new().with_file("/src.txt", "data");
        fs.copy(Path::new("/src.txt"), Path::new("/dst.txt"))
            .unwrap();
        assert_eq!(fs.read_to_string(Path::new("/dst.txt")).unwrap(), "data");
    }

    #[test]
    fn remove_dir_all_clears_subtree() {
        let fs = InMemoryFileSystem::new()
            .with_file("/dir/a.txt", "a")
            .with_file("/dir/sub/b.txt", "b")
            .with_file("/other/c.txt", "c");
        fs.remove_dir_all(Path::new("/dir")).unwrap();
        assert!(!fs.exists(Path::new("/dir/a.txt")));
        assert!(!fs.exists(Path::new("/dir/sub/b.txt")));
        assert!(fs.exists(Path::new("/other/c.txt")));
    }

    #[test]
    fn create_symlink_inserts_and_removes_file() {
        let fs = InMemoryFileSystem::new().with_file("/real.txt", "data");
        fs.create_symlink(Path::new("/real.txt"), Path::new("/link.txt"))
            .unwrap();
        // symlink exists in the symlinks map
        assert!(fs.is_symlink(Path::new("/link.txt")));
        // creating a symlink on an existing file path removes that file entry
        fs.write(Path::new("/file-then-link.txt"), "content")
            .unwrap();
        assert!(fs.is_file(Path::new("/file-then-link.txt")));
        fs.create_symlink(Path::new("/real.txt"), Path::new("/file-then-link.txt"))
            .unwrap();
        assert!(!fs.is_file(Path::new("/file-then-link.txt")));
        assert!(fs.is_symlink(Path::new("/file-then-link.txt")));
    }

    #[test]
    fn read_symlink() {
        let fs = InMemoryFileSystem::new();
        fs.create_symlink(Path::new("/target.txt"), Path::new("/link.txt"))
            .unwrap();
        let target = fs.read_symlink(Path::new("/link.txt")).unwrap();
        assert_eq!(target, PathBuf::from("/target.txt"));
        // reading a non-existent symlink returns NotFound
        let err = fs.read_symlink(Path::new("/no-such-link")).unwrap_err();
        assert!(matches!(err, FsError::NotFound(_)));
    }

    #[test]
    fn is_symlink_detection() {
        let fs = InMemoryFileSystem::new()
            .with_file("/real.txt", "data")
            .with_symlink("/link.txt", "/real.txt");
        assert!(fs.is_symlink(Path::new("/link.txt")));
        assert!(!fs.is_symlink(Path::new("/real.txt")));
        assert!(!fs.is_symlink(Path::new("/absent")));
    }

    #[test]
    fn exists_includes_symlinks() {
        let fs = InMemoryFileSystem::new().with_symlink("/link.txt", "/target.txt");
        assert!(fs.exists(Path::new("/link.txt")));
        assert!(!fs.exists(Path::new("/absent")));
    }

    #[test]
    fn remove_file_removes_symlink() {
        let fs = InMemoryFileSystem::new().with_symlink("/link.txt", "/target.txt");
        assert!(fs.is_symlink(Path::new("/link.txt")));
        fs.remove_file(Path::new("/link.txt")).unwrap();
        assert!(!fs.is_symlink(Path::new("/link.txt")));
        assert!(!fs.exists(Path::new("/link.txt")));
    }

    #[test]
    fn with_symlink_builder() {
        let fs = InMemoryFileSystem::new()
            .with_file("/real.txt", "hello")
            .with_symlink("/link.txt", "/real.txt");
        assert!(fs.is_symlink(Path::new("/link.txt")));
        let target = fs.read_symlink(Path::new("/link.txt")).unwrap();
        assert_eq!(target, PathBuf::from("/real.txt"));
    }
}
