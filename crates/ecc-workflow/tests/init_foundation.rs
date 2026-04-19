//! BL-155 integration tests — Foundation concern end-to-end.
//!
//! Verifies `ecc-workflow init foundation` writes the correct state.json, the
//! `ecc workflow init` delegator matches direct invocation, `worktree-name
//! foundation` returns a valid session-branch name, and the full FSM walk
//! (init → plan → solution → implement → done) preserves `concern=foundation`.

mod common;

use std::process::Command;

fn read_state(temp: &std::path::Path) -> serde_json::Value {
    let state_path = temp.join(".claude/workflow/state.json");
    let content = std::fs::read_to_string(&state_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", state_path.display()));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("state.json invalid JSON: {e}\ncontent: {content}"))
}

fn assert_exit_zero(output: &std::process::Output, step: &str) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{step} must exit 0, got {:?}\nstdout: {}\nstderr: {}",
        output.status.code(),
        std::str::from_utf8(&output.stdout).unwrap_or(""),
        std::str::from_utf8(&output.stderr).unwrap_or(""),
    );
}

/// PC-010 / AC-002.1: `ecc-workflow init foundation "demo"` exits 0 and writes
/// state.json with `concern="foundation"`.
#[test]
fn init_foundation_writes_concern_foundation() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    let temp_dir = tempfile::tempdir().unwrap();
    let output = Command::new(&bin)
        .args(["init", "foundation", "demo"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .output()
        .expect("failed to execute ecc-workflow init foundation");

    assert_exit_zero(&output, "init foundation");

    let state = read_state(temp_dir.path());
    assert_eq!(
        state.get("concern").and_then(|v| v.as_str()),
        Some("foundation"),
        "state.json must have concern=foundation; got {:?}",
        state.get("concern"),
    );
}

/// PC-011 / AC-002.2: `--feature-stdin` delegator path accepts `foundation`
/// identically to direct invocation. Feeds the feature via stdin and verifies
/// the resulting state.json matches the direct-arg form in its concern field.
#[test]
fn delegator_init_foundation_matches_direct() {
    use std::io::Write;

    let bin = common::binary_path();

    // Direct form
    let temp_direct = tempfile::tempdir().unwrap();
    let direct = Command::new(&bin)
        .args(["init", "foundation", "demo"])
        .env("CLAUDE_PROJECT_DIR", temp_direct.path())
        .output()
        .expect("direct init failed");
    assert_exit_zero(&direct, "direct init foundation");

    // Stdin form (the delegator pattern used by `ecc workflow init`)
    let temp_stdin = tempfile::tempdir().unwrap();
    let mut child = Command::new(&bin)
        .args(["init", "foundation", "--feature-stdin"])
        .env("CLAUDE_PROJECT_DIR", temp_stdin.path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn stdin init");
    child
        .stdin
        .as_mut()
        .expect("stdin handle missing")
        .write_all(b"demo")
        .expect("write stdin failed");
    let stdin_output = child.wait_with_output().expect("wait failed");
    assert_exit_zero(&stdin_output, "stdin init foundation");

    let direct_state = read_state(temp_direct.path());
    let stdin_state = read_state(temp_stdin.path());

    assert_eq!(
        direct_state.get("concern"),
        stdin_state.get("concern"),
        "direct and stdin concern fields must match",
    );
    assert_eq!(
        stdin_state.get("concern").and_then(|v| v.as_str()),
        Some("foundation"),
        "stdin form must yield concern=foundation",
    );
}

/// PC-012 / AC-002.3: `worktree-name foundation "demo project"` returns a
/// non-empty valid session-branch name string.
#[test]
fn worktree_name_foundation_returns_valid_slug() {
    let bin = common::binary_path();
    let output = Command::new(&bin)
        .args(["worktree-name", "foundation", "demo project"])
        .output()
        .expect("failed to execute worktree-name");

    assert_exit_zero(&output, "worktree-name foundation");
    let stdout = std::str::from_utf8(&output.stdout).expect("non-utf8 stdout");

    // Output is a JSON envelope: {"status":"pass","message":"ecc-session-..."}
    let value: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("worktree-name output must be JSON");
    let name = value
        .get("message")
        .and_then(|v| v.as_str())
        .expect("message field must be a string");

    assert!(
        name.starts_with("ecc-session-"),
        "worktree name must start with ecc-session-, got: {name}",
    );
    assert!(
        name.contains("demo-project") || name.contains("demo"),
        "worktree name should contain the feature slug; got: {name}",
    );
}

/// PC-013 / AC-002.4: Full FSM walk with `concern=foundation` —
/// init → transition plan → solution → implement → done. Each step exits 0 and
/// the final state.json preserves `concern=foundation`.
#[test]
fn foundation_concern_persists_through_full_fsm_walk() {
    let bin = common::binary_path();
    let temp_dir = tempfile::tempdir().unwrap();
    let env_key = "CLAUDE_PROJECT_DIR";

    let init = Command::new(&bin)
        .args(["init", "foundation", "e2e walk"])
        .env(env_key, temp_dir.path())
        .output()
        .expect("init");
    assert_exit_zero(&init, "init");

    // Each transition must exit 0 and preserve concern=foundation.
    // Note: transitions beyond `solution` require `--artifact <kind>` per
    // ecc-workflow's gating. We pass plan/solution/implement artifacts in order.
    let steps: &[&[&str]] = &[
        &["transition", "solution", "--artifact", "plan"],
        &["transition", "implement", "--artifact", "solution"],
        &["transition", "done", "--artifact", "implement"],
    ];

    for args in steps {
        let output = Command::new(&bin)
            .args(*args)
            .env(env_key, temp_dir.path())
            .output()
            .unwrap_or_else(|e| panic!("failed to execute {:?}: {e}", args));
        assert_exit_zero(&output, &format!("{:?}", args));

        // After each transition, concern must still be foundation.
        let state = read_state(temp_dir.path());
        assert_eq!(
            state.get("concern").and_then(|v| v.as_str()),
            Some("foundation"),
            "concern must remain 'foundation' after {:?}; state={state}",
            args,
        );
    }

    // Final check: phase is done, concern is still foundation.
    let final_state = read_state(temp_dir.path());
    assert_eq!(
        final_state.get("phase").and_then(|v| v.as_str()),
        Some("done"),
        "final phase must be done",
    );
    assert_eq!(
        final_state.get("concern").and_then(|v| v.as_str()),
        Some("foundation"),
        "final concern must be foundation",
    );
}
