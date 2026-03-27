//! `ecc-workflow reset --force` — reset workflow to idle state.

use crate::output::WorkflowOutput;
use std::path::Path;

pub fn run(force: bool, project_dir: &Path) -> WorkflowOutput {
    if !force {
        return WorkflowOutput::block(
            "Reset requires --force flag to prevent accidental state loss. \
             Usage: ecc-workflow reset --force",
        );
    }

    let state_path = project_dir.join(".claude/workflow/state.json");
    if !state_path.exists() {
        return WorkflowOutput::pass("No active workflow to reset");
    }

    match std::fs::remove_file(&state_path) {
        Ok(()) => WorkflowOutput::pass("Workflow reset — state.json deleted"),
        Err(e) => WorkflowOutput::block(format!("Failed to delete state.json: {e}")),
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn reset_force_deletes() {
        let dir = tempfile::tempdir().unwrap();
        let wf_dir = dir.path().join(".claude/workflow");
        std::fs::create_dir_all(&wf_dir).unwrap();
        std::fs::write(wf_dir.join("state.json"), "{}").unwrap();
        assert!(wf_dir.join("state.json").exists());

        let output = run(true, dir.path());
        assert!(output.message.contains("reset"));
        assert!(!wf_dir.join("state.json").exists());
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
}
