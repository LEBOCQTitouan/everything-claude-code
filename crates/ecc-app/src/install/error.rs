//! Typed errors for the install module.

/// Errors that can occur during ECC installation operations.
#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    /// Failed to resolve the ECC root directory.
    #[error("resolve_ecc_root: {0}")]
    ResolveRoot(String),

    /// Filesystem operation failed during installation.
    #[error("install: {source}")]
    Fs {
        #[from]
        source: ecc_ports::fs::FsError,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_resolve_root() {
        let err = InstallError::ResolveRoot("not found".into());
        assert!(err.to_string().contains("resolve_ecc_root"));
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn display_fs_error() {
        let fs_err = ecc_ports::fs::FsError::NotFound(std::path::PathBuf::from("/tmp/missing"));
        let err = InstallError::from(fs_err);
        assert!(err.to_string().contains("install:"));
    }
}
