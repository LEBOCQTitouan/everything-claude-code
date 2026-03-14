use std::path::{Path, PathBuf};

/// Port for filesystem operations.
/// Production: wraps `std::fs`. Tests: in-memory HashMap.
pub trait FileSystem: Send + Sync {
    fn read_to_string(&self, path: &Path) -> Result<String, FsError>;
    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, FsError>;
    fn write(&self, path: &Path, content: &str) -> Result<(), FsError>;
    fn write_bytes(&self, path: &Path, content: &[u8]) -> Result<(), FsError>;
    fn exists(&self, path: &Path) -> bool;
    fn is_dir(&self, path: &Path) -> bool;
    fn is_file(&self, path: &Path) -> bool;
    fn create_dir_all(&self, path: &Path) -> Result<(), FsError>;
    fn remove_file(&self, path: &Path) -> Result<(), FsError>;
    fn remove_dir_all(&self, path: &Path) -> Result<(), FsError>;
    fn copy(&self, from: &Path, to: &Path) -> Result<(), FsError>;
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FsError>;
    fn read_dir_recursive(&self, path: &Path) -> Result<Vec<PathBuf>, FsError>;
}

#[derive(Debug, thiserror::Error)]
pub enum FsError {
    #[error("file not found: {0}")]
    NotFound(PathBuf),

    #[error("permission denied: {0}")]
    PermissionDenied(PathBuf),

    #[error("I/O error on {path}: {message}")]
    Io { path: PathBuf, message: String },
}

impl FsError {
    pub fn io(path: &Path, err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => Self::NotFound(path.to_path_buf()),
            std::io::ErrorKind::PermissionDenied => Self::PermissionDenied(path.to_path_buf()),
            _ => Self::Io {
                path: path.to_path_buf(),
                message: err.to_string(),
            },
        }
    }
}
