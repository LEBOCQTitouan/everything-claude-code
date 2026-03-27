use std::io::Read;
use std::path::Path;

use crate::output::WorkflowOutput;

/// Allowed path prefixes during plan/solution phases.
/// A file_path is allowed if it starts with any of these prefixes
/// (both relative and prefixed with an arbitrary parent directory).
const ALLOWED_PREFIXES: &[&str] = &[
    ".claude/workflow/",
    ".claude/plans/",
    "docs/audits/",
    "docs/backlog/",
    "docs/user-stories/",
    "docs/specs/",
    "docs/plans/",
    "docs/designs/",
    "docs/adr/",
];

/// Destructive Bash command patterns that are blocked during plan/solution phases.
const BLOCKED_BASH_PATTERNS: &[&str] = &[
    "rm -rf",
    "git reset --hard",
    "git clean",
    "git checkout --",
    "cargo publish",
];

/// Run the `phase-gate` subcommand.
///
/// Reads the hook protocol JSON from stdin:
///   `{"tool_name":"Write","tool_input":{"file_path":"src/main.rs"}}`
///
/// Exit behavior (mirrors phase-gate.sh):
/// - No state.json → exit 0 (pass)
/// - phase is implement or done → exit 0 (pass)
/// - Write/Edit/MultiEdit to allowed path → exit 0 (pass)
/// - Write/Edit/MultiEdit to blocked path → exit 2 (block)
/// - Bash with destructive command → exit 2 (block)
/// - All other tools → exit 0 (pass)
pub fn run(project_dir: &Path) -> WorkflowOutput {
    // Read the workflow state; if missing, allow everything.
    let phase = match read_phase(project_dir) {
        PhaseResult::Missing => return WorkflowOutput::pass("No workflow active"),
        PhaseResult::ReadError(e) => {
            return WorkflowOutput::pass(format!("Could not read state.json: {e}"))
        }
        PhaseResult::Phase(p) => p,
    };

    // implement and done phases — no gating
    if phase == "implement" || phase == "done" {
        return WorkflowOutput::pass(format!("Phase {phase}: no gating"));
    }

    // Read stdin JSON from the hooks runtime
    let input = read_stdin();
    let (tool_name, file_path, command) = parse_hook_input(&input);

    match tool_name.as_deref() {
        Some("Write") | Some("Edit") | Some("MultiEdit") => {
            let fp = file_path.as_deref().unwrap_or("");
            if is_allowed_path(fp) {
                WorkflowOutput::pass(format!("Write to allowed path '{fp}' permitted"))
            } else {
                WorkflowOutput::block(format!(
                    "BLOCKED: Cannot write to '{fp}' during {phase} phase. \
                     Only workflow and docs paths are allowed."
                ))
            }
        }
        Some("Bash") => {
            let cmd = command.as_deref().unwrap_or("");
            if is_destructive_bash(cmd) {
                WorkflowOutput::block(format!(
                    "BLOCKED: Destructive command not allowed during {phase} phase. \
                     Command: {cmd}"
                ))
            } else {
                WorkflowOutput::pass("Bash command permitted")
            }
        }
        _ => WorkflowOutput::pass("Tool permitted"),
    }
}

enum PhaseResult {
    Missing,
    ReadError(String),
    Phase(String),
}

fn read_phase(project_dir: &Path) -> PhaseResult {
    let state_path = project_dir.join(".claude/workflow/state.json");
    if !state_path.exists() {
        return PhaseResult::Missing;
    }
    let content = match std::fs::read_to_string(&state_path) {
        Ok(c) => c,
        Err(e) => return PhaseResult::ReadError(e.to_string()),
    };
    let v: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => return PhaseResult::ReadError(e.to_string()),
    };
    let phase = v
        .get("phase")
        .and_then(|p| p.as_str())
        .unwrap_or("done")
        .to_owned();
    PhaseResult::Phase(phase)
}

fn read_stdin() -> String {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf).unwrap_or(0);
    buf
}

fn parse_hook_input(input: &str) -> (Option<String>, Option<String>, Option<String>) {
    let v: serde_json::Value = match serde_json::from_str(input) {
        Ok(v) => v,
        Err(_) => return (None, None, None),
    };
    let tool_name = v
        .get("tool_name")
        .and_then(|t| t.as_str())
        .map(|s| s.to_owned());
    let file_path = v
        .pointer("/tool_input/file_path")
        .and_then(|p| p.as_str())
        .map(|s| s.to_owned());
    let command = v
        .pointer("/tool_input/command")
        .and_then(|c| c.as_str())
        .map(|s| s.to_owned());
    (tool_name, file_path, command)
}

fn is_allowed_path(path: &str) -> bool {
    for prefix in ALLOWED_PREFIXES {
        if path.starts_with(prefix) || path.contains(&format!("/{prefix}")) {
            return true;
        }
        // Also allow exact match without trailing slash for directory itself
        let prefix_no_slash = prefix.trim_end_matches('/');
        if path == prefix_no_slash || path.ends_with(&format!("/{prefix_no_slash}")) {
            return true;
        }
    }
    false
}

fn is_destructive_bash(command: &str) -> bool {
    BLOCKED_BASH_PATTERNS
        .iter()
        .any(|pattern| command.contains(pattern))
}
