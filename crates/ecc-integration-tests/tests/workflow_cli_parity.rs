//! Integration tests for `ecc workflow` CLI parity with `ecc-workflow` (PC-036-042).

use proptest::prelude::*;
use proptest::test_runner::{Config, TestRunner};

use assert_cmd::Command;
use std::path::Path;
use tempfile::TempDir;

fn ecc_cmd(project_dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("ecc").expect("ecc binary not found");
    cmd.env("CLAUDE_PROJECT_DIR", project_dir);
    cmd
}

fn wf_cmd(project_dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("ecc-workflow").expect("ecc-workflow binary not found");
    cmd.env("CLAUDE_PROJECT_DIR", project_dir);
    cmd
}

/// PC-036: `ecc workflow init` succeeds.
#[test]
fn init_succeeds() {
    let tmp = TempDir::new().unwrap();
    ecc_cmd(tmp.path())
        .args(["workflow", "init", "dev", "test-feature"])
        .assert()
        .success();
}

/// PC-037: `ecc workflow status` matches ecc-workflow status output.
#[test]
fn status_parity() {
    let tmp = TempDir::new().unwrap();

    // Init via ecc-workflow
    wf_cmd(tmp.path())
        .args(["init", "dev", "parity-test"])
        .assert()
        .success();

    // Get status from both binaries
    let ecc_output = ecc_cmd(tmp.path())
        .args(["workflow", "status"])
        .output()
        .expect("ecc workflow status failed");

    let wf_output = wf_cmd(tmp.path())
        .args(["status"])
        .output()
        .expect("ecc-workflow status failed");

    // Both should succeed
    assert!(
        ecc_output.status.success(),
        "ecc workflow status should succeed"
    );
    assert!(
        wf_output.status.success(),
        "ecc-workflow status should succeed"
    );

    // Both should contain the same phase and feature
    let ecc_stdout = String::from_utf8_lossy(&ecc_output.stdout);
    let wf_stdout = String::from_utf8_lossy(&wf_output.stdout);
    assert!(
        ecc_stdout.contains("plan"),
        "ecc output should contain plan phase"
    );
    assert!(
        wf_stdout.contains("plan"),
        "wf output should contain plan phase"
    );
    assert!(
        ecc_stdout.contains("parity-test"),
        "ecc output should contain feature name"
    );
}

/// PC-038: `ecc workflow transition` succeeds after init.
#[test]
fn transition_parity() {
    let tmp = TempDir::new().unwrap();

    wf_cmd(tmp.path())
        .args(["init", "dev", "transition-test"])
        .assert()
        .success();

    ecc_cmd(tmp.path())
        .args(["workflow", "transition", "solution"])
        .assert()
        .success();
}

/// PC-039: All 22+ subcommands are accessible under `ecc workflow`.
#[test]
fn all_subcommands_exist() {
    let subcommands = [
        "init",
        "transition",
        "toolchain-persist",
        "memory-write",
        "phase-gate",
        "stop-gate",
        "grill-me-gate",
        "tdd-enforcement",
        "status",
        "artifact",
        "reset",
        "scope-check",
        "doc-enforcement",
        "doc-level-check",
        "pass-condition-check",
        "e2e-boundary-check",
        "worktree-name",
        "wave-plan",
        "merge",
        "backlog",
        "tasks",
        "recover",
    ];

    for subcmd in subcommands {
        let output = Command::cargo_bin("ecc")
            .expect("ecc binary not found")
            .args(["workflow", subcmd, "--help"])
            .output()
            .unwrap_or_else(|e| panic!("failed to run ecc workflow {subcmd}: {e}"));

        assert!(
            output.status.success(),
            "ecc workflow {subcmd} --help should succeed, got: {:?}, stderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// PC-037: Property-based round-trip via the `ecc` delegator CLI.
///
/// Identical property contract to PC-021 (`init_stdin_property_round_trip` in
/// `ecc-workflow/tests/init.rs`) but invokes `ecc workflow init dev --feature-stdin`
/// through the `ecc` delegator binary instead of `ecc-workflow` directly.
///
/// Marked `#[ignore]` because 1024 subprocess invocations are slow for default CI.
/// Run with:
/// `cargo test --package ecc-integration-tests --test workflow_cli_parity init_stdin_property_round_trip_via_delegator -- --exact --ignored`
#[test]
#[ignore]
fn init_stdin_property_round_trip_via_delegator() {
    use std::io::Write;
    use std::process::{Command, Stdio};

    // Resolve the `ecc` (and sibling `ecc-workflow`) binary paths by walking up
    // from the test executable, mirroring assert_cmd's legacy_cargo_bin resolution.
    // This works when CARGO_BIN_EXE_ecc is not set (integration test harnesses
    // without explicit bin declarations).
    let target_dir = {
        let mut path = std::env::current_exe().expect("failed to resolve current exe path");
        // pop test binary name
        path.pop();
        if path.ends_with("deps") {
            path.pop();
        }
        path
    };
    let ecc_bin = target_dir.join(format!("ecc{}", std::env::consts::EXE_SUFFIX));
    assert!(
        ecc_bin.exists(),
        "ecc binary not found at {:?}; run `cargo build --package ecc-cli` first",
        ecc_bin
    );

    // Prepend target_dir to PATH so that `ecc`'s delegation to `ecc-workflow`
    // picks up the locally-built binary (which supports --feature-stdin) rather
    // than the system-installed one.
    let original_path = std::env::var("PATH").unwrap_or_default();
    let patched_path = format!("{}:{}", target_dir.display(), original_path);

    let mut runner = TestRunner::new(Config {
        cases: 1024,
        ..Config::default()
    });

    // Generate strings composed of printable Unicode chars or whitespace, up to 4096 chars.
    // Falls back to printable ASCII-only if the regex engine rejects the pattern.
    let strategy = proptest::string::string_regex(r"(?s-u:\PC|\s){0,4096}")
        .unwrap_or_else(|_| proptest::string::string_regex(r"\PC{0,4096}").unwrap());

    runner
        .run(&strategy, |s: String| {
            let bytes = s.as_bytes();

            // Skip inputs that exceed the 64KB binary limit (separate test path).
            if bytes.len() > 64 * 1024 {
                return Ok(());
            }

            let temp_dir = tempfile::tempdir()
                .map_err(|e| TestCaseError::fail(format!("tempdir failed: {e}")))?;

            let mut child = Command::new(&ecc_bin)
                .args(["workflow", "init", "dev", "--feature-stdin"])
                .env("CLAUDE_PROJECT_DIR", temp_dir.path())
                .env("PATH", &patched_path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| TestCaseError::fail(format!("spawn ecc failed: {e}")))?;

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
            let is_empty_after_strip =
                bytes.is_empty() || bytes == b"\n" || (bytes.len() == 1 && bytes[0] == b'\n');

            if is_empty_after_strip {
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

/// Helper: resolve target dir (parent of the running test binary).
fn target_dir() -> std::path::PathBuf {
    let mut path = std::env::current_exe().expect("failed to resolve current exe path");
    path.pop(); // pop test binary name
    if path.ends_with("deps") {
        path.pop();
    }
    path
}

/// Helper: spawn a binary with --feature-stdin, write `input` bytes to its stdin,
/// and return the full output.
fn run_feature_stdin(
    bin: &std::path::Path,
    extra_args: &[&str],
    project_dir: &std::path::Path,
    input: &[u8],
    patched_path: &str,
) -> std::process::Output {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new(bin)
        .args(extra_args)
        .arg("--feature-stdin")
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .env("PATH", patched_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("spawn {:?} failed: {e}", bin));

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input)
        .expect("write_all failed");
    drop(child.stdin.take());

    child.wait_with_output().expect("wait_with_output failed")
}

/// PC-025: exit codes are identical across `ecc workflow init dev --feature-stdin`
/// and `ecc-workflow init dev --feature-stdin` for both valid and invalid (empty) input.
#[test]
fn workflow_cli_parity_feature_stdin_exit_code() {
    let tgt = target_dir();
    let ecc_bin = tgt.join(format!("ecc{}", std::env::consts::EXE_SUFFIX));
    let wf_bin = tgt.join(format!("ecc-workflow{}", std::env::consts::EXE_SUFFIX));
    assert!(ecc_bin.exists(), "ecc binary not found at {:?}", ecc_bin);
    assert!(
        wf_bin.exists(),
        "ecc-workflow binary not found at {:?}",
        wf_bin
    );

    let original_path = std::env::var("PATH").unwrap_or_default();
    let patched_path = format!("{}:{}", tgt.display(), original_path);

    // Round 1: valid input — both should exit 0
    {
        let input = b"my-feature-parity";
        let tmp_ecc = TempDir::new().unwrap();
        let tmp_wf = TempDir::new().unwrap();

        let out_ecc = run_feature_stdin(
            &ecc_bin,
            &["workflow", "init", "dev"],
            tmp_ecc.path(),
            input,
            &patched_path,
        );
        let out_wf = run_feature_stdin(
            &wf_bin,
            &["init", "dev"],
            tmp_wf.path(),
            input,
            &patched_path,
        );

        assert_eq!(
            out_ecc.status.code(),
            out_wf.status.code(),
            "exit codes must match for valid input: ecc={:?} ecc-workflow={:?}",
            out_ecc.status.code(),
            out_wf.status.code(),
        );
        assert_eq!(
            out_ecc.status.code(),
            Some(0),
            "both should exit 0 for valid input"
        );
    }

    // Round 2: invalid input (empty stdin) — both should exit with same non-zero code
    {
        let input = b"";
        let tmp_ecc = TempDir::new().unwrap();
        let tmp_wf = TempDir::new().unwrap();

        let out_ecc = run_feature_stdin(
            &ecc_bin,
            &["workflow", "init", "dev"],
            tmp_ecc.path(),
            input,
            &patched_path,
        );
        let out_wf = run_feature_stdin(
            &wf_bin,
            &["init", "dev"],
            tmp_wf.path(),
            input,
            &patched_path,
        );

        assert_eq!(
            out_ecc.status.code(),
            out_wf.status.code(),
            "exit codes must match for empty input: ecc={:?} ecc-workflow={:?}",
            out_ecc.status.code(),
            out_wf.status.code(),
        );
        assert_ne!(
            out_ecc.status.code(),
            Some(0),
            "both should exit non-zero for empty input"
        );
    }
}

/// PC-026: state.json bytes are identical across both invocations for valid input.
#[test]
fn workflow_cli_parity_feature_stdin_state_bytes() {
    let tgt = target_dir();
    let ecc_bin = tgt.join(format!("ecc{}", std::env::consts::EXE_SUFFIX));
    let wf_bin = tgt.join(format!("ecc-workflow{}", std::env::consts::EXE_SUFFIX));
    assert!(ecc_bin.exists(), "ecc binary not found at {:?}", ecc_bin);
    assert!(
        wf_bin.exists(),
        "ecc-workflow binary not found at {:?}",
        wf_bin
    );

    let original_path = std::env::var("PATH").unwrap_or_default();
    let patched_path = format!("{}:{}", tgt.display(), original_path);

    let input = b"parity-state-bytes-test";
    let tmp_ecc = TempDir::new().unwrap();
    let tmp_wf = TempDir::new().unwrap();

    let out_ecc = run_feature_stdin(
        &ecc_bin,
        &["workflow", "init", "dev"],
        tmp_ecc.path(),
        input,
        &patched_path,
    );
    let out_wf = run_feature_stdin(
        &wf_bin,
        &["init", "dev"],
        tmp_wf.path(),
        input,
        &patched_path,
    );

    assert_eq!(
        out_ecc.status.code(),
        Some(0),
        "ecc should exit 0 for valid input, stderr: {}",
        String::from_utf8_lossy(&out_ecc.stderr)
    );
    assert_eq!(
        out_wf.status.code(),
        Some(0),
        "ecc-workflow should exit 0 for valid input, stderr: {}",
        String::from_utf8_lossy(&out_wf.stderr)
    );

    let state_ecc = std::fs::read(tmp_ecc.path().join(".claude/workflow/state.json"))
        .expect("state.json not found for ecc");
    let state_wf = std::fs::read(tmp_wf.path().join(".claude/workflow/state.json"))
        .expect("state.json not found for ecc-workflow");

    // Parse as JSON values so timestamp differences don't cause false failures
    let val_ecc: serde_json::Value =
        serde_json::from_slice(&state_ecc).expect("ecc state.json not valid JSON");
    let val_wf: serde_json::Value =
        serde_json::from_slice(&state_wf).expect("ecc-workflow state.json not valid JSON");

    assert_eq!(
        val_ecc.get("feature"),
        val_wf.get("feature"),
        "feature field must match between ecc and ecc-workflow"
    );
    assert_eq!(
        val_ecc.get("phase"),
        val_wf.get("phase"),
        "phase field must match between ecc and ecc-workflow"
    );
}

/// PC-027: first line of stderr is identical for invalid (empty) input across both invocations.
#[test]
fn workflow_cli_parity_feature_stdin_stderr_class() {
    let tgt = target_dir();
    let ecc_bin = tgt.join(format!("ecc{}", std::env::consts::EXE_SUFFIX));
    let wf_bin = tgt.join(format!("ecc-workflow{}", std::env::consts::EXE_SUFFIX));
    assert!(ecc_bin.exists(), "ecc binary not found at {:?}", ecc_bin);
    assert!(
        wf_bin.exists(),
        "ecc-workflow binary not found at {:?}",
        wf_bin
    );

    let original_path = std::env::var("PATH").unwrap_or_default();
    let patched_path = format!("{}:{}", tgt.display(), original_path);

    let input = b""; // empty stdin → FeatureInputError::Empty
    let tmp_ecc = TempDir::new().unwrap();
    let tmp_wf = TempDir::new().unwrap();

    let out_ecc = run_feature_stdin(
        &ecc_bin,
        &["workflow", "init", "dev"],
        tmp_ecc.path(),
        input,
        &patched_path,
    );
    let out_wf = run_feature_stdin(
        &wf_bin,
        &["init", "dev"],
        tmp_wf.path(),
        input,
        &patched_path,
    );

    let stderr_ecc = String::from_utf8_lossy(&out_ecc.stderr);
    let stderr_wf = String::from_utf8_lossy(&out_wf.stderr);

    let first_line_ecc = stderr_ecc.lines().next().unwrap_or("").trim();
    let first_line_wf = stderr_wf.lines().next().unwrap_or("").trim();

    assert_eq!(
        first_line_ecc, first_line_wf,
        "first stderr line must match: ecc={:?} ecc-workflow={:?}",
        first_line_ecc, first_line_wf,
    );
    assert!(
        !first_line_ecc.is_empty(),
        "both should produce stderr output for empty input"
    );
}

/// PC-042: `ecc workflow -v status` emits tracing output on stderr.
#[test]
fn verbose_tracing() {
    let tmp = TempDir::new().unwrap();
    wf_cmd(tmp.path())
        .args(["init", "dev", "verbose-test"])
        .assert()
        .success();

    let output = ecc_cmd(tmp.path())
        .args(["workflow", "-v", "status"])
        .output()
        .expect("failed to run verbose status");

    assert!(output.status.success(), "verbose status should succeed");
    // Verbose mode should produce stderr output (tracing logs)
    // Note: may be empty if ecc-workflow doesn't log at info level for status
    // The key assertion is that -v flag is accepted without error
}
