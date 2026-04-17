//! Workflow transition history display.

use crate::output::WorkflowOutput;
use std::path::Path;

pub fn run(json: bool, state_dir: &Path) -> WorkflowOutput {
    let state_path = state_dir.join("state.json");
    let content = match std::fs::read_to_string(&state_path) {
        Ok(c) => c,
        Err(_) => return WorkflowOutput::pass("No workflow state found"),
    };
    let state: ecc_domain::workflow::state::WorkflowState = match serde_json::from_str(&content) {
        Ok(s) => s,
        Err(e) => return WorkflowOutput::block(format!("Invalid state: {e}")),
    };
    if state.history.is_empty() {
        return WorkflowOutput::pass("No transition history");
    }
    if json {
        let json_str = serde_json::to_string_pretty(&state.history)
            .unwrap_or_else(|e| format!("JSON error: {e}"));
        return WorkflowOutput::pass(&json_str);
    }
    // Text table format
    let mut lines = vec![format!(
        "{:<4} {:<12} {:<12} {:<10} {:<30} {:<24} {}",
        "#", "From", "To", "Direction", "Justification", "Timestamp", "Actor"
    )];
    for (i, record) in state.history.iter().enumerate() {
        let justify = record.justification.as_deref().unwrap_or("\u{2014}");
        lines.push(format!(
            "{:<4} {:<12} {:<12} {:<10} {:<30} {:<24} {}",
            i + 1,
            record.from,
            record.to,
            record.direction,
            justify,
            record.timestamp,
            record.actor
        ));
    }
    WorkflowOutput::pass(lines.join("\n"))
}

#[cfg(test)]
pub mod tests {
    use ecc_domain::workflow::{
        Direction, TransitionRecord,
        concern::Concern,
        phase::Phase,
        state::{Artifacts, Toolchain, WorkflowState},
        timestamp::Timestamp,
    };
    use tempfile::TempDir;

    fn make_state_with_history(records: Vec<TransitionRecord>) -> String {
        let state = WorkflowState {
            phase: Phase::Implement,
            concern: Concern::Dev,
            feature: "BL-129".to_owned(),
            started_at: Timestamp::new("2026-01-01T00:00:00Z"),
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
            history: records,
        };
        serde_json::to_string_pretty(&state).unwrap()
    }

    // PC-017: ecc-workflow history displays records chronologically
    #[test]
    fn history_displays_chronologically() {
        let dir = TempDir::new().unwrap();
        let wf_dir = dir.path().join(".claude/workflow");
        std::fs::create_dir_all(&wf_dir).unwrap();

        let records = vec![
            TransitionRecord {
                from: Phase::Plan,
                to: Phase::Solution,
                direction: Direction::Forward,
                justification: None,
                timestamp: "2026-01-01T00:00:00Z".to_owned(),
                actor: "ecc-workflow".to_owned(),
            },
            TransitionRecord {
                from: Phase::Solution,
                to: Phase::Plan,
                direction: Direction::Backward,
                justification: Some("needed revision".to_owned()),
                timestamp: "2026-01-02T00:00:00Z".to_owned(),
                actor: "ecc-workflow".to_owned(),
            },
        ];
        std::fs::write(wf_dir.join("state.json"), make_state_with_history(records)).unwrap();

        let output = super::run(false, &wf_dir);
        assert!(
            matches!(output.status, crate::output::Status::Pass),
            "history display must pass, got: {:?}: {}",
            output.status,
            output.message
        );
        // Must contain header columns
        assert!(output.message.contains("From"), "must contain From column");
        assert!(output.message.contains("To"), "must contain To column");
        assert!(
            output.message.contains("Direction"),
            "must contain Direction column"
        );
        assert!(
            output.message.contains("Justification"),
            "must contain Justification column"
        );
        assert!(
            output.message.contains("Timestamp"),
            "must contain Timestamp column"
        );
        assert!(
            output.message.contains("Actor"),
            "must contain Actor column"
        );
        // Must contain actual record data
        assert!(output.message.contains("plan"), "must contain plan phase");
        assert!(
            output.message.contains("solution"),
            "must contain solution phase"
        );
        assert!(
            output.message.contains("backward"),
            "must contain backward direction"
        );
        assert!(
            output.message.contains("needed revision"),
            "must contain justification"
        );
        // First record (#1) must appear before second record (#2) — chronological order
        let pos_1 = output.message.find('1').unwrap_or(usize::MAX);
        let pos_2 = output.message.find('2').unwrap_or(usize::MAX);
        assert!(pos_1 < pos_2, "record #1 must appear before #2 in output");
    }

    // PC-018: ecc-workflow history --json outputs valid JSON array of TransitionRecord objects
    #[test]
    fn history_json_output() {
        let dir = TempDir::new().unwrap();
        let wf_dir = dir.path().join(".claude/workflow");
        std::fs::create_dir_all(&wf_dir).unwrap();

        let records = vec![TransitionRecord {
            from: Phase::Plan,
            to: Phase::Solution,
            direction: Direction::Forward,
            justification: None,
            timestamp: "2026-01-01T00:00:00Z".to_owned(),
            actor: "ecc-workflow".to_owned(),
        }];
        std::fs::write(wf_dir.join("state.json"), make_state_with_history(records)).unwrap();

        let output = super::run(true, &wf_dir);
        assert!(
            matches!(output.status, crate::output::Status::Pass),
            "history json must pass, got: {:?}: {}",
            output.status,
            output.message
        );

        // Must be valid JSON array
        let parsed: serde_json::Value =
            serde_json::from_str(&output.message).expect("output must be valid JSON");
        assert!(parsed.is_array(), "output must be a JSON array");
        let arr = parsed.as_array().unwrap();
        assert_eq!(arr.len(), 1, "must have exactly 1 record");
        // Each element must have required fields
        let record = &arr[0];
        assert!(record.get("from").is_some(), "must have 'from' field");
        assert!(record.get("to").is_some(), "must have 'to' field");
        assert!(
            record.get("direction").is_some(),
            "must have 'direction' field"
        );
        assert!(
            record.get("justification").is_some(),
            "must have 'justification' field"
        );
        assert!(
            record.get("timestamp").is_some(),
            "must have 'timestamp' field"
        );
        assert!(record.get("actor").is_some(), "must have 'actor' field");
    }
}
