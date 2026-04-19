//! `SafePath` newtype for path-traversal prevention.
//!
//! All constructors are pure — zero I/O.
//! The caller is responsible for canonicalizing paths at the app boundary.

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    // SafePath and SafePathError are not yet defined — this import will fail to compile,
    // confirming the RED state.
    use super::super::safe_path::{SafePath, SafePathError};

    #[test]
    fn rejects_traversal() {
        let root = PathBuf::from("/home/user/.claude/projects/proj/memory");
        let escaping_child = PathBuf::from("/home/user/.ssh/id_rsa");
        let result = SafePath::from_canonical(root.clone(), escaping_child.clone());
        assert!(matches!(result, Err(SafePathError::Escape { .. })));

        let good_child = root.join("project_bl001_foo.md");
        let result = SafePath::from_canonical(root.clone(), good_child.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().full(), good_child.as_path());
    }
}
