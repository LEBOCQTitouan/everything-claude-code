//! `ecc-workflow reset --force` — reset workflow to idle state.

use ecc_domain::workflow::{
    concern::Concern,
    phase::Phase,
    state::{Artifacts, Toolchain, WorkflowState},
    timestamp::Timestamp,
};
use std::path::Path;

use crate::io::{archive_state, with_state_lock, write_state_atomic};
use crate::output::WorkflowOutput;
use crate::time::utc_now_iso8601;

pub fn run(force: bool, project_dir: &Path) -> WorkflowOutput {
    if !force {
        return WorkflowOutput::block(
            "Reset requires --force flag to prevent accidental state loss. \
             Usage: ecc-workflow reset --force",
        );
    }

    let result = with_state_lock(project_dir, || {
        let workflow_dir = project_dir.join(".claude/workflow");
        let state_path = workflow_dir.join("state.json");
        if !state_path.exists() {
            return WorkflowOutput::pass("No active workflow to reset");
        }

        // Archive state (include done states, unlike init)
        if let Err(e) = archive_state(&workflow_dir, true) {
            return WorkflowOutput::block(format!("Failed to archive state: {e}"));
        }

        // Write minimal Idle state
        let idle_state = WorkflowState {
            phase: Phase::Idle,
            concern: Concern::Dev,
            feature: String::new(),
            started_at: Timestamp::new(utc_now_iso8601()),
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

        match write_state_atomic(project_dir, &idle_state) {
            Ok(()) => {
                WorkflowOutput::pass("Workflow reset - state archived, phase set to idle")
            }
            Err(e) => WorkflowOutput::block(format!("Failed to write idle state: {e}")),
        }
    });

    match result {
        Ok(output) => output,
        Err(e) => WorkflowOutput::block(format!("Failed to acquire state lock: {e}")),
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use ecc_domain::workflow::{
        concern::Concern,
        phase::Phase,
        state::{Artifacts, Toolchain, WorkflowState},
        timestamp::Timestamp,
    };

    fn make_state_json(phase: Phase) -> String {
        let state = WorkflowState {
            phase,
            concern: Concern::Dev,
            feature: "test-feature".to_owned(),
            started_at: Timestamp::new("2026-01-01T00:00:00Z"),
            toolchain: Toolchain { test: None, lint: None, build: None },
            artifacts: Artifacts {
                plan: None, solution: None, implement: None,
                campaign_path: None, spec_path: None, design_path: None, tasks_path: None,
            },
            completed: vec![],
        };
        serde_json::to_string_pretty(&state).unwrap()
    }

    #[test]
    fn reset_force_deletes() {
        let dir = tempfile::tempdir().unwrap();
        let wf_dir = dir.path().join(".claude/workflow");
        std::fs::create_dir_all(&wf_dir).unwrap();
        std::fs::write(wf_dir.join("state.json"), "{}").unwrap();
        assert!(wf_dir.join("state.json").exists());

        let output = run(true, dir.path());
        assert!(output.message.contains("reset"));
        // After rewrite: state.json should contain idle state, not be deleted
        assert!(wf_dir.join("state.json").exists(), "state.json should be replaced with idle state, not deleted");
    }

    #[test]
    fn reset_no_force_errors() {
        let dir = tempfile::tempdir().unwrap();
        let output = run(false, dir.path());
        assert!(output.message.contains("--force"));
    }

    #[test]
    fn reset_no_state_clean() {
        let dir = tempfile::tempdir().unwrap();
        let output = run(true, dir.path());
        assert!(output.message.contains("No active workflow"));
    }

    // PC-020: Reset archives state + writes idle
    #[test]
    fn reset_archives_and_writes_idle() {
        let dir = tempfile::tempdir().unwrap();
        let wf_dir = dir.path().join(".claude/workflow");
        std::fs::create_dir_all(&wf_dir).unwrap();
        std::fs::write(wf_dir.join("state.json"), make_state_json(Phase::Implement)).unwrap();

        let output = run(true, dir.path());

        // Output must be pass
        assert!(
            matches!(output.status, crate::output::Status::Pass),
            "reset should succeed: {:?}",
            output.message
        );
        // state.json must now contain idle state
        assert!(wf_dir.join("state.json").exists(), "state.json must exist with idle state");
        let content = std::fs::read_to_string(wf_dir.join("state.json")).unwrap();
        let state = WorkflowState::from_json(&content).unwrap();
        assert_eq!(state.phase, Phase::Idle, "phase should be idle after reset");
        // archive dir must contain the old state
        let archive_dir = wf_dir.join("archive");
        let entries: Vec<_> = std::fs::read_dir(&archive_dir).unwrap().collect();
        assert!(!entries.is_empty(), "old state must be archived");
    }

    // PC-021: Reset also archives done states (unlike init which skips done)
    #[test]
    fn reset_archives_done_state() {
        let dir = tempfile::tempdir().unwrap();
        let wf_dir = dir.path().join(".claude/workflow");
        std::fs::create_dir_all(&wf_dir).unwrap();
        std::fs::write(wf_dir.join("state.json"), make_state_json(Phase::Done)).unwrap();

        let output = run(true, dir.path());

        assert!(
            matches!(output.status, crate::output::Status::Pass),
            "reset should succeed even for done state: {:?}",
            output.message
        );
        // The done state must be archived
        let archive_dir = wf_dir.join("archive");
        let entries: Vec<_> = std::fs::read_dir(&archive_dir).unwrap().collect();
        assert!(!entries.is_empty(), "done state must also be archived");
        // state.json must now contain idle state
        let content = std::fs::read_to_string(wf_dir.join("state.json")).unwrap();
        let state = WorkflowState::from_json(&content).unwrap();
        assert_eq!(state.phase, Phase::Idle, "phase should be idle after reset");
    }

    // PC-022: Reset creates archive dir if it doesn't exist
    #[test]
    fn reset_creates_archive_dir() {
        let dir = tempfile::tempdir().unwrap();
        let wf_dir = dir.path().join(".claude/workflow");
        std::fs::create_dir_all(&wf_dir).unwrap();
        std::fs::write(wf_dir.join("state.json"), make_state_json(Phase::Plan)).unwrap();

        // Ensure archive dir does not exist before reset
        let archive_dir = wf_dir.join("archive");
        assert!(!archive_dir.exists(), "archive dir should not exist before reset");

        run(true, dir.path());

        assert!(archive_dir.exists(), "reset should create the archive directory");
    }

    // PC-023: Reset with no state.json returns pass
    #[test]
    fn reset_no_state_passes() {
        let dir = tempfile::tempdir().unwrap();
        let output = run(true, dir.path());
        assert!(
            matches!(output.status, crate::output::Status::Pass),
            "reset with no state should pass: {:?}",
            output.message
        );
        assert!(
            output.message.contains("No active workflow"),
            "message should mention no active workflow"
        );
    }

    // PC-024: Reset acquires the state lock
    #[test]
    fn reset_acquires_state_lock() {
        let dir = tempfile::tempdir().unwrap();
        let wf_dir = dir.path().join(".claude/workflow");
        std::fs::create_dir_all(&wf_dir).unwrap();
        std::fs::write(wf_dir.join("state.json"), make_state_json(Phase::Implement)).unwrap();

        run(true, dir.path());

        // The lock file must exist (proves flock was acquired during reset)
        let lock_file = ecc_flock::lock_dir(dir.path()).join("state.lock");
        assert!(
            lock_file.exists(),
            "state.lock file not found at {:?} — reset did not acquire the state lock",
            lock_file
        );
    }

    // PC-036: Archive failure blocks reset (fail-safe)
    #[test]
    fn reset_archive_failure_blocks() {
        let dir = tempfile::tempdir().unwrap();
        let wf_dir = dir.path().join(".claude/workflow");
        std::fs::create_dir_all(&wf_dir).unwrap();
        std::fs::write(wf_dir.join("state.json"), make_state_json(Phase::Implement)).unwrap();

        // Create a FILE at the archive path to prevent create_dir_all from succeeding
        let archive_path = wf_dir.join("archive");
        std::fs::write(&archive_path, b"blocker").unwrap();

        let output = run(true, dir.path());

        assert!(
            matches!(output.status, crate::output::Status::Block),
            "reset should be blocked when archive fails: {:?}",
            output.message
        );
        // state.json must be unchanged (fail-safe: not modified on archive failure)
        assert!(wf_dir.join("state.json").exists(), "state.json must not be modified when archive fails");
        let content = std::fs::read_to_string(wf_dir.join("state.json")).unwrap();
        let state = WorkflowState::from_json(&content).unwrap();
        assert_eq!(state.phase, Phase::Implement, "state.json phase must be unchanged");
    }
}
