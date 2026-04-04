use std::path::Path;

use crate::io::{read_phase, read_stdin};
use crate::output::WorkflowOutput;

/// Test file path patterns that indicate a file is a test file.
const TEST_PATTERNS: &[&str] = &[
    "_test.rs",
    "_test.go",
    ".test.ts",
    ".test.js",
    "__tests__/",
    "/tests/",
    "tests/",
    "spec/",
];

/// Run the `tdd-enforcement` subcommand.
///
/// Reads hook protocol JSON from stdin:
///   `{"tool_name":"Write","tool_input":{"file_path":"tests/foo.rs"}}`
///
/// Maintains TDD state at `.claude/workflow/.tdd-state`.
/// Only active during "implement" phase.
/// Always exits 0 — informational logging only.
pub fn run(state_dir: &Path) -> WorkflowOutput {
    let phase = match read_phase(state_dir) {
        Some(p) => p,
        None => return WorkflowOutput::pass(""),
    };

    if phase != "implement" {
        return WorkflowOutput::pass("");
    }

    let input = read_stdin();
    let (tool_name, file_path) = parse_hook_input(&input);

    match tool_name.as_deref() {
        Some("Write") | Some("Edit") | Some("MultiEdit") => {
            let fp = file_path.as_deref().unwrap_or("");
            if is_test_file(fp) {
                write_tdd_state(state_dir, "RED");
                WorkflowOutput::pass("TDD state: RED")
            } else {
                let current = read_tdd_state(state_dir);
                if current.as_deref() == Some("RED") {
                    write_tdd_state(state_dir, "GREEN");
                    WorkflowOutput::pass("TDD state: GREEN")
                } else {
                    WorkflowOutput::pass("TDD state unchanged")
                }
            }
        }
        _ => WorkflowOutput::pass(""),
    }
}

fn parse_hook_input(input: &str) -> (Option<String>, Option<String>) {
    let v: serde_json::Value = match serde_json::from_str(input) {
        Ok(v) => v,
        Err(_) => return (None, None),
    };
    let tool_name = v
        .get("tool_name")
        .and_then(|t| t.as_str())
        .map(|s| s.to_owned());
    let file_path = v
        .pointer("/tool_input/file_path")
        .and_then(|p| p.as_str())
        .map(|s| s.to_owned());
    (tool_name, file_path)
}

fn is_test_file(path: &str) -> bool {
    TEST_PATTERNS
        .iter()
        .any(|pattern| path.ends_with(pattern) || path.contains(pattern))
}

fn tdd_state_path(state_dir: &Path) -> std::path::PathBuf {
    state_dir.join(".tdd-state")
}

fn read_tdd_state(state_dir: &Path) -> Option<String> {
    std::fs::read_to_string(tdd_state_path(state_dir)).ok()
}

fn write_tdd_state(state_dir: &Path, state: &str) {
    let path = tdd_state_path(state_dir);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    // Atomic write via tempfile + rename
    let tmp_path = path.with_extension("tmp");
    if std::fs::write(&tmp_path, state).is_ok() {
        let _ = std::fs::rename(&tmp_path, &path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// PC-018: tdd_enforcement reads .tdd-state from state_dir (AC-004.5)
    #[test]
    fn tdd_reads_from_state_dir() {
        let tmp = TempDir::new().unwrap();
        let custom_state_dir = tmp.path().join(".git/ecc-workflow");
        std::fs::create_dir_all(&custom_state_dir).unwrap();

        // Write implement phase state to custom state_dir
        let state_json = r#"{"phase":"implement","concern":"dev","feature":"test","started_at":"2026-01-01T00:00:00Z","toolchain":{"test":null,"lint":null,"build":null},"artifacts":{"plan":null,"solution":null,"implement":null,"campaign_path":null,"spec_path":null,"design_path":null,"tasks_path":null},"completed":[],"version":1}"#;
        std::fs::write(custom_state_dir.join("state.json"), state_json).unwrap();

        // Pre-write RED state to the custom state_dir
        std::fs::write(custom_state_dir.join(".tdd-state"), "RED").unwrap();

        // Verify the file is read from custom state_dir (not .claude/workflow/)
        let state_file_path = custom_state_dir.join(".tdd-state");
        assert!(
            state_file_path.exists(),
            ".tdd-state must exist at custom state_dir"
        );
        let content = std::fs::read_to_string(&state_file_path).unwrap();
        assert_eq!(content, "RED", ".tdd-state must be readable from custom state_dir");

        // .tdd-state must NOT exist at .claude/workflow/
        let default_path = tmp.path().join(".claude/workflow/.tdd-state");
        assert!(
            !default_path.exists(),
            ".tdd-state must NOT be at .claude/workflow/ when using custom state_dir"
        );

        // tdd_state_path function must resolve under custom state_dir
        let resolved = tdd_state_path(&custom_state_dir);
        assert!(
            resolved.starts_with(&custom_state_dir),
            "tdd_state_path must resolve under custom state_dir, got {:?}",
            resolved
        );
    }
}
