//! Status use case — reads ECC runtime state for `ecc status`.
//!
//! Returns version info, active workflow, component counts from manifest,
//! and artifact status from `.claude/workflow/state.json`.

use ecc_ports::env::Environment;
use ecc_ports::fs::FileSystem;
use ecc_ports::shell::ShellExecutor;

/// ECC runtime status snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EccStatus {
    /// ECC binary version (from `CARGO_PKG_VERSION`).
    pub ecc_version: String,
    /// `ecc-workflow` binary version, or `None` when the binary is not found.
    pub workflow_version: Option<String>,
    /// Active workflow info, or `None` when no state.json exists.
    pub workflow: Option<WorkflowInfo>,
    /// Component counts from `~/.claude/.ecc-manifest.json`.
    pub components: ComponentCounts,
    /// Artifact presence flags from state.json.
    pub artifacts: ArtifactStatus,
}

/// Active workflow summary from state.json.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkflowInfo {
    pub phase: String,
    pub feature: String,
    pub started_at: String,
}

/// Counts of installed ECC components from the manifest.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ComponentCounts {
    pub agents: usize,
    pub skills: usize,
    pub commands: usize,
    pub rules: usize,
    pub hooks: usize,
}


/// Which spec-driven artifacts are present in the active workflow.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArtifactStatus {
    pub spec: bool,
    pub design: bool,
    pub tasks: bool,
}

/// Collect the ECC runtime status.
///
/// - `ecc_version`: compile-time `CARGO_PKG_VERSION`
/// - `workflow_version`: `ecc-workflow --version` stdout; `None` on failure
/// - `workflow`: parsed from `<cwd>/.claude/workflow/state.json`; `None` if absent
/// - `components`: counts from `<home>/.claude/.ecc-manifest.json`; zeros if absent
/// - `artifacts`: spec/design/tasks presence from state.json; all false if absent
pub fn ecc_status(fs: &dyn FileSystem, env: &dyn Environment, shell: &dyn ShellExecutor) -> EccStatus {
    let ecc_version = crate::version::version();

    let workflow_version = read_workflow_version(shell);

    let (workflow, artifacts) = read_workflow_state(fs, env);

    let components = read_component_counts(fs, env);

    EccStatus {
        ecc_version,
        workflow_version,
        workflow,
        components,
        artifacts,
    }
}

/// Run `ecc-workflow --version`, return trimmed stdout or `None` on any failure.
fn read_workflow_version(shell: &dyn ShellExecutor) -> Option<String> {
    let output = shell.run_command("ecc-workflow", &["--version"]).ok()?;
    if output.success() {
        let trimmed = output.stdout.trim().to_owned();
        if trimmed.is_empty() { None } else { Some(trimmed) }
    } else {
        None
    }
}

/// Read `.claude/workflow/state.json` from the current working directory.
///
/// Returns `(Some(WorkflowInfo), ArtifactStatus)` when the file exists and
/// parses successfully, `(None, ArtifactStatus::default())` otherwise.
fn read_workflow_state(
    fs: &dyn FileSystem,
    env: &dyn Environment,
) -> (Option<WorkflowInfo>, ArtifactStatus) {
    let Some(cwd) = env.current_dir() else {
        return (None, ArtifactStatus::default());
    };

    let state_path = cwd.join(".claude/workflow/state.json");

    let content = match fs.read_to_string(&state_path) {
        Ok(c) => c,
        Err(_) => return (None, ArtifactStatus::default()),
    };

    let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
        return (None, ArtifactStatus::default());
    };

    let phase = value
        .get("phase")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_owned();

    let feature = value
        .get("feature")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_owned();

    let started_at = value
        .get("started_at")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_owned();

    let workflow_info = WorkflowInfo { phase, feature, started_at };

    let artifacts = parse_artifact_status(&value);

    (Some(workflow_info), artifacts)
}

/// Extract artifact presence from the state.json `artifacts` object.
fn parse_artifact_status(value: &serde_json::Value) -> ArtifactStatus {
    let artifacts_obj = match value.get("artifacts") {
        Some(a) => a,
        None => return ArtifactStatus::default(),
    };

    let spec = artifacts_obj
        .get("spec_path")
        .map(|v| !v.is_null())
        .unwrap_or(false);

    let design = artifacts_obj
        .get("design_path")
        .map(|v| !v.is_null())
        .unwrap_or(false);

    let tasks = artifacts_obj
        .get("tasks_path")
        .map(|v| !v.is_null())
        .unwrap_or(false);

    ArtifactStatus { spec, design, tasks }
}

/// Read component counts from `<home>/.claude/.ecc-manifest.json`.
fn read_component_counts(fs: &dyn FileSystem, env: &dyn Environment) -> ComponentCounts {
    let Some(home) = env.home_dir() else {
        return ComponentCounts::default();
    };

    let manifest_path = home.join(".claude/.ecc-manifest.json");

    let content = match fs.read_to_string(&manifest_path) {
        Ok(c) => c,
        Err(_) => return ComponentCounts::default(),
    };

    let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
        return ComponentCounts::default();
    };

    let artifacts = match value.get("artifacts") {
        Some(a) => a,
        None => return ComponentCounts::default(),
    };

    let agents = count_array(artifacts, "agents");
    let commands = count_array(artifacts, "commands");
    let skills = count_array(artifacts, "skills");
    let hooks = count_array(artifacts, "hookDescriptions");

    // rules is a map of group -> [files]; total = sum of all vec lengths
    let rules = artifacts
        .get("rules")
        .and_then(|v| v.as_object())
        .map(|m| m.values().map(|v| v.as_array().map_or(0, |a| a.len())).sum())
        .unwrap_or(0);

    ComponentCounts { agents, skills, commands, rules, hooks }
}

fn count_array(obj: &serde_json::Value, key: &str) -> usize {
    obj.get(key)
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::{InMemoryFileSystem, MockEnvironment, MockExecutor};
    use ecc_ports::shell::CommandOutput;

    fn workflow_version_output(version: &str) -> CommandOutput {
        CommandOutput {
            stdout: version.to_owned(),
            stderr: String::new(),
            exit_code: 0,
        }
    }

    fn failed_command_output() -> CommandOutput {
        CommandOutput {
            stdout: String::new(),
            stderr: "not found".to_owned(),
            exit_code: 1,
        }
    }

    fn sample_manifest_json(agents: usize, skills: usize, commands: usize, hooks: usize) -> String {
        let agents_arr: Vec<String> = (0..agents).map(|i| format!("\"agent{i}.md\"")).collect();
        let skills_arr: Vec<String> = (0..skills).map(|i| format!("\"skill{i}\"")).collect();
        let cmds_arr: Vec<String> = (0..commands).map(|i| format!("\"cmd{i}.md\"")).collect();
        let hooks_arr: Vec<String> = (0..hooks).map(|i| format!("\"hook{i}\"")).collect();
        format!(
            r#"{{
  "version": "4.2.0",
  "installedAt": "2026-01-01T00:00:00Z",
  "updatedAt": "2026-01-01T00:00:00Z",
  "languages": [],
  "artifacts": {{
    "agents": [{agents}],
    "commands": [{cmds}],
    "skills": [{skills}],
    "rules": {{}},
    "hookDescriptions": [{hooks}]
  }}
}}"#,
            agents = agents_arr.join(", "),
            cmds = cmds_arr.join(", "),
            skills = skills_arr.join(", "),
            hooks = hooks_arr.join(", "),
        )
    }

    fn sample_state_json(phase: &str, feature: &str, started_at: &str) -> String {
        format!(
            r#"{{
  "phase": "{phase}",
  "concern": "dev",
  "feature": "{feature}",
  "started_at": "{started_at}",
  "toolchain": {{"test": null, "lint": null, "build": null}},
  "artifacts": {{
    "plan": null,
    "solution": null,
    "implement": null,
    "campaign_path": null,
    "spec_path": "docs/specs/my-feature/spec.md",
    "design_path": "docs/specs/my-feature/design.md",
    "tasks_path": null
  }},
  "completed": []
}}"#
        )
    }

    // PC-010: ecc_status returns ecc_version (non-empty string from CARGO_PKG_VERSION)
    #[test]
    fn ecc_status_returns_non_empty_ecc_version() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new()
            .with_home("/home/test")
            .with_current_dir("/project");
        let shell = MockExecutor::new()
            .on("ecc-workflow", workflow_version_output("ecc-workflow 4.2.0\n"));

        let status = ecc_status(&fs, &env, &shell);

        assert!(!status.ecc_version.is_empty(), "ecc_version must be non-empty");
    }

    // PC-010: ecc_status returns component counts from manifest
    #[test]
    fn ecc_status_returns_component_counts_from_manifest() {
        let fs = InMemoryFileSystem::new()
            .with_file("/home/test/.claude/.ecc-manifest.json", &sample_manifest_json(3, 5, 2, 4));
        let env = MockEnvironment::new()
            .with_home("/home/test")
            .with_current_dir("/project");
        let shell = MockExecutor::new()
            .on("ecc-workflow", workflow_version_output("ecc-workflow 4.2.0\n"));

        let status = ecc_status(&fs, &env, &shell);

        assert_eq!(status.components.agents, 3);
        assert_eq!(status.components.skills, 5);
        assert_eq!(status.components.commands, 2);
        assert_eq!(status.components.hooks, 4);
        assert_eq!(status.components.rules, 0);
    }

    // PC-010: ecc_status returns workflow_version from shell executor
    #[test]
    fn ecc_status_returns_workflow_version_from_shell() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new()
            .with_home("/home/test")
            .with_current_dir("/project");
        let shell = MockExecutor::new()
            .on("ecc-workflow", workflow_version_output("ecc-workflow 4.2.0\n"));

        let status = ecc_status(&fs, &env, &shell);

        assert_eq!(status.workflow_version, Some("ecc-workflow 4.2.0".to_owned()));
    }

    // PC-011: ecc_status shows active workflow info when state.json exists
    #[test]
    fn ecc_status_shows_active_workflow_when_state_json_exists() {
        let state = sample_state_json("implement", "BL-091-diagnostics", "2026-03-30T10:00:00Z");
        let fs = InMemoryFileSystem::new()
            .with_file("/project/.claude/workflow/state.json", &state);
        let env = MockEnvironment::new()
            .with_home("/home/test")
            .with_current_dir("/project");
        let shell = MockExecutor::new()
            .on("ecc-workflow", workflow_version_output("ecc-workflow 4.2.0\n"));

        let status = ecc_status(&fs, &env, &shell);

        let workflow = status.workflow.expect("workflow should be Some when state.json exists");
        assert_eq!(workflow.phase, "implement");
        assert_eq!(workflow.feature, "BL-091-diagnostics");
        assert_eq!(workflow.started_at, "2026-03-30T10:00:00Z");
    }

    // PC-011: ecc_status shows artifact status from state.json
    #[test]
    fn ecc_status_shows_artifact_status_from_state_json() {
        let state = sample_state_json("implement", "BL-091-diagnostics", "2026-03-30T10:00:00Z");
        let fs = InMemoryFileSystem::new()
            .with_file("/project/.claude/workflow/state.json", &state);
        let env = MockEnvironment::new()
            .with_home("/home/test")
            .with_current_dir("/project");
        let shell = MockExecutor::new()
            .on("ecc-workflow", workflow_version_output("ecc-workflow 4.2.0\n"));

        let status = ecc_status(&fs, &env, &shell);

        // state.json has spec_path and design_path set, tasks_path is null
        assert!(status.artifacts.spec, "spec should be present");
        assert!(status.artifacts.design, "design should be present");
        assert!(!status.artifacts.tasks, "tasks should not be present");
    }

    // PC-012: ecc_status shows no workflow when state.json absent
    #[test]
    fn ecc_status_shows_no_workflow_when_state_json_absent() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new()
            .with_home("/home/test")
            .with_current_dir("/project");
        let shell = MockExecutor::new()
            .on("ecc-workflow", workflow_version_output("ecc-workflow 4.2.0\n"));

        let status = ecc_status(&fs, &env, &shell);

        assert!(status.workflow.is_none(), "workflow should be None when state.json absent");
    }

    // PC-012: artifact status is all false when state.json absent
    #[test]
    fn ecc_status_artifact_status_is_all_false_when_no_state_json() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new()
            .with_home("/home/test")
            .with_current_dir("/project");
        let shell = MockExecutor::new()
            .on("ecc-workflow", workflow_version_output("ecc-workflow 4.2.0\n"));

        let status = ecc_status(&fs, &env, &shell);

        assert_eq!(status.artifacts, ArtifactStatus::default());
    }

    // PC-013: ecc_status handles missing ecc-workflow binary gracefully
    #[test]
    fn ecc_status_handles_missing_ecc_workflow_binary() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new()
            .with_home("/home/test")
            .with_current_dir("/project");
        // MockExecutor with no "ecc-workflow" registration returns NotFound error
        let shell = MockExecutor::new();

        let status = ecc_status(&fs, &env, &shell);

        assert_eq!(
            status.workflow_version,
            None,
            "workflow_version must be None when ecc-workflow binary is missing"
        );
    }

    // PC-013: ecc_status does not panic when ecc-workflow returns failure exit code
    #[test]
    fn ecc_status_handles_failed_ecc_workflow_command() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new()
            .with_home("/home/test")
            .with_current_dir("/project");
        let shell = MockExecutor::new()
            .on("ecc-workflow", failed_command_output());

        let status = ecc_status(&fs, &env, &shell);

        assert_eq!(
            status.workflow_version,
            None,
            "workflow_version must be None when ecc-workflow exits with non-zero"
        );
    }

    // PC-010: component counts are zeros when manifest is absent
    #[test]
    fn ecc_status_returns_zero_counts_when_manifest_absent() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new()
            .with_home("/home/test")
            .with_current_dir("/project");
        let shell = MockExecutor::new()
            .on("ecc-workflow", workflow_version_output("ecc-workflow 4.2.0\n"));

        let status = ecc_status(&fs, &env, &shell);

        assert_eq!(status.components, ComponentCounts::default());
    }

    // PC-010: rules count sums all rule groups
    #[test]
    fn ecc_status_counts_rules_across_all_groups() {
        let manifest = r#"{
  "version": "4.2.0",
  "installedAt": "2026-01-01T00:00:00Z",
  "updatedAt": "2026-01-01T00:00:00Z",
  "languages": [],
  "artifacts": {
    "agents": [],
    "commands": [],
    "skills": [],
    "rules": {
      "common": ["rule1.md", "rule2.md"],
      "rust": ["coding-style.md"]
    },
    "hookDescriptions": []
  }
}"#;
        let fs = InMemoryFileSystem::new()
            .with_file("/home/test/.claude/.ecc-manifest.json", manifest);
        let env = MockEnvironment::new()
            .with_home("/home/test")
            .with_current_dir("/project");
        let shell = MockExecutor::new()
            .on("ecc-workflow", workflow_version_output("ecc-workflow 4.2.0\n"));

        let status = ecc_status(&fs, &env, &shell);

        assert_eq!(status.components.rules, 3, "2 common + 1 rust = 3 total rules");
    }
}
