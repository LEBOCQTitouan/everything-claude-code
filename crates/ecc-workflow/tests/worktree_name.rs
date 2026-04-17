mod common;

use std::process::{Command, Stdio};

// PC-022: positional round-trip with metachar/control-byte payload — AC-001.2b
// Invoke `ecc-workflow worktree-name dev <feature>` where feature contains
// METACHAR ∪ CONTROL bytes (NUL excluded per POSIX argv).
// Assert: stdout JSON message starts with "ecc-session-", exit 0.
#[test]
fn worktree_name_positional_round_trips() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    // Canonical metachar payload (NUL excluded per POSIX argv).
    // Covers: backtick, dquote, squote, dollar, backslash, CR, C0 controls, DEL.
    let payload = "feat-`-\"-'-$-\\-\r-\x01-\x1f-\x7f-end";

    let output = Command::new(&bin)
        .args(["worktree-name", "dev", payload])
        .output()
        .expect("failed to execute ecc-workflow worktree-name");

    assert_eq!(
        output.status.code(),
        Some(0),
        "expected exit 0, got: {:?}\nstdout: {}\nstderr: {}",
        output.status.code(),
        std::str::from_utf8(&output.stdout).unwrap_or("<non-utf8>"),
        std::str::from_utf8(&output.stderr).unwrap_or("<non-utf8>"),
    );

    let stdout = std::str::from_utf8(&output.stdout)
        .expect("stdout must be valid UTF-8")
        .trim();

    let value: serde_json::Value = serde_json::from_str(stdout)
        .unwrap_or_else(|e| panic!("stdout is not valid JSON: {e}\nstdout: {stdout}"));

    let message = value
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("missing 'message' field in stdout JSON: {value}"));

    assert!(
        message.starts_with("ecc-session-"),
        "expected stdout message to start with 'ecc-session-', got: {message}"
    );
}

// PC-023: stdin round-trip with metachar/control-byte payload — AC-001.2a
// Invoke `ecc-workflow worktree-name dev --feature-stdin` with same bytes piped via stdin.
// Assert: stdout JSON message starts with "ecc-session-", exit 0.
#[test]
fn worktree_name_stdin_round_trips() {
    use std::io::Write;

    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    // Same canonical metachar payload as PC-022, plus NUL (stdin can carry NUL).
    let payload: &[u8] = b"feat-`-\"-'-$-\\-\r-\x01-\x1f-\x7f-\x00-end";

    let mut child = Command::new(&bin)
        .args(["worktree-name", "dev", "--feature-stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn ecc-workflow worktree-name");

    child.stdin.as_mut().unwrap().write_all(payload).unwrap();
    // Drop stdin to signal EOF
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("failed to wait on child");

    assert_eq!(
        output.status.code(),
        Some(0),
        "expected exit 0, got: {:?}\nstdout: {}\nstderr: {}",
        output.status.code(),
        std::str::from_utf8(&output.stdout).unwrap_or("<non-utf8>"),
        std::str::from_utf8(&output.stderr).unwrap_or("<non-utf8>"),
    );

    let stdout = std::str::from_utf8(&output.stdout)
        .expect("stdout must be valid UTF-8")
        .trim();

    let value: serde_json::Value = serde_json::from_str(stdout)
        .unwrap_or_else(|e| panic!("stdout is not valid JSON: {e}\nstdout: {stdout}"));

    let message = value
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("missing 'message' field in stdout JSON: {value}"));

    assert!(
        message.starts_with("ecc-session-"),
        "expected stdout message to start with 'ecc-session-', got: {message}"
    );
}

// PC-024: TTY guard — AC-001.11
// Spawn `ecc-workflow worktree-name dev --feature-stdin` via rexpect PTY.
// The binary detects TTY on stdin and must exit non-zero.
//
// Timing note: use process_mut().wait() for accurate exit detection (avoids
// the 100ms PTY polling delay of exp_eof()).
#[test]
fn worktree_name_stdin_tty_rejected() {
    let bin_path = env!("CARGO_BIN_EXE_ecc-workflow");
    let cmd = format!("{bin_path} worktree-name dev --feature-stdin");

    let spawn_result = rexpect::spawn(&cmd, Some(2_000));
    let Ok(mut session) = spawn_result else {
        panic!("failed to spawn ecc-workflow via rexpect PTY");
    };

    // Use process_mut().wait() (blocking waitpid) for accurate exit status.
    let exit_status = session.process_mut().wait();
    // Drain remaining PTY output afterward (best-effort; ignore errors)
    let _merged_output = session.exp_eof().unwrap_or_default();

    let non_zero = !matches!(exit_status, Ok(rexpect::process::WaitStatus::Exited(_, 0)));

    assert!(
        non_zero,
        "expected non-zero exit when stdin is a TTY, but got exit 0"
    );
}
