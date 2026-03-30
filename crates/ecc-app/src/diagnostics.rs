//! Diagnostics use case — `ecc status` data gathering.

use ecc_ports::env::Environment;
use ecc_ports::fs::FileSystem;
use serde::Serialize;

/// Full diagnostic snapshot returned by [`gather_status`].
#[derive(Debug, Serialize)]
pub struct DiagnosticReport {
    pub ecc_version: String,
    pub workflow_phase: Option<String>,
    pub workflow_feature: Option<String>,
    pub artifacts: ArtifactStatus,
    pub component_counts: ComponentCounts,
    pub hook_count: usize,
    pub config_path: String,
    pub installed: bool,
}

/// Presence status of workflow artifact files.
#[derive(Debug, Serialize)]
pub struct ArtifactStatus {
    pub spec: bool,
    pub design: bool,
    pub tasks: bool,
}

/// Counts of installed ECC component files.
#[derive(Debug, Serialize)]
pub struct ComponentCounts {
    pub agents: usize,
    pub skills: usize,
    pub commands: usize,
    pub rules: usize,
}

/// Gather a diagnostic snapshot using the provided port implementations.
pub fn gather_status(fs: &dyn FileSystem, env: &dyn Environment) -> DiagnosticReport {
    // Stub: always returns "not installed" — tests must fail.
    let _ = (fs, env);
    DiagnosticReport {
        ecc_version: String::new(),
        workflow_phase: None,
        workflow_feature: None,
        artifacts: ArtifactStatus {
            spec: false,
            design: false,
            tasks: false,
        },
        component_counts: ComponentCounts {
            agents: 0,
            skills: 0,
            commands: 0,
            rules: 0,
        },
        hook_count: 0,
        config_path: String::new(),
        installed: false,
    }
}

/// Format a [`DiagnosticReport`] as human-readable key-value lines.
pub fn format_human(report: &DiagnosticReport) -> String {
    let _ = report;
    String::new()
}

/// Format a [`DiagnosticReport`] as pretty-printed JSON.
pub fn format_json(report: &DiagnosticReport) -> String {
    serde_json::to_string_pretty(report).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::{InMemoryFileSystem, MockEnvironment};

    fn make_state_json(phase: &str, feature: &str, spec: bool, design: bool, tasks: bool) -> String {
        let spec_val = if spec { r#""docs/specs/spec.md""# } else { "null" };
        let design_val = if design { r#""docs/specs/design.md""# } else { "null" };
        let tasks_val = if tasks { r#""docs/specs/tasks.md""# } else { "null" };
        format!(
            r#"{{
  "phase": "{phase}",
  "concern": "dev",
  "feature": "{feature}",
  "started_at": "2026-01-01T00:00:00Z",
  "toolchain": {{"test": null, "lint": null, "build": null}},
  "artifacts": {{
    "plan": null,
    "solution": null,
    "implement": null,
    "campaign_path": null,
    "spec_path": {spec_val},
    "design_path": {design_val},
    "tasks_path": {tasks_val}
  }},
  "completed": []
}}"#
        )
    }

    #[test]
    fn gather_status_with_active_workflow() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new().with_home("/home/test");

        let claude_dir = std::path::Path::new("/home/test/.claude");
        fs.create_dir_all(claude_dir).unwrap();
        fs.create_dir_all(&claude_dir.join("agents")).unwrap();
        fs.create_dir_all(&claude_dir.join("skills")).unwrap();
        fs.create_dir_all(&claude_dir.join("commands")).unwrap();
        fs.create_dir_all(&claude_dir.join("rules")).unwrap();
        fs.create_dir_all(&claude_dir.join("workflow")).unwrap();

        let state_json = make_state_json("implement", "my-feature", true, true, false);
        fs.write(&claude_dir.join("workflow/state.json"), &state_json).unwrap();

        fs.write(&claude_dir.join("agents/planner.md"), "# planner").unwrap();
        fs.write(&claude_dir.join("agents/tdd-guide.md"), "# tdd").unwrap();

        let report = gather_status(&fs, &env);

        assert!(report.installed, "should be installed when ~/.claude/ exists");
        assert_eq!(report.workflow_phase.as_deref(), Some("implement"));
        assert_eq!(report.workflow_feature.as_deref(), Some("my-feature"));
        assert!(report.artifacts.spec, "spec artifact should be present");
        assert!(report.artifacts.design, "design artifact should be present");
        assert!(!report.artifacts.tasks, "tasks artifact should be absent");
        assert_eq!(report.component_counts.agents, 2, "should count 2 agent files");
    }

    #[test]
    fn gather_status_without_workflow() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new().with_home("/home/test");

        let claude_dir = std::path::Path::new("/home/test/.claude");
        fs.create_dir_all(claude_dir).unwrap();

        let report = gather_status(&fs, &env);

        assert!(report.installed, "should be installed when ~/.claude/ exists");
        assert!(report.workflow_phase.is_none(), "no workflow phase without state.json");
        assert!(report.workflow_feature.is_none(), "no workflow feature without state.json");
    }

    #[test]
    fn gather_status_missing_claude_dir() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new().with_home("/home/test");

        let report = gather_status(&fs, &env);

        assert!(!report.installed, "should be not installed when ~/.claude/ is missing");
    }

    #[test]
    fn format_human_contains_required_keys() {
        let report = DiagnosticReport {
            ecc_version: "1.2.3".to_owned(),
            workflow_phase: Some("implement".to_owned()),
            workflow_feature: Some("feature-x".to_owned()),
            artifacts: ArtifactStatus { spec: true, design: false, tasks: true },
            component_counts: ComponentCounts { agents: 5, skills: 3, commands: 2, rules: 4 },
            hook_count: 7,
            config_path: "/home/test/.ecc/config.toml".to_owned(),
            installed: true,
        };

        let output = format_human(&report);

        assert!(output.contains("ECC"), "output must include 'ECC'");
        assert!(output.contains("Phase:"), "output must include 'Phase:'");
        assert!(output.contains("Feature:"), "output must include 'Feature:'");
        assert!(output.contains("Components:"), "output must include 'Components:'");
        assert!(output.contains("Config:"), "output must include 'Config:'");
    }

    #[test]
    fn format_json_parses_as_valid_json() {
        let report = DiagnosticReport {
            ecc_version: "1.0.0".to_owned(),
            workflow_phase: None,
            workflow_feature: None,
            artifacts: ArtifactStatus { spec: false, design: false, tasks: false },
            component_counts: ComponentCounts { agents: 0, skills: 0, commands: 0, rules: 0 },
            hook_count: 0,
            config_path: String::new(),
            installed: false,
        };

        let json_str = format_json(&report);
        let parsed: serde_json::Value =
            serde_json::from_str(&json_str).expect("format_json must produce valid JSON");
        assert!(parsed.is_object(), "JSON output must be an object");
    }
}
