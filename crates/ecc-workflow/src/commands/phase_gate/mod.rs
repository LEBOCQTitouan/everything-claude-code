//! Phase-gate hook subcommand.
//!
//! Reads Claude Code's hook protocol JSON from stdin and decides whether the
//! tool invocation should be permitted, warned about, or blocked based on the
//! current workflow phase.
//!
//! Exit behavior (mirrors phase-gate.sh):
//! - No state.json → exit 0 (pass)
//! - Corrupt/unparseable state.json → exit 0 (warn) — does NOT block
//! - phase.is_gated() == false (Idle, Implement, Done) → exit 0 (pass)
//! - Write/Edit/MultiEdit to allowed path → exit 0 (pass)
//! - Write/Edit/MultiEdit to blocked path → exit 2 (block)
//! - Bash with destructive command → exit 2 (block)
//! - All other tools → exit 0 (pass)

mod validation;

use std::path::Path;

use ecc_domain::workflow::phase::Phase;
use ecc_domain::workflow::state::WorkflowState;

use crate::io::{read_stdin, with_state_lock};
use crate::output::WorkflowOutput;

use validation::{
    allowed_prefixes, contains_encoded_traversal, is_allowed_path, is_destructive_bash,
    resolve_worktree_state_dir,
};

enum PhaseResult {
    Missing,
    Corrupt(String),
    Parsed(Phase),
}

fn read_phase_typed(state_dir: &Path) -> PhaseResult {
    let state_path = state_dir.join("state.json");
    if !state_path.exists() {
        return PhaseResult::Missing;
    }
    let content = match std::fs::read_to_string(&state_path) {
        Ok(c) => c,
        Err(e) => return PhaseResult::Corrupt(e.to_string()),
    };
    match WorkflowState::from_json(&content) {
        Ok(state) => PhaseResult::Parsed(state.phase),
        Err(e) => PhaseResult::Corrupt(e.to_string()),
    }
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

/// Run the `phase-gate` subcommand.
///
/// Reads the hook protocol JSON from stdin:
///   `{"tool_name":"Write","tool_input":{"file_path":"src/main.rs"}}`
///
/// The workflow phase is read under the state lock so that phase-gate never
/// observes a partially-written state.json during a concurrent transition.
pub fn run(project_dir: &Path, state_dir: &Path) -> WorkflowOutput {
    // Read stdin before acquiring the lock — stdin is not state-dependent.
    let input = read_stdin();
    run_with_input(project_dir, state_dir, &input)
}

/// Testable entry point: same as `run` but accepts hook input directly.
pub fn run_with_input(project_dir: &Path, state_dir: &Path, input: &str) -> WorkflowOutput {
    let _ = project_dir; // kept for future use

    // Parse hook input once — reused for both worktree resolution and gate check.
    let (tool_name, file_path, command) = parse_hook_input(input);

    // BL-131: Override state_dir when the gated file path reveals we're in a worktree.
    let effective_state_dir = file_path
        .as_deref()
        .and_then(resolve_worktree_state_dir)
        .unwrap_or_else(|| state_dir.to_path_buf());
    let state_dir = &effective_state_dir;
    // Fast path: if state.json does not exist, skip locking entirely.
    let state_path = state_dir.join("state.json");
    if !state_path.exists() {
        return WorkflowOutput::pass("No workflow active");
    }

    // Acquire state lock to read the phase atomically with respect to transitions.
    let phase_result = match with_state_lock(state_dir, || read_phase_typed(state_dir)) {
        Ok(r) => r,
        Err(e) => return WorkflowOutput::pass(format!("Could not acquire state lock: {e}")),
    };

    let phase = match phase_result {
        PhaseResult::Missing => return WorkflowOutput::pass("No workflow active"),
        PhaseResult::Corrupt(msg) => {
            return WorkflowOutput::warn(format!("Corrupt state.json: {msg}"));
        }
        PhaseResult::Parsed(p) => p,
    };

    // Non-gated phases (Idle, Implement, Done) — no restrictions
    if !phase.is_gated() {
        return WorkflowOutput::pass(format!("Phase {phase}: no gating"));
    }

    tracing::info!(phase = %phase, "phase-gate: evaluating gate for current phase");
    let phase_str = phase.to_string();

    let prefixes = allowed_prefixes(state_dir);
    let result = match tool_name.as_deref() {
        Some("Write") | Some("Edit") | Some("MultiEdit") => {
            let raw_fp = file_path.as_deref().unwrap_or("");
            // SEC-010: block URL-encoded traversal before normalization
            if contains_encoded_traversal(raw_fp) {
                return WorkflowOutput::block(format!(
                    "BLOCKED: URL-encoded traversal detected in path '{raw_fp}' during \
                     {phase_str} phase."
                ));
            }
            let fp = ecc_domain::workflow::path::normalize_path(raw_fp);
            let fp = fp.as_str();
            if is_allowed_path(fp, &prefixes) {
                WorkflowOutput::pass(format!("Write to allowed path '{fp}' permitted"))
            } else {
                WorkflowOutput::block(format!(
                    "BLOCKED: Cannot write to '{fp}' during {phase_str} phase. \
                     Only workflow and docs paths are allowed."
                ))
            }
        }
        Some("Bash") => {
            let cmd = command.as_deref().unwrap_or("");
            if is_destructive_bash(cmd) {
                WorkflowOutput::block(format!(
                    "BLOCKED: Destructive command not allowed during {phase_str} phase. \
                     Command: {cmd}"
                ))
            } else {
                WorkflowOutput::pass("Bash command permitted")
            }
        }
        _ => WorkflowOutput::pass("Tool permitted"),
    };
    let tool = tool_name.as_deref().unwrap_or("none");
    let verdict = format!("{:?}", result.status);
    tracing::info!(phase = %phase_str, tool = %tool, verdict = %verdict, "phase-gate decision");
    result
}
