use std::path::Path;

use crate::io::read_phase;
use crate::output::WorkflowOutput;

/// Run the `doc-enforcement` subcommand.
///
/// Only runs at "done" phase. Reads `.claude/workflow/implement-done.md`
/// and checks for required documentation sections. Warns on stderr if
/// sections are missing, but always exits 0.
///
/// Required sections:
/// - `## Docs Updated` with at least one list item or table row
/// - `## Supplemental Docs`
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
                "implement-done.md not found; please fill in doc sections",
            );
        }
    };

    if let Some(msg) = check_sections(&content) {
        return WorkflowOutput::warn(msg);
    }

    WorkflowOutput::pass("")
}

/// Check that implement-done.md contains the required sections.
/// Returns Some(warning message) if a section is missing, None if all present.
fn check_sections(content: &str) -> Option<String> {
    let has_docs_updated = has_section_with_content(content, "## Docs Updated");
    let has_supplemental = content.contains("## Supplemental Docs");

    match (has_docs_updated, has_supplemental) {
        (false, false) => Some(
            "implement-done.md is missing '## Docs Updated' and '## Supplemental Docs' sections"
                .to_owned(),
        ),
        (false, true) => Some(
            "implement-done.md is missing '## Docs Updated' section (with at least one list item)"
                .to_owned(),
        ),
        (true, false) => {
            Some("implement-done.md is missing '## Supplemental Docs' section".to_owned())
        }
        (true, true) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::Status;
    use tempfile::TempDir;

    /// PC-019: doc_enforcement reads implement-done.md from state_dir (AC-004.5)
    #[test]
    fn doc_enforcement_reads_from_state_dir() {
        let tmp = TempDir::new().unwrap();
        let custom_state_dir = tmp.path().join(".git/ecc-workflow");
        std::fs::create_dir_all(&custom_state_dir).unwrap();

        // Write done phase state to custom state_dir
        let state_json = r#"{"phase":"done","concern":"dev","feature":"test","started_at":"2026-01-01T00:00:00Z","toolchain":{"test":null,"lint":null,"build":null},"artifacts":{"plan":null,"solution":null,"implement":null,"campaign_path":null,"spec_path":null,"design_path":null,"tasks_path":null},"completed":[],"version":1}"#;
        std::fs::write(custom_state_dir.join("state.json"), state_json).unwrap();

        // Write implement-done.md at custom state_dir
        let implement_done = "## Docs Updated\n\n- updated CLAUDE.md\n\n## Supplemental Docs\n\n(none)\n";
        std::fs::write(custom_state_dir.join("implement-done.md"), implement_done).unwrap();

        // Run doc_enforcement with custom state_dir
        let output = run(&custom_state_dir);
        assert!(
            matches!(output.status, Status::Pass),
            "doc_enforcement should pass when implement-done.md is at custom state_dir, got: {:?}: {}",
            output.status,
            output.message
        );

        // Verify .claude/workflow/ was NOT consulted
        assert!(
            !tmp.path().join(".claude/workflow/implement-done.md").exists(),
            "implement-done.md must NOT exist at .claude/workflow/ when using custom state_dir"
        );
    }
}

/// Returns true if the section heading exists and is followed by at least one
/// list item (`- `) or table row (`|`) before the next heading or end of file.
fn has_section_with_content(content: &str, heading: &str) -> bool {
    let Some(start) = content.find(heading) else {
        return false;
    };
    let after = &content[start + heading.len()..];
    // Look for the next heading or use entire remainder
    let section_body = match after.find("\n## ") {
        Some(end) => &after[..end],
        None => after,
    };
    section_body
        .lines()
        .any(|line| line.trim_start().starts_with("- ") || line.trim_start().starts_with('|'))
}
