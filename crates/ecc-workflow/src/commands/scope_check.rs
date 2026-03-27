//! `scope-check` subcommand.
//!
//! Compares the git diff against the expected files listed in the design's
//! File Changes table.  Always exits 0 — informational/warning only.
//!
//! Logic:
//! - No state.json                    → silent (exit 0)
//! - Phase is plan or solution        → silent (exit 0)
//! - Phase is implement or done       → perform the check
//!   - No design_path in artifacts    → warn: "WARNING: No design file path in state"
//!   - design_path but file missing   → warn: "WARNING: Design file not found at <path>"
//!   - design_path present            → extract expected files, compare with git diff,
//!     warn about unexpected files (if any)

use std::path::Path;
use std::process::Command;

use ecc_domain::workflow::phase::Phase;

use crate::output::WorkflowOutput;

/// Extract file paths listed in the "File Changes" table of a design document.
///
/// Looks for a markdown table after a "File Changes" heading and extracts
/// the first column of each data row.  Ignores header and separator rows.
fn extract_expected_files(design_content: &str) -> Vec<String> {
    let mut in_table = false;
    let mut files: Vec<String> = Vec::new();

    for line in design_content.lines() {
        let trimmed = line.trim();
        // Detect "File Changes" section heading
        if trimmed.to_lowercase().contains("file changes") {
            in_table = true;
            continue;
        }
        if !in_table {
            continue;
        }
        // Stop at next heading
        if trimmed.starts_with('#') {
            break;
        }
        // Skip non-table lines
        if !trimmed.starts_with('|') {
            // A blank line after the table ends it
            if trimmed.is_empty() && !files.is_empty() {
                // keep going — table may resume
            }
            continue;
        }
        // Skip separator rows (|---|---|)
        if trimmed.chars().all(|c| c == '-' || c == '|' || c == ' ') {
            continue;
        }
        // Extract first column
        let cols: Vec<&str> = trimmed
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        if let Some(first) = cols.first() {
            let path = first.trim();
            // Skip header rows (case-insensitive check for column headers)
            if path.to_lowercase() == "file"
                || path.to_lowercase() == "path"
                || path.is_empty()
            {
                continue;
            }
            files.push(path.to_string());
        }
    }

    files
}

/// Get changed files from git diff (uncommitted changes + staged changes).
///
/// Uses `git diff --name-only HEAD` to capture all changes since the last commit.
/// Falls back to an empty list on error (e.g., not a git repo).
fn git_changed_files(project_dir: &Path) -> Vec<String> {
    // Try staged + unstaged changes relative to HEAD
    let output = Command::new("git")
        .args(["diff", "--name-only", "HEAD"])
        .current_dir(project_dir)
        .output();

    let mut files: Vec<String> = match output {
        Ok(o) if o.status.success() => std::str::from_utf8(&o.stdout)
            .unwrap_or("")
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(String::from)
            .collect(),
        _ => Vec::new(),
    };

    // Also capture untracked files via `git ls-files --others --exclude-standard`
    let untracked = Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard"])
        .current_dir(project_dir)
        .output();

    if let Ok(o) = untracked && o.status.success() {
        files.extend(
            std::str::from_utf8(&o.stdout)
                .unwrap_or("")
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(String::from),
        );
    }

    files
}

/// Return true if `path` should be excluded from scope-check comparisons.
///
/// Excluded patterns (matching the design spec):
/// - docs/*
/// - .claude/workflow/*
/// - CHANGELOG.md
/// - test files (*_test.*, *.test.*)
/// - lock files (Cargo.lock, package-lock.json, yarn.lock)
fn is_exception(path: &str) -> bool {
    if path.starts_with("docs/") {
        return true;
    }
    if path.starts_with(".claude/workflow/") {
        return true;
    }
    if path == "CHANGELOG.md" {
        return true;
    }
    if path.ends_with("Cargo.lock")
        || path.ends_with("package-lock.json")
        || path.ends_with("yarn.lock")
    {
        return true;
    }
    // Test files
    let filename = path.split('/').next_back().unwrap_or(path);
    if filename.contains("_test.") || filename.contains(".test.") {
        return true;
    }
    false
}

/// Run the `scope-check` subcommand.
pub fn run(project_dir: &Path) -> WorkflowOutput {
    let state = match crate::io::read_state(project_dir) {
        Ok(None) => return WorkflowOutput::pass(""),
        Ok(Some(s)) => s,
        Err(_) => return WorkflowOutput::pass(""),
    };

    // Only run during implement/done phases.
    match state.phase {
        Phase::Plan | Phase::Solution => return WorkflowOutput::pass(""),
        Phase::Implement | Phase::Done => {}
    }

    // Resolve design_path from artifacts.
    let design_path = match state.artifacts.design_path.as_deref() {
        None | Some("") => {
            return WorkflowOutput::warn(
                "WARNING: scope-check skipped — no design file path recorded in workflow state.",
            );
        }
        Some(p) => p,
    };

    // Read the design file.
    let design_content = match std::fs::read_to_string(design_path) {
        Ok(c) => c,
        Err(_) => {
            return WorkflowOutput::warn(format!(
                "WARNING: scope-check skipped — design file not found at '{design_path}'."
            ));
        }
    };

    // Extract expected paths from the File Changes table.
    let expected: std::collections::HashSet<String> =
        extract_expected_files(&design_content).into_iter().collect();

    // Get actually changed files.
    let changed = git_changed_files(project_dir);

    // Find unexpected files (changed but not in expected, and not an exception).
    let unexpected: Vec<&str> = changed
        .iter()
        .map(String::as_str)
        .filter(|f| !is_exception(f) && !expected.contains(*f))
        .collect();

    if unexpected.is_empty() {
        WorkflowOutput::pass("")
    } else {
        WorkflowOutput::warn(format!(
            "WARNING: scope-check found {} unexpected file(s) not listed in the design's \
             File Changes table: {}",
            unexpected.len(),
            unexpected.join(", ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_files_from_file_changes_table() {
        let content = "# Design\n\n## File Changes\n\n\
                       | File | Action |\n\
                       |------|--------|\n\
                       | src/foo.rs | CREATE |\n\
                       | src/bar.rs | MODIFY |\n";
        let files = extract_expected_files(content);
        assert_eq!(files, vec!["src/foo.rs", "src/bar.rs"]);
    }

    #[test]
    fn ignores_header_and_separator_rows() {
        let content = "## File Changes\n\n\
                       | Path | Description |\n\
                       |------|-------------|\n\
                       | crates/x/src/lib.rs | new module |\n";
        let files = extract_expected_files(content);
        assert_eq!(files, vec!["crates/x/src/lib.rs"]);
    }

    #[test]
    fn returns_empty_when_no_file_changes_section() {
        let content = "# Design\n\nSome content without a table.\n";
        let files = extract_expected_files(content);
        assert!(files.is_empty());
    }

    #[test]
    fn is_exception_docs() {
        assert!(is_exception("docs/specs/foo/design.md"));
    }

    #[test]
    fn is_exception_workflow() {
        assert!(is_exception(".claude/workflow/state.json"));
    }

    #[test]
    fn is_exception_lock_files() {
        assert!(is_exception("Cargo.lock"));
        assert!(is_exception("package-lock.json"));
    }

    #[test]
    fn is_exception_test_files() {
        assert!(is_exception("src/foo_test.rs"));
        assert!(is_exception("src/foo.test.ts"));
    }

    #[test]
    fn non_exception_path_not_excluded() {
        assert!(!is_exception("src/commands/scope_check.rs"));
        assert!(!is_exception("crates/ecc-app/src/lib.rs"));
    }
}
