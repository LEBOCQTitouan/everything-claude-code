use std::path::Path;

use ecc_domain::workflow::phase::Phase;
use ecc_domain::workflow::state::WorkflowState;

use crate::io::{read_stdin, with_state_lock};
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
/// - Corrupt/unparseable state.json → exit 0 (warn) — does NOT block
/// - phase.is_gated() == false (Idle, Implement, Done) → exit 0 (pass)
/// - Write/Edit/MultiEdit to allowed path → exit 0 (pass)
/// - Write/Edit/MultiEdit to blocked path → exit 2 (block)
/// - Bash with destructive command → exit 2 (block)
/// - All other tools → exit 0 (pass)
///
/// The workflow phase is read under the state lock so that phase-gate never
/// observes a partially-written state.json during a concurrent transition.
pub fn run(project_dir: &Path) -> WorkflowOutput {
    // Read stdin before acquiring the lock — stdin is not state-dependent.
    let input = read_stdin();
    run_with_input(project_dir, &input)
}

/// Testable entry point: same as `run` but accepts hook input directly.
pub fn run_with_input(project_dir: &Path, input: &str) -> WorkflowOutput {
    // Fast path: if state.json does not exist, skip locking entirely.
    let state_path = project_dir.join(".claude/workflow/state.json");
    if !state_path.exists() {
        return WorkflowOutput::pass("No workflow active");
    }

    // Acquire state lock to read the phase atomically with respect to transitions.
    let phase_result = match with_state_lock(project_dir, || read_phase_typed(project_dir)) {
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
    let (tool_name, file_path, command) = parse_hook_input(input);

    let result = match tool_name.as_deref() {
        Some("Write") | Some("Edit") | Some("MultiEdit") => {
            let fp = file_path.as_deref().unwrap_or("");
            if is_allowed_path(fp) {
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

enum PhaseResult {
    Missing,
    Corrupt(String),
    Parsed(Phase),
}

fn read_phase_typed(project_dir: &Path) -> PhaseResult {
    let state_path = project_dir.join(".claude/workflow/state.json");
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

#[cfg(test)]
mod tests {
    use crate::output::Status;
    use std::sync::{Arc, Barrier};
    use std::time::Duration;
    use tempfile::TempDir;

    fn write_state(dir: &std::path::Path, phase: &str) {
        let workflow_dir = dir.join(".claude/workflow");
        std::fs::create_dir_all(&workflow_dir).unwrap();
        let json = format!(
            r#"{{"phase":"{phase}","concern":"dev","feature":"feat","started_at":"2026-01-01T00:00:00Z","toolchain":{{"test":null,"lint":null,"build":null}},"artifacts":{{"plan":null,"solution":null,"implement":null,"campaign_path":null,"spec_path":null,"design_path":null,"tasks_path":null}},"completed":[]}}"#
        );
        std::fs::write(workflow_dir.join("state.json"), json).unwrap();
    }

    fn write_raw_state(dir: &std::path::Path, content: &str) {
        let workflow_dir = dir.join(".claude/workflow");
        std::fs::create_dir_all(&workflow_dir).unwrap();
        std::fs::write(workflow_dir.join("state.json"), content).unwrap();
    }

    /// PC-015: phase_gate passes for idle state (AC-001.7, AC-002.1)
    #[test]
    fn phase_gate_passes_for_idle() {
        let tmp = TempDir::new().unwrap();
        write_state(tmp.path(), "idle");
        let output = super::run_with_input(tmp.path(), "");
        assert!(
            matches!(output.status, Status::Pass),
            "Expected Pass for idle phase, got {:?}: {}",
            output.status,
            output.message
        );
    }

    /// PC-016: phase_gate warns when state.json has invalid type for phase (AC-002.2)
    #[test]
    fn phase_gate_corrupt_type_warns() {
        let tmp = TempDir::new().unwrap();
        write_raw_state(
            tmp.path(),
            r#"{"phase":123,"concern":"dev","feature":"","started_at":"2026-01-01T00:00:00Z","toolchain":{"test":null,"lint":null,"build":null},"artifacts":{"plan":null,"solution":null,"implement":null,"campaign_path":null,"spec_path":null,"design_path":null,"tasks_path":null},"completed":[]}"#,
        );
        let output = super::run_with_input(tmp.path(), "");
        assert!(
            matches!(output.status, Status::Warn),
            "Expected Warn for corrupt type, got {:?}: {}",
            output.status,
            output.message
        );
        assert!(
            output.message.contains("Corrupt") || output.message.contains("corrupt"),
            "Expected 'corrupt' in message, got: {}",
            output.message
        );
    }

    /// PC-017: phase_gate warns when state.json is missing the phase key (AC-002.3)
    #[test]
    fn phase_gate_missing_phase_warns() {
        let tmp = TempDir::new().unwrap();
        write_raw_state(
            tmp.path(),
            r#"{"concern":"dev","feature":"","started_at":"2026-01-01T00:00:00Z","toolchain":{"test":null,"lint":null,"build":null},"artifacts":{"plan":null,"solution":null,"implement":null,"campaign_path":null,"spec_path":null,"design_path":null,"tasks_path":null},"completed":[]}"#,
        );
        let output = super::run_with_input(tmp.path(), "");
        assert!(
            matches!(output.status, Status::Warn),
            "Expected Warn for missing phase, got {:?}: {}",
            output.status,
            output.message
        );
    }

    /// PC-018: phase_gate blocks Write to non-allowed path when phase is plan (AC-002.4)
    #[test]
    fn phase_gate_plan_blocks_write() {
        let tmp = TempDir::new().unwrap();
        write_state(tmp.path(), "plan");
        let hook_input = r#"{"tool_name":"Write","tool_input":{"file_path":"src/main.rs"}}"#;
        let output = super::run_with_input(tmp.path(), hook_input);
        assert!(
            matches!(output.status, Status::Block),
            "Expected Block for Write to src/main.rs during plan phase, got {:?}: {}",
            output.status,
            output.message
        );
    }

    /// PC-019: phase_gate warns when state.json has unknown phase variant (AC-002.5)
    #[test]
    fn phase_gate_unknown_variant_warns() {
        let tmp = TempDir::new().unwrap();
        write_raw_state(
            tmp.path(),
            r#"{"phase":"banana","concern":"dev","feature":"","started_at":"2026-01-01T00:00:00Z","toolchain":{"test":null,"lint":null,"build":null},"artifacts":{"plan":null,"solution":null,"implement":null,"campaign_path":null,"spec_path":null,"design_path":null,"tasks_path":null},"completed":[]}"#,
        );
        let output = super::run_with_input(tmp.path(), "");
        assert!(
            matches!(output.status, Status::Warn),
            "Expected Warn for unknown variant 'banana', got {:?}: {}",
            output.status,
            output.message
        );
    }

    /// PC-017: phase_gate reads the workflow phase under the state lock.
    ///
    /// Verifies that `run()` waits for the state lock to be available before
    /// reading state.json. A background thread holds the lock; run() must block
    /// until it is released and then complete successfully.
    #[test]
    fn phase_gate_reads_under_lock() {
        let tmp = TempDir::new().unwrap();
        let project_dir = tmp.path().to_path_buf();

        // Create state.json with phase=plan so phase_gate reads it
        let workflow_dir = project_dir.join(".claude/workflow");
        std::fs::create_dir_all(&workflow_dir).unwrap();
        let state_json = serde_json::json!({
            "phase": "plan",
            "concern": "dev",
            "feature": "test-feature",
            "started_at": "2026-01-01T00:00:00Z",
            "toolchain": {"test": null, "lint": null, "build": null},
            "artifacts": {
                "plan": null,
                "solution": null,
                "implement": null,
                "campaign_path": null,
                "spec_path": null,
                "design_path": null,
                "tasks_path": null
            },
            "completed": []
        });
        std::fs::write(
            workflow_dir.join("state.json"),
            serde_json::to_string_pretty(&state_json).unwrap(),
        )
        .unwrap();

        // Barrier: main thread and lock-holder thread synchronize on lock acquisition
        let barrier = Arc::new(Barrier::new(2));
        let barrier_clone = Arc::clone(&barrier);
        let project_dir_clone = project_dir.clone();

        // Background thread: acquire the state lock, signal barrier, hold for 200ms, then release
        let lock_thread = std::thread::spawn(move || {
            let guard = ecc_flock::acquire(&project_dir_clone, "state")
                .expect("background thread failed to acquire state lock");
            // Signal main thread that lock is held
            barrier_clone.wait();
            // Hold the lock for long enough for run() to be blocked
            std::thread::sleep(Duration::from_millis(200));
            // Release by dropping
            drop(guard);
        });

        // Wait until the background thread holds the lock
        barrier.wait();

        // Now call run() — it must acquire the lock itself, so it must wait ~200ms
        let start = std::time::Instant::now();
        let output = super::run(&project_dir);
        let elapsed = start.elapsed();

        lock_thread.join().expect("lock thread panicked");

        // run() must have waited for the lock (at least 100ms, accounting for scheduling jitter)
        assert!(
            elapsed >= Duration::from_millis(100),
            "phase_gate::run() did not wait for the state lock — elapsed {:?} < 100ms. \
             phase_gate must call with_state_lock before reading state.",
            elapsed
        );

        // run() must have completed successfully (tool_name = None → "Tool permitted")
        assert!(
            output.message.contains("permitted") || output.message.contains("gating"),
            "unexpected phase_gate output: {}",
            output.message
        );

        // Lock must be free after run() returns
        ecc_flock::acquire(&project_dir, "state")
            .expect("state lock was not released after phase_gate::run returned");
    }
}
