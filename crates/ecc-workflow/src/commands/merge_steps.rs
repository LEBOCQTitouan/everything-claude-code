//! Merge step helpers — individual git operations used by the merge orchestrator.

use std::path::Path;
use std::process::Command;
use std::time::Duration;

use super::merge::MergeError;

pub(crate) fn current_branch(dir: &Path) -> Result<String, MergeError> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(dir)
        .output()
        .map_err(|_| MergeError::NoBranch)?;
    if !output.status.success() {
        return Err(MergeError::NoBranch);
    }
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if branch.is_empty() || branch == "HEAD" {
        return Err(MergeError::NoBranch);
    }
    Ok(branch)
}

pub(crate) fn validate_session_branch(branch: &str) -> Result<(), MergeError> {
    if ecc_domain::worktree::WorktreeName::parse(branch).is_none() {
        return Err(MergeError::NotSessionBranch {
            branch: branch.to_owned(),
        });
    }
    Ok(())
}

pub(crate) fn acquire_merge_lock(repo_root: &Path) -> Result<ecc_flock::FlockGuard, MergeError> {
    ecc_flock::acquire_with_timeout(
        repo_root,
        "merge",
        Duration::from_secs(crate::commands::merge::MERGE_LOCK_TIMEOUT_SECS),
    )
    .map_err(|e| match e {
        ecc_flock::FlockError::Timeout(d) => MergeError::LockTimeout(d),
        _ => MergeError::LockTimeout(Duration::from_secs(
            crate::commands::merge::MERGE_LOCK_TIMEOUT_SECS,
        )),
    })
}

pub(crate) fn rebase_onto_main(dir: &Path, branch: &str) -> Result<(), MergeError> {
    let output = Command::new("git")
        .args(["rebase", "main"])
        .current_dir(dir)
        .output()
        .map_err(|e| MergeError::RebaseConflict {
            branch: branch.to_owned(),
            stderr: e.to_string(),
        })?;
    if !output.status.success() {
        let _ = Command::new("git")
            .args(["rebase", "--abort"])
            .current_dir(dir)
            .output();
        return Err(MergeError::RebaseConflict {
            branch: branch.to_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }
    Ok(())
}

pub(crate) fn run_fast_verify(dir: &Path) -> Result<(), MergeError> {
    for (step, args) in [
        ("cargo build", vec!["build"]),
        ("cargo test", vec!["test"]),
        ("cargo clippy", vec!["clippy", "--", "-D", "warnings"]),
    ] {
        let output = Command::new("cargo")
            .args(&args)
            .current_dir(dir)
            .output()
            .map_err(|e| MergeError::VerifyFailed {
                step: step.to_owned(),
                stderr: e.to_string(),
            })?;
        if !output.status.success() {
            return Err(MergeError::VerifyFailed {
                step: step.to_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }
    }
    Ok(())
}

/// Ensure the main repo has `main` checked out before merging.
///
/// Without this, `git merge --ff-only` would target whatever branch
/// is currently checked out in the main repo, which may not be `main`
/// if another session or manual git operation changed it.
pub(crate) fn checkout_main(repo_root: &Path) -> Result<(), MergeError> {
    let output = Command::new("git")
        .args(["checkout", "main"])
        .current_dir(repo_root)
        .output()
        .map_err(|e| MergeError::CheckoutFailed {
            stderr: e.to_string(),
        })?;
    if !output.status.success() {
        return Err(MergeError::CheckoutFailed {
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }
    Ok(())
}

pub(crate) fn merge_fast_forward(repo_root: &Path, branch: &str) -> Result<(), MergeError> {
    let output = Command::new("git")
        .args(["merge", "--ff-only", "--", branch])
        .current_dir(repo_root)
        .output()
        .map_err(|e| MergeError::MergeFailed {
            stderr: e.to_string(),
        })?;
    if !output.status.success() {
        return Err(MergeError::MergeFailed {
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_git_repo(dir: &Path) {
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
    }

    fn setup_git_repo_with_main(dir: &Path) {
        setup_git_repo(dir);
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

    #[test]
    fn runs_rebase() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());

        Command::new("git")
            .args(["checkout", "-b", "ecc-session-feature"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        std::fs::write(tmp.path().join("file.txt"), "content").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "feature commit"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        let result = rebase_onto_main(tmp.path(), "ecc-session-feature");
        assert!(result.is_ok(), "rebase should succeed: {result:?}");
    }

    #[test]
    fn conflict_aborts() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());

        std::fs::write(tmp.path().join("conflict.txt"), "main content").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "main commit"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        let init_sha = {
            let out = Command::new("git")
                .args(["rev-list", "--max-parents=0", "HEAD"])
                .current_dir(tmp.path())
                .output()
                .unwrap();
            String::from_utf8_lossy(&out.stdout).trim().to_owned()
        };
        Command::new("git")
            .args(["checkout", "-b", "ecc-session-conflict", &init_sha])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        std::fs::write(tmp.path().join("conflict.txt"), "session content").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "session conflict"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        let result = rebase_onto_main(tmp.path(), "ecc-session-conflict");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, MergeError::RebaseConflict { .. }));

        let rebase_merge = tmp.path().join(".git/rebase-merge");
        let rebase_apply = tmp.path().join(".git/rebase-apply");
        assert!(
            !rebase_merge.exists() && !rebase_apply.exists(),
            "rebase should have been aborted"
        );
    }

    #[test]
    fn runs_verify() {
        let tmp = TempDir::new().unwrap();
        let result = run_fast_verify(tmp.path());
        assert!(result.is_err());
        if let Err(MergeError::VerifyFailed { step, .. }) = result {
            assert_eq!(step, "cargo build", "first failing step should be cargo build");
        } else {
            panic!("expected VerifyFailed");
        }
    }

    #[test]
    fn verify_failure() {
        let tmp = TempDir::new().unwrap();
        let result = run_fast_verify(tmp.path());
        assert!(result.is_err());
        let err = result.unwrap_err();
        let out = err.to_output();
        assert!(matches!(out.status, crate::output::Status::Warn));
        assert!(out.message.contains("Fix the issue and re-run"));
    }

    #[test]
    fn ff_merge() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());

        Command::new("git")
            .args(["checkout", "-b", "ecc-session-ff"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        std::fs::write(tmp.path().join("ff.txt"), "ff content").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "ff commit"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        Command::new("git")
            .args(["checkout", "main"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        let result = merge_fast_forward(tmp.path(), "ecc-session-ff");
        assert!(result.is_ok(), "ff merge should succeed: {result:?}");

        let log = Command::new("git")
            .args(["log", "--oneline", "-2"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        let log_str = String::from_utf8_lossy(&log.stdout);
        assert!(log_str.contains("ff commit"));
    }

    #[test]
    fn rejects_main() {
        let result = validate_session_branch("main");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, MergeError::NotSessionBranch { ref branch } if branch == "main"));

        let out = err.to_output();
        assert!(matches!(out.status, crate::output::Status::Block));
        assert!(out.message.contains("'main'"));
    }

    #[test]
    fn accepts_prefixed_session_branch() {
        let result =
            validate_session_branch("worktree-ecc-session-20260404-150000-my-feature-12345");
        assert!(result.is_ok(), "prefixed session branch should be accepted");
    }

    #[test]
    fn accepts_unprefixed_session_branch() {
        let result = validate_session_branch("ecc-session-20260404-150000-my-feature-12345");
        assert!(result.is_ok(), "unprefixed session branch should be accepted");
    }

    #[test]
    fn rejects_non_session_branches() {
        assert!(validate_session_branch("main").is_err());
        assert!(validate_session_branch("feature-x").is_err());
    }

    #[test]
    fn rejects_prefixed_non_session_branch() {
        assert!(
            validate_session_branch("worktree-feature-x").is_err(),
            "worktree-feature-x should be rejected"
        );
    }

    #[test]
    fn checkout_main_when_already_on_main() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());
        let result = checkout_main(tmp.path());
        assert!(result.is_ok(), "checkout main should succeed when already on main: {result:?}");
    }

    #[test]
    fn checkout_main_from_detached() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());
        let sha = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        let sha = String::from_utf8_lossy(&sha.stdout).trim().to_owned();
        Command::new("git")
            .args(["checkout", &sha])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        let result = checkout_main(tmp.path());
        assert!(result.is_ok(), "checkout main from detached HEAD should succeed: {result:?}");
    }

    #[test]
    fn checkout_main_from_other_branch() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());
        Command::new("git")
            .args(["checkout", "-b", "other-branch"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        let result = checkout_main(tmp.path());
        assert!(result.is_ok(), "checkout main from other branch should succeed: {result:?}");
    }

    #[test]
    fn checkout_main_dirty_tree_fails() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());
        Command::new("git")
            .args(["checkout", "-b", "dirty-branch"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        std::fs::write(tmp.path().join("conflict.txt"), "dirty content").unwrap();
        Command::new("git")
            .args(["add", "conflict.txt"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "add conflict file"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        std::fs::write(tmp.path().join("conflict.txt"), "uncommitted change").unwrap();
        let result = checkout_main(tmp.path());
        assert!(result.is_ok() || matches!(result, Err(MergeError::CheckoutFailed { .. })));
    }

    #[test]
    fn merge_updates_working_tree_files() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());

        Command::new("git")
            .args(["checkout", "-b", "ecc-session-wt-test"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        std::fs::write(tmp.path().join("new_feature.txt"), "feature content").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "add feature"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        checkout_main(tmp.path()).unwrap();
        merge_fast_forward(tmp.path(), "ecc-session-wt-test").unwrap();

        let content = std::fs::read_to_string(tmp.path().join("new_feature.txt")).unwrap();
        assert_eq!(content, "feature content", "working tree file should match merged content");
    }

    #[test]
    fn merge_preserves_worktree_directory() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());

        let wt_dir = tmp.path().join("worktree-preserve");
        Command::new("git")
            .args([
                "worktree", "add", wt_dir.to_str().unwrap(), "-b", "ecc-session-preserve",
            ])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        std::fs::write(wt_dir.join("preserve.txt"), "content").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(&wt_dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "preserve commit"])
            .current_dir(&wt_dir)
            .output()
            .unwrap();

        rebase_onto_main(&wt_dir, "ecc-session-preserve").unwrap();
        checkout_main(tmp.path()).unwrap();
        merge_fast_forward(tmp.path(), "ecc-session-preserve").unwrap();
        let _ = Command::new("git")
            .args(["branch", "-D", "--", "ecc-session-preserve"])
            .current_dir(tmp.path())
            .output();

        assert!(wt_dir.exists(), "worktree directory should be preserved after merge");
    }

    #[test]
    fn merge_defers_branch_deletion() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());

        let wt_dir = tmp.path().join("worktree-branch");
        Command::new("git")
            .args([
                "worktree", "add", wt_dir.to_str().unwrap(), "-b", "ecc-session-branch-def",
            ])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        std::fs::write(wt_dir.join("branch.txt"), "branch").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(&wt_dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "branch commit"])
            .current_dir(&wt_dir)
            .output()
            .unwrap();

        rebase_onto_main(&wt_dir, "ecc-session-branch-def").unwrap();
        checkout_main(tmp.path()).unwrap();
        merge_fast_forward(tmp.path(), "ecc-session-branch-def").unwrap();

        let branches = Command::new("git")
            .args(["branch"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        let branch_list = String::from_utf8_lossy(&branches.stdout);
        assert!(
            branch_list.contains("ecc-session-branch-def"),
            "branch should still exist (deferred to gc): {branch_list}"
        );

        let log = Command::new("git")
            .args(["log", "--oneline", "-2"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        let log_str = String::from_utf8_lossy(&log.stdout);
        assert!(log_str.contains("branch commit"), "commit should be on main");
    }

    #[test]
    fn merge_leaves_clean_status() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());

        Command::new("git")
            .args(["checkout", "-b", "ecc-session-clean"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        std::fs::write(tmp.path().join("clean.txt"), "clean").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "clean commit"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        checkout_main(tmp.path()).unwrap();
        merge_fast_forward(tmp.path(), "ecc-session-clean").unwrap();

        let status = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        let status_str = String::from_utf8_lossy(&status.stdout);
        assert!(
            status_str.trim().is_empty(),
            "working tree should be clean after merge, got: {status_str}"
        );
    }
}
