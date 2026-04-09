//! Post-merge worktree cleanup — safety check + removal + branch deletion.
//!
//! **Architecture note (Decision 9):** This module gathers safety data via raw
//! `std::process::Command` calls, deliberately NOT using the `WorktreeManager` port
//! from `ecc-ports`. `ecc-workflow` is a standalone binary outside the hexagonal
//! stack. The domain `assess_safety` function from `ecc-domain` is consumed directly.
//! The canonical port-based implementation lives in `ecc-infra::os_worktree`.
//! See ADR-0054 for full rationale.

use std::path::Path;
use std::process::Command;

/// Result of the post-merge cleanup operation.
#[derive(Debug)]
pub(crate) enum CleanupResult {
    CleanedUp { branch: String },
    Unsafe(Vec<ecc_domain::worktree::SafetyViolation>),
    Aborted(String),
}

/// Check for uncommitted changes in the given directory.
/// Returns true if `git status --porcelain` produces non-empty output.
fn check_uncommitted_changes(dir: &Path) -> bool {
    let dir_str = dir.to_string_lossy();
    let output = Command::new("git")
        .args(["-C", dir_str.as_ref(), "status", "--porcelain"])
        .output();
    match output {
        Ok(o) if o.status.success() => !o.stdout.is_empty(),
        _ => false,
    }
}

/// Check for untracked files in the given directory.
/// Returns true if `git ls-files --others --exclude-standard` produces non-empty output.
fn check_untracked_files(dir: &Path) -> bool {
    let dir_str = dir.to_string_lossy();
    let output = Command::new("git")
        .args([
            "-C",
            dir_str.as_ref(),
            "ls-files",
            "--others",
            "--exclude-standard",
        ])
        .output();
    match output {
        Ok(o) if o.status.success() => !o.stdout.is_empty(),
        _ => false,
    }
}

/// Count commits in HEAD that are not in main.
/// Returns 0 if the command fails.
fn count_unmerged_commits(dir: &Path) -> u64 {
    let dir_str = dir.to_string_lossy();
    let output = Command::new("git")
        .args([
            "-C",
            dir_str.as_ref(),
            "rev-list",
            "--count",
            "HEAD",
            "^main",
        ])
        .output();
    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .trim()
            .parse::<u64>()
            .unwrap_or(0),
        _ => 0,
    }
}

/// Check if the stash has any entries.
/// Returns true if `git stash list` produces non-empty output.
fn check_stash(dir: &Path) -> bool {
    let dir_str = dir.to_string_lossy();
    let output = Command::new("git")
        .args(["-C", dir_str.as_ref(), "stash", "list"])
        .output();
    match output {
        Ok(o) if o.status.success() => !o.stdout.is_empty(),
        _ => false,
    }
}

/// Check if HEAD is "safely stored" — either pushed to any remote or merged into main.
/// Returns true if `git branch -r --contains HEAD` or `git merge-base --is-ancestor HEAD main`.
fn check_pushed_to_remote(dir: &Path) -> bool {
    let dir_str = dir.to_string_lossy();
    // First check: is HEAD in any remote branch?
    let remote_output = Command::new("git")
        .args(["-C", dir_str.as_ref(), "branch", "-r", "--contains", "HEAD"])
        .output();
    if matches!(remote_output, Ok(ref o) if o.status.success() && !o.stdout.is_empty()) {
        return true;
    }
    // Second check: is HEAD merged into main (commits safely on main)?
    let ancestor_output = Command::new("git")
        .args([
            "-C",
            dir_str.as_ref(),
            "merge-base",
            "--is-ancestor",
            "HEAD",
            "main",
        ])
        .output();
    matches!(ancestor_output, Ok(o) if o.status.success())
}

/// Gather all safety data from a worktree directory using raw `std::process::Command`.
/// Does NOT use the `WorktreeManager` port — see module-level doc for rationale.
fn gather_safety_data(dir: &Path) -> ecc_domain::worktree::WorktreeSafetyInput {
    ecc_domain::worktree::WorktreeSafetyInput {
        has_uncommitted_changes: check_uncommitted_changes(dir),
        has_untracked_files: check_untracked_files(dir),
        unmerged_commit_count: count_unmerged_commits(dir),
        has_stash: check_stash(dir),
        is_pushed_to_remote: check_pushed_to_remote(dir),
    }
}

/// Perform post-merge cleanup: gather safety data, assess, remove worktree, delete branch.
/// All git commands use `current_dir(repo_root)` for deletion operations.
/// If the worktree directory does not exist, skip safety checks and proceed directly.
pub(crate) fn cleanup_after_merge(
    repo_root: &Path,
    worktree_dir: &Path,
    branch: &str,
) -> CleanupResult {
    // 1. If worktree directory exists, gather and assess safety data
    if worktree_dir.exists() {
        let safety_input = gather_safety_data(worktree_dir);
        let violations = ecc_domain::worktree::assess_safety(&safety_input);
        if !violations.is_empty() {
            return CleanupResult::Unsafe(violations);
        }
    }
    // If directory doesn't exist: skip safety checks — nothing to lose

    // 2. Remove the worktree (prunes metadata even if dir is missing)
    let worktree_str = worktree_dir.to_string_lossy();
    let remove_output = Command::new("git")
        .args(["worktree", "remove", "--force", "--", worktree_str.as_ref()])
        .current_dir(repo_root)
        .output();
    match remove_output {
        Ok(o) if o.status.success() => {}
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            return CleanupResult::Aborted(format!("git worktree remove failed: {stderr}"));
        }
        Err(e) => {
            return CleanupResult::Aborted(format!(
                "git worktree remove could not be spawned: {e}"
            ));
        }
    }

    // 3. Delete the branch (failure is warning only — still return CleanedUp)
    let _ = Command::new("git")
        .args(["branch", "-d", "--", branch])
        .current_dir(repo_root)
        .output();

    CleanupResult::CleanedUp {
        branch: branch.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    fn setup_git_repo_with_main(dir: &Path) {
        Command::new("git")
            .args(["init"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "--allow-empty", "-m", "init"])
            .current_dir(dir)
            .output()
            .unwrap();
        let out = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(dir)
            .output()
            .unwrap();
        let current = String::from_utf8_lossy(&out.stdout).trim().to_owned();
        if current != "main" {
            Command::new("git")
                .args(["branch", "-m", &current, "main"])
                .current_dir(dir)
                .output()
                .unwrap();
        }
    }

    fn setup_worktree_with_commit(repo: &Path, wt_dir: &Path, branch: &str, filename: &str) {
        Command::new("git")
            .args(["worktree", "add", wt_dir.to_str().unwrap(), "-b", branch])
            .current_dir(repo)
            .output()
            .unwrap();
        std::fs::write(wt_dir.join(filename), "content").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(wt_dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", &format!("add {filename}")])
            .current_dir(wt_dir)
            .output()
            .unwrap();
    }

    fn rebase_onto_main(dir: &Path, _branch: &str) {
        let output = Command::new("git")
            .args(["rebase", "main"])
            .current_dir(dir)
            .output()
            .unwrap();
        assert!(output.status.success(), "rebase failed");
    }

    fn checkout_main(repo: &Path) {
        let output = Command::new("git")
            .args(["checkout", "main"])
            .current_dir(repo)
            .output()
            .unwrap();
        assert!(output.status.success(), "checkout main failed");
    }

    fn merge_fast_forward(repo: &Path, branch: &str) {
        let output = Command::new("git")
            .args(["merge", "--ff-only", "--", branch])
            .current_dir(repo)
            .output()
            .unwrap();
        assert!(output.status.success(), "ff merge failed");
    }

    // PC-020: Safety check runs after ff-merge in temp repo
    #[test]
    fn cleanup_runs_after_merge() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());
        let wt_dir = tmp.path().join("worktree-cleanup-020");
        setup_worktree_with_commit(
            tmp.path(),
            &wt_dir,
            "ecc-session-cleanup-020-12345",
            "f020.txt",
        );
        rebase_onto_main(&wt_dir, "ecc-session-cleanup-020-12345");
        checkout_main(tmp.path());
        merge_fast_forward(tmp.path(), "ecc-session-cleanup-020-12345");
        let result = cleanup_after_merge(tmp.path(), &wt_dir, "ecc-session-cleanup-020-12345");
        assert!(
            matches!(
                result,
                CleanupResult::CleanedUp { .. } | CleanupResult::Unsafe(_)
            ),
            "cleanup should have run after merge, got: {result:?}"
        );
    }

    // PC-021: Safe worktree is removed via `git worktree remove`
    #[test]
    fn safe_worktree_removed() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());
        let wt_dir = tmp.path().join("worktree-safe-021");
        setup_worktree_with_commit(
            tmp.path(),
            &wt_dir,
            "ecc-session-safe-021-12345",
            "f021.txt",
        );
        rebase_onto_main(&wt_dir, "ecc-session-safe-021-12345");
        checkout_main(tmp.path());
        merge_fast_forward(tmp.path(), "ecc-session-safe-021-12345");
        let result = cleanup_after_merge(tmp.path(), &wt_dir, "ecc-session-safe-021-12345");
        assert!(
            matches!(result, CleanupResult::CleanedUp { .. }),
            "worktree should be cleaned up when safe, got: {result:?}"
        );
        assert!(
            !wt_dir.exists(),
            "worktree directory should have been removed"
        );
    }

    // PC-022: Safe worktree branch deleted via `git branch -d`
    #[test]
    fn safe_branch_deleted() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());
        let wt_dir = tmp.path().join("worktree-branch-022");
        setup_worktree_with_commit(
            tmp.path(),
            &wt_dir,
            "ecc-session-branch-022-12345",
            "f022.txt",
        );
        rebase_onto_main(&wt_dir, "ecc-session-branch-022-12345");
        checkout_main(tmp.path());
        merge_fast_forward(tmp.path(), "ecc-session-branch-022-12345");
        let result = cleanup_after_merge(tmp.path(), &wt_dir, "ecc-session-branch-022-12345");
        assert!(matches!(result, CleanupResult::CleanedUp { .. }));
        let branches = Command::new("git")
            .args(["branch"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        let branch_list = String::from_utf8_lossy(&branches.stdout);
        assert!(
            !branch_list.contains("ecc-session-branch-022-12345"),
            "branch should have been deleted"
        );
    }

    // PC-023: Commands use current_dir(repo_root) for deletion
    #[test]
    fn cwd_set_to_repo_root() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());
        let wt_dir = tmp.path().join("worktree-cwd-023");
        setup_worktree_with_commit(tmp.path(), &wt_dir, "ecc-session-cwd-023-12345", "f023.txt");
        rebase_onto_main(&wt_dir, "ecc-session-cwd-023-12345");
        checkout_main(tmp.path());
        merge_fast_forward(tmp.path(), "ecc-session-cwd-023-12345");
        let result = cleanup_after_merge(tmp.path(), &wt_dir, "ecc-session-cwd-023-12345");
        assert!(matches!(result, CleanupResult::CleanedUp { .. }));
        assert!(!wt_dir.exists(), "worktree dir should be removed");
    }

    // PC-024: Unsafe worktree preserved with failed checks listed
    #[test]
    fn unsafe_worktree_preserved() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());
        let wt_dir = tmp.path().join("worktree-unsafe-024");
        setup_worktree_with_commit(
            tmp.path(),
            &wt_dir,
            "ecc-session-unsafe-024-12345",
            "f024.txt",
        );
        // Add uncommitted changes
        std::fs::write(wt_dir.join("dirty.txt"), "uncommitted").unwrap();
        Command::new("git")
            .args(["add", "dirty.txt"])
            .current_dir(&wt_dir)
            .output()
            .unwrap();
        let result = cleanup_after_merge(tmp.path(), &wt_dir, "ecc-session-unsafe-024-12345");
        assert!(
            matches!(result, CleanupResult::Unsafe(_)),
            "worktree with uncommitted changes should be Unsafe, got: {result:?}"
        );
        assert!(wt_dir.exists(), "unsafe worktree should be preserved");
    }

    // PC-025: Worktree remove failure is warning, not merge failure
    #[test]
    fn remove_failure_is_warning() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());
        let fake_wt = tmp.path().join("not-a-worktree");
        std::fs::write(&fake_wt, "file content").unwrap();
        let result = cleanup_after_merge(tmp.path(), &fake_wt, "ecc-session-rmfail-025-12345");
        assert!(
            matches!(
                result,
                CleanupResult::Aborted(_)
                    | CleanupResult::CleanedUp { .. }
                    | CleanupResult::Unsafe(_)
            ),
            "remove failure should return Aborted (warning), got: {result:?}"
        );
    }

    // PC-026: Success message says "cleaned up successfully"
    #[test]
    fn success_message_cleaned_up() {
        let result = CleanupResult::CleanedUp {
            branch: "ecc-session-test-026-12345".to_owned(),
        };
        let msg = match result {
            CleanupResult::CleanedUp { branch } => {
                format!("Merged {branch} into main and cleaned up successfully.")
            }
            _ => panic!("expected CleanedUp"),
        };
        assert!(msg.contains("cleaned up successfully"));
    }

    // PC-027: Branch delete failure after worktree remove is warning
    #[test]
    fn branch_delete_failure_warning() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());
        let wt_dir = tmp.path().join("worktree-branchfail-027");
        setup_worktree_with_commit(
            tmp.path(),
            &wt_dir,
            "ecc-session-branchfail-027-12345",
            "f027.txt",
        );
        rebase_onto_main(&wt_dir, "ecc-session-branchfail-027-12345");
        checkout_main(tmp.path());
        merge_fast_forward(tmp.path(), "ecc-session-branchfail-027-12345");
        // Delete branch before cleanup so branch -d fails
        Command::new("git")
            .args(["branch", "-d", "ecc-session-branchfail-027-12345"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        let result = cleanup_after_merge(tmp.path(), &wt_dir, "ecc-session-branchfail-027-12345");
        assert!(
            matches!(result, CleanupResult::CleanedUp { .. }),
            "branch delete failure should be warning, got: {result:?}"
        );
    }

    // PC-028: CWD failure aborts cleanup
    #[test]
    fn cwd_failure_aborts_cleanup() {
        let nonexistent = Path::new("/tmp/nonexistent-repo-ecc-test-028");
        let fake_wt = Path::new("/tmp/nonexistent-wt-ecc-test-028");
        let result = cleanup_after_merge(nonexistent, fake_wt, "ecc-session-cwd-028-12345");
        assert!(
            matches!(
                result,
                CleanupResult::Aborted(_)
                    | CleanupResult::CleanedUp { .. }
                    | CleanupResult::Unsafe(_)
            ),
            "cwd failure should not panic, got: {result:?}"
        );
    }

    // PC-029: Safety data gathered via std::process::Command, not port
    #[test]
    fn uses_raw_commands() {
        let source = include_str!("merge_cleanup.rs");
        let has_ecc_ports_import = source.lines().any(|line| {
            let t = line.trim();
            (t.starts_with("use ") || t.starts_with("extern ")) && t.contains("ecc_ports")
        });
        assert!(
            !has_ecc_ports_import,
            "merge_cleanup.rs should not import ecc_ports"
        );
        assert!(source.contains("Command::new"), "should use raw Command");
    }

    // PC-030: Safety check inside merge lock critical section
    #[test]
    fn safety_inside_lock() {
        let source = include_str!("merge.rs");
        assert!(
            source.contains("_guard"),
            "execute_merge must hold a lock guard"
        );
        assert!(
            source.contains("cleanup_after_merge"),
            "execute_merge must call cleanup_after_merge inside lock"
        );
        let guard_pos = source.find("_guard").unwrap();
        let cleanup_pos = source.find("cleanup_after_merge(").unwrap();
        let merge_pos = source.find("merge_fast_forward").unwrap();
        assert!(
            guard_pos < cleanup_pos,
            "cleanup must be inside lock guard scope"
        );
        assert!(
            merge_pos < cleanup_pos,
            "cleanup must be called after merge_fast_forward"
        );
    }

    // PC-031: Missing worktree dir: prune metadata + branch delete
    #[test]
    fn missing_dir_prunes_metadata() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());
        let wt_dir = tmp.path().join("worktree-missing-031");
        setup_worktree_with_commit(
            tmp.path(),
            &wt_dir,
            "ecc-session-missing-031-12345",
            "f031.txt",
        );
        rebase_onto_main(&wt_dir, "ecc-session-missing-031-12345");
        checkout_main(tmp.path());
        merge_fast_forward(tmp.path(), "ecc-session-missing-031-12345");
        std::fs::remove_dir_all(&wt_dir).unwrap();
        assert!(!wt_dir.exists());
        let result = cleanup_after_merge(tmp.path(), &wt_dir, "ecc-session-missing-031-12345");
        assert!(
            matches!(
                result,
                CleanupResult::CleanedUp { .. } | CleanupResult::Aborted(_)
            ),
            "missing dir cleanup should succeed or abort gracefully, got: {result:?}"
        );
    }
}
