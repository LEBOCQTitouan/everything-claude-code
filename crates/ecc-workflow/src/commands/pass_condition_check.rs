use std::path::Path;

use crate::io::read_phase;
use crate::output::WorkflowOutput;

/// Run the `pass-condition-check` subcommand.
///
/// Only runs at "done" phase. Reads `.claude/workflow/implement-done.md`
/// and checks for "## Pass Condition Results" heading and failures.
/// Warns on stderr if the section is missing or any ❌ failures are found,
/// but always exits 0.
pub fn run(state_dir: &Path) -> WorkflowOutput {
    let phase = match read_phase(state_dir) {
        Some(p) => p,
        None => return WorkflowOutput::pass(""),
    };

    if phase != "done" {
        return WorkflowOutput::pass("");
    }

    let implement_done_path = state_dir.join("implement-done.md");
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::Status;
    use tempfile::TempDir;

    /// PC-020: pass_condition_check reads implement-done.md from state_dir (AC-004.5)
    #[test]
    fn pass_condition_reads_from_state_dir() {
        let tmp = TempDir::new().unwrap();
        let custom_state_dir = tmp.path().join(".git/ecc-workflow");
        std::fs::create_dir_all(&custom_state_dir).unwrap();

        // Write done phase state to custom state_dir
        let state_json = r#"{"phase":"done","concern":"dev","feature":"test","started_at":"2026-01-01T00:00:00Z","toolchain":{"test":null,"lint":null,"build":null},"artifacts":{"plan":null,"solution":null,"implement":null,"campaign_path":null,"spec_path":null,"design_path":null,"tasks_path":null},"completed":[],"version":1}"#;
        std::fs::write(custom_state_dir.join("state.json"), state_json).unwrap();

        // Write implement-done.md with passing results at custom state_dir
        let implement_done = "## Pass Condition Results\n\n- PC-001: passed\n";
        std::fs::write(custom_state_dir.join("implement-done.md"), implement_done).unwrap();

        // Run pass_condition_check with custom state_dir
        let output = run(&custom_state_dir);
        assert!(
            matches!(output.status, Status::Pass),
            "pass_condition_check should pass when implement-done.md is at custom state_dir, got: {:?}: {}",
            output.status,
            output.message
        );

        // Verify .claude/workflow/ was NOT consulted
        assert!(
            !tmp.path()
                .join(".claude/workflow/implement-done.md")
                .exists(),
            "implement-done.md must NOT be at .claude/workflow/ when using custom state_dir"
        );
    }
}
