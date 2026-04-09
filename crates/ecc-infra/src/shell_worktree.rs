//! [`ShellWorktreeManager`] — a [`WorktreeManager`] backed by a [`ShellExecutor`].
//!
//! Parses `git worktree list --porcelain` output for listing, and uses raw git
//! commands for removal, matching the previous GC implementation.

use ecc_ports::shell::ShellExecutor;
use ecc_ports::worktree::{WorktreeError, WorktreeInfo, WorktreeManager};
use std::path::Path;

/// A `WorktreeManager` that delegates list/remove/branch operations to a
/// [`ShellExecutor`]. Used when only a shell is available (e.g., session hooks).
///
/// Parses `git worktree list --porcelain` output for listing, and uses raw
/// git commands for removal — matching the previous GC implementation.
pub struct ShellWorktreeManager<'a> {
    shell: &'a dyn ShellExecutor,
}

impl<'a> ShellWorktreeManager<'a> {
    /// Create a new manager backed by the given shell executor.
    pub fn new(shell: &'a dyn ShellExecutor) -> Self {
        Self { shell }
    }

    fn parse_porcelain(output: &str) -> Vec<WorktreeInfo> {
        let mut out = Vec::new();
        let mut cur_path: Option<String> = None;
        let mut cur_branch: Option<String> = None;

        for line in output.lines() {
            if let Some(path) = line.strip_prefix("worktree ") {
                if let Some(p) = cur_path.take() {
                    out.push(WorktreeInfo {
                        path: p,
                        branch: cur_branch.take(),
                    });
                }
                cur_path = Some(path.to_owned());
                cur_branch = None;
            } else if let Some(branch_ref) = line.strip_prefix("branch ") {
                let name = branch_ref
                    .strip_prefix("refs/heads/")
                    .unwrap_or(branch_ref)
                    .to_owned();
                cur_branch = Some(name);
            }
        }
        if let Some(p) = cur_path {
            out.push(WorktreeInfo {
                path: p,
                branch: cur_branch,
            });
        }
        out
    }
}

impl WorktreeManager for ShellWorktreeManager<'_> {
    fn has_uncommitted_changes(&self, _worktree_path: &Path) -> Result<bool, WorktreeError> {
        Ok(false)
    }

    fn has_untracked_files(&self, _worktree_path: &Path) -> Result<bool, WorktreeError> {
        Ok(false)
    }

    fn unmerged_commit_count(
        &self,
        _worktree_path: &Path,
        _target_branch: &str,
    ) -> Result<u64, WorktreeError> {
        Ok(0)
    }

    fn has_stash(&self, _worktree_path: &Path) -> Result<bool, WorktreeError> {
        Ok(false)
    }

    fn is_pushed_to_remote(
        &self,
        _worktree_path: &Path,
        _branch: &str,
    ) -> Result<bool, WorktreeError> {
        Ok(true)
    }

    fn remove_worktree(
        &self,
        repo_root: &Path,
        worktree_path: &Path,
    ) -> Result<(), WorktreeError> {
        let path_str = worktree_path.to_string_lossy();
        let out = self
            .shell
            .run_command_in_dir(
                "git",
                &["worktree", "remove", "--force", "--", &path_str],
                repo_root,
            )
            .map_err(|e| WorktreeError::CommandFailed(e.to_string()))?;
        if out.success() {
            Ok(())
        } else {
            Err(WorktreeError::CommandFailed(out.stderr))
        }
    }

    fn delete_branch(&self, repo_root: &Path, branch: &str) -> Result<(), WorktreeError> {
        let out = self
            .shell
            .run_command_in_dir("git", &["branch", "-D", "--", branch], repo_root)
            .map_err(|e| WorktreeError::CommandFailed(e.to_string()))?;
        if out.success() {
            Ok(())
        } else {
            Err(WorktreeError::CommandFailed(out.stderr))
        }
    }

    fn list_worktrees(&self, repo_root: &Path) -> Result<Vec<WorktreeInfo>, WorktreeError> {
        let out = self
            .shell
            .run_command_in_dir("git", &["worktree", "list", "--porcelain"], repo_root)
            .map_err(|e| WorktreeError::CommandFailed(e.to_string()))?;
        Ok(Self::parse_porcelain(&out.stdout))
    }
}
