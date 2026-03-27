//! `ecc-workflow status` — display current workflow state.

use crate::io;
use crate::output::WorkflowOutput;
use std::path::Path;

pub fn run(project_dir: &Path) -> WorkflowOutput {
    let state = match io::read_state(project_dir) {
        Ok(Some(s)) => s,
        Ok(None) => return WorkflowOutput::pass("No active workflow"),
        Err(e) => return WorkflowOutput::warn(format!("Failed to read state: {e}")),
    };

    let mut lines = Vec::new();
    lines.push(format!("Phase:      {}", state.phase));
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
        let output = run(dir.path());
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
        let output = run(dir.path());
        assert!(output.message.contains("Phase:"));
        assert!(output.message.contains("plan"));
        assert!(output.message.contains("test feature"));
        assert!(output.message.contains("Spec:"));
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
        let output = run(dir.path());
        // Verify labeled fields
        assert!(output.message.contains("Phase:"));
        assert!(output.message.contains("Concern:"));
        assert!(output.message.contains("Feature:"));
        assert!(output.message.contains("Started at:"));
    }
}
