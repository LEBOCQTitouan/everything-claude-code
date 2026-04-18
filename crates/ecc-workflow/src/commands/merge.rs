use std::path::Path;
use std::time::Duration;

use crate::commands::merge_cleanup::{CleanupResult, cleanup_after_merge};
use crate::commands::merge_steps::{
    acquire_merge_lock, checkout_main, current_branch, merge_fast_forward, rebase_onto_main,
    run_fast_verify, validate_session_branch,
};
use crate::output::WorkflowOutput;

pub(crate) const MERGE_LOCK_TIMEOUT_SECS: u64 = 60;

/// Errors that can occur during the merge process.
#[derive(Debug)]
pub(crate) enum MergeError {
    LockTimeout(Duration),
    RebaseConflict { branch: String, stderr: String },
    VerifyFailed { step: String, stderr: String },
    CheckoutFailed { stderr: String },
    MergeFailed { stderr: String },
    NotSessionBranch { branch: String },
    NoBranch,
}

impl MergeError {
    pub(crate) fn to_output(&self) -> WorkflowOutput {
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
            MergeError::CheckoutFailed { stderr } => WorkflowOutput::warn(format!(
                "git checkout main failed. The main repo may have a dirty working tree.\n{stderr}"
            )),
            MergeError::MergeFailed { stderr } => {
                WorkflowOutput::warn(format!("git merge --ff-only failed.\n{stderr}"))
            }
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
pub fn run(project_dir: &Path, state_dir: &Path) -> WorkflowOutput {
    let _ = state_dir; // kept for future use (Wave 4)
    match execute_merge(project_dir) {
        Ok(msg) => WorkflowOutput::pass(msg),
        Err(e) => e.to_output(),
    }
}

/// Orchestrator — one level of abstraction.
///
/// <!-- keep in sync with: ff_merge -->
/// ```text
/// current_branch --> validate_session_branch?
///     |
///     +--N--> block NotSessionBranch
///     |
///     Y
///     v
/// acquire_merge_lock (timeout? --Y--> block LockTimeout)
///     |
///     v
/// rebase_onto_main (conflict? --Y--> warn RebaseConflict)
///     |
///     v
/// run_fast_verify (fail? --Y--> warn VerifyFailed)
///     |
///     v
/// checkout_main --> merge_fast_forward --> cleanup_after_merge
///     |
///     v
/// CleanedUp | Unsafe(violations) | Aborted(reason)
/// ```
fn execute_merge(project_dir: &Path) -> Result<String, MergeError> {
    let repo_root = ecc_flock::resolve_repo_root(project_dir);
    let branch = current_branch(project_dir)?;
    validate_session_branch(&branch)?;
    let _guard = acquire_merge_lock(&repo_root)?;
    rebase_onto_main(project_dir, &branch)?;
    run_fast_verify(project_dir)?;
    checkout_main(&repo_root)?;
    merge_fast_forward(&repo_root, &branch)?;
    // Cleanup after successful merge (inside lock — _guard is alive until end of function)
    let worktree_dir = project_dir.to_path_buf();
    let cleanup = cleanup_after_merge(&repo_root, &worktree_dir, &branch);
    match cleanup {
        CleanupResult::CleanedUp { branch } => Ok(format!(
            "Merged {branch} into main and cleaned up successfully."
        )),
        CleanupResult::Unsafe(violations) => {
            let checks: Vec<String> = violations.iter().map(|v| format!("{v:?}")).collect();
            Ok(format!(
                "Merged {branch} into main. Cleanup blocked: {}",
                checks.join(", ")
            ))
        }
        CleanupResult::Aborted(reason) => Ok(format!(
            "Merged {branch} into main. Cleanup failed: {reason}. Worktree preserved."
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::merge_steps::acquire_merge_lock;
    use tempfile::TempDir;

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
}
