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
pub fn run(state_dir: &Path) -> WorkflowOutput {
    let state = match crate::io::read_state(state_dir) {
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

    // Version >= 2: block if campaign_path is absent or file missing.
    if state.version >= 2 {
        match &state.artifacts.campaign_path {
            None => {
                return WorkflowOutput::block(
                    "Campaign file not found. Run: ecc-workflow campaign init <spec-dir>"
                        .to_string(),
                );
            }
            Some(path) => {
                if !std::path::Path::new(path).exists() {
                    return WorkflowOutput::block(format!(
                        "Campaign file not found at {path}. Run: ecc-workflow campaign init <spec-dir>"
                    ));
                }
            }
        }
    }

    // Collect the paths to check for grill-me markers.
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

    fn write_state(dir: &std::path::Path, phase: &str, campaign_path: Option<&str>, version: u32) {
        std::fs::create_dir_all(dir).unwrap();
        let cp = campaign_path
            .map(|p| format!(r#""{p}""#))
            .unwrap_or_else(|| "null".to_string());
        let json = format!(
            r#"{{"phase":"{phase}","concern":"dev","feature":"test","started_at":"2026-01-01T00:00:00Z","toolchain":{{"test":null,"lint":null,"build":null}},"artifacts":{{"plan":null,"solution":null,"implement":null,"campaign_path":{cp},"spec_path":null,"design_path":null,"tasks_path":null}},"completed":[],"version":{version}}}"#
        );
        std::fs::write(dir.join("state.json"), json).unwrap();
    }

    #[test]
    fn blocks_v2_plan_no_campaign() {
        let tmp = TempDir::new().unwrap();
        write_state(tmp.path(), "plan", None, 2);
        let out = super::run(tmp.path());
        assert!(
            matches!(out.status, Status::Block),
            "Expected Block, got {:?}: {}",
            out.status,
            out.message
        );
        assert!(out.message.contains("Campaign file not found"));
    }

    #[test]
    fn passes_v1_plan() {
        let tmp = TempDir::new().unwrap();
        write_state(tmp.path(), "plan", None, 1);
        let out = super::run(tmp.path());
        // v1 grandfathered — should not block
        assert!(
            !matches!(out.status, Status::Block),
            "v1 should not block, got {:?}: {}",
            out.status,
            out.message
        );
    }

    #[test]
    fn passes_non_spec_phases() {
        for phase in &["idle", "implement", "done"] {
            let tmp = TempDir::new().unwrap();
            write_state(tmp.path(), phase, None, 2);
            let out = super::run(tmp.path());
            assert!(
                matches!(out.status, Status::Pass),
                "Phase {phase} should pass, got {:?}: {}",
                out.status,
                out.message
            );
        }
    }

    #[test]
    fn passes_when_campaign_present() {
        let tmp = TempDir::new().unwrap();
        let campaign = tmp.path().join("campaign.md");
        std::fs::write(&campaign, "## Grill-Me Decisions").unwrap();
        write_state(tmp.path(), "plan", Some(campaign.to_str().unwrap()), 2);
        let out = super::run(tmp.path());
        assert!(
            matches!(out.status, Status::Pass),
            "Expected Pass when campaign exists, got {:?}: {}",
            out.status,
            out.message
        );
    }

    /// PC-038: grill_me_gate passes through for Idle phase (AC-001.1)
    #[test]
    fn grill_me_gate_idle_passes() {
        let tmp = TempDir::new().unwrap();
        let workflow_dir = tmp.path().join(".claude/workflow");
        std::fs::create_dir_all(&workflow_dir).unwrap();
        let json = r#"{"phase":"idle","concern":"dev","feature":"","started_at":"2026-01-01T00:00:00Z","toolchain":{"test":null,"lint":null,"build":null},"artifacts":{"plan":null,"solution":null,"implement":null,"campaign_path":null,"spec_path":null,"design_path":null,"tasks_path":null},"completed":[]}"#;
        std::fs::write(workflow_dir.join("state.json"), json).unwrap();

        let output = super::run(&workflow_dir);
        assert!(
            matches!(output.status, Status::Pass),
            "Expected Pass for idle phase in grill_me_gate, got {:?}: {}",
            output.status,
            output.message
        );
    }
}
