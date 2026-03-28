/// Integration tests for observable CLI operations (US-001).
///
/// These tests build the binary and invoke it as a subprocess to verify
/// logging and error-reporting behaviour at the process level.
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

/// Return the path to the `ecc` release binary.
fn ecc_release_bin() -> std::path::PathBuf {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // ecc-cli
    path.pop(); // crates
    path.push("target");
    path.push("release");
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

/// PC-002 — --verbose flag produces debug-level output.
///
/// Simulates `env -u RUST_LOG ./target/release/ecc --verbose version 2>&1 | grep -i debug`.
/// The release binary must be pre-built.
#[test]
#[ignore]
fn verbose_debug_output() {
    let bin = ecc_release_bin();
    let output = Command::new(&bin)
        .args(["--verbose", "version"])
        .env_remove("RUST_LOG")
        .output()
        .expect("failed to execute ecc binary — run `cargo build --release` first");

    let combined = String::from_utf8_lossy(&output.stderr).to_lowercase()
        + &String::from_utf8_lossy(&output.stdout).to_lowercase();

    assert!(
        combined.contains("debug"),
        "Expected debug-level output when --verbose is passed, got stdout: {} stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

/// PC-005 — Failure banner `Error: <desc>` on stderr before exit(1).
///
/// Simulates `./target/release/ecc install --ecc-root /nonexistent 2>&1 | grep '^Error:'`.
/// The release binary must be pre-built.
#[test]
#[ignore]
fn failure_banner_error_on_stderr() {
    let bin = ecc_release_bin();
    let output = Command::new(&bin)
        .args(["install", "--ecc-root", "/nonexistent", "--no-interactive", "--force"])
        .output()
        .expect("failed to execute ecc binary — run `cargo build --release` first");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{stderr}{stdout}");

    // The binary must print a line starting with "Error:" on stderr (or combined output)
    assert!(
        combined.lines().any(|l| l.starts_with("Error:")),
        "Expected a line starting with 'Error:' on stderr/stdout, got:\nstdout: {stdout}\nstderr: {stderr}"
    );
}
