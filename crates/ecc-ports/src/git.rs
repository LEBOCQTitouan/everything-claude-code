//! Git repository information port.

use std::fmt;
use std::path::{Path, PathBuf};

/// Error type for git operations.
#[derive(Debug)]
pub enum GitError {
    /// The directory is not inside a git repository.
    NotARepo,
    /// The git command failed or produced unexpected output.
    CommandFailed(String),
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotARepo => write!(f, "not a git repository"),
            Self::CommandFailed(msg) => write!(f, "git command failed: {msg}"),
        }
    }
}

impl std::error::Error for GitError {}

/// Port for querying git repository information.
pub trait GitInfo: Send + Sync {
    /// Return the git directory for the given working directory.
    ///
    /// For a normal repo, this is `<repo>/.git`.
    /// For a worktree, this is `<repo>/.git/worktrees/<name>`.
    /// For a bare repo, this is the repo directory itself.
    fn git_dir(&self, working_dir: &Path) -> Result<PathBuf, GitError>;
}
