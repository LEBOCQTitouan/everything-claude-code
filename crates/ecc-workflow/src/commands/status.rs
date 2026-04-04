//! `ecc-workflow status` — display current workflow state.

use crate::io;
use crate::output::WorkflowOutput;
use std::path::Path;

pub fn run(state_dir: &Path) -> WorkflowOutput {
    let state = match io::read_state(state_dir) {
        Ok(Some(s)) => s,
        Ok(None) => return WorkflowOutput::pass("No active workflow"),
        Err(e) => return WorkflowOutput::warn(format!("Failed to read state: {e}")),
    };

    let mut lines = Vec::new();

    // Check staleness for non-idle, non-done phases
    let now = crate::time::utc_now_iso8601();
    let stale_suffix =
        if ecc_domain::workflow::staleness::is_stale(
            state.started_at.as_str(),
            &now,
            ecc_domain::workflow::staleness::DEFAULT_STALENESS_THRESHOLD_SECS,
        ) && !matches!(
            state.phase,
            ecc_domain::workflow::phase::Phase::Idle | ecc_domain::workflow::phase::Phase::Done
        ) {
            " (STALE)"
        } else {
            ""
        };

    lines.push(format!("Phase:      {}{stale_suffix}", state.phase));
    lines.push(format!("Concern:    {}", state.concern));
    lines.push(format!("Feature:    {}", state.feature));
    lines.push(format!("Started at: {}", state.started_at));

    if let Some(ref p) = state.artifacts.spec_path {
        lines.push(format!("Spec:       {p}"));
    }
    if let Some(ref p) = state.artifacts.design_path {
        lines.push(format!("Design:     {p}"));
    }
    if let Some(ref p) = state.artifacts.tasks_path {
        lines.push(format!("Tasks:      {p}"));
    }
    if let Some(ref p) = state.artifacts.campaign_path {
        lines.push(format!("Campaign:   {p}"));
    }

    WorkflowOutput::pass(lines.join("\n"))
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn status_no_workflow() {
        let dir = tempfile::tempdir().unwrap();
        let state_dir = dir.path().join(".claude/workflow");
        let output = run(&state_dir);
        assert_eq!(output.message, "No active workflow");
    }

    #[test]
    fn status_active_workflow() {
        let dir = tempfile::tempdir().unwrap();
        let wf_dir = dir.path().join(".claude/workflow");
        std::fs::create_dir_all(&wf_dir).unwrap();
        std::fs::write(
            wf_dir.join("state.json"),
            r#"{"phase":"plan","concern":"dev","feature":"test feature","started_at":"2026-01-01T00:00:00Z","toolchain":{"test":null,"lint":null,"build":null},"artifacts":{"plan":null,"solution":null,"implement":null,"campaign_path":null,"spec_path":"docs/specs/test/spec.md","design_path":null,"tasks_path":null},"completed":[]}"#,
        ).unwrap();
        let output = run(&wf_dir);
        assert!(output.message.contains("Phase:"));
        assert!(output.message.contains("plan"));
        assert!(output.message.contains("test feature"));
        assert!(output.message.contains("Spec:"));
    }

    /// PC-024: status shows STALE when threshold exceeded
    #[test]
    fn status_shows_stale() {
        let dir = tempfile::tempdir().unwrap();
        let wf_dir = dir.path().join(".claude/workflow");
        std::fs::create_dir_all(&wf_dir).unwrap();
        // Use a timestamp far in the past (2020) so it's always stale
        std::fs::write(
            wf_dir.join("state.json"),
            r#"{"phase":"plan","concern":"dev","feature":"test","started_at":"2020-01-01T00:00:00Z","toolchain":{"test":null,"lint":null,"build":null},"artifacts":{"plan":null,"solution":null,"implement":null,"campaign_path":null,"spec_path":null,"design_path":null,"tasks_path":null},"completed":[],"version":1}"#,
        ).unwrap();
        let output = run(&wf_dir);
        assert!(
            output.message.contains("(STALE)"),
            "status should show STALE for old workflow, got: {}",
            output.message
        );
    }

    #[test]
    fn status_labeled_output() {
        let dir = tempfile::tempdir().unwrap();
        let wf_dir = dir.path().join(".claude/workflow");
        std::fs::create_dir_all(&wf_dir).unwrap();
        std::fs::write(
            wf_dir.join("state.json"),
            r#"{"phase":"implement","concern":"fix","feature":"bug fix","started_at":"2026-01-01T00:00:00Z","toolchain":{"test":null,"lint":null,"build":null},"artifacts":{"plan":null,"solution":null,"implement":null,"campaign_path":null},"completed":[]}"#,
        ).unwrap();
        let output = run(&wf_dir);
        // Verify labeled fields
        assert!(output.message.contains("Phase:"));
        assert!(output.message.contains("Concern:"));
        assert!(output.message.contains("Feature:"));
        assert!(output.message.contains("Started at:"));
    }
}
