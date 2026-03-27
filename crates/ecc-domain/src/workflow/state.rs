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
mod tests {
    use super::*;
    use crate::workflow::phase::Phase;

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
