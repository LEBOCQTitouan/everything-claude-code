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

    #[cfg(unix)]
    fn create_symlink(&self, target: &Path, link: &Path) -> Result<(), FsError> {
        if std::fs::symlink_metadata(link).is_ok() {
            if link.is_dir() && !self.is_symlink(link) {
                std::fs::remove_dir_all(link).map_err(|e| FsError::io(link, e))?;
            } else {
                std::fs::remove_file(link).map_err(|e| FsError::io(link, e))?;
            }
        }
        std::os::unix::fs::symlink(target, link).map_err(|e| FsError::io(link, e))
    }

    #[cfg(not(unix))]
    fn create_symlink(&self, _target: &Path, link: &Path) -> Result<(), FsError> {
        Err(FsError::Unsupported(format!(
            "symlinks are not supported on this platform: {}",
            link.display()
        )))
    }

    fn read_symlink(&self, link: &Path) -> Result<PathBuf, FsError> {
        std::fs::read_link(link).map_err(|e| FsError::io(link, e))
    }

    fn is_symlink(&self, path: &Path) -> bool {
        std::fs::symlink_metadata(path)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
    }
}
