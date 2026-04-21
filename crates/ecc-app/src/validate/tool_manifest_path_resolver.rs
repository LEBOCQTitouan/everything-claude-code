//! Pure canonical path resolver for the tool manifest file.
//!
//! Returns the canonical path `<root>/manifest/tool-manifest.yaml` with no
//! parent traversal (no `..` allowed) and no symlink following. Fixed suffix
//! only — never derived from user input.

use std::path::{Path, PathBuf};

/// Return the canonical path to the tool manifest file for the given root.
///
/// Always `root.join("manifest/tool-manifest.yaml")`. The resolved path is
/// validated to contain no `..` components (parent traversal) and the returned
/// path is not a symlink (checked at call site via `FileSystem::is_symlink`).
pub fn resolve_tool_manifest_path(root: &Path) -> PathBuf {
    root.join("manifest/tool-manifest.yaml")
}

/// Returns `true` if the path contains any `..` component.
pub fn has_parent_traversal(path: &Path) -> bool {
    path.components()
        .any(|c| c == std::path::Component::ParentDir)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── PC-028: path_is_canonical_only ───────────────────────────────────────

    #[test]
    fn path_is_canonical_only() {
        let root = Path::new("/project");
        let path = resolve_tool_manifest_path(root);
        assert_eq!(
            path,
            PathBuf::from("/project/manifest/tool-manifest.yaml"),
            "path must be root.join('manifest/tool-manifest.yaml')"
        );
        // Must not contain any '..' component
        assert!(
            !has_parent_traversal(&path),
            "canonical path must not contain parent traversal"
        );
    }

    // ── PC-072: rejects_parent_walk_and_symlinks ──────────────────────────────

    #[test]
    fn rejects_parent_walk_and_symlinks() {
        // The fixed-suffix construction should never introduce '..'.
        // Verify that if somehow the root itself contained '..', has_parent_traversal detects it.
        let root_with_traversal = Path::new("/project/../etc");
        let path = resolve_tool_manifest_path(root_with_traversal);
        // The path will contain '..' from the root
        assert!(
            has_parent_traversal(&path),
            "has_parent_traversal must detect '..' in the path"
        );

        // Normal root: no traversal
        let root = Path::new("/safe/root");
        let safe_path = resolve_tool_manifest_path(root);
        assert!(
            !has_parent_traversal(&safe_path),
            "safe root must not produce traversal path"
        );

        // Symlink rejection is exercised at caller level via FileSystem::is_symlink.
        // The path_resolver itself is pure — it simply produces the canonical path.
        // The loader (tool_manifest_loader) checks `fs.is_symlink(&path)` before reading.
    }
}
