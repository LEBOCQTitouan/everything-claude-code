//! `doc-level-check` subcommand.
//!
//! Warns when key documentation files exceed recommended size limits.
//! Always exits 0 — informational/warning only.
//!
//! Logic:
//! - No state.json                    → silent (exit 0)
//! - Phase is not "done"              → silent (exit 0)
//! - Phase is "done"                  → check file sizes:
//!   - CLAUDE.md > 200 lines          → warn
//!   - README.md > 300 lines          → warn
//!   - ARCHITECTURE.md code blocks    → warn if any block > 20 lines

use std::path::Path;

use ecc_domain::workflow::phase::Phase;

use crate::output::WorkflowOutput;

/// Count the number of lines in a file.  Returns `None` if the file does not exist.
fn count_lines(path: &Path) -> Option<usize> {
    let content = std::fs::read_to_string(path).ok()?;
    Some(content.lines().count())
}

/// Find the longest fenced code block (``` ... ```) in a file.
///
/// Returns the line count of the longest code block found, or 0 if none.
fn longest_code_block(path: &Path) -> usize {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return 0,
    };

    let mut max_lines: usize = 0;
    let mut in_block = false;
    let mut current_count: usize = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            if in_block {
                // End of block
                if current_count > max_lines {
                    max_lines = current_count;
                }
                in_block = false;
                current_count = 0;
            } else {
                // Start of block
                in_block = true;
                current_count = 0;
            }
        } else if in_block {
            current_count += 1;
        }
    }

    max_lines
}

/// Run the `doc-level-check` subcommand.
pub fn run(project_dir: &Path, state_dir: &Path) -> WorkflowOutput {
    let state = match crate::io::read_state(state_dir) {
        Ok(None) => return WorkflowOutput::pass(""),
        Ok(Some(s)) => s,
        Err(_) => return WorkflowOutput::pass(""),
    };

    // Only run at "done" phase.
    if state.phase != Phase::Done {
        return WorkflowOutput::pass("");
    }

    let mut warnings: Vec<String> = Vec::new();

    // Check CLAUDE.md > 200 lines
    let claude_path = project_dir.join("CLAUDE.md");
    if let Some(lines) = count_lines(&claude_path)
        && lines > 200
    {
        warnings.push(format!(
            "WARNING: CLAUDE.md has {lines} lines (limit: 200). \
             Consider splitting into focused sections."
        ));
    }

    // Check README.md > 300 lines
    let readme_path = project_dir.join("README.md");
    if let Some(lines) = count_lines(&readme_path)
        && lines > 300
    {
        warnings.push(format!(
            "WARNING: README.md has {lines} lines (limit: 300). \
             Consider moving details to dedicated docs."
        ));
    }

    // Check ARCHITECTURE.md — code blocks > 20 lines
    let arch_path = project_dir.join("ARCHITECTURE.md");
    let max_block = longest_code_block(&arch_path);
    if max_block > 20 {
        warnings.push(format!(
            "WARNING: ARCHITECTURE.md contains a code block with {max_block} lines (limit: 20). \
             Consider extracting large examples to separate files."
        ));
    }

    if warnings.is_empty() {
        WorkflowOutput::pass("")
    } else {
        WorkflowOutput::warn(warnings.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_lines_returns_none_for_missing_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("nonexistent.md");
        assert_eq!(count_lines(&path), None);
    }

    #[test]
    fn count_lines_counts_correctly() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("test.md");
        std::fs::write(&path, "line1\nline2\nline3\n").unwrap();
        assert_eq!(count_lines(&path), Some(3));
    }

    #[test]
    fn longest_code_block_returns_zero_for_missing_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("nonexistent.md");
        assert_eq!(longest_code_block(&path), 0);
    }

    #[test]
    fn longest_code_block_finds_block_size() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("arch.md");
        let content = "# Doc\n```\nline1\nline2\nline3\n```\n";
        std::fs::write(&path, content).unwrap();
        assert_eq!(longest_code_block(&path), 3);
    }

    #[test]
    fn longest_code_block_returns_max_of_multiple_blocks() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("arch.md");
        let content = "```\na\nb\n```\n\n```\nx\ny\nz\nw\n```\n";
        std::fs::write(&path, content).unwrap();
        assert_eq!(longest_code_block(&path), 4);
    }
}
