//! Characterization tests for session hooks (PC-028, PC-029).
//!
//! These tests capture the current behavior of session_start and session_end
//! hooks so that the refactoring does not silently change behavior.

use assert_cmd::Command;

fn ecc_hook_cmd() -> Command {
    let mut cmd = Command::cargo_bin("ecc").expect("ecc binary not found");
    cmd.env("ECC_HOOK_PROFILE", "standard");
    cmd
}

/// PC-028: session_start hook returns exit 0 with output.
#[test]
fn session_start_characterization() {
    let stdin_payload = serde_json::json!({
        "session_id": "test-session-123",
        "transcript_path": "/tmp/test.jsonl",
        "cwd": "/tmp",
        "hook_event_name": "SessionStart",
        "source": "startup",
        "model": "test-model"
    });

    let result = ecc_hook_cmd()
        .args(["hook", "session:start", "standard"])
        .write_stdin(serde_json::to_string(&stdin_payload).unwrap())
        .output()
        .expect("failed to run session:start hook");

    // Session start should exit 0 (not block)
    assert!(
        result.status.success(),
        "session:start should exit 0, got: {:?}, stderr: {}",
        result.status,
        String::from_utf8_lossy(&result.stderr)
    );
}

/// PC-029: session_end hook returns exit 0.
#[test]
fn session_end_characterization() {
    let stdin_payload = serde_json::json!({
        "session_id": "test-session-123",
        "hook_event_name": "SessionEnd"
    });

    let result = ecc_hook_cmd()
        .args(["hook", "session:end:marker", "standard"])
        .write_stdin(serde_json::to_string(&stdin_payload).unwrap())
        .output()
        .expect("failed to run session:end hook");

    assert!(
        result.status.success(),
        "session:end:marker should exit 0, got: {:?}, stderr: {}",
        result.status,
        String::from_utf8_lossy(&result.stderr)
    );
}
