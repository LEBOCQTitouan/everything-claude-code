//! Integration tests for `ecc hook` behavior (PC-043, PC-044).

use assert_cmd::Command;

fn ecc_cmd() -> Command {
    Command::cargo_bin("ecc").expect("ecc binary not found")
}

/// PC-043: `ecc hook check:hook:enabled` returns exit 0.
#[test]
fn check_hook_enabled_parity() {
    let output = ecc_cmd()
        .args(["hook", "check:hook:enabled", "standard"])
        .write_stdin("{}")
        .output()
        .expect("ecc hook failed");

    assert!(
        output.status.success(),
        "ecc hook check:hook:enabled should exit 0, got: {:?}, stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

/// PC-044: `ecc hook` handles unknown hook IDs gracefully (passthrough).
#[test]
fn thin_wrapper_fallback() {
    let output = ecc_cmd()
        .args(["hook", "test:nonexistent:hook", "standard"])
        .write_stdin("{}")
        .output()
        .expect("ecc hook failed");

    // Unknown hooks should exit 0 (passthrough with warning)
    assert!(
        output.status.success(),
        "ecc hook should pass for unknown hook, got: {:?}",
        output.status
    );
}
