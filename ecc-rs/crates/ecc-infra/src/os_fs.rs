use ecc_ports::fs::{FileSystem, FsError};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Production filesystem adapter backed by `std::fs`.
pub struct OsFileSystem;

impl FileSystem for OsFileSystem {
    fn read_to_string(&self, path: &Path) -> Result<String, FsError> {
        std::fs::read_to_string(path).map_err(|e| FsError::io(path, e))
    }

    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, FsError> {
        std::fs::read(path).map_err(|e| FsError::io(path, e))
    }

    fn write(&self, path: &Path, content: &str) -> Result<(), FsError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| FsError::io(parent, e))?;
        }
        std::fs::write(path, content).map_err(|e| FsError::io(path, e))
    }

    fn write_bytes(&self, path: &Path, content: &[u8]) -> Result<(), FsError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| FsError::io(parent, e))?;
        }
        std::fs::write(path, content).map_err(|e| FsError::io(path, e))
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn create_dir_all(&self, path: &Path) -> Result<(), FsError> {
        std::fs::create_dir_all(path).map_err(|e| FsError::io(path, e))
    }

    fn remove_file(&self, path: &Path) -> Result<(), FsError> {
        std::fs::remove_file(path).map_err(|e| FsError::io(path, e))
    }

    fn remove_dir_all(&self, path: &Path) -> Result<(), FsError> {
        std::fs::remove_dir_all(path).map_err(|e| FsError::io(path, e))
    }

    fn copy(&self, from: &Path, to: &Path) -> Result<(), FsError> {
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent).map_err(|e| FsError::io(parent, e))?;
        }
        std::fs::copy(from, to)
            .map(|_| ())
            .map_err(|e| FsError::io(from, e))
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FsError> {
        let entries = std::fs::read_dir(path).map_err(|e| FsError::io(path, e))?;
        let mut paths = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| FsError::io(path, e))?;
            paths.push(entry.path());
        }
        paths.sort();
        Ok(paths)
    }

    fn read_dir_recursive(&self, path: &Path) -> Result<Vec<PathBuf>, FsError> {
        let mut paths = Vec::new();
        for entry in WalkDir::new(path).min_depth(1) {
            let entry = entry.map_err(|e| FsError::Io {
                path: path.to_path_buf(),
                message: e.to_string(),
            })?;
            paths.push(entry.into_path());
        }
        paths.sort();
        Ok(paths)
    }
}
