//! App-layer path resolution for the memory system.
//!
//! Canonicalizes and bounds-checks `ECC_PROJECT_MEMORY_ROOT` at the app boundary.

use ecc_domain::memory::{SafePath, SafePathError};
use ecc_ports::env::Environment;
use ecc_ports::fs::FileSystem;
use std::path::{Path, PathBuf};

/// Errors returned by [`resolve_project_memory_root`].
#[derive(Debug, thiserror::Error)]
pub enum PathResolutionError {
    /// `HOME` environment variable is not set.
    #[error("HOME env var not set")]
    HomeNotSet,
    /// `ECC_PROJECT_MEMORY_ROOT` canonicalizes outside `$HOME`.
    #[error("ECC_PROJECT_MEMORY_ROOT canonicalizes outside $HOME: {path:?}")]
    OutsideHome {
        /// The canonical path that escaped `$HOME`.
        path: PathBuf,
    },
    /// Canonicalization failed for the given path.
    #[error("canonicalization failed for {path:?}: {source}")]
    CanonicalizeFailed {
        /// The path that failed to canonicalize.
        path: PathBuf,
        /// The underlying I/O error.
        source: std::io::Error,
    },
    /// The resulting path is structurally invalid.
    #[error(transparent)]
    SafePathInvalid(#[from] SafePathError),
}

/// Resolve the project memory root at the app boundary.
///
/// Reads `ECC_PROJECT_MEMORY_ROOT` if set, else falls back to
/// `$HOME/.claude/projects/default/memory`. Canonicalizes both paths and
/// constructs a [`SafePath`] rooted at `$HOME` (or tighter).
///
/// # Errors
///
/// Returns [`PathResolutionError`] if `HOME` is not set, either path fails
/// canonicalization, or the resolved root escapes `$HOME`.
pub fn resolve_project_memory_root(
    env: &dyn Environment,
    fs: &dyn FileSystem,
) -> Result<SafePath, PathResolutionError> {
    let home = env.var("HOME").ok_or(PathResolutionError::HomeNotSet)?;
    let home_canonical = fs
        .canonicalize(Path::new(&home))
        .map_err(|e| PathResolutionError::CanonicalizeFailed {
            path: PathBuf::from(&home),
            source: e,
        })?;

    let raw = env.var("ECC_PROJECT_MEMORY_ROOT").unwrap_or_else(|| {
        format!("{home}/.claude/projects/default/memory")
    });
    let raw_path = PathBuf::from(&raw);
    let canonical = fs
        .canonicalize(&raw_path)
        .map_err(|e| PathResolutionError::CanonicalizeFailed {
            path: raw_path.clone(),
            source: e,
        })?;

    if !canonical.starts_with(&home_canonical) {
        return Err(PathResolutionError::OutsideHome { path: canonical });
    }

    SafePath::from_canonical(home_canonical, canonical).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::{InMemoryFileSystem, MockEnvironment};

    /// Verify that the HOME-fallback path uses the same project-hash derivation
    /// algorithm as `ecc-workflow/memory_write::resolve_project_memory_dir`:
    /// `path.trim_start_matches('/').replace('/', "-")`.
    #[test]
    fn resolve_root_hash_algorithm_vectors() {
        // Vector 1: /Users/alice/project/foo → Users-alice-project-foo
        let expected_hash_1 = "Users-alice-project-foo";
        let fs = InMemoryFileSystem::new()
            .with_dir("/home/alice")
            .with_dir(&format!("/home/alice/.claude/projects/{expected_hash_1}/memory"));
        let env = MockEnvironment::new()
            .with_var("HOME", "/home/alice")
            .with_var("CLAUDE_PROJECT_DIR", "/Users/alice/project/foo");

        let result = resolve_project_memory_root(&env, &fs);
        let safe = result.expect("should resolve");
        assert!(
            safe.full()
                .to_string_lossy()
                .contains(expected_hash_1),
            "expected path to contain hash {expected_hash_1}, got {:?}",
            safe.full()
        );

        // Vector 2: /home/bob/repos/myapp → home-bob-repos-myapp
        let expected_hash_2 = "home-bob-repos-myapp";
        let fs2 = InMemoryFileSystem::new()
            .with_dir("/home/bob")
            .with_dir(&format!("/home/bob/.claude/projects/{expected_hash_2}/memory"));
        let env2 = MockEnvironment::new()
            .with_var("HOME", "/home/bob")
            .with_var("CLAUDE_PROJECT_DIR", "/home/bob/repos/myapp");

        let result2 = resolve_project_memory_root(&env2, &fs2);
        let safe2 = result2.expect("should resolve for vector 2");
        assert!(
            safe2
                .full()
                .to_string_lossy()
                .contains(expected_hash_2),
            "expected path to contain hash {expected_hash_2}, got {:?}",
            safe2.full()
        );
    }

    #[test]
    fn resolve_root_env_override() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/home/alice")
            .with_dir("/home/alice/custom-memory");
        let env = MockEnvironment::new()
            .with_var("HOME", "/home/alice")
            .with_var("ECC_PROJECT_MEMORY_ROOT", "/home/alice/custom-memory");

        let result = resolve_project_memory_root(&env, &fs);
        let safe = result.expect("should resolve");
        assert_eq!(safe.full(), Path::new("/home/alice/custom-memory"));
    }

    #[test]
    fn rejects_root_outside_home() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/home/alice")
            .with_dir("/etc/secrets");
        let env = MockEnvironment::new()
            .with_var("HOME", "/home/alice")
            .with_var("ECC_PROJECT_MEMORY_ROOT", "/etc/secrets");

        let result = resolve_project_memory_root(&env, &fs);
        assert!(
            matches!(result, Err(PathResolutionError::OutsideHome { .. })),
            "must reject root outside HOME; got {result:?}"
        );
    }
}
