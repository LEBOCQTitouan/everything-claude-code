mod common;

use std::process::{Command, Stdio};

/// PC-021: Property-based round-trip for `--feature-stdin` with 1024 generated strings.
///
/// Generates strings via `proptest` (up to 4096 chars of printable/whitespace),
/// pipes each via `--feature-stdin` to `ecc-workflow init dev`, and asserts that
/// `state.json.feature == input_minus_single_trailing_lf`.
///
/// Marked `#[ignore]` because 1024 subprocess invocations are slow for default CI.
/// Run with: `cargo test --package ecc-workflow --test init init_stdin_property_round_trip -- --exact --ignored`
#[test]
#[ignore]
fn init_stdin_property_round_trip() {
    use proptest::prelude::*;
    use proptest::test_runner::{Config, TestRunner};
    use std::io::Write;

    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    let mut runner = TestRunner::new(Config {
        cases: 1024,
        ..Config::default()
    });

    // Use printable non-control chars + whitespace to stay within valid UTF-8 and
    // avoid the 64KB limit. The regex `(?s-u:\PC|\s){0,4096}` generates strings
    // composed of printable Unicode chars or whitespace, length 0..=4096.
    //
    // Note: proptest's string_regex uses the `regex` crate with Unicode mode off
    // (`-u`) so `\PC` means non-control ASCII. We fall back to a simpler strategy
    // if the regex engine rejects the pattern.
    let strategy = proptest::string::string_regex(r"(?s-u:\PC|\s){0,4096}").unwrap_or_else(|_| {
        // Fallback: printable ASCII only, max 4096 chars
        proptest::string::string_regex(r"\PC{0,4096}").unwrap()
    });

    runner
        .run(&strategy, |s: String| {
            let bytes = s.as_bytes();

            // Skip inputs that exceed the 64KB binary limit — those are rejected by
            // the binary with a different error path tested separately in PC-016.
            if bytes.len() > 64 * 1024 {
                return Ok(());
            }

            let temp_dir = tempfile::tempdir()
                .map_err(|e| TestCaseError::fail(format!("tempdir failed: {e}")))?;

            let mut child = Command::new(&bin)
                .args(["init", "dev", "--feature-stdin"])
                .env("CLAUDE_PROJECT_DIR", temp_dir.path())
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| TestCaseError::fail(format!("spawn failed: {e}")))?;

            child
                .stdin
                .as_mut()
                .unwrap()
                .write_all(bytes)
                .map_err(|e| TestCaseError::fail(format!("write_all failed: {e}")))?;
            drop(child.stdin.take());

            let output = child
                .wait_with_output()
                .map_err(|e| TestCaseError::fail(format!("wait_with_output failed: {e}")))?;

            // Inputs that are empty after trailing-LF strip will be rejected (exit 2).
            // Detect this case: input is either empty or "\n" only.
            let is_empty_after_strip =
                bytes.is_empty() || bytes == b"\n" || (bytes.len() == 1 && bytes[0] == b'\n');

            if is_empty_after_strip {
                // Must exit non-zero with "feature is empty" diagnostic
                prop_assert_ne!(
                    output.status.code(),
                    Some(0),
                    "expected non-zero exit for effectively-empty input {:?}",
                    s
                );
                return Ok(());
            }

            prop_assert_eq!(
                output.status.code(),
                Some(0),
                "expected exit 0 for input {:?}, stderr: {}",
                s,
                std::str::from_utf8(&output.stderr).unwrap_or("<non-utf8>")
            );

            let state_path = temp_dir.path().join(".claude/workflow/state.json");
            prop_assert!(
                state_path.exists(),
                "state.json not found for input {:?}",
                s
            );

            let content = std::fs::read_to_string(&state_path)
                .map_err(|e| TestCaseError::fail(format!("read state.json failed: {e}")))?;
            let value: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| TestCaseError::fail(format!("parse state.json failed: {e}")))?;

            let stored_feature = value
                .get("feature")
                .and_then(|v| v.as_str())
                .ok_or_else(|| TestCaseError::fail("missing 'feature' field in state.json"))?;

            // Compute expected: strip exactly one trailing LF if present.
            let expected: &[u8] = if bytes.last() == Some(&b'\n') {
                &bytes[..bytes.len() - 1]
            } else {
                bytes
            };

            prop_assert_eq!(
                stored_feature.as_bytes(),
                expected,
                "round-trip mismatch for input {:?}",
                s
            );

            Ok(())
        })
        .unwrap();
}

// PC-020: TTY guard via rexpect PTY — AC-001.11
// Spawning `ecc-workflow init dev --feature-stdin` with a real PTY gives the child
// a TTY as stdin. The binary must detect this and exit non-zero within 100ms.
//
// Timing note: rexpect's reader polls every 100ms; we use process_mut().wait() for
// precise exit timing instead of exp_eof(), then drain the PTY output afterward.
//
// rexpect merges stdout+stderr via PTY; the "stdin is a TTY" diagnostic (emitted to
// stderr by the binary) appears in the merged PTY output.
//
// Ignored by default: this test is inherently timing-sensitive (asserts 3 of 5
// trials exit in <100ms wall-clock) and fails under load or cold binary-cache
// conditions. The guarded behavior (non-zero exit on TTY stdin) is also
// covered by unit tests in the binary; this PTY-level assertion is valuable
// only in an unloaded CI environment. Run explicitly with
// `cargo test --test init init_stdin_tty_rejected -- --ignored`.
#[test]
#[ignore]
fn init_stdin_tty_rejected_within_100ms_median_of_5() {
    use std::time::{Duration, Instant};

    let bin_path = env!("CARGO_BIN_EXE_ecc-workflow");
    let cmd = format!("{bin_path} init dev --feature-stdin");

    let mut successes = 0u32;

    for _ in 0..5 {
        let start = Instant::now();
        let spawn_result = rexpect::spawn(&cmd, Some(2_000));
        let Ok(mut session) = spawn_result else {
            continue;
        };
        // Use process_mut().wait() (blocking waitpid) for accurate exit timing.
        // This does NOT go through the 100ms-polling PTY reader.
        let exit_status = session.process_mut().wait();
        let elapsed = start.elapsed();
        // Drain remaining PTY output afterward (best-effort; ignore errors)
        let _merged_output = session.exp_eof().unwrap_or_default();

        let non_zero = !matches!(exit_status, Ok(rexpect::process::WaitStatus::Exited(_, 0)));
        if non_zero && elapsed < Duration::from_millis(100) {
            successes += 1;
        }
    }

    assert!(
        successes >= 3,
        "At least 3 of 5 trials must satisfy exit!=0 AND elapsed<100ms; got {successes}"
    );
}

#[test]
fn init_positional_round_trips_metachars() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    // Canonical metachar payload (NUL excluded per POSIX argv).
    // Covers: backtick, dquote, squote, dollar, backslash, CR, C0 controls, DEL.
    let payload = "feat-`-\"-'-$-\\-\r-\x01-\x1f-\x7f-end";

    let temp_dir = tempfile::tempdir().unwrap();

    let output = Command::new(&bin)
        .args(["init", "dev", payload])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .output()
        .expect("failed to execute ecc-workflow init");

    assert_eq!(
        output.status.code(),
        Some(0),
        "expected exit 0, got: {:?}\nstdout: {}\nstderr: {}",
        output.status.code(),
        std::str::from_utf8(&output.stdout).unwrap_or(""),
        std::str::from_utf8(&output.stderr).unwrap_or(""),
    );

    let state_path = temp_dir.path().join(".claude/workflow/state.json");
    assert!(
        state_path.exists(),
        "state.json was not created at {:?}",
        state_path
    );

    let content = std::fs::read_to_string(&state_path).expect("failed to read state.json");
    let value: serde_json::Value = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("state.json is not valid JSON: {e}\ncontent: {content}"));

    let stored_feature = value
        .get("feature")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("missing 'feature' field in state.json: {value}"));

    assert_eq!(
        stored_feature, payload,
        "round-trip failed: stored feature does not match input payload\nexpected: {payload:?}\ngot:      {stored_feature:?}"
    );
}

#[test]
fn init_feature_stdin_without_concern_fails_cleanly() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    // Invoking `init --feature-stdin` WITHOUT the required `concern` positional
    // must be rejected by clap with exit code 2. No stdin read should occur.
    let output = Command::new(&bin)
        .args(["init", "--feature-stdin"])
        .output()
        .expect("failed to execute ecc-workflow init");

    assert_eq!(
        output.status.code(),
        Some(2),
        "expected exit code 2 for missing required positional, got: {:?}\nstdout: {}\nstderr: {}",
        output.status.code(),
        std::str::from_utf8(&output.stdout).unwrap_or(""),
        std::str::from_utf8(&output.stderr).unwrap_or(""),
    );

    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");
    assert!(
        stderr.to_lowercase().contains("concern") || stderr.to_lowercase().contains("required"),
        "expected stderr to mention 'concern' or 'required', got: {stderr}"
    );
}

#[test]
fn init_stdin_and_positional_exits_two() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    let temp_dir = tempfile::tempdir().unwrap();

    // Passing BOTH positional feature AND --feature-stdin must be rejected by clap
    // with exit code 2 and a conflict diagnostic on stderr.
    let output = Command::new(&bin)
        .args(["init", "dev", "feat", "--feature-stdin"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .output()
        .expect("failed to execute ecc-workflow init");

    assert_eq!(
        output.status.code(),
        Some(2),
        "expected exit code 2 for arg conflict, got: {:?}\nstdout: {}\nstderr: {}",
        output.status.code(),
        std::str::from_utf8(&output.stdout).unwrap_or(""),
        std::str::from_utf8(&output.stderr).unwrap_or(""),
    );

    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");
    assert!(
        stderr.contains("cannot be used with"),
        "expected clap conflict diagnostic in stderr, got: {stderr}"
    );
}

#[test]
fn missing_state_exits_zero_with_warning() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    // Create a temp dir with NO state.json inside
    let temp_dir = tempfile::tempdir().unwrap();

    let output = std::process::Command::new(&bin)
        .args(["transition", "solution"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .output()
        .expect("failed to execute ecc-workflow transition");

    // Exit code must be 0
    assert_eq!(
        output.status.code(),
        Some(0),
        "expected exit 0 when state.json is missing, got: {:?}",
        output.status.code()
    );

    // stderr must contain JSON with status "warn"
    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");
    assert!(
        !stderr.trim().is_empty(),
        "expected non-empty stderr when state.json is missing"
    );

    let value: serde_json::Value = serde_json::from_str(stderr.trim())
        .unwrap_or_else(|e| panic!("stderr is not valid JSON: {e}\nstderr was: {stderr}"));

    let status = value
        .get("status")
        .unwrap_or_else(|| panic!("JSON missing 'status' field: {value}"))
        .as_str()
        .unwrap_or_else(|| panic!("'status' is not a string: {value}"));

    assert_eq!(
        status, "warn",
        "expected status 'warn' when state.json missing, got '{status}'"
    );

    let message = value
        .get("message")
        .unwrap_or_else(|| panic!("JSON missing 'message' field: {value}"))
        .as_str()
        .unwrap_or_else(|| panic!("'message' is not a string: {value}"));

    // The message must indicate the state is missing / workflow not initialized,
    // not just "not yet implemented"
    assert!(
        message.to_lowercase().contains("state")
            || message.to_lowercase().contains("not initialized")
            || message.to_lowercase().contains("workflow"),
        "warn message should indicate missing state, got: '{message}'"
    );
}

#[test]
fn output_is_structured_json() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    let temp_dir = tempfile::tempdir().unwrap();

    // Test: init subcommand produces structured JSON
    let output = Command::new(&bin)
        .args(["init", "dev", "test-feature"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .output()
        .expect("failed to execute ecc-workflow init");

    common::assert_structured_json_output(&output);
}

#[test]
fn init_creates_state_json() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    let temp_dir = tempfile::tempdir().unwrap();

    // Run init
    let output = Command::new(&bin)
        .args(["init", "dev", "test feature"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .output()
        .expect("failed to execute ecc-workflow init");

    assert_eq!(
        output.status.code(),
        Some(0),
        "expected exit 0, got: {:?}\nstdout: {}\nstderr: {}",
        output.status.code(),
        std::str::from_utf8(&output.stdout).unwrap_or(""),
        std::str::from_utf8(&output.stderr).unwrap_or(""),
    );

    // Read state.json from the temp dir
    let state_path = temp_dir.path().join(".claude/workflow/state.json");
    assert!(
        state_path.exists(),
        "state.json was not created at {:?}",
        state_path
    );

    let content = std::fs::read_to_string(&state_path).expect("failed to read state.json");

    let value: serde_json::Value = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("state.json is not valid JSON: {e}\ncontent: {content}"));

    // concern == "dev"
    assert_eq!(
        value.get("concern").and_then(|v| v.as_str()),
        Some("dev"),
        "expected concern 'dev', got: {:?}",
        value.get("concern")
    );

    // phase == "plan"
    assert_eq!(
        value.get("phase").and_then(|v| v.as_str()),
        Some("plan"),
        "expected phase 'plan', got: {:?}",
        value.get("phase")
    );

    // feature == "test feature"
    assert_eq!(
        value.get("feature").and_then(|v| v.as_str()),
        Some("test feature"),
        "expected feature 'test feature', got: {:?}",
        value.get("feature")
    );

    // started_at is ISO 8601 format: YYYY-MM-DDTHH:MM:SSZ
    let started_at = value
        .get("started_at")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("missing 'started_at' field in state.json"));
    assert!(
        started_at.len() == 20 && started_at.ends_with('Z') && started_at.contains('T'),
        "started_at must be ISO 8601 UTC (YYYY-MM-DDTHH:MM:SSZ), got: '{started_at}'"
    );

    // toolchain fields are all null
    let toolchain = value
        .get("toolchain")
        .unwrap_or_else(|| panic!("missing 'toolchain' field in state.json"));
    assert_eq!(
        toolchain.get("test"),
        Some(&serde_json::Value::Null),
        "expected toolchain.test == null"
    );
    assert_eq!(
        toolchain.get("lint"),
        Some(&serde_json::Value::Null),
        "expected toolchain.lint == null"
    );
    assert_eq!(
        toolchain.get("build"),
        Some(&serde_json::Value::Null),
        "expected toolchain.build == null"
    );

    // artifacts fields are all null
    let artifacts = value
        .get("artifacts")
        .unwrap_or_else(|| panic!("missing 'artifacts' field in state.json"));
    for key in &[
        "plan",
        "solution",
        "implement",
        "campaign_path",
        "spec_path",
        "design_path",
        "tasks_path",
    ] {
        assert_eq!(
            artifacts.get(*key),
            Some(&serde_json::Value::Null),
            "expected artifacts.{key} == null"
        );
    }

    // completed == []
    let completed = value
        .get("completed")
        .unwrap_or_else(|| panic!("missing 'completed' field in state.json"));
    assert_eq!(
        completed,
        &serde_json::Value::Array(vec![]),
        "expected completed == []"
    );
}

#[test]
fn init_stdin_round_trips_metachars_and_nul() {
    use std::io::Write;

    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    let temp_dir = tempfile::tempdir().unwrap();

    // Canonical payload: backtick, dquote, squote, dollar, backslash, CR,
    // C0 control U+0001, U+001F, DEL U+007F, NUL U+0000 — no trailing LF.
    let payload: &[u8] = b"feat-`-\"-'-$-\\-\r-\x01-\x1f-\x7f-\x00-end";

    let mut child = Command::new(&bin)
        .args(["init", "dev", "--feature-stdin"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn ecc-workflow init");

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

    let state_path = temp_dir.path().join(".claude/workflow/state.json");
    assert!(
        state_path.exists(),
        "state.json was not created at {:?}",
        state_path
    );

    let content = std::fs::read_to_string(&state_path).expect("failed to read state.json");
    let value: serde_json::Value = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("state.json is not valid JSON: {e}\ncontent: {content}"));

    let stored_feature = value
        .get("feature")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("missing 'feature' field in state.json: {value}"));

    // Compare as bytes: payload has no trailing LF, so stored bytes must equal payload exactly.
    assert_eq!(
        stored_feature.as_bytes(),
        payload,
        "round-trip failed: stored feature bytes do not match input payload\nexpected: {:?}\ngot:      {:?}",
        payload,
        stored_feature.as_bytes(),
    );
}

#[test]
fn init_stdin_invalid_utf8_rejected_no_state() {
    use std::io::Write;

    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    let temp_dir = tempfile::tempdir().unwrap();

    // Pipe a lone invalid UTF-8 byte (0xFF) via --feature-stdin.
    let mut child = Command::new(&bin)
        .args(["init", "dev", "--feature-stdin"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn ecc-workflow init");

    child.stdin.as_mut().unwrap().write_all(b"\xff").unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("failed to wait on child");

    // Assert exit code is non-zero
    assert_ne!(
        output.status.code(),
        Some(0),
        "expected non-zero exit for invalid UTF-8 stdin, got exit 0\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&output.stdout).unwrap_or("<non-utf8>"),
        std::str::from_utf8(&output.stderr).unwrap_or("<non-utf8>"),
    );

    // Assert stderr contains the pinned diagnostic
    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");
    assert!(
        stderr.contains("invalid UTF-8 on stdin"),
        "expected stderr to contain 'invalid UTF-8 on stdin', got: {stderr}"
    );

    // Assert state.json was NOT written
    let state_path = temp_dir.path().join(".claude/workflow/state.json");
    assert!(
        !state_path.exists(),
        "state.json must NOT exist after invalid UTF-8 failure, found at {:?}",
        state_path
    );
}

#[test]
fn init_stdin_65537_bytes_rejected() {
    use std::io::Write;

    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    let temp_dir = tempfile::tempdir().unwrap();

    // 64KB + 1 byte of valid ASCII — must be rejected with the size-cap diagnostic.
    let payload = vec![b'a'; 64 * 1024 + 1];

    let mut child = Command::new(&bin)
        .args(["init", "dev", "--feature-stdin"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn ecc-workflow init");

    child.stdin.as_mut().unwrap().write_all(&payload).unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("failed to wait on child");

    // Assert exit code is non-zero
    assert_ne!(
        output.status.code(),
        Some(0),
        "expected non-zero exit for 64KB+1 stdin, got exit 0\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&output.stdout).unwrap_or("<non-utf8>"),
        std::str::from_utf8(&output.stderr).unwrap_or("<non-utf8>"),
    );

    // Assert stderr contains the pinned size-cap diagnostic
    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");
    assert!(
        stderr.contains("stdin exceeds 64KB limit"),
        "expected stderr to contain 'stdin exceeds 64KB limit', got: {stderr}"
    );

    // Assert state.json was NOT written
    let state_path = temp_dir.path().join(".claude/workflow/state.json");
    assert!(
        !state_path.exists(),
        "state.json must NOT exist after size-cap rejection, found at {:?}",
        state_path
    );
}

#[test]
fn init_stdin_65536_bytes_accepted() {
    use std::io::Write;

    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    let temp_dir = tempfile::tempdir().unwrap();

    // Exactly 64KB of valid ASCII — must be accepted (boundary case, PC-018).
    let payload = vec![b'a'; 64 * 1024];

    let mut child = Command::new(&bin)
        .args(["init", "dev", "--feature-stdin"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn ecc-workflow init");

    child.stdin.as_mut().unwrap().write_all(&payload).unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("failed to wait on child");

    assert_eq!(
        output.status.code(),
        Some(0),
        "expected exit 0 for exactly 64KB stdin, got: {:?}\nstdout: {}\nstderr: {}",
        output.status.code(),
        std::str::from_utf8(&output.stdout).unwrap_or("<non-utf8>"),
        std::str::from_utf8(&output.stderr).unwrap_or("<non-utf8>"),
    );

    // Assert state.json was written
    let state_path = temp_dir.path().join(".claude/workflow/state.json");
    assert!(
        state_path.exists(),
        "state.json must exist after successful 64KB stdin init, not found at {:?}",
        state_path
    );

    let content = std::fs::read_to_string(&state_path).expect("failed to read state.json");
    let value: serde_json::Value = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("state.json is not valid JSON: {e}\ncontent: {content}"));

    let stored_feature = value
        .get("feature")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("missing 'feature' field in state.json: {value}"));

    // The stored feature must be exactly 65_536 'a' characters
    assert_eq!(
        stored_feature.len(),
        64 * 1024,
        "expected stored feature length {}, got {}",
        64 * 1024,
        stored_feature.len()
    );
    assert!(
        stored_feature.bytes().all(|b| b == b'a'),
        "expected stored feature to contain only 'a' chars"
    );
}

#[test]
fn init_stdin_trailing_newline_policy() {
    use std::io::Write;

    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    // Helper: spawn binary with given stdin bytes, return stored feature string.
    let run_scenario = |stdin_bytes: &[u8]| -> String {
        let temp_dir = tempfile::tempdir().unwrap();

        let mut child = Command::new(&bin)
            .args(["init", "dev", "--feature-stdin"])
            .env("CLAUDE_PROJECT_DIR", temp_dir.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to spawn ecc-workflow init");

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(stdin_bytes)
            .unwrap();
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

        let state_path = temp_dir.path().join(".claude/workflow/state.json");
        let content = std::fs::read_to_string(&state_path)
            .unwrap_or_else(|e| panic!("failed to read state.json: {e}"));
        let value: serde_json::Value = serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("state.json is not valid JSON: {e}"));
        value
            .get("feature")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("missing 'feature' field in state.json"))
            .to_owned()
    };

    // Sub-assertion 1: single trailing LF is stripped.
    assert_eq!(
        run_scenario(b"foo\n"),
        "foo",
        "single trailing LF must be stripped"
    );

    // Sub-assertion 2: double trailing LF — only the last LF is stripped.
    assert_eq!(
        run_scenario(b"foo\n\n"),
        "foo\n",
        "only the final trailing LF must be stripped, leaving one LF"
    );

    // Sub-assertion 3: CRLF — only the trailing LF is stripped, CR is preserved.
    assert_eq!(
        run_scenario(b"foo\r\n"),
        "foo\r",
        "trailing LF stripped but CR must be preserved"
    );
}

/// PC-028: Canonical payload survives `sh -c` with env-var + stdin pipe.
///
/// Simulates the post-fix template invocation pattern: pass the feature text via an
/// environment variable, then pipe it to `ecc-workflow init dev --feature-stdin` using
/// `printf %s "$FEATURE_PAYLOAD"`. This bypasses shell-argv interpolation entirely and
/// verifies AC-001.1b (shell-proxy behavioral test).
#[test]
fn canonical_payload_survives_sh_c_with_env_var() {
    let bin = env!("CARGO_BIN_EXE_ecc-workflow");
    // Canonical metachar payload (no NUL, no trailing LF):
    // backtick, dquote, squote, dollar, backslash, CR, C0 controls, DEL
    let canonical_payload = "feat-`-\"-'-$-\\-\r-\x01-\x1f-\x7f-end";

    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap().to_owned();

    // Use `printf %s "$FEATURE_PAYLOAD" | cmd` — portable across sh/bash/dash/zsh.
    // This avoids `<<<` which is bash-only (here-string).
    let sh_cmd = format!(r#"printf %s "$FEATURE_PAYLOAD" | {bin} init dev --feature-stdin"#);

    let output = std::process::Command::new("sh")
        .env("FEATURE_PAYLOAD", canonical_payload)
        .env("CLAUDE_PROJECT_DIR", &temp_path)
        .args(["-c", &sh_cmd])
        .output()
        .expect("sh must spawn");

    assert!(
        output.status.success(),
        "sh -c with env var + stdin must succeed\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let state_path = temp_dir.path().join(".claude/workflow/state.json");
    assert!(
        state_path.exists(),
        "state.json must be created at {:?}",
        state_path
    );

    let content = std::fs::read_to_string(&state_path).expect("failed to read state.json");
    let value: serde_json::Value =
        serde_json::from_str(&content).expect("state.json must be valid JSON");

    let stored_feature = value
        .get("feature")
        .and_then(|v| v.as_str())
        .expect("missing 'feature' field in state.json");

    assert_eq!(
        stored_feature, canonical_payload,
        "round-trip failed via sh-c+env-var: stored feature does not match\nexpected: {canonical_payload:?}\ngot:      {stored_feature:?}"
    );
}

#[test]
fn init_empty_stdin_exits_two() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    let temp_dir = tempfile::tempdir().unwrap();

    // Spawn with piped stdin so we can close it immediately (empty EOF)
    let mut child = Command::new(&bin)
        .args(["init", "dev", "--feature-stdin"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn ecc-workflow init");

    // Close stdin immediately — produces empty EOF
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("failed to wait on child");

    // Assert exit code is non-zero (block = exit 2)
    assert_ne!(
        output.status.code(),
        Some(0),
        "expected non-zero exit for empty stdin, got exit 0\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&output.stdout).unwrap_or(""),
        std::str::from_utf8(&output.stderr).unwrap_or(""),
    );

    // Assert stderr contains the pinned diagnostic from FeatureInputError::Empty
    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");
    assert!(
        stderr.contains("feature is empty"),
        "expected stderr to contain 'feature is empty', got: {stderr}"
    );

    // Assert state.json was NOT written
    let state_path = temp_dir.path().join(".claude/workflow/state.json");
    assert!(
        !state_path.exists(),
        "state.json must NOT exist after empty-stdin failure, found at {:?}",
        state_path
    );
}
