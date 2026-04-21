use std::path::Path;
use std::str::FromStr;

use ecc_domain::workflow::concern::Concern;
use ecc_domain::workflow::phase::Phase;
use ecc_domain::workflow::state::{Artifacts, Toolchain, WorkflowState};
use ecc_domain::workflow::timestamp::Timestamp;

use crate::io::{archive_state, with_state_lock, write_state_atomic};
use crate::output::WorkflowOutput;
use crate::time::utc_now_iso8601;

/// Run the `init` subcommand: initialize workflow state for a new session.
///
/// Creates `.claude/workflow/state.json` with phase=plan, the given concern and feature,
/// a current UTC timestamp, and all toolchain/artifact fields set to null.
///
/// If a previous state.json exists and its phase is not "done", it is archived to
/// `.claude/workflow/archive/state-YYYYMMDD-HHMMSS.json` before the new state is written.
/// Artifact files `implement-done.md` and `.tdd-state` are cleaned up on every init.
///
/// ```text
/// acquire state lock
///     |
///     v
/// archive previous state (skip if phase==done)
///     |
///     v
/// cleanup artifact files (implement-done.md, .tdd-state)
///     |
///     v
/// parse concern --> build WorkflowState { phase: Plan, ... }
///     |
///     v
/// write_state_atomic --> write .state-dir anchor (best-effort)
///     |
///     v
/// pass "Workflow initialized"
/// ```
pub fn run(concern: &str, feature: &str, project_dir: &Path, state_dir: &Path) -> WorkflowOutput {
    let result = with_state_lock(state_dir, || {
        // Archive stale state if present and not done
        if let Err(e) = archive_state(state_dir, false) {
            return WorkflowOutput::block(format!("Failed to archive stale state: {e}"));
        }

        // Clean previous artifact files
        cleanup_artifacts(state_dir);

        let concern_enum = match Concern::from_str(concern) {
            Ok(c) => c,
            Err(e) => return WorkflowOutput::block(format!("Invalid concern: {e}")),
        };

        let started_at = Timestamp::new(utc_now_iso8601());

        let state = WorkflowState {
            phase: Phase::Plan,
            concern: concern_enum,
            feature: feature.to_owned(),
            started_at,
            toolchain: Toolchain {
                test: None,
                lint: None,
                build: None,
            },
            artifacts: Artifacts {
                plan: None,
                solution: None,
                implement: None,
                campaign_path: None,
                spec_path: None,
                design_path: None,
                tasks_path: None,
            },
            completed: vec![],
            version: 1,
            history: vec![],
        };

        match write_state_atomic(state_dir, &state) {
            Ok(()) => {
                // Best-effort: write .state-dir anchor (AC-001.1, AC-001.8)
                write_anchor(project_dir, state_dir);
                WorkflowOutput::pass(format!(
                    "Workflow initialized: concern={concern}, feature=\"{feature}\""
                ))
            }
            Err(e) => WorkflowOutput::block(format!("Failed to write state.json: {e}")),
        }
    });

    match result {
        Ok(output) => output,
        Err(e) => WorkflowOutput::block(format!("Failed to acquire state lock: {e}")),
    }
}

/// Best-effort write of `.claude/workflow/.state-dir` anchor file.
///
/// The anchor contains the absolute path to the state directory so that
/// hook subprocesses can resolve state without depending on CWD-based git
/// resolution. Written atomically via temp + rename. Failures are logged
/// but never cause init to fail (AC-001.8).
fn write_anchor(project_dir: &Path, state_dir: &Path) {
    let anchor_dir = project_dir.join(".claude/workflow");
    if let Err(e) = std::fs::create_dir_all(&anchor_dir) {
        tracing::warn!("Failed to create anchor dir {}: {e}", anchor_dir.display());
        return;
    }
    let anchor_path = anchor_dir.join(".state-dir");
    let content = format!("{}\n", state_dir.display());
    let tmp_path = anchor_dir.join(".state-dir.tmp");
    match std::fs::write(&tmp_path, &content) {
        Ok(()) => {
            if let Err(e) = std::fs::rename(&tmp_path, &anchor_path) {
                tracing::warn!("Failed to rename anchor file: {e}");
                let _ = std::fs::remove_file(&tmp_path);
            }
        }
        Err(e) => {
            tracing::warn!("Failed to write anchor file: {e}");
        }
    }
}

/// Delete `implement-done.md` and `.tdd-state` from the workflow directory if they exist.
fn cleanup_artifacts(workflow_dir: &Path) {
    let _ = std::fs::remove_file(workflow_dir.join("implement-done.md"));
    let _ = std::fs::remove_file(workflow_dir.join(".tdd-state"));
}

#[cfg(test)]
mod tests {
    use crate::output::Status;
    use tempfile::TempDir;

    /// PC-017: init creates state at state_dir, not .claude/workflow/ (AC-004.3)
    #[test]
    fn init_creates_at_state_dir() {
        let tmp = TempDir::new().unwrap();
        let project_dir = tmp.path();
        // Use a non-default state_dir: .git/ecc-workflow
        let custom_state_dir = tmp.path().join(".git/ecc-workflow");

        let result = super::run("dev", "test-feature", project_dir, &custom_state_dir);
        assert!(
            matches!(result.status, Status::Pass),
            "init should succeed: {:?}",
            result.message
        );

        // state.json must exist at custom_state_dir, NOT at .claude/workflow/
        assert!(
            custom_state_dir.join("state.json").exists(),
            "state.json must be created at custom state_dir {:?}",
            custom_state_dir
        );
        assert!(
            !project_dir.join(".claude/workflow/state.json").exists(),
            "state.json must NOT be at .claude/workflow/ when using custom state_dir"
        );
    }

    /// PC-005: init writes .claude/workflow/.state-dir with state_dir path (AC-001.1, AC-001.8)
    #[test]
    fn init_writes_state_dir_anchor() {
        let tmp = TempDir::new().unwrap();
        let project_dir = tmp.path();
        let state_dir = tmp.path().join(".git/ecc-workflow");

        let result = super::run("dev", "test-feature", project_dir, &state_dir);
        assert!(matches!(result.status, Status::Pass));

        // Anchor must exist
        let anchor_path = project_dir.join(".claude/workflow/.state-dir");
        assert!(anchor_path.exists(), "anchor file must be created");

        // Content must be the state_dir path
        let content = std::fs::read_to_string(&anchor_path).unwrap();
        let trimmed = content.trim();
        assert_eq!(
            trimmed,
            state_dir.to_string_lossy(),
            "anchor must contain the state_dir path"
        );

        // state.json must also exist (written before anchor)
        assert!(state_dir.join("state.json").exists());
    }

    /// PC-006: init succeeds even when anchor write fails (AC-001.8)
    #[test]
    fn init_succeeds_without_anchor() {
        let tmp = TempDir::new().unwrap();
        // Use a project_dir that doesn't exist — anchor write will fail
        let project_dir = std::path::Path::new("/nonexistent/project");
        let state_dir = tmp.path().join(".git/ecc-workflow");

        let result = super::run("dev", "test-feature", project_dir, &state_dir);

        // Init must succeed despite anchor failure
        assert!(
            matches!(result.status, Status::Pass),
            "init must succeed even when anchor write fails: {:?}",
            result.message
        );

        // state.json must exist
        assert!(state_dir.join("state.json").exists());

        // Anchor must NOT exist (write failed)
        assert!(
            !project_dir.join(".claude/workflow/.state-dir").exists(),
            "anchor should not exist when project_dir is invalid"
        );
    }

    #[test]
    fn init_acquires_state_lock() {
        let tmp = TempDir::new().unwrap();
        let project_dir = tmp.path();
        let state_dir = project_dir.join(".claude/workflow");

        let result = super::run("dev", "test-feature", project_dir, &state_dir);
        assert!(
            matches!(result.status, Status::Pass),
            "init should succeed: {:?}",
            result.message
        );

        // The lock file must exist (proves acquire was called during init)
        let lock_file = ecc_flock::lock_dir_for(&state_dir).join("state.lock");
        assert!(
            lock_file.exists(),
            "state.lock file not found at {:?} — init did not acquire the state lock",
            lock_file
        );
    }
}
