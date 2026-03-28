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
pub fn run(concern: &str, feature: &str, project_dir: &Path) -> WorkflowOutput {
    let workflow_dir = project_dir.join(".claude/workflow");

    let result = with_state_lock(project_dir, || {
        // Archive stale state if present and not done
        if let Err(e) = archive_state(&workflow_dir, false) {
            return WorkflowOutput::block(format!("Failed to archive stale state: {e}"));
        }

        // Clean previous artifact files
        cleanup_artifacts(&workflow_dir);

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
        };

        match write_state_atomic(project_dir, &state) {
            Ok(()) => WorkflowOutput::pass(format!(
                "Workflow initialized: concern={concern}, feature=\"{feature}\""
            )),
            Err(e) => WorkflowOutput::block(format!("Failed to write state.json: {e}")),
        }
    });

    match result {
        Ok(output) => output,
        Err(e) => WorkflowOutput::block(format!("Failed to acquire state lock: {e}")),
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

    #[test]
    fn init_acquires_state_lock() {
        let tmp = TempDir::new().unwrap();
        let project_dir = tmp.path();

        let result = super::run("dev", "test-feature", project_dir);
        assert!(
            matches!(result.status, Status::Pass),
            "init should succeed: {:?}",
            result.message
        );

        // The lock file must exist (proves acquire was called during init)
        let lock_file = ecc_flock::lock_dir(project_dir).join("state.lock");
        assert!(
            lock_file.exists(),
            "state.lock file not found at {:?} — init did not acquire the state lock",
            lock_file
        );
    }
}
