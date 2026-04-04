use std::path::Path;

use ecc_domain::workflow::phase::Phase;

use crate::output::WorkflowOutput;

/// Run the `stop-gate` subcommand.
///
/// Called by the Stop hook when a Claude session ends.
/// Warns on stderr if the workflow is in an incomplete phase, exits 0 always.
///
/// Exit behavior:
/// - No state.json → exit 0, silent
/// - Phase `Done`  → exit 0, silent
/// - Any other phase → exit 0, warn on stderr:
///   "WARNING: Workflow is in '<phase>' phase (not done)."
pub fn run(state_dir: &Path) -> WorkflowOutput {
    let state = match crate::io::read_state(state_dir) {
        Ok(None) => return WorkflowOutput::pass(""),
        Ok(Some(s)) => s,
        Err(_) => return WorkflowOutput::pass(""),
    };

    if matches!(state.phase, Phase::Done | Phase::Idle) {
        return WorkflowOutput::pass("");
    }

    WorkflowOutput::warn(format!(
        "WARNING: Workflow is in '{}' phase (not done). Feature: '{}'. \
         Complete the workflow or run `ecc-workflow transition done`.",
        state.phase, state.feature
    ))
}

#[cfg(test)]
mod tests {
    use crate::output::Status;
    use tempfile::TempDir;

    /// PC-039: stop_gate treats Idle like Done — no warning (AC-001.1)
    #[test]
    fn stop_gate_idle_no_warning() {
        let tmp = TempDir::new().unwrap();
        let workflow_dir = tmp.path().join(".claude/workflow");
        std::fs::create_dir_all(&workflow_dir).unwrap();
        let json = r#"{"phase":"idle","concern":"dev","feature":"","started_at":"2026-01-01T00:00:00Z","toolchain":{"test":null,"lint":null,"build":null},"artifacts":{"plan":null,"solution":null,"implement":null,"campaign_path":null,"spec_path":null,"design_path":null,"tasks_path":null},"completed":[]}"#;
        std::fs::write(workflow_dir.join("state.json"), json).unwrap();

        let output = super::run(&workflow_dir);
        assert!(
            matches!(output.status, Status::Pass),
            "Expected Pass (no warning) for idle phase in stop_gate, got {:?}: {}",
            output.status,
            output.message
        );
        assert!(
            !output.message.contains("WARNING"),
            "Expected no WARNING in stop_gate output for idle phase, got: {}",
            output.message
        );
    }
}
