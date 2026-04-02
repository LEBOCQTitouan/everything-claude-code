//! Mock implementation of [`ecc_ports::git::GitInfo`] for testing.

use ecc_ports::git::{GitError, GitInfo};
use std::path::{Path, PathBuf};

/// Configurable mock for git info queries.
///
/// By default, returns `NotARepo` for all queries. Configure with builder methods.
pub struct MockGitInfo {
    git_dir: Option<PathBuf>,
    is_worktree: bool,
}

impl MockGitInfo {
    /// Create a mock that simulates a non-git directory.
    pub fn not_a_repo() -> Self {
        Self {
            git_dir: None,
            is_worktree: false,
        }
    }

    /// Create a mock that simulates a normal git repository.
    pub fn repo(git_dir: impl Into<PathBuf>) -> Self {
        Self {
            git_dir: Some(git_dir.into()),
            is_worktree: false,
        }
    }

    /// Create a mock that simulates a git worktree.
    pub fn worktree(git_dir: impl Into<PathBuf>) -> Self {
        Self {
            git_dir: Some(git_dir.into()),
            is_worktree: true,
        }
    }
}

impl GitInfo for MockGitInfo {
    fn git_dir(&self, _working_dir: &Path) -> Result<PathBuf, GitError> {
        match &self.git_dir {
            Some(path) => Ok(path.clone()),
            None => Err(GitError::NotARepo),
        }
    }

    fn is_inside_worktree(&self, _working_dir: &Path) -> bool {
        self.is_worktree
    }
}
