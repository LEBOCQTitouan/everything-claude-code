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

/// Orchestrator — one level of abstraction
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
    if ecc_domain::worktree::WorktreeName::parse(branch).is_none() {
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
        if let Err(e) = Command::new("git")
            .args(["rebase", "--abort"])
            .current_dir(dir)
            .output()
        {
            tracing::warn!(error = %e, "failed to abort rebase during error recovery");
        }
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

/// Ensure the main repo has `main` checked out before merging.
///
/// Without this, `git merge --ff-only` would target whatever branch
/// is currently checked out in the main repo, which may not be `main`
/// if another session or manual git operation changed it.
fn checkout_main(repo_root: &Path) -> Result<(), MergeError> {
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

#[cfg(test)]
#[path = "merge_tests.rs"]
mod tests;
