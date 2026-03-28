//! Integration tests for `ecc worktree gc` end-to-end with a real git repo.

use assert_cmd::Command;
use std::path::Path;
use tempfile::TempDir;

/// Create a temp dir, init a git repo, make an initial commit.
fn init_git_repo() -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path();

    run_git(path, &["init"]);
    run_git(path, &["config", "user.email", "test@ecc.test"]);
    run_git(path, &["config", "user.name", "ECC Test"]);

    // Create an initial commit so worktrees can be added
    let readme = path.join("README.md");
    std::fs::write(&readme, "# test\n").expect("failed to write README");
    run_git(path, &["add", "README.md"]);
    run_git(path, &["commit", "-m", "Initial commit"]);

    dir
}

fn run_git(dir: &Path, args: &[&str]) {
    let status = std::process::Command::new("git")
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

fn ecc_cmd() -> Command {
    Command::cargo_bin("ecc").expect("ecc binary not found")
}

/// A stale session worktree name: PID 999999 is almost certainly dead,
/// and the timestamp is in the past (year 2020).
const STALE_WORKTREE: &str = "ecc-session-20200101-120000-test-999999";

#[test]
#[ignore] // Requires: git repo with worktrees
fn gc_removes_stale_session_worktree() {
    let repo = init_git_repo();
    let repo_path = repo.path();

    // Create a branch for the worktree
    run_git(repo_path, &["branch", STALE_WORKTREE]);

    // Add a worktree with the ecc-session-* name
    let worktree_path = repo_path.join(STALE_WORKTREE);
    run_git(
        repo_path,
        &[
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
            STALE_WORKTREE,
        ],
    );

    // Verify the worktree exists before GC
    assert!(
        worktree_path.exists(),
        "worktree directory should exist before gc"
    );

    // Run `ecc worktree gc` on the repo
    let output = ecc_cmd()
        .args(["worktree", "gc"])
        .current_dir(repo_path)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // The stale worktree should be reported as removed
    assert!(
        stdout.contains("Removed") || stdout.contains(STALE_WORKTREE),
        "expected 'Removed' or worktree name in output, got: {stdout}"
    );

    // The worktree directory should be gone
    assert!(
        !worktree_path.exists(),
        "stale worktree directory should be removed after gc"
    );
}

#[test]
#[ignore] // Requires: git repo
fn gc_reports_no_worktrees_when_none_exist() {
    let repo = init_git_repo();
    let repo_path = repo.path();

    // No ecc-session-* worktrees — only the main worktree
    let output = ecc_cmd()
        .args(["worktree", "gc"])
        .current_dir(repo_path)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    assert!(
        stdout.contains("No ECC session worktrees found"),
        "expected 'No ECC session worktrees found' in output, got: {stdout}"
    );
}
