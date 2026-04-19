/// Hook-layer error types for ecc-app.

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
