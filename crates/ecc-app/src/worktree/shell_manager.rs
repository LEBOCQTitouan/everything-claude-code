//! [`ShellWorktreeManager`] — a [`WorktreeManager`] backed by a [`ShellExecutor`].
//!
//! Kept in ecc-app for use by session lifecycle hooks (which live in ecc-app).
//! Also available in ecc-infra for infrastructure-layer use.

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

    /// Run `git rev-list --count <range>` in `dir` and parse the numeric result.
    ///
    /// Returns `Err(WorktreeError::CommandFailed)` on non-zero exit or parse failure.
    fn run_git_count(
        shell: &dyn ShellExecutor,
        dir: &Path,
        range: &str,
    ) -> Result<u64, WorktreeError> {
        let out = shell
            .run_command_in_dir("git", &["rev-list", "--count", range], dir)
            .map_err(|e| WorktreeError::CommandFailed(e.to_string()))?;
        if !out.success() {
            return Err(WorktreeError::CommandFailed(out.stderr));
        }
        out.stdout
            .trim()
            .parse::<u64>()
            .map_err(|e| WorktreeError::CommandFailed(format!("failed to parse count: {e}")))
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
    fn has_uncommitted_changes(&self, worktree_path: &Path) -> Result<bool, WorktreeError> {
        let out = self
            .shell
            .run_command_in_dir("git", &["status", "--porcelain"], worktree_path)
            .map_err(|e| WorktreeError::CommandFailed(e.to_string()))?;
        if out.success() {
            Ok(!out.stdout.is_empty())
        } else {
            Err(WorktreeError::CommandFailed(out.stderr))
        }
    }

    fn has_untracked_files(&self, worktree_path: &Path) -> Result<bool, WorktreeError> {
        let out = self
            .shell
            .run_command_in_dir(
                "git",
                &["ls-files", "--others", "--exclude-standard"],
                worktree_path,
            )
            .map_err(|e| WorktreeError::CommandFailed(e.to_string()))?;
        if out.success() {
            Ok(!out.stdout.is_empty())
        } else {
            Err(WorktreeError::CommandFailed(out.stderr))
        }
    }

    fn unmerged_commit_count(
        &self,
        worktree_path: &Path,
        target_branch: &str,
    ) -> Result<u64, WorktreeError> {
        let range = format!("{target_branch}..HEAD");
        Self::run_git_count(self.shell, worktree_path, &range)
    }

    /// Check whether the current stash list is non-empty.
    ///
    /// **Note**: `git stash` is repo-global, not worktree-scoped. This method
    /// reports stash entries for the entire repository regardless of which
    /// worktree path is supplied.
    fn has_stash(&self, worktree_path: &Path) -> Result<bool, WorktreeError> {
        let out = self
            .shell
            .run_command_in_dir("git", &["stash", "list"], worktree_path)
            .map_err(|e| WorktreeError::CommandFailed(e.to_string()))?;
        if out.success() {
            Ok(!out.stdout.is_empty())
        } else {
            Err(WorktreeError::CommandFailed(out.stderr))
        }
    }

    fn is_pushed_to_remote(
        &self,
        worktree_path: &Path,
        branch: &str,
    ) -> Result<bool, WorktreeError> {
        let range = format!("{branch}..origin/{branch}");
        // Non-zero exit (e.g., remote not found) → treat as unpushed, not an error.
        match Self::run_git_count(self.shell, worktree_path, &range) {
            Ok(count) => Ok(count == 0),
            Err(_) => Ok(false),
        }
    }

    fn remove_worktree(&self, repo_root: &Path, worktree_path: &Path) -> Result<(), WorktreeError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_ports::shell::CommandOutput;
    use ecc_test_support::MockExecutor;
    use std::path::PathBuf;

    fn ok_output(stdout: &str) -> CommandOutput {
        CommandOutput {
            stdout: stdout.to_string(),
            stderr: String::new(),
            exit_code: 0,
        }
    }

    fn fail_output(stderr: &str) -> CommandOutput {
        CommandOutput {
            stdout: String::new(),
            stderr: stderr.to_string(),
            exit_code: 1,
        }
    }

    fn dummy_path() -> PathBuf {
        PathBuf::from("/repo/worktree")
    }

    // PC-001: unmerged_commit_count invokes `git rev-list --count <base>..HEAD`
    #[test]
    fn unmerged_commit_count_invokes_git_rev_list() {
        let mock = MockExecutor::new()
            .on_args("git", &["rev-list", "--count", "main..HEAD"], ok_output("3\n"));
        let mgr = ShellWorktreeManager::new(&mock);
        let count = mgr
            .unmerged_commit_count(&dummy_path(), "main")
            .expect("unmerged_commit_count");
        assert_eq!(count, 3);
    }

    // PC-002: has_uncommitted_changes checks `git status --porcelain` non-empty
    #[test]
    fn has_uncommitted_changes_checks_porcelain() {
        // non-empty → true
        let mock = MockExecutor::new()
            .on_args("git", &["status", "--porcelain"], ok_output(" M src/lib.rs\n"));
        let mgr = ShellWorktreeManager::new(&mock);
        assert!(mgr.has_uncommitted_changes(&dummy_path()).expect("has_uncommitted_changes"));

        // empty → false
        let mock_clean = MockExecutor::new()
            .on_args("git", &["status", "--porcelain"], ok_output(""));
        let mgr_clean = ShellWorktreeManager::new(&mock_clean);
        assert!(!mgr_clean.has_uncommitted_changes(&dummy_path()).expect("has_uncommitted_changes"));
    }

    // PC-003: has_untracked_files via `git ls-files --others --exclude-standard`
    #[test]
    fn has_untracked_files_checks_ls_files() {
        // non-empty → true
        let mock = MockExecutor::new().on_args(
            "git",
            &["ls-files", "--others", "--exclude-standard"],
            ok_output("untracked.txt\n"),
        );
        let mgr = ShellWorktreeManager::new(&mock);
        assert!(mgr.has_untracked_files(&dummy_path()).expect("has_untracked_files"));

        // empty → false
        let mock_clean = MockExecutor::new().on_args(
            "git",
            &["ls-files", "--others", "--exclude-standard"],
            ok_output(""),
        );
        let mgr_clean = ShellWorktreeManager::new(&mock_clean);
        assert!(!mgr_clean.has_untracked_files(&dummy_path()).expect("has_untracked_files"));
    }

    // PC-004: has_stash via `git stash list` non-empty
    /// Note: stash is repo-global, not worktree-scoped.
    #[test]
    fn has_stash_checks_stash_list() {
        // non-empty → true
        let mock = MockExecutor::new()
            .on_args("git", &["stash", "list"], ok_output("stash@{0}: WIP on main\n"));
        let mgr = ShellWorktreeManager::new(&mock);
        assert!(mgr.has_stash(&dummy_path()).expect("has_stash"));

        // empty → false
        let mock_empty = MockExecutor::new()
            .on_args("git", &["stash", "list"], ok_output(""));
        let mgr_empty = ShellWorktreeManager::new(&mock_empty);
        assert!(!mgr_empty.has_stash(&dummy_path()).expect("has_stash"));
    }

    // PC-005: is_pushed_to_remote via `git rev-list --count <branch>..origin/<branch>` returns 0
    #[test]
    fn is_pushed_checks_count_against_origin() {
        // 0 unpushed commits → pushed
        let mock = MockExecutor::new().on_args(
            "git",
            &["rev-list", "--count", "main..origin/main"],
            ok_output("0\n"),
        );
        let mgr = ShellWorktreeManager::new(&mock);
        assert!(mgr.is_pushed_to_remote(&dummy_path(), "main").expect("is_pushed_to_remote"));

        // non-zero → not pushed
        let mock_unpushed = MockExecutor::new().on_args(
            "git",
            &["rev-list", "--count", "main..origin/main"],
            ok_output("2\n"),
        );
        let mgr_unpushed = ShellWorktreeManager::new(&mock_unpushed);
        assert!(!mgr_unpushed.is_pushed_to_remote(&dummy_path(), "main").expect("is_pushed_to_remote"));
    }

    // PC-006: Shell failure → Err(WorktreeError) propagation
    #[test]
    fn shell_failure_propagates_as_err() {
        let mock = MockExecutor::new()
            .on_args("git", &["status", "--porcelain"], fail_output("fatal: not a git repo"));
        let mgr = ShellWorktreeManager::new(&mock);
        let result = mgr.has_uncommitted_changes(&dummy_path());
        assert!(result.is_err(), "expected Err on git failure");
        match result.unwrap_err() {
            WorktreeError::CommandFailed(msg) => {
                assert!(msg.contains("fatal: not a git repo"), "error should contain stderr");
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    // PC-008: Non-numeric stdout → Err(ParseError)
    #[test]
    fn non_numeric_stdout_propagates_err() {
        let mock = MockExecutor::new()
            .on_args("git", &["rev-list", "--count", "main..HEAD"], ok_output("not-a-number\n"));
        let mgr = ShellWorktreeManager::new(&mock);
        let result = mgr.unmerged_commit_count(&dummy_path(), "main");
        assert!(result.is_err(), "expected Err for non-numeric stdout");
    }

    // PC-009: BOM/locale noise in stdout → Err
    #[test]
    fn bom_in_stdout_propagates_err() {
        // BOM prefix \u{FEFF} before the digit — trim should not strip BOM, parse must fail
        let mock = MockExecutor::new().on_args(
            "git",
            &["rev-list", "--count", "main..HEAD"],
            ok_output("\u{feff}5\n"),
        );
        let mgr = ShellWorktreeManager::new(&mock);
        let result = mgr.unmerged_commit_count(&dummy_path(), "main");
        assert!(result.is_err(), "expected Err for BOM-prefixed stdout");
    }

    // PC-012: Remote-not-found → Ok(false) graceful
    #[test]
    fn remote_not_found_returns_unpushed() {
        // Non-zero exit when remote doesn't exist → Ok(false), NOT Err
        let mock = MockExecutor::new().on_args(
            "git",
            &["rev-list", "--count", "main..origin/main"],
            fail_output("fatal: ambiguous argument 'origin/main': unknown revision"),
        );
        let mgr = ShellWorktreeManager::new(&mock);
        let result = mgr.is_pushed_to_remote(&dummy_path(), "main");
        assert!(result.is_ok(), "remote-not-found should be Ok(false), not Err");
        assert!(!result.unwrap(), "remote-not-found should return false (unpushed)");
    }
}
