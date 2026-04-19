/// Hook-layer error types for ecc-app.
use std::path::PathBuf;

/// Errors produced by hook execution.
#[derive(Debug, thiserror::Error)]
pub enum HookError {
    /// An I/O error occurred during a cartography operation.
    #[error("cartography I/O: {operation} at {path:?}: {source}")]
    CartographyIo {
        /// The operation that failed (e.g., "create_dir").
        operation: String,
        /// The filesystem path involved.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::path::PathBuf;

    #[test]
    fn cartography_io_display() {
        let err = HookError::CartographyIo {
            operation: "create_dir".to_string(),
            path: PathBuf::from("/tmp/foo"),
            source: io::Error::new(io::ErrorKind::PermissionDenied, "denied"),
        };
        let s = err.to_string();
        assert!(s.contains("create_dir"));
        assert!(s.contains("/tmp/foo"));
        assert!(s.contains("denied"));
    }
}
