use std::io::Read;
use std::path::Path;

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
pub fn run(project_dir: &Path) -> WorkflowOutput {
    let phase = match read_phase(project_dir) {
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
                write_tdd_state(project_dir, "RED");
                WorkflowOutput::pass("TDD state: RED")
            } else {
                let current = read_tdd_state(project_dir);
                if current.as_deref() == Some("RED") {
                    write_tdd_state(project_dir, "GREEN");
                    WorkflowOutput::pass("TDD state: GREEN")
                } else {
                    WorkflowOutput::pass("TDD state unchanged")
                }
            }
        }
        _ => WorkflowOutput::pass(""),
    }
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

fn read_stdin() -> String {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf).unwrap_or(0);
    buf
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

fn tdd_state_path(project_dir: &Path) -> std::path::PathBuf {
    project_dir.join(".claude/workflow/.tdd-state")
}

fn read_tdd_state(project_dir: &Path) -> Option<String> {
    std::fs::read_to_string(tdd_state_path(project_dir)).ok()
}

fn write_tdd_state(project_dir: &Path, state: &str) {
    let path = tdd_state_path(project_dir);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    // Atomic write via tempfile + rename
    let tmp_path = path.with_extension("tmp");
    if std::fs::write(&tmp_path, state).is_ok() {
        let _ = std::fs::rename(&tmp_path, &path);
    }
}
