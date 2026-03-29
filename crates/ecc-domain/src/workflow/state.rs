//! WorkflowState aggregate for the ECC workflow state machine.

use serde::{Deserialize, Deserializer, Serialize};
use std::str::FromStr;

use super::concern::Concern;
use super::error::WorkflowError;
use super::phase::Phase;
use super::timestamp::Timestamp;

/// Deserializer for `Completion.phase` that falls back to [`Phase::Unknown`]
/// for any string that is not a recognized phase value.
fn deserialize_phase_with_fallback<'de, D>(deserializer: D) -> Result<Phase, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Phase::from_str(&s).unwrap_or(Phase::Unknown))
}

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
    /// The phase that was completed. Unrecognized strings deserialize as [`Phase::Unknown`].
    #[serde(deserialize_with = "deserialize_phase_with_fallback")]
    pub phase: Phase,
    /// Path to the artifact produced in this phase.
    pub file: String,
    /// ISO 8601 timestamp when this phase completed.
    pub at: String,
}

/// The root aggregate for workflow state machine data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowState {
    pub phase: Phase,
    pub concern: Concern,
    pub feature: String,
    pub started_at: Timestamp,
    pub toolchain: Toolchain,
    pub artifacts: Artifacts,
    pub completed: Vec<Completion>,
}

impl WorkflowState {
    /// Deserialize a `WorkflowState` from a JSON string, mapping parse errors to
    /// [`WorkflowError::InvalidState`] with a descriptive message.
    pub fn from_json(json: &str) -> Result<Self, WorkflowError> {
        serde_json::from_str(json).map_err(|e| WorkflowError::InvalidState(e.to_string()))
    }
}

impl crate::traits::Transitionable for WorkflowState {
    fn transition_to(self, target: Phase) -> Result<Self, WorkflowError> {
        use super::transition::resolve_transition;
        let new_phase = resolve_transition(self.phase, target)?;
        Ok(Self {
            phase: new_phase,
            ..self
        })
    }
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
    use crate::workflow::concern::Concern;
    use crate::workflow::phase::Phase;
    use crate::workflow::timestamp::Timestamp;

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
            spec_path: Some("docs/specs/2026-03-26-replace-hooks-with-rust/spec.md".to_owned()),
            design_path: Some("docs/specs/2026-03-26-replace-hooks-with-rust/design.md".to_owned()),
            tasks_path: Some("docs/specs/2026-03-26-replace-hooks-with-rust/tasks.md".to_owned()),
        };

        let original = WorkflowState {
            phase: Phase::Implement,
            concern: Concern::Dev,
            feature: "BL-052: Replace shell hooks with compiled Rust binaries".to_owned(),
            started_at: Timestamp::new("2026-03-26T21:45:00Z"),
            toolchain,
            artifacts,
            completed: vec![],
        };

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&original).expect("serialization must succeed");

        // Verify required keys are present
        assert!(
            json.contains(r#""concern""#),
            "JSON must contain 'concern' key"
        );
        assert!(json.contains(r#""phase""#), "JSON must contain 'phase' key");
        assert!(
            json.contains(r#""feature""#),
            "JSON must contain 'feature' key"
        );
        assert!(
            json.contains(r#""started_at""#),
            "JSON must contain 'started_at' key"
        );
        assert!(
            json.contains(r#""toolchain""#),
            "JSON must contain 'toolchain' key"
        );
        assert!(
            json.contains(r#""artifacts""#),
            "JSON must contain 'artifacts' key"
        );
        assert!(
            json.contains(r#""completed""#),
            "JSON must contain 'completed' key"
        );

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
        assert_eq!(state.concern, Concern::Dev);
        assert_eq!(
            state.feature,
            "BL-052: Replace shell hooks with compiled Rust binaries"
        );
        assert_eq!(state.started_at, Timestamp::new("2026-03-26T21:45:00Z"));
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
            spec_path: Some("docs/specs/2026-03-26-replace-hooks-with-rust/spec.md".to_owned()),
            design_path: Some("docs/specs/2026-03-26-replace-hooks-with-rust/design.md".to_owned()),
            tasks_path: Some("docs/specs/2026-03-26-replace-hooks-with-rust/tasks.md".to_owned()),
        };

        let completion = Completion {
            phase: Phase::Solution,
            file: "docs/specs/foo/design.md".to_owned(),
            at: "2026-03-26T22:00:00Z".to_owned(),
        };

        let state = WorkflowState {
            phase: Phase::Implement,
            concern: Concern::Dev,
            feature: "BL-052: Replace shell hooks with compiled Rust binaries".to_owned(),
            started_at: Timestamp::new("2026-03-26T21:45:00Z"),
            toolchain: toolchain.clone(),
            artifacts: artifacts.clone(),
            completed: vec![completion.clone()],
        };

        // Verify phase field
        assert_eq!(state.phase, Phase::Implement);

        // Verify concern field
        assert_eq!(state.concern, Concern::Dev);

        // Verify feature field
        assert_eq!(
            state.feature,
            "BL-052: Replace shell hooks with compiled Rust binaries"
        );

        // Verify started_at field
        assert_eq!(state.started_at, Timestamp::new("2026-03-26T21:45:00Z"));

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
        assert_eq!(state.completed[0].phase, Phase::Solution);
        assert_eq!(state.completed[0].file, "docs/specs/foo/design.md");
        assert_eq!(state.completed[0].at, "2026-03-26T22:00:00Z");
    }

    #[test]
    fn concern_enum_roundtrip() {
        // Concern serializes as lowercase, deserializes from lowercase
        let json_dev = serde_json::to_string(&Concern::Dev).expect("serialization must succeed");
        assert_eq!(json_dev, r#""dev""#);
        let restored: Concern =
            serde_json::from_str(&json_dev).expect("deserialization must succeed");
        assert_eq!(restored, Concern::Dev);

        let json_fix = serde_json::to_string(&Concern::Fix).expect("serialization must succeed");
        assert_eq!(json_fix, r#""fix""#);

        let json_refactor =
            serde_json::to_string(&Concern::Refactor).expect("serialization must succeed");
        assert_eq!(json_refactor, r#""refactor""#);
    }

    #[test]
    fn completion_phase_is_typed() {
        // Completion.phase must be a Phase enum, not a String
        let completion = Completion {
            phase: Phase::Plan,
            file: "docs/specs/foo/spec.md".to_owned(),
            at: "2026-01-01T00:00:00Z".to_owned(),
        };
        assert_eq!(completion.phase, Phase::Plan);
    }

    #[test]
    fn unknown_phase_fallback() {
        // Deserializing a completion with an unrecognized phase string falls back to Phase::Unknown
        let json = r#"{
            "phase": "banana",
            "file": "docs/specs/foo/spec.md",
            "at": "2026-01-01T00:00:00Z"
        }"#;
        let completion: Completion = serde_json::from_str(json)
            .expect("should not fail — unknown phase maps to Phase::Unknown");
        assert_eq!(completion.phase, Phase::Unknown);
    }

    #[test]
    fn phase_backward_compat() {
        // Phase::Unknown serializes as the string "unknown"
        let completion = Completion {
            phase: Phase::Unknown,
            file: "docs/specs/foo/spec.md".to_owned(),
            at: "2026-01-01T00:00:00Z".to_owned(),
        };
        let json = serde_json::to_string(&completion).expect("serialization must succeed");
        assert!(json.contains(r#""phase":"unknown""#) || json.contains(r#""phase": "unknown""#));
        // Round-trip: deserialize back
        let restored: Completion =
            serde_json::from_str(&json).expect("deserialization must succeed");
        assert_eq!(restored.phase, Phase::Unknown);
    }
}
