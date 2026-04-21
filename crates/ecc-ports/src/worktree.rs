//! Worktree management port.
//!
//! Defines the [`WorktreeManager`] trait for querying and managing git worktrees.

use std::fmt;
use std::path::Path;

/// Error type for worktree operations.
#[derive(Debug)]
pub enum WorktreeError {
    /// A git command failed.
    CommandFailed(String),
    /// The worktree or branch was not found.
    NotFound(String),
}

impl fmt::Display for WorktreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CommandFailed(msg) => write!(f, "worktree command failed: {msg}"),
            Self::NotFound(msg) => write!(f, "not found: {msg}"),
        }
    }
}

impl std::error::Error for WorktreeError {}

/// Information about a single git worktree.
#[derive(Debug, Clone)]
pub struct WorktreeInfo {
    /// Absolute path of the worktree.
    pub path: String,
    /// Branch checked out in the worktree, if any.
    pub branch: Option<String>,
}

/// Port for querying and managing git worktrees.
///
/// # Pattern
///
/// Port \[Hexagonal Architecture\]
pub trait WorktreeManager: Send + Sync {
    /// Return `true` if the worktree at `worktree_path` has uncommitted changes.
    fn has_uncommitted_changes(&self, worktree_path: &Path) -> Result<bool, WorktreeError>;

    /// Return `true` if the worktree at `worktree_path` has untracked files.
    fn has_untracked_files(&self, worktree_path: &Path) -> Result<bool, WorktreeError>;

    /// Return the number of commits reachable from HEAD but not from `target_branch`.
    fn unmerged_commit_count(
        &self,
        worktree_path: &Path,
        target_branch: &str,
    ) -> Result<u64, WorktreeError>;

    /// Return `true` if the worktree at `worktree_path` has a stash.
    fn has_stash(&self, worktree_path: &Path) -> Result<bool, WorktreeError>;

    /// Return `true` if `branch` is fully pushed to a remote.
    fn is_pushed_to_remote(
        &self,
        worktree_path: &Path,
        branch: &str,
    ) -> Result<bool, WorktreeError>;

    /// Remove the worktree at `worktree_path` from the repo at `repo_root`.
    fn remove_worktree(&self, repo_root: &Path, worktree_path: &Path) -> Result<(), WorktreeError>;

    /// Delete the local branch `branch` in the repo at `repo_root`.
    fn delete_branch(&self, repo_root: &Path, branch: &str) -> Result<(), WorktreeError>;

    /// List all worktrees for the repo at `repo_root`.
    fn list_worktrees(&self, repo_root: &Path) -> Result<Vec<WorktreeInfo>, WorktreeError>;
}
