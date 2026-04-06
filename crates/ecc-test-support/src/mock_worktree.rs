//! Mock implementation of [`ecc_ports::worktree::WorktreeManager`] for testing.

use ecc_ports::worktree::{WorktreeError, WorktreeInfo, WorktreeManager};
use std::path::Path;

/// Configurable mock for worktree management operations.
///
/// By default returns safe values: no uncommitted changes, no untracked files,
/// 0 unmerged commits, no stash, pushed to remote, remove succeeds, delete succeeds,
/// empty list.
#[derive(Default)]
pub struct MockWorktreeManager {
    has_uncommitted_changes: bool,
    has_untracked_files: bool,
    unmerged_commit_count: u64,
    has_stash: bool,
    is_pushed_to_remote: bool,
    remove_worktree_result: bool,
    delete_branch_result: bool,
    worktrees: Vec<WorktreeInfo>,
}

impl MockWorktreeManager {
    /// Create a new mock with all-safe defaults.
    pub fn new() -> Self {
        Self {
            has_uncommitted_changes: false,
            has_untracked_files: false,
            unmerged_commit_count: 0,
            has_stash: false,
            is_pushed_to_remote: true,
            remove_worktree_result: true,
            delete_branch_result: true,
            worktrees: Vec::new(),
        }
    }

    /// Configure whether `has_uncommitted_changes` returns true.
    pub fn with_uncommitted_changes(mut self, value: bool) -> Self {
        self.has_uncommitted_changes = value;
        self
    }

    /// Configure whether `has_untracked_files` returns true.
    pub fn with_untracked_files(mut self, value: bool) -> Self {
        self.has_untracked_files = value;
        self
    }

    /// Configure the unmerged commit count.
    pub fn with_unmerged_commit_count(mut self, count: u64) -> Self {
        self.unmerged_commit_count = count;
        self
    }

    /// Configure whether `has_stash` returns true.
    pub fn with_stash(mut self, value: bool) -> Self {
        self.has_stash = value;
        self
    }

    /// Configure whether `is_pushed_to_remote` returns true.
    pub fn with_pushed(mut self, value: bool) -> Self {
        self.is_pushed_to_remote = value;
        self
    }

    /// Configure whether `remove_worktree` succeeds.
    pub fn with_remove_succeeds(mut self, value: bool) -> Self {
        self.remove_worktree_result = value;
        self
    }

    /// Configure whether `delete_branch` succeeds.
    pub fn with_delete_succeeds(mut self, value: bool) -> Self {
        self.delete_branch_result = value;
        self
    }

    /// Configure the list of worktrees returned by `list_worktrees`.
    pub fn with_worktrees(mut self, worktrees: Vec<WorktreeInfo>) -> Self {
        self.worktrees = worktrees;
        self
    }
}

impl WorktreeManager for MockWorktreeManager {
    fn has_uncommitted_changes(
        &self,
        _worktree_path: &Path,
    ) -> Result<bool, WorktreeError> {
        Ok(self.has_uncommitted_changes)
    }

    fn has_untracked_files(
        &self,
        _worktree_path: &Path,
    ) -> Result<bool, WorktreeError> {
        Ok(self.has_untracked_files)
    }

    fn unmerged_commit_count(
        &self,
        _worktree_path: &Path,
        _target_branch: &str,
    ) -> Result<u64, WorktreeError> {
        Ok(self.unmerged_commit_count)
    }

    fn has_stash(&self, _worktree_path: &Path) -> Result<bool, WorktreeError> {
        Ok(self.has_stash)
    }

    fn is_pushed_to_remote(
        &self,
        _worktree_path: &Path,
        _branch: &str,
    ) -> Result<bool, WorktreeError> {
        Ok(self.is_pushed_to_remote)
    }

    fn remove_worktree(
        &self,
        _repo_root: &Path,
        _worktree_path: &Path,
    ) -> Result<(), WorktreeError> {
        if self.remove_worktree_result {
            Ok(())
        } else {
            Err(WorktreeError::CommandFailed("mock remove failed".to_owned()))
        }
    }

    fn delete_branch(
        &self,
        _repo_root: &Path,
        _branch: &str,
    ) -> Result<(), WorktreeError> {
        if self.delete_branch_result {
            Ok(())
        } else {
            Err(WorktreeError::CommandFailed("mock delete failed".to_owned()))
        }
    }

    fn list_worktrees(
        &self,
        _repo_root: &Path,
    ) -> Result<Vec<WorktreeInfo>, WorktreeError> {
        Ok(self.worktrees.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn defaults_are_safe() {
        let mgr = MockWorktreeManager::new();
        let p = Path::new("/tmp/fake");

        assert!(!mgr.has_uncommitted_changes(p).unwrap());
        assert!(!mgr.has_untracked_files(p).unwrap());
        assert_eq!(mgr.unmerged_commit_count(p, "main").unwrap(), 0);
        assert!(!mgr.has_stash(p).unwrap());
        assert!(mgr.is_pushed_to_remote(p, "main").unwrap());
        assert!(mgr.remove_worktree(p, p).is_ok());
        assert!(mgr.delete_branch(p, "branch").is_ok());
        assert!(mgr.list_worktrees(p).unwrap().is_empty());
    }

    #[test]
    fn configured_values_returned() {
        let mgr = MockWorktreeManager::new()
            .with_uncommitted_changes(true)
            .with_untracked_files(true)
            .with_unmerged_commit_count(3)
            .with_stash(true)
            .with_pushed(false)
            .with_remove_succeeds(false)
            .with_delete_succeeds(false)
            .with_worktrees(vec![
                WorktreeInfo {
                    path: "/tmp/wt1".to_owned(),
                    branch: Some("feature-1".to_owned()),
                },
                WorktreeInfo {
                    path: "/tmp/wt2".to_owned(),
                    branch: None,
                },
            ]);

        let p = Path::new("/tmp/fake");

        assert!(mgr.has_uncommitted_changes(p).unwrap());
        assert!(mgr.has_untracked_files(p).unwrap());
        assert_eq!(mgr.unmerged_commit_count(p, "main").unwrap(), 3);
        assert!(mgr.has_stash(p).unwrap());
        assert!(!mgr.is_pushed_to_remote(p, "main").unwrap());
        assert!(mgr.remove_worktree(p, p).is_err());
        assert!(mgr.delete_branch(p, "branch").is_err());

        let wts = mgr.list_worktrees(p).unwrap();
        assert_eq!(wts.len(), 2);
        assert_eq!(wts[0].path, "/tmp/wt1");
        assert_eq!(wts[0].branch.as_deref(), Some("feature-1"));
        assert_eq!(wts[1].path, "/tmp/wt2");
        assert!(wts[1].branch.is_none());
    }
}
