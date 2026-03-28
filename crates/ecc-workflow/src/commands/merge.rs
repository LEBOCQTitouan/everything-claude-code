use std::path::Path;
use std::process::Command;
use std::time::Duration;

use crate::output::WorkflowOutput;

const MERGE_LOCK_TIMEOUT_SECS: u64 = 60;

/// Errors that can occur during the merge process.
#[derive(Debug)]
enum MergeError {
    LockTimeout(Duration),
    RebaseConflict { branch: String, stderr: String },
    VerifyFailed { step: String, stderr: String },
    MergeFailed { stderr: String },
    CleanupFailed { reason: String },
    NotSessionBranch { branch: String },
    NoBranch,
}

impl MergeError {
    fn to_output(&self) -> WorkflowOutput {
        match self {
            MergeError::LockTimeout(d) => WorkflowOutput::block(format!(
                "Merge lock held by another session. Timed out after {}s.",
                d.as_secs()
            )),
            MergeError::RebaseConflict { branch, stderr } => WorkflowOutput::warn(format!(
                "Rebase conflict on {branch}. Rebase aborted. Resolve conflicts manually.\n{stderr}"
            )),
            MergeError::VerifyFailed { step, stderr } => WorkflowOutput::warn(format!(
                "Fast verify failed at '{step}'. Fix the issue and re-run.\n{stderr}"
            )),
            MergeError::MergeFailed { stderr } => {
                WorkflowOutput::warn(format!("git merge --ff-only failed.\n{stderr}"))
            }
            MergeError::CleanupFailed { reason } => WorkflowOutput::warn(format!(
                "Worktree cleanup failed: {reason}. Run 'ecc worktree gc' to clean up."
            )),
            MergeError::NotSessionBranch { branch } => WorkflowOutput::block(format!(
                "Refusing to merge non-session branch '{branch}'. Only ecc-session-* branches can be merged."
            )),
            MergeError::NoBranch => {
                WorkflowOutput::block("Cannot determine current branch (detached HEAD?)".to_owned())
            }
        }
    }
}

/// Top-level run function — thin wrapper mapping MergeError to WorkflowOutput
pub fn run(project_dir: &Path) -> WorkflowOutput {
    match execute_merge(project_dir) {
        Ok(msg) => WorkflowOutput::pass(msg),
        Err(e) => e.to_output(),
    }
}

/// Orchestrator — one level of abstraction
fn execute_merge(project_dir: &Path) -> Result<String, MergeError> {
    let repo_root = ecc_flock::resolve_repo_root(project_dir);
    let branch = current_branch(project_dir)?;
    validate_session_branch(&branch)?;
    let _guard = acquire_merge_lock(&repo_root)?;
    rebase_onto_main(project_dir, &branch)?;
    run_fast_verify(project_dir)?;
    merge_fast_forward(&repo_root, &branch)?;
    cleanup_worktree(&repo_root, project_dir, &branch)?;
    Ok(format!("Merged {branch} into main. Worktree cleaned up."))
}

fn current_branch(dir: &Path) -> Result<String, MergeError> {
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

fn validate_session_branch(branch: &str) -> Result<(), MergeError> {
    if !branch.starts_with("ecc-session-") {
        return Err(MergeError::NotSessionBranch {
            branch: branch.to_owned(),
        });
    }
    Ok(())
}

fn acquire_merge_lock(repo_root: &Path) -> Result<ecc_flock::FlockGuard, MergeError> {
    ecc_flock::acquire_with_timeout(
        repo_root,
        "merge",
        Duration::from_secs(MERGE_LOCK_TIMEOUT_SECS),
    )
    .map_err(|e| match e {
        ecc_flock::FlockError::Timeout(d) => MergeError::LockTimeout(d),
        _ => MergeError::LockTimeout(Duration::from_secs(MERGE_LOCK_TIMEOUT_SECS)),
    })
}

fn rebase_onto_main(dir: &Path, branch: &str) -> Result<(), MergeError> {
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

fn run_fast_verify(dir: &Path) -> Result<(), MergeError> {
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

fn merge_fast_forward(repo_root: &Path, branch: &str) -> Result<(), MergeError> {
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

fn cleanup_worktree(
    repo_root: &Path,
    worktree_path: &Path,
    branch: &str,
) -> Result<(), MergeError> {
    let wt_str = worktree_path.to_string_lossy();
    let output = Command::new("git")
        .args(["worktree", "remove", "--force", "--", &wt_str])
        .current_dir(repo_root)
        .output()
        .map_err(|e| MergeError::CleanupFailed {
            reason: e.to_string(),
        })?;
    if !output.status.success() {
        return Err(MergeError::CleanupFailed {
            reason: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }
    let _ = Command::new("git")
        .args(["branch", "-D", "--", branch])
        .current_dir(repo_root)
        .output();
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
        // Rename default branch to main if needed
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
    fn error_mapping() {
        // LockTimeout → block
        let err = MergeError::LockTimeout(Duration::from_secs(60));
        let out = err.to_output();
        assert!(matches!(out.status, crate::output::Status::Block));
        assert!(out.message.contains("60s"));

        // RebaseConflict → warn
        let err = MergeError::RebaseConflict {
            branch: "ecc-session-test".to_owned(),
            stderr: "conflict".to_owned(),
        };
        let out = err.to_output();
        assert!(matches!(out.status, crate::output::Status::Warn));
        assert!(out.message.contains("ecc-session-test"));

        // VerifyFailed → warn
        let err = MergeError::VerifyFailed {
            step: "cargo build".to_owned(),
            stderr: "error".to_owned(),
        };
        let out = err.to_output();
        assert!(matches!(out.status, crate::output::Status::Warn));
        assert!(out.message.contains("cargo build"));

        // MergeFailed → warn
        let err = MergeError::MergeFailed {
            stderr: "not possible".to_owned(),
        };
        let out = err.to_output();
        assert!(matches!(out.status, crate::output::Status::Warn));
        assert!(out.message.contains("ff-only"));

        // CleanupFailed → warn
        let err = MergeError::CleanupFailed {
            reason: "permission denied".to_owned(),
        };
        let out = err.to_output();
        assert!(matches!(out.status, crate::output::Status::Warn));
        assert!(out.message.contains("ecc worktree gc"));

        // NotSessionBranch → block
        let err = MergeError::NotSessionBranch {
            branch: "main".to_owned(),
        };
        let out = err.to_output();
        assert!(matches!(out.status, crate::output::Status::Block));
        assert!(out.message.contains("main"));

        // NoBranch → block
        let err = MergeError::NoBranch;
        let out = err.to_output();
        assert!(matches!(out.status, crate::output::Status::Block));
        assert!(out.message.contains("detached HEAD"));
    }

    #[test]
    fn acquires_lock() {
        let tmp = TempDir::new().unwrap();
        // acquire_merge_lock uses 60s timeout
        let guard = acquire_merge_lock(tmp.path()).unwrap();
        let lock_path = guard.lock_path().to_path_buf();
        // Lock file is at .claude/workflow/.locks/merge.lock
        assert!(lock_path.ends_with(".claude/workflow/.locks/merge.lock"));
        assert!(lock_path.exists());
    }

    #[test]
    fn timeout_blocks() {
        let err = MergeError::LockTimeout(Duration::from_secs(60));
        let out = err.to_output();
        assert!(matches!(out.status, crate::output::Status::Block));
        assert!(out.message.contains("Timed out after 60s"));
    }

    #[test]
    fn runs_rebase() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());

        // Create a session branch
        Command::new("git")
            .args(["checkout", "-b", "ecc-session-feature"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        // Add a commit on the session branch
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

        // Rebase onto main should succeed (no conflicts)
        let result = rebase_onto_main(tmp.path(), "ecc-session-feature");
        assert!(result.is_ok(), "rebase should succeed: {result:?}");
    }

    #[test]
    fn conflict_aborts() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());

        // Create conflicting file on main
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

        // Create session branch from the initial commit (before conflict file)
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

        // Add conflicting content on session branch
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

        // rebase_onto_main should fail and abort
        let result = rebase_onto_main(tmp.path(), "ecc-session-conflict");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, MergeError::RebaseConflict { .. }));

        // Confirm rebase was aborted (no .git/rebase-merge dir)
        let rebase_merge = tmp.path().join(".git/rebase-merge");
        let rebase_apply = tmp.path().join(".git/rebase-apply");
        assert!(
            !rebase_merge.exists() && !rebase_apply.exists(),
            "rebase should have been aborted"
        );
    }

    #[test]
    fn runs_verify() {
        // This test verifies run_fast_verify attempts build, test, clippy in order.
        // We test this by running in a non-cargo directory — the first step (cargo build)
        // will fail immediately, and we get a VerifyFailed with step "cargo build".
        let tmp = TempDir::new().unwrap();
        let result = run_fast_verify(tmp.path());
        assert!(result.is_err());
        if let Err(MergeError::VerifyFailed { step, .. }) = result {
            assert_eq!(
                step, "cargo build",
                "first failing step should be cargo build"
            );
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

        // Create session branch and add commit
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

        // Switch back to main for the merge
        Command::new("git")
            .args(["checkout", "main"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        // ff-only merge should succeed
        let result = merge_fast_forward(tmp.path(), "ecc-session-ff");
        assert!(result.is_ok(), "ff merge should succeed: {result:?}");

        // Verify the commit is on main
        let log = Command::new("git")
            .args(["log", "--oneline", "-2"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        let log_str = String::from_utf8_lossy(&log.stdout);
        assert!(log_str.contains("ff commit"));
    }

    #[test]
    fn cleanup() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());

        // Create a second worktree dir for the session
        let wt_dir = tmp.path().join("worktree-session");

        // Create session branch and worktree
        Command::new("git")
            .args([
                "worktree",
                "add",
                wt_dir.to_str().unwrap(),
                "-b",
                "ecc-session-cleanup",
            ])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        // Add a commit in worktree
        std::fs::write(wt_dir.join("cleanup.txt"), "cleanup").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(&wt_dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "cleanup commit"])
            .current_dir(&wt_dir)
            .output()
            .unwrap();

        // Switch to main and ff-merge
        Command::new("git")
            .args(["checkout", "main"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["merge", "--ff-only", "ecc-session-cleanup"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        // Now cleanup: remove worktree and branch
        let result = cleanup_worktree(tmp.path(), &wt_dir, "ecc-session-cleanup");
        assert!(result.is_ok(), "cleanup should succeed: {result:?}");

        // Worktree dir should be gone
        assert!(!wt_dir.exists(), "worktree dir should be removed");

        // Branch should be deleted
        let branches = Command::new("git")
            .args(["branch"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        let branch_list = String::from_utf8_lossy(&branches.stdout);
        assert!(
            !branch_list.contains("ecc-session-cleanup"),
            "branch should be deleted"
        );
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
}
