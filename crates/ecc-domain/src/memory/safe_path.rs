//! `SafePath` newtype for path-traversal prevention.
//!
//! All constructors are pure — zero I/O.
//! The caller is responsible for canonicalizing paths at the app boundary.

use std::path::{Path, PathBuf};

/// A path guaranteed to reside under a given root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafePath {
    root: PathBuf,
    full: PathBuf,
}

/// Errors returned by [`SafePath::from_canonical`].
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SafePathError {
    /// The child path does not start with the root, indicating a traversal attempt.
    #[error("path escapes root: {path:?} not under {root:?}")]
    Escape { root: PathBuf, path: PathBuf },
    /// The path is structurally invalid.
    #[error("invalid path: {reason}")]
    Invalid { reason: String },
}

impl SafePath {
    /// Construct from already-canonicalized paths. PURE — no I/O.
    ///
    /// The caller is responsible for canonicalizing both `root` and `child`
    /// via `std::fs::canonicalize` at the app boundary.
    pub fn from_canonical(root: PathBuf, child: PathBuf) -> Result<Self, SafePathError> {
        if !child.starts_with(&root) {
            return Err(SafePathError::Escape { root, path: child });
        }
        Ok(SafePath { root, full: child })
    }

    /// Returns the root path.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the full (child) path.
    pub fn full(&self) -> &Path {
        &self.full
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn rejects_escape_pure() {
        // Sibling dir with shared prefix but not a child:
        //   root=/home/user/mem, sibling=/home/user/memotherdir — NOT under root
        let root = PathBuf::from("/home/user/mem");
        let sibling = PathBuf::from("/home/user/memotherdir/file.md");
        // starts_with is path-component aware in Rust — "/home/user/mem" is not
        // a component prefix of "/home/user/memotherdir", so this correctly rejects.
        let result = SafePath::from_canonical(root.clone(), sibling);
        assert!(matches!(result, Err(SafePathError::Escape { .. })));

        // Exact root path is accepted (degenerate case)
        let root_exact = root.clone();
        let result = SafePath::from_canonical(root.clone(), root_exact);
        assert!(result.is_ok(), "exact root path is a valid SafePath");

        // Deep nested child accepted
        let deep = root.join("sub/dir/file.md");
        let result = SafePath::from_canonical(root, deep);
        assert!(result.is_ok());
    }

    #[test]
    fn rejects_traversal() {
        // Traversal-like inputs (pre-canonicalization should handle ..
        // but assume caller passes already-canonical paths).
        // The pure newtype relies on string-prefix check: if child
        // canonicalized does not start_with root, it's an escape.
        let root = PathBuf::from("/home/user/.claude/projects/proj/memory");
        // Escape: child is OUTSIDE root after canonicalization
        let escaping_child = PathBuf::from("/home/user/.ssh/id_rsa");
        let result = SafePath::from_canonical(root.clone(), escaping_child.clone());
        assert!(matches!(result, Err(SafePathError::Escape { .. })));

        // OK: child is under root
        let good_child = root.join("project_bl001_foo.md");
        let result = SafePath::from_canonical(root.clone(), good_child.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().full(), good_child.as_path());
    }
}
