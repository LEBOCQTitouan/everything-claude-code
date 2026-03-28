/// Integration tests for observable CLI operations (US-001).
///
/// These tests build the binary and invoke it as a subprocess to verify
/// logging and error-reporting behaviour at the process level.
use std::process::Command;

/// Return the path to the `ecc` binary built for tests.
fn ecc_bin() -> std::path::PathBuf {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // Navigate to workspace root and find the binary
    path.pop(); // ecc-cli
    path.pop(); // crates
    path.push("target");
    path.push("debug");
    path.push("ecc");
    path
}

/// PC-001 — Default invocation emits warn! on stderr.
///
/// Runs `ecc install --ecc-root /nonexistent --no-interactive --force`
/// and asserts that stderr contains a WARN-level log message.
/// The binary must be pre-built (`cargo build`).
#[test]
#[ignore]
fn warn_on_stderr() {
    let bin = ecc_bin();
    let output = Command::new(&bin)
        .args([
            "install",
            "--ecc-root",
            "/nonexistent_ecc_root_path",
            "--no-interactive",
            "--force",
        ])
        .env_remove("RUST_LOG") // ensure default filter is used
        .output()
        .expect("failed to execute ecc binary — run `cargo build` first");

    let stderr = String::from_utf8_lossy(&output.stderr);
    // The default log level is "warn"; at least one WARN message must appear on stderr.
    assert!(
        stderr.to_lowercase().contains("[warn]") || stderr.contains("WARN"),
        "Expected WARN-level log on stderr, got: {stderr}"
    );
}
