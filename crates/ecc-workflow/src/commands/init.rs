use std::path::Path;

use ecc_domain::workflow::phase::Phase;
use ecc_domain::workflow::state::{Artifacts, Toolchain, WorkflowState};

use crate::io::write_state_atomic;
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

    // Archive stale state if present and not done
    if let Err(e) = archive_stale_state(&workflow_dir) {
        return WorkflowOutput::block(format!("Failed to archive stale state: {e}"));
    }

    // Clean previous artifact files
    cleanup_artifacts(&workflow_dir);

    let started_at = utc_now_iso8601();

    let state = WorkflowState {
        phase: Phase::Plan,
        concern: concern.to_owned(),
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
}

/// Archive `state.json` to `archive/state-YYYYMMDD-HHMMSS.json` when the current phase
/// is not "done". This mirrors the shell script's stale-workflow-archiving behavior.
fn archive_stale_state(workflow_dir: &Path) -> Result<(), anyhow::Error> {
    let state_path = workflow_dir.join("state.json");
    if !state_path.exists() {
        return Ok(());
    }

    // Read the existing state to determine its phase
    let phase_is_done = match read_state_phase(&state_path) {
        Ok(phase_str) => phase_str == "done",
        // Unreadable / corrupt state — archive it anyway
        Err(_) => false,
    };

    if !phase_is_done {
        let archive_dir = workflow_dir.join("archive");
        std::fs::create_dir_all(&archive_dir).map_err(|e| {
            anyhow::anyhow!("Failed to create archive directory: {e}")
        })?;

        let ts = utc_now_iso8601().replace(['T', ':', 'Z'], "");
        // ts is now "YYYYMMDDHHMMSS"
        let archive_name = format!("state-{ts}.json");
        let archive_path = archive_dir.join(&archive_name);

        std::fs::rename(&state_path, &archive_path).map_err(|e| {
            anyhow::anyhow!("Failed to archive state.json to {archive_name}: {e}")
        })?;
    }

    Ok(())
}

/// Read only the `phase` field from a state.json file without full deserialization.
fn read_state_phase(state_path: &Path) -> Result<String, anyhow::Error> {
    let content = std::fs::read_to_string(state_path)
        .map_err(|e| anyhow::anyhow!("Failed to read state.json: {e}"))?;
    let v: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse state.json: {e}"))?;
    Ok(v.get("phase")
        .and_then(|p| p.as_str())
        .unwrap_or("unknown")
        .to_owned())
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

        let result = super::run("test-concern", "test-feature", project_dir);
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

