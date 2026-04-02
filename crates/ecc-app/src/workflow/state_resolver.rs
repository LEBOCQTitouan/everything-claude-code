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
use std::path::PathBuf;

/// Warning emitted during state resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warning {
    /// Human-readable warning message.
    pub message: String,
}

/// Resolve the state directory for the current context.
///
/// Returns `(state_dir, warnings)` where `state_dir` is the absolute path to
/// the directory containing `state.json`, and `warnings` are any non-fatal
/// issues encountered during resolution.
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

    // Step 3: Check for old-location migration
    let old_location = project_dir.join(".claude/workflow");
    let new_state_file = state_dir.join("state.json");
    let old_state_file = old_location.join("state.json");

    if state_dir != old_location {
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
}
