use std::path::{Path, PathBuf};
use thiserror::Error;

/// Error variants for tarball extraction.
#[derive(Debug, Error)]
pub enum ExtractError {
    /// The archive is corrupt or invalid.
    #[error("corrupt archive: {0}")]
    CorruptArchive(String),
    /// Path traversal (zip-slip) attack detected.
    #[error("zip-slip path traversal detected: {0}")]
    ZipSlip(String),
    /// I/O error during extraction.
    #[error("I/O error: {0}")]
    Io(String),
}

/// Port for extracting release tarballs.
///
/// Production adapter: `FlateExtractor` in `ecc-infra`.
/// Test double: `MockExtractor` in `ecc-test-support`.
pub trait TarballExtractor: Send + Sync {
    /// Extract the tarball at `tarball` into `dest`.
    ///
    /// Returns the list of extracted file paths relative to `dest`.
    ///
    /// # Errors
    ///
    /// Returns `ExtractError::CorruptArchive` if the archive is invalid.
    /// Returns `ExtractError::ZipSlip` if a path traversal is detected.
    /// Returns `ExtractError::Io` on I/O failure.
    fn extract(&self, tarball: &Path, dest: &Path) -> Result<Vec<PathBuf>, ExtractError>;
}
