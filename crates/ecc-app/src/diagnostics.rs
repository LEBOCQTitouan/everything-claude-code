//! Diagnostics use case — `ecc status` data gathering.
//!
//! Reads workflow state, counts components, and assembles a [`DiagnosticReport`]
//! from the [`FileSystem`] and [`Environment`] ports. No direct I/O.

use ecc_ports::env::Environment;
use ecc_ports::fs::FileSystem;
use serde::Serialize;

/// Full diagnostic snapshot returned by [`gather_status`].
#[derive(Debug, Serialize)]
pub struct DiagnosticReport {
    /// ECC binary version string.
    pub ecc_version: String,
    /// Current workflow phase, or `None` when no workflow is active.
    pub workflow_phase: Option<String>,
    /// Active workflow feature name, or `None`.
    pub workflow_feature: Option<String>,
    /// Presence of key artifact files.
    pub artifacts: ArtifactStatus,
    /// Counts of installed ECC components.
    pub component_counts: ComponentCounts,
    /// Number of installed hooks.
    pub hook_count: usize,
    /// Resolved path to the global config file.
    pub config_path: String,
    /// Whether ECC appears to be installed (i.e. `~/.claude/` exists).
    pub installed: bool,
}

/// Presence status of workflow artifact files.
#[derive(Debug, Serialize)]
pub struct ArtifactStatus {
    /// Whether a spec artifact path is recorded in state.json.
    pub spec: bool,
    /// Whether a design artifact path is recorded in state.json.
    pub design: bool,
    /// Whether a tasks artifact path is recorded in state.json.
    pub tasks: bool,
}

/// Counts of installed ECC component files.
#[derive(Debug, Serialize)]
pub struct ComponentCounts {
    /// Number of agent files.
    pub agents: usize,
    /// Number of skill files.
    pub skills: usize,
    /// Number of command files.
    pub commands: usize,
    /// Number of rule files.
    pub rules: usize,
}

/// Gather a diagnostic snapshot using the provided port implementations.
///
/// - Checks `~/.claude/` existence to determine `installed`.
/// - Reads `~/.claude/workflow/state.json` for phase/feature/artifacts.
/// - Counts files in `~/.claude/agents/`, `skills/`, `commands/`, `rules/`.
pub fn gather_status(fs: &dyn FileSystem, env: &dyn Environment) -> DiagnosticReport {
    let ecc_version = crate::version::version(env);

    let home = match env.home_dir() {
        Some(h) => h,
        None => {
            return not_installed_report(ecc_version, env);
        }
    };

    let claude_dir = home.join(".claude");
    if !fs.is_dir(&claude_dir) {
        return not_installed_report(ecc_version, env);
    }

    let config_path = home
        .join(".ecc/config.toml")
        .to_string_lossy()
        .into_owned();

    // Read workflow state
    let state_path = claude_dir.join("workflow/state.json");
    let (workflow_phase, workflow_feature, artifacts) = if fs.is_file(&state_path) {
        parse_state(fs, &state_path)
    } else {
        (None, None, ArtifactStatus { spec: false, design: false, tasks: false })
    };

    // Count component files
    let component_counts = ComponentCounts {
        agents: count_files(fs, &claude_dir.join("agents")),
        skills: count_files(fs, &claude_dir.join("skills")),
        commands: count_files(fs, &claude_dir.join("commands")),
        rules: count_files(fs, &claude_dir.join("rules")),
    };

    DiagnosticReport {
        ecc_version,
        workflow_phase,
        workflow_feature,
        artifacts,
        component_counts,
        hook_count: 0,
        config_path,
        installed: true,
    }
}

fn not_installed_report(ecc_version: String, env: &dyn Environment) -> DiagnosticReport {
    let config_path = env
        .home_dir()
        .map(|h| h.join(".ecc/config.toml").to_string_lossy().into_owned())
        .unwrap_or_default();
    DiagnosticReport {
        ecc_version,
        workflow_phase: None,
        workflow_feature: None,
        artifacts: ArtifactStatus { spec: false, design: false, tasks: false },
        component_counts: ComponentCounts { agents: 0, skills: 0, commands: 0, rules: 0 },
        hook_count: 0,
        config_path,
        installed: false,
    }
}

fn parse_state(
    fs: &dyn FileSystem,
    state_path: &std::path::Path,
) -> (Option<String>, Option<String>, ArtifactStatus) {
    let content = match fs.read_to_string(state_path) {
        Ok(c) => c,
        Err(_) => return (None, None, ArtifactStatus { spec: false, design: false, tasks: false }),
    };
    let v: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return (None, None, ArtifactStatus { spec: false, design: false, tasks: false }),
    };

    let phase = v.get("phase").and_then(|p| p.as_str()).map(str::to_owned);
    let feature = v.get("feature").and_then(|f| f.as_str()).map(str::to_owned);

    let artifacts = if let Some(arts) = v.get("artifacts") {
        ArtifactStatus {
            spec: arts.get("spec_path").is_some_and(|v| !v.is_null()),
            design: arts.get("design_path").is_some_and(|v| !v.is_null()),
            tasks: arts.get("tasks_path").is_some_and(|v| !v.is_null()),
        }
    } else {
        ArtifactStatus { spec: false, design: false, tasks: false }
    };

    (phase, feature, artifacts)
}

fn count_files(fs: &dyn FileSystem, dir: &std::path::Path) -> usize {
    fs.read_dir(dir).map(|entries| entries.len()).unwrap_or(0)
}

/// Format a [`DiagnosticReport`] as human-readable key-value lines.
pub fn format_human(report: &DiagnosticReport) -> String {
    let mut lines = Vec::new();

    lines.push(format!("ECC {}", report.ecc_version));

    if !report.installed {
        lines.push("ECC not installed".to_owned());
        return lines.join("\n");
    }

    match &report.workflow_phase {
        Some(phase) => {
            lines.push(format!("Phase: {phase}"));
            if let Some(feature) = &report.workflow_feature {
                lines.push(format!("Feature: {feature}"));
            }
            let spec_mark = if report.artifacts.spec { "✓" } else { "✗" };
            let design_mark = if report.artifacts.design { "✓" } else { "✗" };
            let tasks_mark = if report.artifacts.tasks { "✓" } else { "✗" };
            lines.push(format!(
                "Artifacts: spec [{spec_mark}] design [{design_mark}] tasks [{tasks_mark}]"
            ));
        }
        None => {
            lines.push("No active workflow".to_owned());
        }
    }

    let c = &report.component_counts;
    lines.push(format!(
        "Components: {} agents, {} skills, {} commands, {} rules",
        c.agents, c.skills, c.commands, c.rules
    ));
    lines.push(format!("Hooks: {} installed", report.hook_count));
    lines.push(format!("Config: {}", report.config_path));

    lines.join("\n")
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
