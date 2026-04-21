//! Worktree-scoped state directory resolution.
//!
//! Resolves the location of workflow state using a fallback chain:
//! 1. Get project dir from `CLAUDE_PROJECT_DIR` env var or current working directory
//! 2. Run `git rev-parse --git-dir` to find the git directory
//! 3. If inside a git repo: state lives at `<git-dir>/ecc-workflow/`
//! 4. If not a git repo: fall back to `<project-dir>/.claude/workflow/`
//! 5. If state exists at old location but not new: read from old (backward compat)

use ecc_ports::env::Environment;
use ecc_ports::fs::FileSystem;
use ecc_ports::git::{GitError, GitInfo};
use std::path::{Path, PathBuf};

/// Warning emitted during state resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warning {
    /// Human-readable warning message.
    pub message: String,
}

/// Maximum anchor file size to prevent DoS from corrupted files.
const ANCHOR_MAX_BYTES: usize = 4096;

/// Anchor file name for pinning state directory resolution.
const ANCHOR_FILE_NAME: &str = ".state-dir";

/// Read the `.state-dir` anchor file from `<project_dir>/.claude/workflow/`.
///
/// Returns `(Some(dir), warnings)` if the anchor is valid and the directory exists.
/// Returns `(None, warnings)` to signal "fall through to git resolution".
fn read_anchor(fs: &dyn FileSystem, project_dir: &Path) -> (Option<PathBuf>, Vec<Warning>) {
    let anchor_path = project_dir.join(".claude/workflow").join(ANCHOR_FILE_NAME);
    let content = match fs.read_to_string(&anchor_path) {
        Ok(c) => c,
        Err(_) => return (None, vec![]),
    };
    if content.len() > ANCHOR_MAX_BYTES {
        return (
            None,
            vec![Warning {
                message: format!(
                    ".state-dir anchor exceeds {} bytes — ignoring",
                    ANCHOR_MAX_BYTES
                ),
            }],
        );
    }
    let trimmed = content.trim();
    let anchor_dir = PathBuf::from(trimmed);
    if trimmed.is_empty() || !anchor_dir.is_absolute() {
        return (
            None,
            vec![Warning {
                message: format!(
                    ".state-dir anchor at {} has invalid content — falling back to git resolution",
                    anchor_path.display()
                ),
            }],
        );
    }
    if !fs.exists(&anchor_dir) {
        return (
            None,
            vec![Warning {
                message: format!(
                    ".state-dir anchor points to nonexistent {} — falling back to git resolution",
                    anchor_dir.display()
                ),
            }],
        );
    }
    (Some(anchor_dir), vec![])
}

/// Resolve the state directory for the current context.
///
/// Returns `(state_dir, warnings)` where `state_dir` is the absolute path to
/// the directory containing `state.json`, and `warnings` are any non-fatal
/// issues encountered during resolution.
///
/// Flow/decision diagram — anchor-first chain with git + fallback:
///
/// <!-- keep in sync with: anchor_file_overrides_git_resolution -->
/// ```text
/// resolve_state_dir(env, git, fs)
///        |
///        v
/// read project_dir from CLAUDE_PROJECT_DIR or cwd
///        |
///        v
/// +------- read .state-dir anchor? -------+
/// |  path valid + dir exists?             |
/// +---------------------------------------+
///        |--Y--> return anchor_dir (no git lookup)
///        |--N-->
///        v
/// +------- git.git_dir(project_dir) ------+
/// +---------------------------------------+
///        |--Ok(git_dir)--> state = git_dir/ecc-workflow
///        |--NotARepo  --> warn + state = project/.claude/workflow
///        |--CmdFailed --> warn + state = project/.claude/workflow
///        v
/// +------- is_worktree path? -------------+
/// +---------------------------------------+
///        |--Y--> never fall back, return state_dir
///        |--N-->
///        v
/// old exists and new doesn't? --Y--> return old_location (migrate)
///                              --N--> return state_dir
/// ```
///
/// # Pattern
///
/// Chain of Responsibility \[GoF\] — anchor -> git -> legacy fallback.
pub fn resolve_state_dir(
    env: &dyn Environment,
    git: &dyn GitInfo,
    fs: &dyn FileSystem,
) -> (PathBuf, Vec<Warning>) {
    let mut warnings = Vec::new();

    // Step 1: Determine project directory
    let project_dir = env
        .var("CLAUDE_PROJECT_DIR")
        .map(PathBuf::from)
        .or_else(|| env.current_dir())
        .unwrap_or_else(|| PathBuf::from("."));

    // Step 1.5: Check .state-dir anchor file — overrides git resolution when valid
    let (anchor_result, anchor_warnings) = read_anchor(fs, &project_dir);
    warnings.extend(anchor_warnings);
    if let Some(dir) = anchor_result {
        return (dir, warnings);
    }

    // Step 2: Try git-dir resolution
    let state_dir = match git.git_dir(&project_dir) {
        Ok(git_dir) => git_dir.join("ecc-workflow"),
        Err(GitError::NotARepo) => {
            warnings.push(Warning {
                message: "Not a git repository — state is not worktree-isolated".to_owned(),
            });
            project_dir.join(".claude/workflow")
        }
        Err(GitError::CommandFailed(msg)) => {
            warnings.push(Warning {
                message: format!("git command failed: {msg} — falling back to project dir"),
            });
            project_dir.join(".claude/workflow")
        }
    };

    // Step 3: Check for old-location migration (main repo only, never worktrees)
    let old_location = project_dir.join(".claude/workflow");
    let new_state_file = state_dir.join("state.json");
    let old_state_file = old_location.join("state.json");

    // Worktrees have git-dir paths like `.git/worktrees/<name>/ecc-workflow`.
    // They must NEVER fall back to the main repo's `.claude/workflow/` —
    // that would cause one session's state to bleed into another.
    let is_worktree = state_dir.components().any(|c| c.as_os_str() == "worktrees");

    if state_dir != old_location && !is_worktree {
        let new_exists = fs.exists(&new_state_file);
        let old_exists = fs.exists(&old_state_file);

        if !new_exists && old_exists {
            warnings.push(Warning {
                message: "Migrating state to worktree-scoped location".to_owned(),
            });
            return (old_location, warnings);
        }
    }

    (state_dir, warnings)
}

/// Migrate state from old location to new location if needed.
///
/// Must be called under the state lock. Copies (not moves) old state.json
/// to the new location. The old file is preserved for rollback.
///
/// Returns Ok(true) if migration occurred, Ok(false) if not needed.
///
/// Flow/decision diagram — three early exits before copy:
///
/// <!-- keep in sync with: migrate_copies_old_to_new -->
/// ```text
/// migrate_if_needed(old_dir, new_dir, fs)
///        |
///        v
/// old_dir == new_dir?  --Y--> Ok(false)
///                      --N-->
///        v
/// new/state.json exists? --Y--> Ok(false)
///                        --N-->
///        v
/// old/state.json exists? --N--> Ok(false)
///                        --Y-->
///        v
/// read old -> create new_dir -> write new -> Ok(true)
/// ```
pub fn migrate_if_needed(
    old_dir: &Path,
    new_dir: &Path,
    fs: &dyn FileSystem,
) -> Result<bool, String> {
    if old_dir == new_dir {
        return Ok(false);
    }

    let new_state = new_dir.join("state.json");
    if fs.exists(&new_state) {
        return Ok(false);
    }

    let old_state = old_dir.join("state.json");
    if !fs.exists(&old_state) {
        return Ok(false);
    }

    tracing::warn!(
        "Migrating workflow state from {} to {}",
        old_dir.display(),
        new_dir.display()
    );

    let content = fs
        .read_to_string(&old_state)
        .map_err(|e| format!("failed to read old state: {e}"))?;
    fs.create_dir_all(new_dir)
        .map_err(|e| format!("failed to create new dir: {e}"))?;
    fs.write(&new_state, &content)
        .map_err(|e| format!("failed to write new state: {e}"))?;

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::{InMemoryFileSystem, MockEnvironment, MockGitInfo};

    fn make_env(project_dir: Option<&str>, _cwd: Option<&str>) -> MockEnvironment {
        let env = MockEnvironment::new();
        match project_dir {
            Some(pd) => env.with_var("CLAUDE_PROJECT_DIR", pd),
            None => env,
        }
    }

    #[test]
    fn worktree_returns_git_dir() {
        let env = make_env(Some("/project"), None);
        let git = MockGitInfo::worktree("/project/.git/worktrees/my-branch");
        let fs = InMemoryFileSystem::new();

        let (dir, warnings) = resolve_state_dir(&env, &git, &fs);
        assert_eq!(
            dir,
            PathBuf::from("/project/.git/worktrees/my-branch/ecc-workflow")
        );
        assert!(warnings.is_empty());
    }

    #[test]
    fn worktree_independent_from_main() {
        let env_main = make_env(Some("/project"), None);
        let git_main = MockGitInfo::repo("/project/.git");
        let fs = InMemoryFileSystem::new();

        let env_wt = make_env(Some("/project"), None);
        let git_wt = MockGitInfo::worktree("/project/.git/worktrees/feature");

        let (dir_main, _) = resolve_state_dir(&env_main, &git_main, &fs);
        let (dir_wt, _) = resolve_state_dir(&env_wt, &git_wt, &fs);

        assert_ne!(
            dir_main, dir_wt,
            "main and worktree must have different state dirs"
        );
    }

    #[test]
    fn uses_claude_project_dir() {
        let env = make_env(Some("/custom/project"), None);
        let git = MockGitInfo::repo("/custom/project/.git");
        let fs = InMemoryFileSystem::new();

        let (dir, _) = resolve_state_dir(&env, &git, &fs);
        assert_eq!(dir, PathBuf::from("/custom/project/.git/ecc-workflow"));
    }

    #[test]
    fn non_git_fallback() {
        let env = make_env(Some("/no-git"), None);
        let git = MockGitInfo::not_a_repo();
        let fs = InMemoryFileSystem::new();

        let (dir, warnings) = resolve_state_dir(&env, &git, &fs);
        assert_eq!(dir, PathBuf::from("/no-git/.claude/workflow"));
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("Not a git repository"));
    }

    #[test]
    fn old_location_fallback() {
        let env = make_env(Some("/project"), None);
        let git = MockGitInfo::repo("/project/.git");
        let fs = InMemoryFileSystem::new();
        fs.write(
            &PathBuf::from("/project/.claude/workflow/state.json"),
            r#"{"phase":"plan"}"#,
        )
        .unwrap();

        let (dir, warnings) = resolve_state_dir(&env, &git, &fs);
        assert_eq!(
            dir,
            PathBuf::from("/project/.claude/workflow"),
            "should fall back to old location"
        );
        assert!(warnings.iter().any(|w| w.message.contains("Migrating")));
    }

    #[test]
    fn bare_repo_support() {
        let env = make_env(Some("/bare-repo.git"), None);
        let git = MockGitInfo::repo("/bare-repo.git");
        let fs = InMemoryFileSystem::new();

        let (dir, warnings) = resolve_state_dir(&env, &git, &fs);
        assert_eq!(dir, PathBuf::from("/bare-repo.git/ecc-workflow"));
        assert!(warnings.is_empty());
    }

    /// PC-001: Worktree with no local state but main repo has state.json →
    /// must return the worktree's git-dir-scoped path, NOT the main repo path.
    #[test]
    fn worktree_ignores_main_repo_state() {
        let env = make_env(Some("/project"), None);
        let git = MockGitInfo::worktree("/project/.git/worktrees/feature-x");
        let fs = InMemoryFileSystem::new();
        // Main repo has state.json at old location
        fs.write(
            &PathBuf::from("/project/.claude/workflow/state.json"),
            r#"{"phase":"plan"}"#,
        )
        .unwrap();
        // Worktree does NOT have its own state yet

        let (dir, warnings) = resolve_state_dir(&env, &git, &fs);

        // Must return worktree path, not main repo path
        assert_eq!(
            dir,
            PathBuf::from("/project/.git/worktrees/feature-x/ecc-workflow"),
            "worktree must NOT fall back to main repo state"
        );
        // No migration warning — worktrees don't migrate
        assert!(
            !warnings.iter().any(|w| w.message.contains("Migrating")),
            "no migration warning for worktrees"
        );
    }

    /// PC-002: Worktree with its own state.json → returns worktree path.
    #[test]
    fn worktree_with_own_state() {
        let env = make_env(Some("/project"), None);
        let git = MockGitInfo::worktree("/project/.git/worktrees/my-session");
        let fs = InMemoryFileSystem::new();
        // Both main and worktree have state
        fs.write(
            &PathBuf::from("/project/.claude/workflow/state.json"),
            r#"{"phase":"plan"}"#,
        )
        .unwrap();
        fs.write(
            &PathBuf::from("/project/.git/worktrees/my-session/ecc-workflow/state.json"),
            r#"{"phase":"implement"}"#,
        )
        .unwrap();

        let (dir, _) = resolve_state_dir(&env, &git, &fs);

        assert_eq!(
            dir,
            PathBuf::from("/project/.git/worktrees/my-session/ecc-workflow"),
            "worktree must use its own state"
        );
    }

    // --- migrate_if_needed tests ---

    #[test]
    fn migrate_copies_old_to_new() {
        let fs = InMemoryFileSystem::new();
        let old_dir = PathBuf::from("/project/.claude/workflow");
        let new_dir = PathBuf::from("/project/.git/ecc-workflow");
        let old_state = old_dir.join("state.json");
        let content = r#"{"phase":"plan"}"#;
        fs.write(&old_state, content).unwrap();

        let result = migrate_if_needed(&old_dir, &new_dir, &fs);
        assert_eq!(result, Ok(true));

        let new_state = new_dir.join("state.json");
        let written = fs.read_to_string(&new_state).unwrap();
        assert_eq!(written, content);
    }

    #[test]
    fn migrate_noop_when_new_exists() {
        let fs = InMemoryFileSystem::new();
        let old_dir = PathBuf::from("/project/.claude/workflow");
        let new_dir = PathBuf::from("/project/.git/ecc-workflow");
        fs.write(&old_dir.join("state.json"), r#"{"phase":"plan"}"#)
            .unwrap();
        fs.write(&new_dir.join("state.json"), r#"{"phase":"idle"}"#)
            .unwrap();

        let result = migrate_if_needed(&old_dir, &new_dir, &fs);
        assert_eq!(result, Ok(false));
    }

    #[test]
    fn migrate_noop_same_dir() {
        let fs = InMemoryFileSystem::new();
        let dir = PathBuf::from("/project/.claude/workflow");
        fs.write(&dir.join("state.json"), r#"{"phase":"plan"}"#)
            .unwrap();

        let result = migrate_if_needed(&dir, &dir, &fs);
        assert_eq!(result, Ok(false));
    }

    #[test]
    fn migrate_noop_no_state() {
        let fs = InMemoryFileSystem::new();
        let old_dir = PathBuf::from("/project/.claude/workflow");
        let new_dir = PathBuf::from("/project/.git/ecc-workflow");

        let result = migrate_if_needed(&old_dir, &new_dir, &fs);
        assert_eq!(result, Ok(false));
    }

    #[test]
    #[tracing_test::traced_test]
    fn migrate_emits_warning() {
        let fs = InMemoryFileSystem::new();
        let old_dir = PathBuf::from("/project/.claude/workflow");
        let new_dir = PathBuf::from("/project/.git/ecc-workflow");
        fs.write(&old_dir.join("state.json"), r#"{"phase":"plan"}"#)
            .unwrap();

        let result = migrate_if_needed(&old_dir, &new_dir, &fs);
        assert_eq!(result, Ok(true));
        assert!(logs_contain("Migrating"));
    }

    // --- PC-034: Concurrent migration serialization test ---

    /// PC-001: Anchor file at `.claude/workflow/.state-dir` with valid absolute path
    /// that exists — resolve_state_dir() returns that path directly, no warnings.
    #[test]
    fn anchor_file_overrides_git_resolution() {
        let env = make_env(Some("/project"), None);
        // Git would resolve to feature-x, but anchor points to OTHER-SESSION
        let git = MockGitInfo::worktree("/project/.git/worktrees/feature-x");
        let fs = InMemoryFileSystem::new();

        // Anchor points to a DIFFERENT worktree than git would resolve
        let anchor_dir = PathBuf::from("/project/.git/worktrees/OTHER-SESSION/ecc-workflow");
        fs.create_dir_all(&anchor_dir).unwrap();

        fs.write(
            &PathBuf::from("/project/.claude/workflow/.state-dir"),
            "/project/.git/worktrees/OTHER-SESSION/ecc-workflow\n",
        )
        .unwrap();

        let (dir, warnings) = resolve_state_dir(&env, &git, &fs);

        // Must return anchor path, NOT git-resolved path
        assert_eq!(dir, anchor_dir, "anchor should override git resolution");
        assert_ne!(
            dir,
            PathBuf::from("/project/.git/worktrees/feature-x/ecc-workflow"),
            "must NOT return git-resolved path when anchor exists"
        );
        assert!(warnings.is_empty(), "no warnings expected: {warnings:?}");
    }

    /// PC-003: Anchor file exists but contains a relative path — invalid.
    /// resolve_state_dir() must fall back to git resolution and emit a warning
    /// containing ".state-dir".
    #[test]
    fn corrupt_anchor_falls_back() {
        let env = make_env(Some("/project"), None);
        let git = MockGitInfo::repo("/project/.git");
        let fs = InMemoryFileSystem::new();

        // Write anchor with relative (invalid) path
        fs.write(
            &PathBuf::from("/project/.claude/workflow/.state-dir"),
            "not-absolute-path\n",
        )
        .unwrap();

        let (dir, warnings) = resolve_state_dir(&env, &git, &fs);

        // Must fall back to git resolution
        assert_eq!(
            dir,
            PathBuf::from("/project/.git/ecc-workflow"),
            "should fall back to git resolution"
        );
        assert!(
            warnings.iter().any(|w| w.message.contains(".state-dir")),
            "expected warning mentioning .state-dir, got: {warnings:?}"
        );
    }

    /// PC-004: Anchor file contains an absolute path that does not exist on disk.
    /// resolve_state_dir() must fall back to git resolution and emit a warning.
    #[test]
    fn stale_anchor_falls_back() {
        let env = make_env(Some("/project"), None);
        let git = MockGitInfo::repo("/project/.git");
        let fs = InMemoryFileSystem::new();

        // Write anchor with nonexistent absolute path
        fs.write(
            &PathBuf::from("/project/.claude/workflow/.state-dir"),
            "/nonexistent/path/ecc-workflow\n",
        )
        .unwrap();

        let (dir, warnings) = resolve_state_dir(&env, &git, &fs);

        // Must fall back to git resolution
        assert_eq!(
            dir,
            PathBuf::from("/project/.git/ecc-workflow"),
            "should fall back to git resolution"
        );
        assert!(
            !warnings.is_empty(),
            "expected at least one warning for stale anchor"
        );
    }

    /// PC-034: Two processes race on old→new migration; exactly one copy
    /// occurs, no corruption. Simulated sequentially since InMemoryFileSystem
    /// is single-threaded, but validates the re-check logic:
    /// - First call: old exists, new does not → migrates → Ok(true)
    /// - Second call: new already exists → no-op → Ok(false)
    /// - Content is identical (no corruption)
    #[test]
    fn migrate_concurrent_serialized() {
        let fs = InMemoryFileSystem::new();
        let old_dir = PathBuf::from("/project/.claude/workflow");
        let new_dir = PathBuf::from("/project/.git/ecc-workflow");
        let content = r#"{"phase":"plan","feature":"concurrent-test"}"#;

        fs.write(&old_dir.join("state.json"), content).unwrap();

        // First "process": should migrate
        let first = migrate_if_needed(&old_dir, &new_dir, &fs);
        assert_eq!(first, Ok(true), "first call must migrate");

        // Second "process": new state already exists, must be no-op
        let second = migrate_if_needed(&old_dir, &new_dir, &fs);
        assert_eq!(
            second,
            Ok(false),
            "second call must be no-op (already migrated)"
        );

        // Verify no corruption: content in new location matches original
        let new_state = new_dir.join("state.json");
        let written = fs.read_to_string(&new_state).unwrap();
        assert_eq!(written, content, "migrated content must not be corrupted");
    }
}
