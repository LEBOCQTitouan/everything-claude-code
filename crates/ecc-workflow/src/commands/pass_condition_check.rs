use std::path::Path;

use crate::output::WorkflowOutput;

/// Run the `pass-condition-check` subcommand.
///
/// Only runs at "done" phase. Reads `.claude/workflow/implement-done.md`
/// and checks for "## Pass Condition Results" heading and failures.
/// Warns on stderr if the section is missing or any ❌ failures are found,
/// but always exits 0.
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
                "implement-done.md not found; pass condition results cannot be verified",
            );
        }
    };

    if let Some(msg) = check_pass_conditions(&content) {
        return WorkflowOutput::warn(msg);
    }

    WorkflowOutput::pass("")
}

/// Check the "## Pass Condition Results" section for failures.
/// Returns Some(warning message) if issues found, None if all pass.
fn check_pass_conditions(content: &str) -> Option<String> {
    let heading = "## Pass Condition Results";

    let Some(start) = content.find(heading) else {
        return Some(
            "implement-done.md is missing '## Pass Condition Results' section; \
             pass condition results were not recorded"
                .to_owned(),
        );
    };

    let after = &content[start + heading.len()..];
    let section_body = match after.find("\n## ") {
        Some(end) => &after[..end],
        None => after,
    };

    if section_body.contains('❌') {
        return Some(
            "implement-done.md has ❌ failures in '## Pass Condition Results'; \
             not all pass conditions were met"
                .to_owned(),
        );
    }

    None
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
