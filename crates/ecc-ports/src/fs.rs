use std::path::{Path, PathBuf};

/// Port for filesystem operations.
/// Production: wraps `std::fs`. Tests: in-memory HashMap.
pub trait FileSystem: Send + Sync {
    /// Read the entire contents of a file as a UTF-8 string.
    fn read_to_string(&self, path: &Path) -> Result<String, FsError>;
    /// Read the entire contents of a file as raw bytes.
    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, FsError>;
    /// Write a UTF-8 string to a file, creating or truncating it.
    fn write(&self, path: &Path, content: &str) -> Result<(), FsError>;
    /// Write raw bytes to a file, creating or truncating it.
    fn write_bytes(&self, path: &Path, content: &[u8]) -> Result<(), FsError>;
    /// Return `true` if a path exists (file, dir, or symlink).
    fn exists(&self, path: &Path) -> bool;
    /// Return `true` if the path is an existing directory.
    fn is_dir(&self, path: &Path) -> bool;
    /// Return `true` if the path is an existing regular file.
    fn is_file(&self, path: &Path) -> bool;
    /// Recursively create all missing directories in the path.
    fn create_dir_all(&self, path: &Path) -> Result<(), FsError>;
    /// Remove a single file.
    fn remove_file(&self, path: &Path) -> Result<(), FsError>;
    /// Recursively remove a directory and all its contents.
    fn remove_dir_all(&self, path: &Path) -> Result<(), FsError>;
    /// Copy a file from `from` to `to`, overwriting if `to` exists.
    fn copy(&self, from: &Path, to: &Path) -> Result<(), FsError>;
    /// List direct children of a directory (non-recursive).
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FsError>;
    /// List all files in a directory tree recursively.
    fn read_dir_recursive(&self, path: &Path) -> Result<Vec<PathBuf>, FsError>;
    /// Create a symbolic link at `link` pointing to `target`.
    fn create_symlink(&self, target: &Path, link: &Path) -> Result<(), FsError>;
    /// Read the target of a symbolic link.
    fn read_symlink(&self, link: &Path) -> Result<PathBuf, FsError>;
    /// Return `true` if the path is a symbolic link.
    fn is_symlink(&self, path: &Path) -> bool;
    /// Set UNIX file permissions (mode bits) on a path.
    fn set_permissions(&self, path: &Path, mode: u32) -> Result<(), FsError>;
    /// Return `true` if the file has executable bits set.
    fn is_executable(&self, path: &Path) -> bool;
    /// Rename (move) a file or directory from `from` to `to`.
    fn rename(&self, from: &Path, to: &Path) -> Result<(), FsError>;
    /// Canonicalize a path by resolving symlinks and `..` components.
    ///
    /// In-memory implementations may return the path as-is (already canonical).
    /// Production implementations call `std::fs::canonicalize`.
    fn canonicalize(&self, path: &Path) -> Result<PathBuf, std::io::Error>;
}

/// Errors that can occur during filesystem operations.
#[derive(Debug, thiserror::Error)]
pub enum FsError {
    /// The requested file or directory was not found.
    #[error("file not found: {0}")]
    NotFound(PathBuf),

    /// Access to the path was denied by the OS.
    #[error("permission denied: {0}")]
    PermissionDenied(PathBuf),

    /// A general I/O error occurred on the path.
    #[error("I/O error on {path}: {message}")]
    Io {
        /// The path on which the error occurred.
        path: PathBuf,
        /// Human-readable error description.
        message: String,
    },

    /// The filesystem operation is not supported by this implementation.
    #[error("unsupported operation: {0}")]
    Unsupported(String),
}

impl FsError {
    /// Construct an `FsError` from a `std::io::Error`, mapping common kinds to typed variants.
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
