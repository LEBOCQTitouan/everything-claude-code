//! Phase 4 CLI tests: tracing subscriber, -v/-q flags, status and config commands.
//!
//! These tests verify that the ecc-cli binary compiles with the expected commands
//! and CLI flags described in PC-Phase4.

use std::process::Command;

/// Return the path to the `ecc` debug binary.
fn ecc_bin() -> std::path::PathBuf {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // ecc-cli
    path.pop(); // crates
    path.push("target");
    path.push("debug");
    path.push("ecc");
    path
}

/// Verify that `ecc status` exits 0 (binary must be pre-built).
#[test]
#[ignore]
fn status_command_exits_zero() {
    let bin = ecc_bin();
    let output = Command::new(&bin)
        .arg("status")
        .env_remove("ECC_LOG")
        .env_remove("RUST_LOG")
        .output()
        .expect("failed to execute ecc binary — run `cargo build` first");

    assert!(
        output.status.success(),
        "Expected `ecc status` to exit 0, got: {:?}\nstdout: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

/// Verify that `ecc status --json` produces valid JSON output.
#[test]
#[ignore]
fn status_json_flag_produces_valid_json() {
    let bin = ecc_bin();
    let output = Command::new(&bin)
        .args(["status", "--json"])
        .env_remove("ECC_LOG")
        .env_remove("RUST_LOG")
        .output()
        .expect("failed to execute ecc binary — run `cargo build` first");

    assert!(
        output.status.success(),
        "Expected `ecc status --json` to exit 0, got: {:?}",
        output.status
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(
        parsed.is_ok(),
        "Expected valid JSON from `ecc status --json`, got: {stdout}"
    );
}

/// Verify that `-v` and `-q` are mutually exclusive (clap should return error).
#[test]
#[ignore]
fn verbose_and_quiet_are_mutually_exclusive() {
    let bin = ecc_bin();
    let output = Command::new(&bin)
        .args(["-v", "-q", "version"])
        .output()
        .expect("failed to execute ecc binary — run `cargo build` first");

    assert!(
        !output.status.success(),
        "Expected `ecc -v -q version` to fail with a clap error"
    );
}

/// Verify that `--verbose` is NOT recognized (clean break from old flag).
#[test]
#[ignore]
fn verbose_long_flag_not_recognized() {
    let bin = ecc_bin();
    let output = Command::new(&bin)
        .args(["--verbose", "version"])
        .output()
        .expect("failed to execute ecc binary — run `cargo build` first");

    assert!(
        !output.status.success(),
        "Expected `--verbose` to be unrecognized after migration to -v"
    );
}

/// Compile-time assertion: ecc-cli must not contain log:: or env_logger calls.
///
/// This is enforced by the pass condition: `grep -rn "log::|env_logger" crates/ecc-cli/src/`
/// must return exit code 1 (no matches).
#[test]
fn no_log_crate_calls_in_ecc_cli_src() {
    let src_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut found = Vec::new();

    for entry in walkdir::WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        for (line_no, line) in content.lines().enumerate() {
            if line.contains("log::") || line.contains("env_logger") {
                found.push(format!(
                    "{}:{}: {}",
                    entry.path().display(),
                    line_no + 1,
                    line.trim()
                ));
            }
        }
    }

    assert!(
        found.is_empty(),
        "Found log::/env_logger calls in ecc-cli/src — these must be migrated to tracing::\n{}",
        found.join("\n")
    );
}
