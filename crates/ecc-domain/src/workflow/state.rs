//! WorkflowState aggregate for the ECC workflow state machine.

use serde::{Deserialize, Serialize};

use super::phase::Phase;

/// Toolchain commands used during this workflow run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Toolchain {
    pub test: Option<String>,
    pub lint: Option<String>,
    pub build: Option<String>,
}

/// Artifact timestamps and paths accumulated during the workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Artifacts {
    pub plan: Option<String>,
    pub solution: Option<String>,
    pub implement: Option<String>,
    pub campaign_path: Option<String>,
    pub spec_path: Option<String>,
    pub design_path: Option<String>,
    pub tasks_path: Option<String>,
}

/// A single completed phase record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Completion {
    pub phase: String,
    pub file: String,
    pub at: String,
}

/// The root aggregate for workflow state machine data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowState {
    pub phase: Phase,
    pub concern: String,
    pub feature: String,
    pub started_at: String,
    pub toolchain: Toolchain,
    pub artifacts: Artifacts,
    pub completed: Vec<Completion>,
}

#[cfg(test)]
mod corrupted_json {
    use super::*;
    use crate::workflow::error::WorkflowError;

    #[test]
    fn invalid_json_syntax_returns_invalid_state() {
        let result = WorkflowState::from_json("not valid json {{{");
        assert!(
            matches!(result, Err(WorkflowError::InvalidState(_))),
            "expected InvalidState, got {result:?}"
        );
        let Err(WorkflowError::InvalidState(msg)) = result else {
            panic!("unreachable");
        };
        assert!(!msg.is_empty(), "error message must not be empty");
    }

    #[test]
    fn missing_required_field_returns_invalid_state() {
        // Valid JSON but missing the required "phase" field
        let json = r#"{
            "concern": "dev",
            "feature": "test",
            "started_at": "2026-01-01T00:00:00Z",
            "toolchain": {"test": null, "lint": null, "build": null},
            "artifacts": {"plan": null, "solution": null, "implement": null, "campaign_path": null, "spec_path": null, "design_path": null, "tasks_path": null},
            "completed": []
        }"#;
        let result = WorkflowState::from_json(json);
        assert!(
            matches!(result, Err(WorkflowError::InvalidState(_))),
            "expected InvalidState for missing 'phase' field, got {result:?}"
        );
        let Err(WorkflowError::InvalidState(msg)) = result else {
            panic!("unreachable");
        };
        assert!(!msg.is_empty(), "error message must not be empty");
    }

    #[test]
    fn wrong_type_for_phase_returns_invalid_state() {
        // Valid JSON but "phase" is a number instead of a string
        let json = r#"{
            "phase": 123,
            "concern": "dev",
            "feature": "test",
            "started_at": "2026-01-01T00:00:00Z",
            "toolchain": {"test": null, "lint": null, "build": null},
            "artifacts": {"plan": null, "solution": null, "implement": null, "campaign_path": null, "spec_path": null, "design_path": null, "tasks_path": null},
            "completed": []
        }"#;
        let result = WorkflowState::from_json(json);
        assert!(
            matches!(result, Err(WorkflowError::InvalidState(_))),
            "expected InvalidState for wrong type, got {result:?}"
        );
        let Err(WorkflowError::InvalidState(msg)) = result else {
            panic!("unreachable");
        };
        assert!(!msg.is_empty(), "error message must not be empty");
    }

    #[test]
    fn unknown_phase_value_returns_invalid_state() {
        // Valid JSON but "phase" has an unknown value
        let json = r#"{
            "phase": "banana",
            "concern": "dev",
            "feature": "test",
            "started_at": "2026-01-01T00:00:00Z",
            "toolchain": {"test": null, "lint": null, "build": null},
            "artifacts": {"plan": null, "solution": null, "implement": null, "campaign_path": null, "spec_path": null, "design_path": null, "tasks_path": null},
            "completed": []
        }"#;
        let result = WorkflowState::from_json(json);
        assert!(
            matches!(result, Err(WorkflowError::InvalidState(_))),
            "expected InvalidState for unknown phase 'banana', got {result:?}"
        );
        let Err(WorkflowError::InvalidState(msg)) = result else {
            panic!("unreachable");
        };
        assert!(!msg.is_empty(), "error message must not be empty");
    }

    #[test]
    fn empty_string_returns_invalid_state() {
        let result = WorkflowState::from_json("");
        assert!(
            matches!(result, Err(WorkflowError::InvalidState(_))),
            "expected InvalidState for empty string, got {result:?}"
        );
        let Err(WorkflowError::InvalidState(msg)) = result else {
            panic!("unreachable");
        };
        assert!(!msg.is_empty(), "error message must not be empty");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::phase::Phase;

    #[test]
    fn json_round_trip() {
        let toolchain = Toolchain {
            test: Some("cargo test".to_owned()),
            lint: Some("cargo clippy -- -D warnings".to_owned()),
            build: Some("cargo build".to_owned()),
        };

        let artifacts = Artifacts {
            plan: Some("2026-03-26T22:09:19Z".to_owned()),
            solution: Some("2026-03-26T22:21:47Z".to_owned()),
            implement: Some("2026-03-26T22:28:30Z".to_owned()),
            campaign_path: None,
            spec_path: Some(
                "docs/specs/2026-03-26-replace-hooks-with-rust/spec.md".to_owned(),
            ),
            design_path: Some(
                "docs/specs/2026-03-26-replace-hooks-with-rust/design.md".to_owned(),
            ),
            tasks_path: Some(
                "docs/specs/2026-03-26-replace-hooks-with-rust/tasks.md".to_owned(),
            ),
        };

        let original = WorkflowState {
            phase: Phase::Implement,
            concern: "dev".to_owned(),
            feature: "BL-052: Replace shell hooks with compiled Rust binaries".to_owned(),
            started_at: "2026-03-26T21:45:00Z".to_owned(),
            toolchain,
            artifacts,
            completed: vec![],
        };

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&original).expect("serialization must succeed");

        // Verify required keys are present
        assert!(json.contains(r#""concern""#), "JSON must contain 'concern' key");
        assert!(json.contains(r#""phase""#), "JSON must contain 'phase' key");
        assert!(json.contains(r#""feature""#), "JSON must contain 'feature' key");
        assert!(json.contains(r#""started_at""#), "JSON must contain 'started_at' key");
        assert!(json.contains(r#""toolchain""#), "JSON must contain 'toolchain' key");
        assert!(json.contains(r#""artifacts""#), "JSON must contain 'artifacts' key");
        assert!(json.contains(r#""completed""#), "JSON must contain 'completed' key");

        // Verify phase serializes as lowercase string
        assert!(
            json.contains(r#""phase": "implement""#),
            "phase must serialize as lowercase 'implement', got: {json}"
        );

        // Deserialize back and assert equality
        let restored: WorkflowState =
            serde_json::from_str(&json).expect("deserialization must succeed");
        assert_eq!(original, restored);
    }

    #[test]
    fn deserializes_from_fixture() {
        let fixture = r#"{
  "concern": "dev",
  "phase": "implement",
  "feature": "BL-052: Replace shell hooks with compiled Rust binaries",
  "started_at": "2026-03-26T21:45:00Z",
  "toolchain": {
    "test": "cargo test",
    "lint": "cargo clippy -- -D warnings",
    "build": "cargo build"
  },
  "artifacts": {
    "plan": "2026-03-26T22:09:19Z",
    "solution": "2026-03-26T22:21:47Z",
    "implement": "2026-03-26T22:28:30Z",
    "campaign_path": null,
    "spec_path": "docs/specs/2026-03-26-replace-hooks-with-rust/spec.md",
    "design_path": "docs/specs/2026-03-26-replace-hooks-with-rust/design.md",
    "tasks_path": "docs/specs/2026-03-26-replace-hooks-with-rust/tasks.md"
  },
  "completed": []
}"#;

        let state: WorkflowState =
            serde_json::from_str(fixture).expect("fixture deserialization must succeed");

        assert_eq!(state.phase, Phase::Implement);
        assert_eq!(state.concern, "dev");
        assert_eq!(
            state.feature,
            "BL-052: Replace shell hooks with compiled Rust binaries"
        );
        assert_eq!(state.started_at, "2026-03-26T21:45:00Z");
        assert_eq!(state.toolchain.test, Some("cargo test".to_owned()));
        assert_eq!(state.artifacts.campaign_path, None);
        assert_eq!(
            state.artifacts.spec_path,
            Some("docs/specs/2026-03-26-replace-hooks-with-rust/spec.md".to_owned())
        );
        assert!(state.completed.is_empty());
    }

    #[test]
    fn creates_workflow_state_with_all_fields() {
        let toolchain = Toolchain {
            test: Some("cargo test".to_owned()),
            lint: Some("cargo clippy -- -D warnings".to_owned()),
            build: Some("cargo build".to_owned()),
        };

        let artifacts = Artifacts {
            plan: Some("2026-03-26T22:09:19Z".to_owned()),
            solution: Some("2026-03-26T22:21:47Z".to_owned()),
            implement: Some("2026-03-26T22:28:30Z".to_owned()),
            campaign_path: None,
            spec_path: Some(
                "docs/specs/2026-03-26-replace-hooks-with-rust/spec.md".to_owned(),
            ),
            design_path: Some(
                "docs/specs/2026-03-26-replace-hooks-with-rust/design.md".to_owned(),
            ),
            tasks_path: Some(
                "docs/specs/2026-03-26-replace-hooks-with-rust/tasks.md".to_owned(),
            ),
        };

        let completion = Completion {
            phase: "solution".to_owned(),
            file: "docs/specs/foo/design.md".to_owned(),
            at: "2026-03-26T22:00:00Z".to_owned(),
        };

        let state = WorkflowState {
            phase: Phase::Implement,
            concern: "dev".to_owned(),
            feature: "BL-052: Replace shell hooks with compiled Rust binaries".to_owned(),
            started_at: "2026-03-26T21:45:00Z".to_owned(),
            toolchain: toolchain.clone(),
            artifacts: artifacts.clone(),
            completed: vec![completion.clone()],
        };

        // Verify phase field
        assert_eq!(state.phase, Phase::Implement);

        // Verify concern field
        assert_eq!(state.concern, "dev");

        // Verify feature field
        assert_eq!(
            state.feature,
            "BL-052: Replace shell hooks with compiled Rust binaries"
        );

        // Verify started_at field
        assert_eq!(state.started_at, "2026-03-26T21:45:00Z");

        // Verify toolchain fields
        assert_eq!(state.toolchain.test, Some("cargo test".to_owned()));
        assert_eq!(
            state.toolchain.lint,
            Some("cargo clippy -- -D warnings".to_owned())
        );
        assert_eq!(state.toolchain.build, Some("cargo build".to_owned()));

        // Verify artifacts fields
        assert_eq!(
            state.artifacts.plan,
            Some("2026-03-26T22:09:19Z".to_owned())
        );
        assert_eq!(
            state.artifacts.solution,
            Some("2026-03-26T22:21:47Z".to_owned())
        );
        assert_eq!(
            state.artifacts.implement,
            Some("2026-03-26T22:28:30Z".to_owned())
        );
        assert_eq!(state.artifacts.campaign_path, None);
        assert_eq!(
            state.artifacts.spec_path,
            Some("docs/specs/2026-03-26-replace-hooks-with-rust/spec.md".to_owned())
        );
        assert_eq!(
            state.artifacts.design_path,
            Some("docs/specs/2026-03-26-replace-hooks-with-rust/design.md".to_owned())
        );
        assert_eq!(
            state.artifacts.tasks_path,
            Some("docs/specs/2026-03-26-replace-hooks-with-rust/tasks.md".to_owned())
        );

        // Verify completed field
        assert_eq!(state.completed.len(), 1);
        assert_eq!(state.completed[0].phase, "solution");
        assert_eq!(state.completed[0].file, "docs/specs/foo/design.md");
        assert_eq!(state.completed[0].at, "2026-03-26T22:00:00Z");
    }
}
