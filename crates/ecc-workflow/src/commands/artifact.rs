//! `ecc-workflow artifact <type>` — resolve and validate artifact paths.

use crate::io;
use crate::output::WorkflowOutput;
use std::path::Path;

pub fn run(artifact_type: &str, project_dir: &Path) -> WorkflowOutput {
    let state = match io::read_state(project_dir) {
        Ok(Some(s)) => s,
        Ok(None) => return WorkflowOutput::block("No active workflow"),
        Err(e) => return WorkflowOutput::block(format!("Failed to read state: {e}")),
    };

    let path = match artifact_type {
        "spec" => state.artifacts.spec_path.as_deref(),
        "design" => state.artifacts.design_path.as_deref(),
        "tasks" => state.artifacts.tasks_path.as_deref(),
        "campaign" => state.artifacts.campaign_path.as_deref(),
        other => {
            return WorkflowOutput::block(format!(
                "Unknown artifact type: {other}. Supported: spec, design, tasks, campaign"
            ));
        }
    };

    match path {
        None => WorkflowOutput::block(format!("no {artifact_type} artifact registered")),
        Some(p) => {
            let full_path = project_dir.join(p);
            if full_path.exists() {
                WorkflowOutput::pass(p.to_string())
            } else {
                WorkflowOutput::block(format!("artifact file not found: {p}"))
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn setup_state(dir: &Path, spec_path: Option<&str>) {
        let wf_dir = dir.join(".claude/workflow");
        std::fs::create_dir_all(&wf_dir).unwrap();
        let spec = spec_path
            .map(|p| format!(r#""spec_path":"{p}""#))
            .unwrap_or_else(|| r#""spec_path":null"#.to_string());
        std::fs::write(
            wf_dir.join("state.json"),
            format!(
                r#"{{"phase":"plan","concern":"dev","feature":"test","started_at":"2026-01-01T00:00:00Z","toolchain":{{"test":null,"lint":null,"build":null}},"artifacts":{{"plan":null,"solution":null,"implement":null,"campaign_path":null,{spec},"design_path":null,"tasks_path":null}},"completed":[]}}"#
            ),
        ).unwrap();
    }

    #[test]
    fn artifact_valid_spec() {
        let dir = tempfile::tempdir().unwrap();
        setup_state(dir.path(), Some("docs/specs/test/spec.md"));
        // Create the artifact file
        let spec_dir = dir.path().join("docs/specs/test");
        std::fs::create_dir_all(&spec_dir).unwrap();
        std::fs::write(spec_dir.join("spec.md"), "# Spec").unwrap();

        let output = run("spec", dir.path());
        assert_eq!(output.message, "docs/specs/test/spec.md");
    }

    #[test]
    fn artifact_file_missing() {
        let dir = tempfile::tempdir().unwrap();
        setup_state(dir.path(), Some("docs/specs/test/spec.md"));
        // Don't create the file
        let output = run("spec", dir.path());
        assert!(output.message.contains("artifact file not found"));
    }

    #[test]
    fn artifact_path_null() {
        let dir = tempfile::tempdir().unwrap();
        setup_state(dir.path(), None);
        let output = run("spec", dir.path());
        assert!(output.message.contains("no spec artifact registered"));
    }

    #[test]
    fn artifact_all_types() {
        let dir = tempfile::tempdir().unwrap();
        setup_state(dir.path(), None);
        // All 4 types should be recognized (even if null)
        for t in &["spec", "design", "tasks", "campaign"] {
            let output = run(t, dir.path());
            assert!(
                output.message.contains("no") || output.message.contains("artifact"),
                "type {t} should be recognized"
            );
        }
        // Unknown type should error
        let output = run("unknown", dir.path());
        assert!(output.message.contains("Unknown artifact type"));
    }
}
