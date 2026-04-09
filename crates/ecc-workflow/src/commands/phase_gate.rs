use std::path::Path;

use ecc_domain::workflow::phase::Phase;
use ecc_domain::workflow::state::WorkflowState;

use crate::io::{read_stdin, with_state_lock};
use crate::output::WorkflowOutput;

/// Returns the dynamic allowlist of path prefixes for the given state directory.
///
/// Always includes standard doc paths. Adds the resolved `state_dir` as a prefix
/// (so writes to the state directory itself are always permitted). Also always
/// includes the legacy `.claude/workflow/` prefix for backward compatibility
/// unless the `state_dir` already points there.
fn allowed_prefixes(state_dir: &Path) -> Vec<String> {
    let mut prefixes = vec![
        ".claude/plans/".to_owned(),
        "docs/audits/".to_owned(),
        "docs/backlog/".to_owned(),
        "docs/user-stories/".to_owned(),
        "docs/specs/".to_owned(),
        "docs/plans/".to_owned(),
        "docs/designs/".to_owned(),
        "docs/adr/".to_owned(),
        "docs/prds/".to_owned(),
        "docs/refactors/".to_owned(),
        "docs/cartography/".to_owned(),
        "docs/domain/".to_owned(),
        "docs/guides/".to_owned(),
        "docs/diagrams/".to_owned(),
    ];
    let state_str = state_dir.to_string_lossy();
    let with_slash = if state_str.ends_with('/') {
        state_str.to_string()
    } else {
        format!("{state_str}/")
    };
    prefixes.push(with_slash);
    // Always include legacy for backward compat
    prefixes.push(".claude/workflow/".to_owned());
    prefixes
}

/// Returns `true` if the path contains URL-encoded traversal sequences.
///
/// Detects `%2e%2e` (dot-dot), `%2f` (slash), and `%5c` (backslash)
/// to block attempts to escape the allowlisted prefix via percent-encoding.
fn contains_encoded_traversal(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.contains("%2e%2e") || lower.contains("%2f") || lower.contains("%5c")
}

/// Maximum bytes to read from a `.git` file when detecting worktree gitdir.
const GIT_FILE_MAX_BYTES: usize = 4096;

/// Maximum parent directory traversal depth for worktree detection.
const WORKTREE_DEPTH_LIMIT: usize = 50;

/// Derive the worktree-scoped state directory from a gated file path.
///
/// When Claude Code's hook subprocess sets `CLAUDE_PROJECT_DIR` to the main repo
/// root, `resolve_state_dir()` reads the wrong `.state-dir` anchor. This function
/// bypasses that by walking the gated file path's parents to find the worktree's
/// `.git` file, then reading its `gitdir:` line to find the correct git-dir.
///
/// Returns `Some(state_dir)` if the file is inside a worktree checkout.
/// Returns `None` if the file is in a main repo, not absolute, or detection fails.
fn resolve_worktree_state_dir(file_path: &str) -> Option<std::path::PathBuf> {
    let path = std::path::Path::new(file_path);
    if !path.is_absolute() {
        return None;
    }

    let mut current = path.parent()?;
    for _ in 0..WORKTREE_DEPTH_LIMIT {
        let git_entry = current.join(".git");
        if git_entry.exists() {
            if git_entry.is_file() {
                // Worktree: .git is a file containing "gitdir: <path>"
                use std::io::Read;
                let file = std::fs::File::open(&git_entry).ok()?;
                let mut content = String::new();
                file.take(GIT_FILE_MAX_BYTES as u64 + 1)
                    .read_to_string(&mut content)
                    .ok()?;
                if content.len() > GIT_FILE_MAX_BYTES {
                    return None;
                }
                let gitdir_line = content.lines().find(|l| l.starts_with("gitdir:"))?;
                let raw_path = gitdir_line.strip_prefix("gitdir:")?.trim();
                let gitdir = if std::path::Path::new(raw_path).is_absolute() {
                    std::path::PathBuf::from(raw_path)
                } else {
                    current.join(raw_path)
                };
                return Some(gitdir.join("ecc-workflow"));
            }
            // .git is a directory — main repo checkout, not a worktree
            return None;
        }
        current = current.parent()?;
    }
    None
}

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

fn is_allowed_path(path: &str, prefixes: &[String]) -> bool {
    for prefix in prefixes {
        if path.starts_with(prefix.as_str()) || path.contains(&format!("/{prefix}")) {
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

    fn state_dir_for(tmp: &TempDir) -> std::path::PathBuf {
        tmp.path().join(".claude/workflow")
    }

    /// PC-015: phase_gate passes for idle state (AC-001.7, AC-002.1)
    #[test]
    fn phase_gate_passes_for_idle() {
        let tmp = TempDir::new().unwrap();
        write_state(tmp.path(), "idle");
        let state_dir = state_dir_for(&tmp);
        let output = super::run_with_input(tmp.path(), &state_dir, "");
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
        let state_dir = state_dir_for(&tmp);
        let output = super::run_with_input(tmp.path(), &state_dir, "");
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
        let state_dir = state_dir_for(&tmp);
        let output = super::run_with_input(tmp.path(), &state_dir, "");
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
        let state_dir = state_dir_for(&tmp);
        let hook_input = r#"{"tool_name":"Write","tool_input":{"file_path":"src/main.rs"}}"#;
        let output = super::run_with_input(tmp.path(), &state_dir, hook_input);
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
        let state_dir = state_dir_for(&tmp);
        let output = super::run_with_input(tmp.path(), &state_dir, "");
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
        let workflow_dir_clone = workflow_dir.clone();

        // Background thread: acquire the state lock, signal barrier, hold for 200ms, then release
        let lock_thread = std::thread::spawn(move || {
            let guard = ecc_flock::acquire_for(&workflow_dir_clone, "state")
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
        let output = super::run(&project_dir, &workflow_dir);
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
        ecc_flock::acquire_for(&workflow_dir, "state")
            .expect("state lock was not released after phase_gate::run returned");
    }

    // ----------------------------------------------------------------
    // Wave 3 tests: dynamic allowlisting (PC-011-015) + SEC-010 (PC-026)
    // ----------------------------------------------------------------

    /// PC-026: contains_encoded_traversal detects %2e%2e, %2f, %5c (AC-003.3)
    #[test]
    fn contains_encoded_traversal_detects() {
        assert!(
            super::contains_encoded_traversal("docs/specs/%2e%2e/src/evil.rs"),
            "should detect %2e%2e"
        );
        assert!(
            super::contains_encoded_traversal("docs%2fspecs%2fevil.rs"),
            "should detect %2f"
        );
        assert!(
            super::contains_encoded_traversal("docs%5cevil.rs"),
            "should detect %5c"
        );
        assert!(
            !super::contains_encoded_traversal("docs/specs/normal.md"),
            "should not flag normal path"
        );
        // Case-insensitive
        assert!(
            super::contains_encoded_traversal("docs/specs/%2E%2E/src/evil.rs"),
            "should detect %2E%2E (uppercase)"
        );
    }

    /// PC-011: phase_gate allows writes to resolved state_dir during spec (AC-003.1)
    #[test]
    fn phase_gate_allows_resolved_state_dir() {
        let tmp = TempDir::new().unwrap();
        // Create a custom state dir (simulating worktree state dir)
        let state_dir = tmp.path().join("some/worktree/.ecc-workflow");
        std::fs::create_dir_all(&state_dir).unwrap();
        let json = r#"{"phase":"plan","concern":"dev","feature":"feat","started_at":"2026-01-01T00:00:00Z","toolchain":{"test":null,"lint":null,"build":null},"artifacts":{"plan":null,"solution":null,"implement":null,"campaign_path":null,"spec_path":null,"design_path":null,"tasks_path":null},"completed":[]}"#;
        std::fs::write(state_dir.join("state.json"), json).unwrap();

        // A file inside the state_dir should be allowed
        let file_in_state_dir = state_dir.join("custom.json");
        let file_path_str = file_in_state_dir.to_string_lossy();
        let hook_input =
            format!(r#"{{"tool_name":"Write","tool_input":{{"file_path":"{file_path_str}"}}}}"#);
        let output = super::run_with_input(tmp.path(), &state_dir, &hook_input);
        assert!(
            matches!(output.status, Status::Pass),
            "Expected Pass for write inside resolved state_dir, got {:?}: {}",
            output.status,
            output.message
        );
    }

    /// PC-012: phase_gate allows writes to .claude/workflow/ during spec (fallback) (AC-003.2)
    #[test]
    fn phase_gate_allows_fallback_state_dir() {
        let tmp = TempDir::new().unwrap();
        write_state(tmp.path(), "plan");
        let state_dir = state_dir_for(&tmp);
        // Write to the legacy .claude/workflow/ path — must be allowed
        let hook_input =
            r#"{"tool_name":"Write","tool_input":{"file_path":".claude/workflow/state.json"}}"#;
        let output = super::run_with_input(tmp.path(), &state_dir, hook_input);
        assert!(
            matches!(output.status, Status::Pass),
            "Expected Pass for write to .claude/workflow/ (fallback), got {:?}: {}",
            output.status,
            output.message
        );
    }

    /// PC-013: phase_gate blocks URL-encoded traversal %2e%2e (AC-003.3)
    #[test]
    fn phase_gate_blocks_url_encoded_traversal() {
        let tmp = TempDir::new().unwrap();
        write_state(tmp.path(), "plan");
        let state_dir = state_dir_for(&tmp);
        let hook_input =
            r#"{"tool_name":"Write","tool_input":{"file_path":"docs/specs/%2e%2e/src/evil.rs"}}"#;
        let output = super::run_with_input(tmp.path(), &state_dir, hook_input);
        assert!(
            matches!(output.status, Status::Block),
            "Expected Block for URL-encoded traversal, got {:?}: {}",
            output.status,
            output.message
        );
    }

    /// PC-014: phase_gate with worktree state_dir in implement allows src/ writes (AC-001.1)
    #[test]
    fn phase_gate_worktree_implement_allows() {
        let tmp = TempDir::new().unwrap();
        // Worktree state_dir (non-standard path)
        let state_dir = tmp.path().join("worktree-state");
        std::fs::create_dir_all(&state_dir).unwrap();
        let json = r#"{"phase":"implement","concern":"dev","feature":"feat","started_at":"2026-01-01T00:00:00Z","toolchain":{"test":null,"lint":null,"build":null},"artifacts":{"plan":null,"solution":null,"implement":null,"campaign_path":null,"spec_path":null,"design_path":null,"tasks_path":null},"completed":[]}"#;
        std::fs::write(state_dir.join("state.json"), json).unwrap();

        let hook_input = r#"{"tool_name":"Write","tool_input":{"file_path":"src/main.rs"}}"#;
        let output = super::run_with_input(tmp.path(), &state_dir, hook_input);
        assert!(
            matches!(output.status, Status::Pass),
            "Expected Pass for src/main.rs in implement phase, got {:?}: {}",
            output.status,
            output.message
        );
    }

    /// PC-015: phase_gate with worktree state_dir in spec blocks src/ writes (AC-001.2)
    #[test]
    fn phase_gate_worktree_spec_blocks() {
        let tmp = TempDir::new().unwrap();
        // Worktree state_dir (non-standard path)
        let state_dir = tmp.path().join("worktree-state");
        std::fs::create_dir_all(&state_dir).unwrap();
        let json = r#"{"phase":"plan","concern":"dev","feature":"feat","started_at":"2026-01-01T00:00:00Z","toolchain":{"test":null,"lint":null,"build":null},"artifacts":{"plan":null,"solution":null,"implement":null,"campaign_path":null,"spec_path":null,"design_path":null,"tasks_path":null},"completed":[]}"#;
        std::fs::write(state_dir.join("state.json"), json).unwrap();

        let hook_input = r#"{"tool_name":"Write","tool_input":{"file_path":"src/main.rs"}}"#;
        let output = super::run_with_input(tmp.path(), &state_dir, hook_input);
        assert!(
            matches!(output.status, Status::Block),
            "Expected Block for src/main.rs in plan phase, got {:?}: {}",
            output.status,
            output.message
        );
    }

    // ----------------------------------------------------------------

    /// PC-005: phase_gate blocks path traversal attack after normalization
    #[test]
    fn phase_gate_blocks_traversal_attack() {
        let tmp = TempDir::new().unwrap();
        write_state(tmp.path(), "plan");
        let state_dir = state_dir_for(&tmp);
        let hook_input =
            r#"{"tool_name":"Write","tool_input":{"file_path":"docs/specs/../../src/evil.rs"}}"#;
        let output = super::run_with_input(tmp.path(), &state_dir, hook_input);
        assert!(
            matches!(output.status, Status::Block),
            "Expected Block for traversal attack path during plan phase, got {:?}: {}",
            output.status,
            output.message
        );
    }

    /// PC-006: phase_gate blocks absolute path outside allowed prefixes
    #[test]
    fn phase_gate_blocks_absolute_outside() {
        let tmp = TempDir::new().unwrap();
        write_state(tmp.path(), "plan");
        let state_dir = state_dir_for(&tmp);
        let hook_input = r#"{"tool_name":"Write","tool_input":{"file_path":"/etc/passwd"}}"#;
        let output = super::run_with_input(tmp.path(), &state_dir, hook_input);
        assert!(
            matches!(output.status, Status::Block),
            "Expected Block for absolute path /etc/passwd during plan phase, got {:?}: {}",
            output.status,
            output.message
        );
    }

    /// PC-006: phase_gate allows writes to docs/prds/ during plan phase (AC-004.1, AC-004.3)
    #[test]
    fn phase_gate_allows_prds_dir() {
        let tmp = TempDir::new().unwrap();
        write_state(tmp.path(), "plan");
        let state_dir = state_dir_for(&tmp);
        let hook_input =
            r#"{"tool_name":"Write","tool_input":{"file_path":"docs/prds/my-feature-prd.md"}}"#;
        let output = super::run_with_input(tmp.path(), &state_dir, hook_input);
        assert!(
            matches!(output.status, Status::Pass),
            "Expected Pass for docs/prds/ during plan phase, got {:?}: {}",
            output.status,
            output.message
        );
    }

    // ----------------------------------------------------------------
    // BL-131 Wave 1: File-path-based worktree state resolution
    // ----------------------------------------------------------------

    /// PC-001: When gated file path is inside a worktree checkout (with .git file),
    /// phase_gate resolves state from that worktree's git-dir, not from the
    /// dispatch state_dir (which may point to the main repo's state).
    #[test]
    fn phase_gate_worktree_file_path_overrides_state_dir() {
        let tmp = TempDir::new().unwrap();

        // Simulate a worktree checkout: create a .git FILE (not dir) pointing to a gitdir
        let worktree_root = tmp.path().join("worktrees/my-feature");
        std::fs::create_dir_all(&worktree_root).unwrap();

        let gitdir_path = tmp.path().join(".git/worktrees/my-feature");
        std::fs::create_dir_all(&gitdir_path).unwrap();

        // Write .git file in worktree root
        std::fs::write(
            worktree_root.join(".git"),
            format!(
                "gitdir: {}
",
                gitdir_path.display()
            ),
        )
        .unwrap();

        // Write state.json in the worktree's git-dir with phase=implement
        let worktree_state_dir = gitdir_path.join("ecc-workflow");
        std::fs::create_dir_all(&worktree_state_dir).unwrap();
        let implement_json = r#"{"phase":"implement","concern":"dev","feature":"feat","started_at":"2026-01-01T00:00:00Z","toolchain":{"test":null,"lint":null,"build":null},"artifacts":{"plan":null,"solution":null,"implement":null,"campaign_path":null,"spec_path":null,"design_path":null,"tasks_path":null},"completed":[]}"#;
        std::fs::write(worktree_state_dir.join("state.json"), implement_json).unwrap();

        // Dispatch state_dir points to a DIFFERENT location with phase=plan (simulating BL-131 bug)
        let wrong_state_dir = tmp.path().join("main-repo-state");
        std::fs::create_dir_all(&wrong_state_dir).unwrap();
        let plan_json = r#"{"phase":"plan","concern":"dev","feature":"other","started_at":"2026-01-01T00:00:00Z","toolchain":{"test":null,"lint":null,"build":null},"artifacts":{"plan":null,"solution":null,"implement":null,"campaign_path":null,"spec_path":null,"design_path":null,"tasks_path":null},"completed":[]}"#;
        std::fs::write(wrong_state_dir.join("state.json"), plan_json).unwrap();

        // Gated file is inside the worktree checkout
        let file_in_worktree = worktree_root.join("src/main.rs");
        let file_path_str = file_in_worktree.to_string_lossy();
        let hook_input =
            format!(r#"{{"tool_name":"Write","tool_input":{{"file_path":"{file_path_str}"}}}}"#);

        // run_with_input uses the wrong_state_dir (plan phase → would block),
        // but the file path should cause override to worktree state (implement → pass)
        let output = super::run_with_input(tmp.path(), &wrong_state_dir, &hook_input);
        assert!(
            matches!(output.status, Status::Pass),
            "Expected Pass (implement phase from worktree), got {:?}: {}.              The file path should have overridden the dispatch state_dir.",
            output.status,
            output.message
        );
    }

    /// PC-002: When gated file path is relative, falls back to dispatch state_dir.
    #[test]
    fn phase_gate_relative_path_uses_dispatch_state_dir() {
        let tmp = TempDir::new().unwrap();
        write_state(tmp.path(), "plan");
        let state_dir = state_dir_for(&tmp);
        // Relative path — cannot walk parents to find .git
        let hook_input = r#"{"tool_name":"Write","tool_input":{"file_path":"src/main.rs"}}"#;
        let output = super::run_with_input(tmp.path(), &state_dir, hook_input);
        assert!(
            matches!(output.status, Status::Block),
            "Expected Block for relative path (falls back to dispatch state_dir with plan phase), got {:?}: {}",
            output.status,
            output.message
        );
    }

    /// PC-003: resolve_worktree_state_dir correctly parses a .git file and returns the state dir.
    #[test]
    fn resolve_worktree_from_git_file() {
        let tmp = TempDir::new().unwrap();

        // Create worktree structure
        let worktree_root = tmp.path().join("checkout");
        std::fs::create_dir_all(worktree_root.join("src")).unwrap();

        let gitdir = tmp.path().join("repo.git/worktrees/my-branch");
        std::fs::create_dir_all(&gitdir).unwrap();

        std::fs::write(
            worktree_root.join(".git"),
            format!(
                "gitdir: {}
",
                gitdir.display()
            ),
        )
        .unwrap();

        let result =
            super::resolve_worktree_state_dir(&worktree_root.join("src/lib.rs").to_string_lossy());
        assert_eq!(
            result,
            Some(gitdir.join("ecc-workflow")),
            "should resolve to gitdir/ecc-workflow"
        );
    }

    /// PC-004: When the worktree's state dir has no state.json, phase_gate passes.
    #[test]
    fn worktree_no_state_json_passes() {
        let tmp = TempDir::new().unwrap();

        // Worktree checkout with .git file
        let worktree_root = tmp.path().join("wt");
        std::fs::create_dir_all(&worktree_root).unwrap();
        let gitdir = tmp.path().join(".git/worktrees/wt");
        std::fs::create_dir_all(&gitdir).unwrap();
        std::fs::write(
            worktree_root.join(".git"),
            format!(
                "gitdir: {}
",
                gitdir.display()
            ),
        )
        .unwrap();

        // Do NOT create state.json in the worktree's ecc-workflow dir
        // (create the dir but leave it empty)
        std::fs::create_dir_all(gitdir.join("ecc-workflow")).unwrap();

        // Dispatch state_dir has plan phase (would block if used)
        let wrong_state_dir = tmp.path().join("wrong");
        std::fs::create_dir_all(&wrong_state_dir).unwrap();
        let plan_json = r#"{"phase":"plan","concern":"dev","feature":"other","started_at":"2026-01-01T00:00:00Z","toolchain":{"test":null,"lint":null,"build":null},"artifacts":{"plan":null,"solution":null,"implement":null,"campaign_path":null,"spec_path":null,"design_path":null,"tasks_path":null},"completed":[]}"#;
        std::fs::write(wrong_state_dir.join("state.json"), plan_json).unwrap();

        let file_path = worktree_root.join("src/main.rs");
        let hook_input = format!(
            r#"{{"tool_name":"Write","tool_input":{{"file_path":"{}"}}}}"#,
            file_path.display()
        );

        let output = super::run_with_input(tmp.path(), &wrong_state_dir, &hook_input);
        assert!(
            matches!(output.status, Status::Pass),
            "Expected Pass (no state.json in worktree → no workflow active), got {:?}: {}",
            output.status,
            output.message
        );
    }

    /// PC-005: When .git is a directory (main repo), resolve_worktree_state_dir returns None.
    #[test]
    fn resolve_worktree_stops_at_git_dir() {
        let tmp = TempDir::new().unwrap();

        // Main repo: .git is a DIRECTORY
        let repo_root = tmp.path().join("repo");
        std::fs::create_dir_all(repo_root.join(".git")).unwrap();
        std::fs::create_dir_all(repo_root.join("src")).unwrap();

        let result =
            super::resolve_worktree_state_dir(&repo_root.join("src/main.rs").to_string_lossy());
        assert_eq!(
            result, None,
            "should return None for main repo (.git is a directory)"
        );
    }

    /// PC-006: resolve_worktree_state_dir stops after WORKTREE_DEPTH_LIMIT iterations.
    #[test]
    fn resolve_worktree_depth_limit() {
        // Create a deeply nested path with no .git anywhere
        let mut deep_path = String::from("/tmp/bl131-depth-test");
        for i in 0..60 {
            deep_path.push_str(&format!("/d{i}"));
        }
        deep_path.push_str("/file.rs");

        let result = super::resolve_worktree_state_dir(&deep_path);
        assert_eq!(
            result, None,
            "should return None after exceeding depth limit"
        );
    }

    /// PC-003b: resolve_worktree_state_dir handles relative gitdir: paths.
    #[test]
    fn resolve_worktree_from_relative_gitdir() {
        let tmp = TempDir::new().unwrap();

        // Create worktree structure with a relative gitdir path
        let worktree_root = tmp.path().join("checkouts/my-feature");
        std::fs::create_dir_all(worktree_root.join("src")).unwrap();

        let gitdir = tmp.path().join("repo.git/worktrees/my-feature");
        // Create both the gitdir AND ecc-workflow subdir (for canonicalize)
        std::fs::create_dir_all(gitdir.join("ecc-workflow")).unwrap();

        // Write .git file with RELATIVE path
        std::fs::write(
            worktree_root.join(".git"),
            "gitdir: ../../repo.git/worktrees/my-feature\n",
        )
        .unwrap();

        let result =
            super::resolve_worktree_state_dir(&worktree_root.join("src/lib.rs").to_string_lossy());
        let resolved = result.expect("should resolve relative gitdir");
        let canonical_expected = gitdir.join("ecc-workflow").canonicalize().unwrap();
        let canonical_actual = resolved.canonicalize().unwrap();
        assert_eq!(
            canonical_actual, canonical_expected,
            "relative gitdir should resolve to the same absolute path"
        );
    }

    /// phase_gate allows writes to docs/refactors/ during plan phase (AC-004.3)
    #[test]
    fn phase_gate_allows_refactors_dir() {
        let tmp = TempDir::new().unwrap();
        write_state(tmp.path(), "plan");
        let state_dir = state_dir_for(&tmp);
        let hook_input = r#"{"tool_name":"Write","tool_input":{"file_path":"docs/refactors/my-refactor-plan.md"}}"#;
        let output = super::run_with_input(tmp.path(), &state_dir, hook_input);
        assert!(
            matches!(output.status, Status::Pass),
            "Expected Pass for docs/refactors/ during plan phase, got {:?}: {}",
            output.status,
            output.message
        );
    }

    /// BL-142: phase_gate allows writes to docs/cartography/ during plan phase
    #[test]
    fn phase_gate_allows_cartography_dir() {
        let tmp = TempDir::new().unwrap();
        write_state(tmp.path(), "plan");
        let state_dir = state_dir_for(&tmp);
        let hook_input =
            r#"{"tool_name":"Write","tool_input":{"file_path":"docs/cartography/journeys/test.md"}}"#;
        let output = super::run_with_input(tmp.path(), &state_dir, hook_input);
        assert!(
            matches!(output.status, Status::Pass),
            "Expected Pass for docs/cartography/ during plan phase, got {:?}: {}",
            output.status,
            output.message
        );
    }

    /// BL-142: phase_gate allows writes to docs/domain/ during plan phase
    #[test]
    fn phase_gate_allows_domain_dir() {
        let tmp = TempDir::new().unwrap();
        write_state(tmp.path(), "plan");
        let state_dir = state_dir_for(&tmp);
        let hook_input = r#"{"tool_name":"Write","tool_input":{"file_path":"docs/domain/bounded-contexts.md"}}"#;
        let output = super::run_with_input(tmp.path(), &state_dir, hook_input);
        assert!(
            matches!(output.status, Status::Pass),
            "Expected Pass for docs/domain/ during plan phase, got {:?}: {}",
            output.status,
            output.message
        );
    }

    /// BL-142: phase_gate allows writes to docs/guides/ during plan phase
    #[test]
    fn phase_gate_allows_guides_dir() {
        let tmp = TempDir::new().unwrap();
        write_state(tmp.path(), "plan");
        let state_dir = state_dir_for(&tmp);
        let hook_input = r#"{"tool_name":"Write","tool_input":{"file_path":"docs/guides/getting-started.md"}}"#;
        let output = super::run_with_input(tmp.path(), &state_dir, hook_input);
        assert!(
            matches!(output.status, Status::Pass),
            "Expected Pass for docs/guides/ during plan phase, got {:?}: {}",
            output.status,
            output.message
        );
    }

    /// BL-142: phase_gate allows writes to docs/diagrams/ during plan phase
    #[test]
    fn phase_gate_allows_diagrams_dir() {
        let tmp = TempDir::new().unwrap();
        write_state(tmp.path(), "plan");
        let state_dir = state_dir_for(&tmp);
        let hook_input = r#"{"tool_name":"Write","tool_input":{"file_path":"docs/diagrams/flow.md"}}"#;
        let output = super::run_with_input(tmp.path(), &state_dir, hook_input);
        assert!(
            matches!(output.status, Status::Pass),
            "Expected Pass for docs/diagrams/ during plan phase, got {:?}: {}",
            output.status,
            output.message
        );
    }
}
