//! Real-git integration tests for [`ShellWorktreeManager`].
//!
//! All tests are `#[ignore]`'d per e2e-testing convention — run with:
//!   `cargo test -p ecc-integration-tests -- --ignored shell_worktree_manager_real_git`

use ecc_app::worktree::shell_manager::ShellWorktreeManager;
use ecc_infra::process_executor::ProcessExecutor;
use ecc_ports::worktree::WorktreeManager;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Create a minimal git repo in a temp directory with an initial commit.
fn init_repo() -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path();

    git(path, &["init"]);
    git(path, &["config", "user.email", "test@ecc.test"]);
    git(path, &["config", "user.name", "ECC Test"]);

    std::fs::write(path.join("README.md"), "# test\n").expect("write README");
    git(path, &["add", "README.md"]);
    git(path, &["commit", "-m", "initial commit"]);

    dir
}

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .unwrap_or_else(|e| panic!("git {args:?} failed to start: {e}"));
    assert!(
        status.success(),
        "git {args:?} exited non-zero in {}",
        dir.display()
    );
}

#[test]
#[ignore] // Requires: filesystem + git binary
fn shell_worktree_manager_real_git_has_no_uncommitted_changes_on_clean_repo() {
    let repo = init_repo();
    let executor = ProcessExecutor;
    let mgr = ShellWorktreeManager::new(&executor);

    let result = mgr
        .has_uncommitted_changes(repo.path())
        .expect("has_uncommitted_changes");
    assert!(!result, "clean repo should have no uncommitted changes");
}

#[test]
#[ignore] // Requires: filesystem + git binary
fn shell_worktree_manager_real_git_detects_uncommitted_changes() {
    let repo = init_repo();
    std::fs::write(repo.path().join("README.md"), "modified\n").expect("write");

    let executor = ProcessExecutor;
    let mgr = ShellWorktreeManager::new(&executor);

    let result = mgr
        .has_uncommitted_changes(repo.path())
        .expect("has_uncommitted_changes");
    assert!(result, "modified tracked file should appear as uncommitted change");
}

#[test]
#[ignore] // Requires: filesystem + git binary
fn shell_worktree_manager_real_git_detects_untracked_files() {
    let repo = init_repo();
    let executor = ProcessExecutor;
    let mgr = ShellWorktreeManager::new(&executor);

    // Initially no untracked files
    assert!(
        !mgr.has_untracked_files(repo.path())
            .expect("has_untracked_files"),
        "clean repo should have no untracked files"
    );

    std::fs::write(repo.path().join("new.txt"), "untracked\n").expect("write");
    assert!(
        mgr.has_untracked_files(repo.path())
            .expect("has_untracked_files"),
        "new file should be reported as untracked"
    );
}

#[test]
#[ignore] // Requires: filesystem + git binary
fn shell_worktree_manager_real_git_counts_unmerged_commits() {
    let repo = init_repo();
    git(repo.path(), &["checkout", "-b", "feature"]);
    std::fs::write(repo.path().join("feature.txt"), "feature\n").expect("write");
    git(repo.path(), &["add", "feature.txt"]);
    git(repo.path(), &["commit", "-m", "feature commit"]);

    let executor = ProcessExecutor;
    let mgr = ShellWorktreeManager::new(&executor);

    let count = mgr
        .unmerged_commit_count(repo.path(), "main")
        .expect("unmerged_commit_count");
    assert_eq!(count, 1, "feature branch should have 1 unmerged commit");
}

#[test]
#[ignore] // Requires: filesystem + git binary
fn shell_worktree_manager_real_git_no_remote_returns_not_pushed() {
    let repo = init_repo();
    let executor = ProcessExecutor;
    let mgr = ShellWorktreeManager::new(&executor);

    // No remote configured — should return false (not pushed), not an error
    let pushed = mgr
        .is_pushed_to_remote(repo.path(), "main")
        .expect("is_pushed_to_remote should not error when remote missing");
    assert!(!pushed, "no remote configured → not pushed");
}
