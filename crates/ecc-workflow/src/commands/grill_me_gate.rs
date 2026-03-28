//! `grill-me-gate` subcommand.
//!
//! Called during plan/solution phases to verify that spec/campaign files
//! contain a grill-me interview section.  Always exits 0 — informational only.
//!
//! Logic:
//! - No state.json        → silent (exit 0)
//! - Phase is implement/done → skip (exit 0)
//! - spec_path or campaign_path found in artifacts → search for grill-me markers
//!   - Markers found → silent (exit 0)
//!   - Markers absent → warn on stderr: "WARNING: No grill-me interview markers found"

use std::path::Path;

use ecc_domain::workflow::phase::Phase;

use crate::output::WorkflowOutput;

/// Grill-me markers that indicate an interview section has been conducted.
const MARKERS: &[&str] = &["### Grill-Me", "Grill-Me Decisions"];

/// Return true when the file at `path` contains at least one grill-me marker.
fn file_has_grill_me_marker(path: &str) -> bool {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    MARKERS.iter().any(|marker| content.contains(marker))
}

/// Run the `grill-me-gate` subcommand.
pub fn run(project_dir: &Path) -> WorkflowOutput {
    let state = match crate::io::read_state(project_dir) {
        Ok(None) => return WorkflowOutput::pass(""),
        Ok(Some(s)) => s,
        Err(_) => return WorkflowOutput::pass(""),
    };

    // Only check during plan/solution phases.
    match state.phase {
        Phase::Plan | Phase::Solution => {}
        Phase::Implement | Phase::Done | Phase::Idle | Phase::Unknown => {
            return WorkflowOutput::pass("");
        }
    }

    // Collect the paths to check.
    let paths_to_check: Vec<&str> = [
        state.artifacts.spec_path.as_deref(),
        state.artifacts.campaign_path.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect();

    // If no artifact paths are recorded, nothing to check.
    if paths_to_check.is_empty() {
        return WorkflowOutput::pass("");
    }

    // Check each path for grill-me markers.
    let has_marker = paths_to_check.iter().any(|p| file_has_grill_me_marker(p));

    if has_marker {
        WorkflowOutput::pass("")
    } else {
        WorkflowOutput::warn(
            "WARNING: No grill-me interview markers found in spec/campaign. \
             Run the grill-me interview and add a '### Grill-Me Decisions' section.",
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::output::Status;
    use tempfile::TempDir;

    /// PC-038: grill_me_gate passes through for Idle phase (AC-001.1)
    #[test]
    fn grill_me_gate_idle_passes() {
        let tmp = TempDir::new().unwrap();
        let workflow_dir = tmp.path().join(".claude/workflow");
        std::fs::create_dir_all(&workflow_dir).unwrap();
        let json = r#"{"phase":"idle","concern":"","feature":"","started_at":"2026-01-01T00:00:00Z","toolchain":{"test":null,"lint":null,"build":null},"artifacts":{"plan":null,"solution":null,"implement":null,"campaign_path":null,"spec_path":null,"design_path":null,"tasks_path":null},"completed":[]}"#;
        std::fs::write(workflow_dir.join("state.json"), json).unwrap();

        let output = super::run(tmp.path());
        assert!(
            matches!(output.status, Status::Pass),
            "Expected Pass for idle phase in grill_me_gate, got {:?}: {}",
            output.status,
            output.message
        );
    }
}
