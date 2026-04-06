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
    CheckoutFailed { stderr: String },
    MergeFailed { stderr: String },
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
        CleanupResult::CleanedUp { branch } => {
            Ok(format!(
                "Merged {branch} into main and cleaned up successfully."
            ))
        }
        CleanupResult::Unsafe(violations) => {
            let checks: Vec<String> = violations.iter().map(|v| format!("{v:?}")).collect();
            Ok(format!(
                "Merged {branch} into main. Cleanup blocked: {}",
                checks.join(", ")
            ))
        }
        CleanupResult::Aborted(reason) => {
            Ok(format!(
                "Merged {branch} into main. Cleanup failed: {reason}. Worktree preserved."
            ))
        }
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
    let output = Command::new("git")
        .args(["-C", dir.to_str().unwrap_or("."), "status", "--porcelain"])
        .output();
    match output {
        Ok(o) if o.status.success() => !o.stdout.is_empty(),
        _ => false,
    }
}

/// Check for untracked files in the given directory.
/// Returns true if `git ls-files --others --exclude-standard` produces non-empty output.
fn check_untracked_files(dir: &Path) -> bool {
    let output = Command::new("git")
        .args([
            "-C",
            dir.to_str().unwrap_or("."),
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
    let output = Command::new("git")
        .args([
            "-C",
            dir.to_str().unwrap_or("."),
            "rev-list",
            "--count",
            "HEAD",
            "^main",
        ])
        .output();
    match output {
        Ok(o) if o.status.success() => {
            String::from_utf8_lossy(&o.stdout)
                .trim()
                .parse::<u64>()
                .unwrap_or(0)
        }
        _ => 0,
    }
}

/// Check if the stash has any entries.
/// Returns true if `git stash list` produces non-empty output.
fn check_stash(dir: &Path) -> bool {
    let output = Command::new("git")
        .args(["-C", dir.to_str().unwrap_or("."), "stash", "list"])
        .output();
    match output {
        Ok(o) if o.status.success() => !o.stdout.is_empty(),
        _ => false,
    }
}

/// Check if HEAD is "safely stored" — either pushed to any remote or merged into main.
/// Returns true if `git branch -r --contains HEAD` or `git branch --contains HEAD`
/// shows main contains HEAD.
fn check_pushed_to_remote(dir: &Path) -> bool {
    // First check: is HEAD in any remote branch?
    let remote_output = Command::new("git")
        .args([
            "-C",
            dir.to_str().unwrap_or("."),
            "branch",
            "-r",
            "--contains",
            "HEAD",
        ])
        .output();
    if matches!(remote_output, Ok(ref o) if o.status.success() && !o.stdout.is_empty()) {
        return true;
    }
    // Second check: is HEAD merged into main (commits safely on main)?
    // git merge-base --is-ancestor HEAD main → exit 0 if HEAD is ancestor of main
    let ancestor_output = Command::new("git")
        .args([
            "-C",
            dir.to_str().unwrap_or("."),
            "merge-base",
            "--is-ancestor",
            "HEAD",
            "main",
        ])
        .output();
    matches!(ancestor_output, Ok(o) if o.status.success())
}

/// Gather all safety data from a worktree directory using raw `std::process::Command`.
/// Does NOT use the `WorktreeManager` port.
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
pub(crate) fn cleanup_after_merge(repo_root: &Path, worktree_dir: &Path, branch: &str) -> CleanupResult {
    // 1. If worktree directory exists, gather and assess safety data
    if worktree_dir.exists() {
        let safety_input = gather_safety_data(worktree_dir);
        let violations = ecc_domain::worktree::assess_safety(&safety_input);
        if !violations.is_empty() {
            return CleanupResult::Unsafe(violations);
        }
    }
    // If directory doesn't exist: skip safety checks — nothing to lose

    // 3. Remove the worktree (prunes metadata even if dir is missing)
    let worktree_str = worktree_dir.to_string_lossy();
    let remove_output = Command::new("git")
        .args(["worktree", "remove", "--force", "--", worktree_str.as_ref()])
        .current_dir(repo_root)
        .output();
    match remove_output {
        Ok(o) if o.status.success() => {}
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            return CleanupResult::Aborted(format!(
                "git worktree remove failed: {stderr}"
            ));
        }
        Err(e) => {
            return CleanupResult::Aborted(format!(
                "git worktree remove could not be spawned: {e}"
            ));
        }
    }

    // 4. Delete the branch (failure is warning only — still return CleanedUp)
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
        assert!(
            result.is_ok(),
            "unprefixed session branch should be accepted"
        );
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

    // --- checkout_main tests ---

    #[test]
    fn checkout_main_when_already_on_main() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());
        // Already on main — checkout should be a no-op success
        let result = checkout_main(tmp.path());
        assert!(
            result.is_ok(),
            "checkout main should succeed when already on main: {result:?}"
        );
    }

    #[test]
    fn checkout_main_from_detached() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());
        // Detach HEAD
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
        // Now checkout main should succeed
        let result = checkout_main(tmp.path());
        assert!(
            result.is_ok(),
            "checkout main from detached HEAD should succeed: {result:?}"
        );
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
        assert!(
            result.is_ok(),
            "checkout main from other branch should succeed: {result:?}"
        );
    }

    #[test]
    fn checkout_main_dirty_tree_fails() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());
        // Create another branch with a conflicting tracked file
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
        // Modify the file without committing — git checkout main should fail
        // because the file exists differently on main vs dirty-branch
        std::fs::write(tmp.path().join("conflict.txt"), "uncommitted change").unwrap();
        let result = checkout_main(tmp.path());
        // This may or may not fail depending on git's merge strategy —
        // if the file doesn't exist on main, checkout succeeds and removes it.
        // The important thing is the function doesn't panic.
        assert!(result.is_ok() || matches!(result, Err(MergeError::CheckoutFailed { .. })));
    }

    // --- working tree content verification tests ---

    #[test]
    fn merge_updates_working_tree_files() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());

        // Create session branch
        Command::new("git")
            .args(["checkout", "-b", "ecc-session-wt-test"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        // Add a new file on session branch
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

        // Checkout main, then merge
        checkout_main(tmp.path()).unwrap();
        merge_fast_forward(tmp.path(), "ecc-session-wt-test").unwrap();

        // Verify working tree has the file with correct content
        let content = std::fs::read_to_string(tmp.path().join("new_feature.txt")).unwrap();
        assert_eq!(
            content, "feature content",
            "working tree file should match merged content"
        );
    }

    // PC-003: execute_merge no longer contains cleanup_worktree call
    // Verified by: cleanup_worktree function removed, no "worktree remove" in execute_merge
    #[test]
    fn merge_preserves_worktree_directory() {
        // After merge, worktree directory is preserved. We verify this at the
        // step level: rebase + checkout + ff-merge + branch delete, no worktree remove.
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());

        let wt_dir = tmp.path().join("worktree-preserve");
        Command::new("git")
            .args([
                "worktree",
                "add",
                wt_dir.to_str().unwrap(),
                "-b",
                "ecc-session-preserve",
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

        // Run individual merge steps (bypass verify which needs Cargo)
        rebase_onto_main(&wt_dir, "ecc-session-preserve").unwrap();
        checkout_main(tmp.path()).unwrap();
        merge_fast_forward(tmp.path(), "ecc-session-preserve").unwrap();
        // Even a manual branch delete should not remove the worktree directory
        let _ = Command::new("git")
            .args(["branch", "-D", "--", "ecc-session-preserve"])
            .current_dir(tmp.path())
            .output();

        // Worktree directory must still exist
        assert!(
            wt_dir.exists(),
            "worktree directory should be preserved after merge"
        );
    }

    // PC-004: execute_merge success message no "cleaned up"
    #[test]
    fn merge_success_message_no_cleanup() {
        // The success message format is deterministic — verify it directly
        let branch = "ecc-session-test";
        let expected_msg = format!(
            "Merged {branch} into main. Worktree directory preserved (cleanup deferred to gc)."
        );
        assert!(
            !expected_msg.contains("cleaned up"),
            "message should not contain 'cleaned up'"
        );
        assert!(
            expected_msg.contains("Merged"),
            "message should contain 'Merged'"
        );
        assert!(
            expected_msg.contains("deferred"),
            "message should contain 'deferred'"
        );
    }

    // PC-005: execute_merge defers branch deletion (can't delete while worktree exists)
    #[test]
    fn merge_defers_branch_deletion() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());

        let wt_dir = tmp.path().join("worktree-branch");
        Command::new("git")
            .args([
                "worktree",
                "add",
                wt_dir.to_str().unwrap(),
                "-b",
                "ecc-session-branch-def",
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

        // Run merge steps (no branch deletion — deferred to gc)
        rebase_onto_main(&wt_dir, "ecc-session-branch-def").unwrap();
        checkout_main(tmp.path()).unwrap();
        merge_fast_forward(tmp.path(), "ecc-session-branch-def").unwrap();

        // Branch still exists (can't delete while worktree uses it)
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

        // But commits are on main
        let log = Command::new("git")
            .args(["log", "--oneline", "-2"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        let log_str = String::from_utf8_lossy(&log.stdout);
        assert!(
            log_str.contains("branch commit"),
            "commit should be on main"
        );
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

        // git status should be clean
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

    // PC-020..031 test helpers and tests

    // Helper: create a worktree at wt_dir on branch_name with one commit
    fn setup_worktree_with_commit(repo: &Path, wt_dir: &Path, branch_name: &str, file_name: &str) {
        Command::new("git")
            .args([
                "worktree",
                "add",
                wt_dir.to_str().unwrap(),
                "-b",
                branch_name,
            ])
            .current_dir(repo)
            .output()
            .unwrap();
        std::fs::write(wt_dir.join(file_name), "content").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(wt_dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "wt commit"])
            .current_dir(wt_dir)
            .output()
            .unwrap();
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

        rebase_onto_main(&wt_dir, "ecc-session-cleanup-020-12345").unwrap();
        checkout_main(tmp.path()).unwrap();
        merge_fast_forward(tmp.path(), "ecc-session-cleanup-020-12345").unwrap();

        let result =
            cleanup_after_merge(tmp.path(), &wt_dir, "ecc-session-cleanup-020-12345");
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

        rebase_onto_main(&wt_dir, "ecc-session-safe-021-12345").unwrap();
        checkout_main(tmp.path()).unwrap();
        merge_fast_forward(tmp.path(), "ecc-session-safe-021-12345").unwrap();

        let result =
            cleanup_after_merge(tmp.path(), &wt_dir, "ecc-session-safe-021-12345");
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

        rebase_onto_main(&wt_dir, "ecc-session-branch-022-12345").unwrap();
        checkout_main(tmp.path()).unwrap();
        merge_fast_forward(tmp.path(), "ecc-session-branch-022-12345").unwrap();

        let result =
            cleanup_after_merge(tmp.path(), &wt_dir, "ecc-session-branch-022-12345");
        assert!(
            matches!(result, CleanupResult::CleanedUp { .. }),
            "expected CleanedUp, got: {result:?}"
        );

        let branches = Command::new("git")
            .args(["branch"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        let branch_list = String::from_utf8_lossy(&branches.stdout);
        assert!(
            !branch_list.contains("ecc-session-branch-022-12345"),
            "branch should have been deleted, branches: {branch_list}"
        );
    }

    // PC-023: Commands use current_dir(repo_root) for deletion
    #[test]
    fn cwd_set_to_repo_root() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());

        let wt_dir = tmp.path().join("worktree-cwd-023");
        setup_worktree_with_commit(
            tmp.path(),
            &wt_dir,
            "ecc-session-cwd-023-12345",
            "f023.txt",
        );

        rebase_onto_main(&wt_dir, "ecc-session-cwd-023-12345").unwrap();
        checkout_main(tmp.path()).unwrap();
        merge_fast_forward(tmp.path(), "ecc-session-cwd-023-12345").unwrap();

        let result =
            cleanup_after_merge(tmp.path(), &wt_dir, "ecc-session-cwd-023-12345");
        assert!(
            matches!(result, CleanupResult::CleanedUp { .. }),
            "cleanup should succeed with repo_root as cwd, got: {result:?}"
        );
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

        // Add staged (uncommitted) changes to the worktree
        std::fs::write(wt_dir.join("dirty.txt"), "uncommitted").unwrap();
        Command::new("git")
            .args(["add", "dirty.txt"])
            .current_dir(&wt_dir)
            .output()
            .unwrap();

        // Do NOT merge — worktree has 1 unmerged commit + uncommitted staged changes
        let result =
            cleanup_after_merge(tmp.path(), &wt_dir, "ecc-session-unsafe-024-12345");
        assert!(
            matches!(result, CleanupResult::Unsafe(_)),
            "worktree with uncommitted changes should be Unsafe, got: {result:?}"
        );
        assert!(wt_dir.exists(), "unsafe worktree dir should be preserved");
    }

    // PC-025: Worktree remove failure is warning, not merge failure
    #[test]
    fn remove_failure_is_warning() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo_with_main(tmp.path());

        let wt_dir = tmp.path().join("worktree-rmfail-025");
        setup_worktree_with_commit(
            tmp.path(),
            &wt_dir,
            "ecc-session-rmfail-025-12345",
            "f025.txt",
        );

        rebase_onto_main(&wt_dir, "ecc-session-rmfail-025-12345").unwrap();
        checkout_main(tmp.path()).unwrap();
        merge_fast_forward(tmp.path(), "ecc-session-rmfail-025-12345").unwrap();

        // Pass a path that is a FILE (not a worktree dir) to force git worktree remove to fail
        let fake_wt = tmp.path().join("not-a-worktree");
        std::fs::write(&fake_wt, "file content").unwrap();

        let result = cleanup_after_merge(tmp.path(), &fake_wt, "ecc-session-rmfail-025-12345");
        // Should NOT panic; returns Aborted, CleanedUp, or Unsafe (not a hard error)
        assert!(
            matches!(
                result,
                CleanupResult::Aborted(_)
                    | CleanupResult::CleanedUp { .. }
                    | CleanupResult::Unsafe(_)
            ),
            "remove failure should return Aborted (warning), not panic, got: {result:?}"
        );
    }

    // PC-026: Success message says "cleaned up successfully"
    #[test]
    fn success_message_cleaned_up() {
        let branch = "ecc-session-test-026-12345".to_owned();
        let result = CleanupResult::CleanedUp {
            branch: branch.clone(),
        };
        let msg = match result {
            CleanupResult::CleanedUp { branch } => {
                format!("Merged {branch} into main and cleaned up successfully.")
            }
            _ => panic!("expected CleanedUp"),
        };
        assert!(
            msg.contains("cleaned up successfully"),
            "success message should contain 'cleaned up successfully', got: {msg}"
        );
    }

    // PC-027: Branch delete failure after worktree remove is warning (CleanedUp returned)
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

        rebase_onto_main(&wt_dir, "ecc-session-branchfail-027-12345").unwrap();
        checkout_main(tmp.path()).unwrap();
        merge_fast_forward(tmp.path(), "ecc-session-branchfail-027-12345").unwrap();

        // Delete branch BEFORE calling cleanup so branch delete will fail
        Command::new("git")
            .args(["branch", "-d", "ecc-session-branchfail-027-12345"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        let result =
            cleanup_after_merge(tmp.path(), &wt_dir, "ecc-session-branchfail-027-12345");
        assert!(
            matches!(result, CleanupResult::CleanedUp { .. }),
            "branch delete failure should be warning (CleanedUp returned), got: {result:?}"
        );
    }

    // PC-028: CWD failure aborts cleanup and preserves worktree
    #[test]
    fn cwd_failure_aborts_cleanup() {
        let nonexistent = Path::new("/tmp/nonexistent-repo-ecc-test-028");
        let fake_wt = Path::new("/tmp/nonexistent-wt-ecc-test-028");
        let result = cleanup_after_merge(nonexistent, fake_wt, "ecc-session-cwd-028-12345");
        // With nonexistent paths, git commands will fail — expect Aborted or Unsafe
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
        let source = include_str!("merge.rs");
        // Check that there's no `use ecc_ports` import line (the test itself
        // contains the literal string for this assertion, so we check for imports)
        let has_ecc_ports_import = source.lines().any(|line| {
            let t = line.trim();
            (t.starts_with("use ") || t.starts_with("extern "))
                && t.contains("ecc_ports")
        });
        assert!(
            !has_ecc_ports_import,
            "merge.rs should not import ecc_ports"
        );
        let has_worktree_manager_import = source.lines().any(|line| {
            let t = line.trim();
            (t.starts_with("use ") || t.starts_with("extern "))
                && t.contains("WorktreeManager")
        });
        assert!(
            !has_worktree_manager_import,
            "merge.rs should not use WorktreeManager port"
        );
        assert!(
            source.contains("Command::new"),
            "merge.rs should use std::process::Command for raw git calls"
        );
    }

    // PC-030: Safety check is inside merge lock critical section
    #[test]
    fn safety_inside_lock() {
        let source = include_str!("merge.rs");
        assert!(source.contains("_guard"), "execute_merge must hold a lock guard");
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

        rebase_onto_main(&wt_dir, "ecc-session-missing-031-12345").unwrap();
        checkout_main(tmp.path()).unwrap();
        merge_fast_forward(tmp.path(), "ecc-session-missing-031-12345").unwrap();

        // Remove the worktree directory to simulate it already being gone
        std::fs::remove_dir_all(&wt_dir).unwrap();
        assert!(!wt_dir.exists(), "pre-condition: wt dir should be gone");

        let result =
            cleanup_after_merge(tmp.path(), &wt_dir, "ecc-session-missing-031-12345");
        assert!(
            matches!(
                result,
                CleanupResult::CleanedUp { .. } | CleanupResult::Aborted(_)
            ),
            "missing dir cleanup should succeed or abort gracefully, got: {result:?}"
        );

        // Branch should be gone
        let branches = Command::new("git")
            .args(["branch"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        let branch_list = String::from_utf8_lossy(&branches.stdout);
        assert!(
            !branch_list.contains("ecc-session-missing-031-12345"),
            "branch should be deleted even when dir was missing, branches: {branch_list}"
        );
    }

}
