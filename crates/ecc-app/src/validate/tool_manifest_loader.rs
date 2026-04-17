//! Loads and parses the tool manifest via the `FileSystem` port.
//!
//! Single-error semantics: any failure produces ONE error message pointing to
//! `manifest/tool-manifest.yaml`, never per-file cascade.

use ecc_domain::config::tool_manifest::{ToolManifest, parse_tool_manifest};
use ecc_ports::fs::FileSystem;
use std::path::Path;

use super::tool_manifest_path_resolver::{has_parent_traversal, resolve_tool_manifest_path};

/// Errors from `load_tool_manifest`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestLoadError {
    /// The manifest file was not found at the expected path.
    NotFound(String),
    /// The manifest file is a symlink (rejected for security).
    SymlinkRejected(String),
    /// The manifest path contains a parent traversal component (`..`).
    PathTraversal(String),
    /// An I/O error occurred while reading the manifest.
    ReadError(String),
    /// The manifest content could not be parsed.
    ParseError(String),
}

impl std::fmt::Display for ManifestLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(p) => write!(f, "tool manifest not found at {p}"),
            Self::SymlinkRejected(p) => write!(f, "tool manifest path is a symlink: {p}"),
            Self::PathTraversal(p) => {
                write!(f, "tool manifest path contains parent traversal: {p}")
            }
            Self::ReadError(msg) => write!(f, "tool manifest read error: {msg}"),
            Self::ParseError(msg) => write!(f, "tool manifest parse error at manifest/tool-manifest.yaml: {msg}"),
        }
    }
}

impl std::error::Error for ManifestLoadError {}

/// Load and parse the tool manifest from the filesystem.
///
/// Reads `<root>/manifest/tool-manifest.yaml` via the `FileSystem` port.
/// Returns `ManifestLoadError` on any failure — callers emit ONE error line
/// to stderr and skip per-file validation.
pub fn load_tool_manifest(
    fs: &dyn FileSystem,
    root: &Path,
) -> Result<ToolManifest, ManifestLoadError> {
    let path = resolve_tool_manifest_path(root);

    // Security: reject parent traversal in path
    if has_parent_traversal(&path) {
        return Err(ManifestLoadError::PathTraversal(
            path.display().to_string(),
        ));
    }

    // Security: reject symlinks
    if fs.is_symlink(&path) {
        return Err(ManifestLoadError::SymlinkRejected(
            path.display().to_string(),
        ));
    }

    // Missing file
    if !fs.exists(&path) {
        return Err(ManifestLoadError::NotFound(
            "manifest/tool-manifest.yaml".to_string(),
        ));
    }

    let content = fs
        .read_to_string(&path)
        .map_err(|e| ManifestLoadError::ReadError(e.to_string()))?;

    parse_tool_manifest(&content).map_err(|e| ManifestLoadError::ParseError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::InMemoryFileSystem;
    use std::path::PathBuf;

    // ── PC-022: missing_manifest_single_error ────────────────────────────────

    #[test]
    fn missing_manifest_single_error() {
        let fs = InMemoryFileSystem::new();
        let root = PathBuf::from("/project");
        let result = load_tool_manifest(&fs, &root);
        assert!(
            matches!(&result, Err(ManifestLoadError::NotFound(p)) if p.contains("manifest/tool-manifest.yaml")),
            "expected NotFound with path, got: {result:?}"
        );
        // The error message should mention the canonical path
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("manifest/tool-manifest.yaml"),
            "error message must contain the canonical path, got: {err_msg}"
        );
    }

    // ── PC-025: parse_error_once_with_path ───────────────────────────────────

    #[test]
    fn parse_error_once_with_path() {
        let fs = InMemoryFileSystem::new();
        let root = PathBuf::from("/project");
        // Write an invalid YAML manifest
        fs.write(
            &root.join("manifest/tool-manifest.yaml"),
            "this: is: not: valid: yaml: :::::",
        )
        .unwrap();
        let result = load_tool_manifest(&fs, &root);
        assert!(
            matches!(&result, Err(ManifestLoadError::ParseError(_))),
            "expected ParseError, got: {result:?}"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("manifest/tool-manifest.yaml"),
            "parse error must reference the manifest path, got: {err_msg}"
        );
    }

    // ── PC-011: canonical_manifest_has_six_plus_presets ──────────────────────

    #[test]
    fn canonical_manifest_has_six_plus_presets() {
        let fs = InMemoryFileSystem::new();
        // Embed the canonical manifest from the workspace
        let canonical = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../manifest/tool-manifest.yaml"
        ));
        let root = PathBuf::from("/project");
        fs.write(&root.join("manifest/tool-manifest.yaml"), canonical)
            .unwrap();
        let manifest = load_tool_manifest(&fs, &root).expect("canonical manifest must load");
        assert!(
            manifest.presets.len() >= 6,
            "canonical manifest must have at least 6 presets, got {}",
            manifest.presets.len()
        );
    }
}
