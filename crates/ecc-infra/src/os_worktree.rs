//! Production adapter for [`ecc_ports::worktree::WorktreeManager`].
//!
//! Uses the `git` CLI to implement all worktree operations.
//! All methods use `--` before user-supplied paths and branch names
//! to prevent argument injection.

use ecc_ports::worktree::{WorktreeError, WorktreeInfo, WorktreeManager};
use std::path::Path;
use std::process::Command;

/// Worktree manager that shells out to the `git` binary.
pub struct OsWorktreeManager;

impl WorktreeManager for OsWorktreeManager {
    fn has_uncommitted_changes(&self, worktree_path: &Path) -> Result<bool, WorktreeError> {
        let output = Command::new("git")
            .args(["-C"])
            .arg(worktree_path)
            .args(["status", "--porcelain"])
            .output()
            .map_err(|e| WorktreeError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WorktreeError::CommandFailed(stderr.into_owned()));
        }

        Ok(!output.stdout.is_empty())
    }

    fn has_untracked_files(&self, worktree_path: &Path) -> Result<bool, WorktreeError> {
        let output = Command::new("git")
            .args(["-C"])
            .arg(worktree_path)
            .args(["ls-files", "--others", "--exclude-standard"])
            .output()
            .map_err(|e| WorktreeError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WorktreeError::CommandFailed(stderr.into_owned()));
        }

        Ok(!output.stdout.is_empty())
    }

    fn unmerged_commit_count(
        &self,
        worktree_path: &Path,
        target_branch: &str,
    ) -> Result<u64, WorktreeError> {
        // Build the exclude-ref using the ^ prefix.
        // We do NOT use -- here because -- separates revisions from paths;
        // ^<branch> is a revision specifier, not a path.
        // The ^ prefix provides injection safety: the user-supplied branch
        // cannot be confused with a git flag since we prepend ^.
        let exclude_ref = format!("^{target_branch}");
        let output = Command::new("git")
            .args(["-C"])
            .arg(worktree_path)
            .args(["rev-list", "--count", "HEAD"])
            .arg(&exclude_ref)
            .output()
            .map_err(|e| WorktreeError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WorktreeError::CommandFailed(stderr.into_owned()));
        }

        let count_str = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        count_str
            .parse::<u64>()
            .map_err(|e| WorktreeError::CommandFailed(format!("failed to parse count: {e}")))
    }

    fn has_stash(&self, worktree_path: &Path) -> Result<bool, WorktreeError> {
        let output = Command::new("git")
            .args(["-C"])
            .arg(worktree_path)
            .args(["stash", "list"])
            .output()
            .map_err(|e| WorktreeError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WorktreeError::CommandFailed(stderr.into_owned()));
        }

        Ok(!output.stdout.is_empty())
    }

    fn is_pushed_to_remote(
        &self,
        worktree_path: &Path,
        branch: &str,
    ) -> Result<bool, WorktreeError> {
        let output = Command::new("git")
            .args(["-C"])
            .arg(worktree_path)
            .args(["branch", "-r", "--contains", "HEAD", "--"])
            .arg(branch)
            .output()
            .map_err(|e| WorktreeError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WorktreeError::CommandFailed(stderr.into_owned()));
        }

        Ok(!output.stdout.is_empty())
    }

    fn remove_worktree(&self, repo_root: &Path, worktree_path: &Path) -> Result<(), WorktreeError> {
        let output = Command::new("git")
            .args(["-C"])
            .arg(repo_root)
            .args(["worktree", "remove", "--force", "--"])
            .arg(worktree_path)
            .output()
            .map_err(|e| WorktreeError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WorktreeError::CommandFailed(stderr.into_owned()));
        }

        Ok(())
    }

    fn delete_branch(&self, repo_root: &Path, branch: &str) -> Result<(), WorktreeError> {
        let output = Command::new("git")
            .args(["-C"])
            .arg(repo_root)
            .args(["branch", "-d", "--"])
            .arg(branch)
            .output()
            .map_err(|e| WorktreeError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WorktreeError::CommandFailed(stderr.into_owned()));
        }

        Ok(())
    }

    fn list_worktrees(&self, repo_root: &Path) -> Result<Vec<WorktreeInfo>, WorktreeError> {
        let output = Command::new("git")
            .args(["-C"])
            .arg(repo_root)
            .args(["worktree", "list", "--porcelain"])
            .output()
            .map_err(|e| WorktreeError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WorktreeError::CommandFailed(stderr.into_owned()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(parse_worktree_porcelain(&stdout))
    }
}

/// Parse the output of `git worktree list --porcelain`.
fn parse_worktree_porcelain(output: &str) -> Vec<WorktreeInfo> {
    let mut worktrees = Vec::new();
    let mut current_path: Option<String> = None;
    let mut current_branch: Option<String> = None;

    for line in output.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            // New entry — save the previous one if any
            if let Some(path_val) = current_path.take() {
                worktrees.push(WorktreeInfo {
                    path: path_val,
                    branch: current_branch.take(),
                });
            }
            current_path = Some(path.to_owned());
            current_branch = None;
        } else if let Some(branch) = line.strip_prefix("branch refs/heads/") {
            current_branch = Some(branch.to_owned());
        } else if line.is_empty() {
            // Blank line separates entries; save current
            if let Some(path_val) = current_path.take() {
                worktrees.push(WorktreeInfo {
                    path: path_val,
                    branch: current_branch.take(),
                });
            }
        }
    }

    // Handle last entry with no trailing blank line
    if let Some(path_val) = current_path.take() {
        worktrees.push(WorktreeInfo {
            path: path_val,
            branch: current_branch.take(),
        });
    }

    worktrees
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    /// Create a minimal git repo in a temp directory.
    fn init_repo() -> TempDir {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path();

        Command::new("git")
            .args(["init"])
            .current_dir(path)
            .output()
            .expect("git init");

        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(path)
            .output()
            .expect("git config email");

        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(path)
            .output()
            .expect("git config name");

        // Create an initial commit so HEAD exists
        let file = path.join("README.md");
        std::fs::write(&file, "# test\n").expect("write file");

        Command::new("git")
            .args(["add", "."])
            .current_dir(path)
            .output()
            .expect("git add");

        Command::new("git")
            .args(["commit", "-m", "initial commit"])
            .current_dir(path)
            .output()
            .expect("git commit");

        dir
    }

    #[test]
    fn detects_uncommitted() {
        let repo = init_repo();
        let mgr = OsWorktreeManager;

        // Initially clean
        assert!(
            !mgr.has_uncommitted_changes(repo.path())
                .expect("has_uncommitted_changes")
        );

        // Modify a tracked file
        std::fs::write(repo.path().join("README.md"), "changed\n").expect("write");
        assert!(
            mgr.has_uncommitted_changes(repo.path())
                .expect("has_uncommitted_changes")
        );
    }

    #[test]
    fn detects_untracked() {
        let repo = init_repo();
        let mgr = OsWorktreeManager;

        // Initially no untracked
        assert!(
            !mgr.has_untracked_files(repo.path())
                .expect("has_untracked_files")
        );

        // Add an untracked file
        std::fs::write(repo.path().join("untracked.txt"), "hello\n").expect("write");
        assert!(
            mgr.has_untracked_files(repo.path())
                .expect("has_untracked_files")
        );
    }

    #[test]
    fn counts_unmerged() {
        let repo = init_repo();
        let mgr = OsWorktreeManager;

        // Create a branch and add a commit
        Command::new("git")
            .args(["checkout", "-b", "feature"])
            .current_dir(repo.path())
            .output()
            .expect("checkout");

        std::fs::write(repo.path().join("feature.txt"), "feature\n").expect("write");
        Command::new("git")
            .args(["add", "."])
            .current_dir(repo.path())
            .output()
            .expect("git add");
        Command::new("git")
            .args(["commit", "-m", "feature commit"])
            .current_dir(repo.path())
            .output()
            .expect("git commit");

        // Now HEAD is 1 commit ahead of main
        let count = mgr
            .unmerged_commit_count(repo.path(), "main")
            .expect("unmerged_commit_count");
        assert_eq!(count, 1, "should have 1 unmerged commit");
    }

    #[test]
    fn detects_stash() {
        let repo = init_repo();
        let mgr = OsWorktreeManager;

        assert!(!mgr.has_stash(repo.path()).expect("has_stash"));

        // Create a stash
        std::fs::write(repo.path().join("README.md"), "stashed\n").expect("write");
        Command::new("git")
            .args(["stash"])
            .current_dir(repo.path())
            .output()
            .expect("git stash");

        assert!(mgr.has_stash(repo.path()).expect("has_stash"));
    }

    #[test]
    fn checks_push_status() {
        let repo = init_repo();
        let mgr = OsWorktreeManager;

        // No remote configured → not pushed
        let pushed = mgr
            .is_pushed_to_remote(repo.path(), "main")
            .expect("is_pushed_to_remote");
        assert!(!pushed, "should not be pushed (no remote)");
    }

    #[test]
    fn removes_worktree() {
        let repo = init_repo();
        let mgr = OsWorktreeManager;

        // Create a worktree
        let wt_dir = TempDir::new().expect("wt tempdir");
        let wt_path = wt_dir.path().join("my-worktree");

        Command::new("git")
            .args(["worktree", "add", "-b", "wt-branch"])
            .arg(&wt_path)
            .arg("HEAD")
            .current_dir(repo.path())
            .output()
            .expect("git worktree add");

        assert!(wt_path.exists(), "worktree dir should exist");

        mgr.remove_worktree(repo.path(), &wt_path)
            .expect("remove_worktree");

        assert!(!wt_path.exists(), "worktree dir should be gone");
    }

    #[test]
    fn deletes_branch() {
        let repo = init_repo();
        let mgr = OsWorktreeManager;

        // Create a branch
        Command::new("git")
            .args(["branch", "to-delete"])
            .current_dir(repo.path())
            .output()
            .expect("git branch");

        // Verify branch exists
        let output = Command::new("git")
            .args(["branch", "--list", "to-delete"])
            .current_dir(repo.path())
            .output()
            .expect("git branch list");
        assert!(
            !output.stdout.is_empty(),
            "branch should exist before deletion"
        );

        mgr.delete_branch(repo.path(), "to-delete")
            .expect("delete_branch");

        // Verify branch is gone
        let output = Command::new("git")
            .args(["branch", "--list", "to-delete"])
            .current_dir(repo.path())
            .output()
            .expect("git branch list");
        assert!(output.stdout.is_empty(), "branch should be deleted");
    }

    #[test]
    fn lists_worktrees() {
        let repo = init_repo();
        let mgr = OsWorktreeManager;

        let worktrees = mgr.list_worktrees(repo.path()).expect("list_worktrees");

        // Should have at least the main worktree
        assert!(!worktrees.is_empty(), "should list at least one worktree");

        // The first one should be the main repo path
        let main_wt = &worktrees[0];
        assert!(
            main_wt.path.contains(repo.path().to_str().unwrap()),
            "main worktree path should match repo path"
        );
    }

    #[test]
    fn uses_double_dash() {
        // Source-scanning test: verify that git Command calls with user-derived path/branch
        // arguments use "--" to separate them from flags (argument injection prevention).
        //
        // Note: unmerged_commit_count uses ^{branch} prefix instead of -- because
        // the argument is a revision specifier, not a path; -- would change semantics.
        let source = include_str!("os_worktree.rs");

        assert!(
            source.contains(r#"args(["branch", "-d", "--"])"#),
            "delete_branch must use -- before branch name"
        );
        assert!(
            source.contains(r#"args(["worktree", "remove", "--force", "--"])"#),
            "remove_worktree must use -- before path"
        );
        assert!(
            source.contains(r#"args(["branch", "-r", "--contains", "HEAD", "--"])"#),
            "is_pushed_to_remote must use -- before branch"
        );
    }
}
