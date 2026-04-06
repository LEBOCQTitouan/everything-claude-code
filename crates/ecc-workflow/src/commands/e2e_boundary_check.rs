use std::path::Path;

use crate::io::read_phase;
use crate::output::WorkflowOutput;

/// Run the `e2e-boundary-check` subcommand.
///
/// Only runs at "done" phase. Reads `.claude/workflow/implement-done.md`
/// and checks for the "## E2E Tests" section. Warns on stderr if the
/// section is missing, but always exits 0.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::Status;
    use tempfile::TempDir;

    /// PC-021: e2e_boundary_check reads implement-done.md from state_dir (AC-004.5)
    #[test]
    fn e2e_boundary_reads_from_state_dir() {
        let tmp = TempDir::new().unwrap();
        let custom_state_dir = tmp.path().join(".git/ecc-workflow");
        std::fs::create_dir_all(&custom_state_dir).unwrap();

        // Write done phase state to custom state_dir
        let state_json = r#"{"phase":"done","concern":"dev","feature":"test","started_at":"2026-01-01T00:00:00Z","toolchain":{"test":null,"lint":null,"build":null},"artifacts":{"plan":null,"solution":null,"implement":null,"campaign_path":null,"spec_path":null,"design_path":null,"tasks_path":null},"completed":[],"version":1}"#;
        std::fs::write(custom_state_dir.join("state.json"), state_json).unwrap();

        // Write implement-done.md with E2E section at custom state_dir
        let implement_done = "## E2E Tests\n\nNo new E2E tests required.\n";
        std::fs::write(custom_state_dir.join("implement-done.md"), implement_done).unwrap();

        // Run e2e_boundary_check with custom state_dir
        let output = run(&custom_state_dir);
        assert!(
            matches!(output.status, Status::Pass),
            "e2e_boundary_check should pass when implement-done.md is at custom state_dir, got: {:?}: {}",
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
