//! WorkflowState aggregate for the ECC workflow state machine.

use serde::{Deserialize, Deserializer, Serialize};
use std::str::FromStr;

use super::concern::Concern;
use super::error::WorkflowError;
use super::phase::Phase;
use super::timestamp::Timestamp;
use super::transition::Direction;

/// Deserializer for `Completion.phase` that falls back to [`Phase::Unknown`]
/// for any string that is not a recognized phase value.
///
/// # Arguments
///
/// * `deserializer` — The serde deserializer.
///
/// # Returns
///
/// The deserialized phase, or `Phase::Unknown` if unrecognized.
fn deserialize_phase_with_fallback<'de, D>(deserializer: D) -> Result<Phase, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Phase::from_str(&s).unwrap_or(Phase::Unknown))
}

/// Toolchain commands used during this workflow run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Toolchain {
    /// Test command (e.g., `cargo test`).
    pub test: Option<String>,
    /// Lint command (e.g., `cargo clippy -- -D warnings`).
    pub lint: Option<String>,
    /// Build command (e.g., `cargo build`).
    pub build: Option<String>,
}

/// Artifact timestamps and paths accumulated during the workflow.
///
/// Composition diagram — pairs each phase with its timestamp and path:
///
/// ```text
/// +--------------- Artifacts ---------------+
/// | plan:       Option<String> (timestamp)  |
/// | solution:   Option<String> (timestamp)  |
/// | implement:  Option<String> (timestamp)  |
/// | campaign_path: Option<String>           |
/// | spec_path:  Option<String>              |
/// | design_path:Option<String>              |
/// | tasks_path: Option<String>              |
/// +-----------------------------------------+
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifacts {
    /// ISO 8601 timestamp when the Plan phase completed.
    pub plan: Option<String>,
    /// ISO 8601 timestamp when the Solution phase completed.
    pub solution: Option<String>,
    /// ISO 8601 timestamp when the Implement phase completed.
    pub implement: Option<String>,
    /// Path to the campaign artifacts file (if applicable).
    pub campaign_path: Option<String>,
    /// Path to the spec file from the Plan phase.
    pub spec_path: Option<String>,
    /// Path to the design file from the Solution phase.
    pub design_path: Option<String>,
    /// Path to the tasks file from the Implement phase.
    pub tasks_path: Option<String>,
}

impl Artifacts {
    /// Clear timestamps and path fields for phases between `to` (inclusive) and `from` (inclusive).
    ///
    /// Pipeline order: Plan → Solution → Implement
    /// - Plan maps to: `plan` timestamp, `spec_path`
    /// - Solution maps to: `solution` timestamp, `design_path`
    /// - Implement maps to: `implement` timestamp, `tasks_path`
    ///
    /// # Arguments
    ///
    /// * `from` — The starting phase (inclusive).
    /// * `to` — The ending phase (inclusive).
    pub fn clear_artifacts_for_rollback(&mut self, from: Phase, to: Phase) {
        // Pipeline order encoded as indices: Plan=0, Solution=1, Implement=2
        let phase_index = |p: Phase| match p {
            Phase::Plan => Some(0usize),
            Phase::Solution => Some(1),
            Phase::Implement => Some(2),
            _ => None,
        };
        let Some(from_idx) = phase_index(from) else {
            return;
        };
        let Some(to_idx) = phase_index(to) else {
            return;
        };
        let (lo, hi) = if to_idx <= from_idx {
            (to_idx, from_idx)
        } else {
            (from_idx, to_idx)
        };
        if lo == 0 {
            self.plan = None;
            self.spec_path = None;
        }
        if lo <= 1 && 1 <= hi {
            self.solution = None;
            self.design_path = None;
        }
        if hi >= 2 {
            self.implement = None;
            self.tasks_path = None;
        }
    }
}

/// A single transition record appended to the workflow history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransitionRecord {
    /// The phase being transitioned from.
    pub from: Phase,
    /// The phase being transitioned to.
    pub to: Phase,
    /// Whether this is a forward or backward transition.
    pub direction: Direction,
    /// Justification for backward transitions (required for rollbacks).
    pub justification: Option<String>,
    /// ISO 8601 timestamp of the transition.
    pub timestamp: String,
    /// The actor who initiated the transition (e.g., `ecc-workflow`).
    pub actor: String,
}

/// A single completed phase record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Completion {
    /// The phase that was completed. Unrecognized strings deserialize as [`Phase::Unknown`].
    #[serde(deserialize_with = "deserialize_phase_with_fallback")]
    pub phase: Phase,
    /// Path to the artifact produced in this phase.
    pub file: String,
    /// ISO 8601 timestamp when this phase completed.
    pub at: String,
}

/// Default version number for the workflow state format.
fn default_version() -> u32 {
    1
}

/// The root aggregate for workflow state machine data.
///
/// Note: `deny_unknown_fields` must NEVER be added — forward compatibility
/// requires ignoring unknown fields so older readers can parse newer state files.
///
/// Composition diagram (aggregate root + owned value objects):
///
/// ```text
/// +---------------------- WorkflowState -----------------------+
/// | phase: Phase           (current FSM state)                 |
/// | concern: Concern       (dev | fix | refactor)              |
/// | feature: String        (human feature label)               |
/// | started_at: Timestamp  (session open ISO 8601)             |
/// | toolchain: Toolchain   +--> { test, lint, build: Option<String> }
/// | artifacts: Artifacts   +--> { plan, solution, implement, ...
/// |                                campaign_path, spec_path,   |
/// |                                design_path, tasks_path }   |
/// | completed: Vec<Completion>                                 |
/// | version: u32           (default = 1)                       |
/// | history: Vec<TransitionRecord>                             |
/// +------------------------------------------------------------+
/// ```
///
/// # Pattern
///
/// Aggregate Root \[DDD\] — single write-through entry point for workflow state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowState {
    /// The current workflow phase.
    pub phase: Phase,
    /// The type of concern (dev/fix/refactor).
    pub concern: Concern,
    /// The feature being implemented.
    pub feature: String,
    /// ISO 8601 timestamp when the workflow started.
    pub started_at: Timestamp,
    /// Toolchain commands used.
    pub toolchain: Toolchain,
    /// Timestamps and paths for produced artifacts.
    pub artifacts: Artifacts,
    /// Phases that have been completed.
    pub completed: Vec<Completion>,
    /// Format version of this state (defaults to 1).
    #[serde(default = "default_version")]
    pub version: u32,
    /// Full history of all transitions (forward and backward).
    #[serde(default)]
    pub history: Vec<TransitionRecord>,
}

impl WorkflowState {
    /// Deserialize a `WorkflowState` from a JSON string, mapping parse errors to
    /// [`WorkflowError::InvalidState`] with a descriptive message.
    ///
    /// # Arguments
    ///
    /// * `json` — The JSON string to deserialize.
    ///
    /// # Returns
    ///
    /// `Ok(WorkflowState)` on success, or `Err(WorkflowError::InvalidState)` on parse failure.
    pub fn from_json(json: &str) -> Result<Self, WorkflowError> {
        serde_json::from_str(json).map_err(|e| WorkflowError::InvalidState(e.to_string()))
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
            version: 1,
            history: vec![],
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
            version: 1,
            history: vec![],
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

        // BL-155: Foundation variant round-trip.
        let json_foundation =
            serde_json::to_string(&Concern::Foundation).expect("serialization must succeed");
        assert_eq!(json_foundation, r#""foundation""#);
        let restored_foundation: Concern =
            serde_json::from_str(&json_foundation).expect("deserialization must succeed");
        assert_eq!(restored_foundation, Concern::Foundation);
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

    #[test]
    fn version_field_default() {
        // Old JSON without version field should deserialize with version=1
        let json = r#"{
            "phase": "plan",
            "concern": "dev",
            "feature": "test",
            "started_at": "2026-01-01T00:00:00Z",
            "toolchain": {"test": null, "lint": null, "build": null},
            "artifacts": {"plan": null, "solution": null, "implement": null, "campaign_path": null, "spec_path": null, "design_path": null, "tasks_path": null},
            "completed": []
        }"#;
        let state = WorkflowState::from_json(json).expect("should deserialize without version");
        assert_eq!(state.version, 1, "version must default to 1 when absent");
    }

    #[test]
    fn version_field_serialized() {
        let state_v = WorkflowState {
            phase: Phase::Idle,
            concern: Concern::Dev,
            feature: "test".to_owned(),
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
            history: vec![],
        };
        let json = serde_json::to_string_pretty(&state_v).expect("serialization must succeed");
        assert!(
            json.contains(r#""version": 1"#),
            "JSON must contain version field, got: {json}"
        );
    }

    #[test]
    fn ignores_unknown_fields() {
        // JSON with an extra unknown field should deserialize without error
        let json = r#"{
            "phase": "plan",
            "concern": "dev",
            "feature": "test",
            "started_at": "2026-01-01T00:00:00Z",
            "toolchain": {"test": null, "lint": null, "build": null},
            "artifacts": {"plan": null, "solution": null, "implement": null, "campaign_path": null, "spec_path": null, "design_path": null, "tasks_path": null},
            "completed": [],
            "version": 1,
            "future_field": true
        }"#;
        let state = WorkflowState::from_json(json).expect("should ignore unknown fields");
        assert_eq!(state.phase, Phase::Plan);
        assert_eq!(state.version, 1);
    }
}

#[cfg(test)]
mod artifacts {
    use super::*;
    use crate::workflow::phase::Phase;

    fn make_artifacts() -> Artifacts {
        Artifacts {
            plan: Some("2026-01-01T00:00:00Z".to_owned()),
            solution: Some("2026-01-02T00:00:00Z".to_owned()),
            implement: Some("2026-01-03T00:00:00Z".to_owned()),
            campaign_path: None,
            spec_path: Some("docs/specs/foo/spec.md".to_owned()),
            design_path: Some("docs/specs/foo/design.md".to_owned()),
            tasks_path: Some("docs/specs/foo/tasks.md".to_owned()),
        }
    }

    #[test]
    fn clear_impl_to_solution() {
        // Rollback from Implement to Solution:
        // clears solution + implement timestamps AND design_path + tasks_path
        // preserves plan timestamp and spec_path
        let mut artifacts = make_artifacts();
        artifacts.clear_artifacts_for_rollback(Phase::Implement, Phase::Solution);
        assert_eq!(
            artifacts.plan,
            Some("2026-01-01T00:00:00Z".to_owned()),
            "plan timestamp must be preserved"
        );
        assert!(
            artifacts.solution.is_none(),
            "solution timestamp must be cleared"
        );
        assert!(
            artifacts.implement.is_none(),
            "implement timestamp must be cleared"
        );
        assert_eq!(
            artifacts.spec_path,
            Some("docs/specs/foo/spec.md".to_owned()),
            "spec_path must be preserved"
        );
        assert!(
            artifacts.design_path.is_none(),
            "design_path must be cleared"
        );
        assert!(artifacts.tasks_path.is_none(), "tasks_path must be cleared");
    }

    #[test]
    fn clear_solution_to_plan() {
        // Rollback from Solution to Plan:
        // clears plan + solution timestamps AND spec_path + design_path
        // preserves implement timestamp
        let mut artifacts = make_artifacts();
        artifacts.clear_artifacts_for_rollback(Phase::Solution, Phase::Plan);
        assert!(artifacts.plan.is_none(), "plan timestamp must be cleared");
        assert!(
            artifacts.solution.is_none(),
            "solution timestamp must be cleared"
        );
        assert_eq!(
            artifacts.implement,
            Some("2026-01-03T00:00:00Z".to_owned()),
            "implement timestamp must be preserved"
        );
        assert!(artifacts.spec_path.is_none(), "spec_path must be cleared");
        assert!(
            artifacts.design_path.is_none(),
            "design_path must be cleared"
        );
        assert_eq!(
            artifacts.tasks_path,
            Some("docs/specs/foo/tasks.md".to_owned()),
            "tasks_path must be preserved"
        );
    }

    #[test]
    fn clear_impl_to_plan() {
        // Rollback from Implement to Plan:
        // clears ALL three timestamps AND spec_path + design_path + tasks_path
        let mut artifacts = make_artifacts();
        artifacts.clear_artifacts_for_rollback(Phase::Implement, Phase::Plan);
        assert!(artifacts.plan.is_none(), "plan timestamp must be cleared");
        assert!(
            artifacts.solution.is_none(),
            "solution timestamp must be cleared"
        );
        assert!(
            artifacts.implement.is_none(),
            "implement timestamp must be cleared"
        );
        assert!(artifacts.spec_path.is_none(), "spec_path must be cleared");
        assert!(
            artifacts.design_path.is_none(),
            "design_path must be cleared"
        );
        assert!(artifacts.tasks_path.is_none(), "tasks_path must be cleared");
    }
}

#[cfg(test)]
mod state {
    use super::*;
    use crate::workflow::concern::Concern;
    use crate::workflow::phase::Phase;
    use crate::workflow::timestamp::Timestamp;
    use crate::workflow::transition::Direction;

    fn make_state() -> WorkflowState {
        WorkflowState {
            phase: Phase::Plan,
            concern: Concern::Dev,
            feature: "test".to_owned(),
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
            history: vec![],
        }
    }

    #[test]
    fn transition_record_serde() {
        // TransitionRecord serialization roundtrip: contains all required fields
        let record = TransitionRecord {
            from: Phase::Plan,
            to: Phase::Solution,
            direction: Direction::Forward,
            justification: Some("moved forward".to_owned()),
            timestamp: "2026-01-01T00:00:00Z".to_owned(),
            actor: "ecc-workflow".to_owned(),
        };
        let json = serde_json::to_string(&record).expect("serialization must succeed");
        assert!(json.contains(r#""from""#), "must contain from");
        assert!(json.contains(r#""to""#), "must contain to");
        assert!(json.contains(r#""direction""#), "must contain direction");
        assert!(
            json.contains(r#""justification""#),
            "must contain justification"
        );
        assert!(json.contains(r#""timestamp""#), "must contain timestamp");
        assert!(json.contains(r#""actor""#), "must contain actor");
        let restored: TransitionRecord =
            serde_json::from_str(&json).expect("deserialization must succeed");
        assert_eq!(restored.from, Phase::Plan);
        assert_eq!(restored.to, Phase::Solution);
        assert_eq!(restored.direction, Direction::Forward);
        assert_eq!(restored.justification, Some("moved forward".to_owned()));
        assert_eq!(restored.timestamp, "2026-01-01T00:00:00Z");
        assert_eq!(restored.actor, "ecc-workflow");
    }

    #[test]
    fn history_default_empty() {
        // Old JSON without 'history' field deserializes with history=[]
        let json = r#"{
            "phase": "plan",
            "concern": "dev",
            "feature": "test",
            "started_at": "2026-01-01T00:00:00Z",
            "toolchain": {"test": null, "lint": null, "build": null},
            "artifacts": {"plan": null, "solution": null, "implement": null, "campaign_path": null, "spec_path": null, "design_path": null, "tasks_path": null},
            "completed": [],
            "version": 1
        }"#;
        let state = WorkflowState::from_json(json).expect("must deserialize without history field");
        assert!(
            state.history.is_empty(),
            "history must default to empty vec when absent"
        );
    }
}
