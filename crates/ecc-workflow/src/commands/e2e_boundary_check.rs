use std::path::Path;

use crate::output::WorkflowOutput;

/// Run the `e2e-boundary-check` subcommand.
///
/// Only runs at "done" phase. Reads `.claude/workflow/implement-done.md`
/// and checks for the "## E2E Tests" section. Warns on stderr if the
/// section is missing, but always exits 0.
pub fn run(project_dir: &Path) -> WorkflowOutput {
    let phase = match read_phase(project_dir) {
        Some(p) => p,
        None => return WorkflowOutput::pass(""),
    };

    if phase != "done" {
        return WorkflowOutput::pass("");
    }

    let implement_done_path = project_dir.join(".claude/workflow/implement-done.md");
    let content = match std::fs::read_to_string(&implement_done_path) {
        Ok(c) => c,
        Err(_) => {
            return WorkflowOutput::warn(
                "implement-done.md not found; add an '## E2E Tests' section",
            );
        }
    };

    if !content.contains("## E2E Tests") {
        return WorkflowOutput::warn(
            "implement-done.md is missing '## E2E Tests' section; \
             document whether E2E tests were added, updated, or not required",
        );
    }

    WorkflowOutput::pass("")
}

fn read_phase(project_dir: &Path) -> Option<String> {
    let state_path = project_dir.join(".claude/workflow/state.json");
    if !state_path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&state_path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    v.get("phase")
        .and_then(|p| p.as_str())
        .map(|s| s.to_owned())
}
