//! Integration tests for ECC_ROOT resolution via the real `ecc` binary.
//!
//! These tests invoke `ecc install --dry-run` with different env configurations
//! to verify end-to-end ECC root resolution behaviour.
//!
//! Guards:
//! - Skipped on GitHub CI (`GITHUB_ACTIONS=true`).
//! - Marked `#[ignore]` — run with `cargo test --test ecc_root_integration -- --ignored`.

use std::path::PathBuf;
use std::process::Command;

/// Find the workspace root by walking up from the manifest dir.
fn project_root() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.pop(); // crates/
    dir.pop(); // workspace root
    dir
}

/// Path to the debug binary built from `ecc-cli`.
fn ecc_binary() -> PathBuf {
    project_root().join("target/debug/ecc")
}

fn on_github_ci() -> bool {
    std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true")
}

/// Build the binary once; returns true on success.
fn ensure_binary_built() -> bool {
    Command::new("cargo")
        .args(["build", "-p", "ecc-cli"])
        .current_dir(project_root())
        .status()
        .is_ok_and(|s| s.success())
}

#[test]
#[ignore]
fn ecc_root_env_var_resolves() {
    if on_github_ci() {
        eprintln!("WARN: Skipping on GitHub CI");
        return;
    }
    assert!(ensure_binary_built(), "failed to build ecc-cli");

    let root = project_root();
    let output = Command::new(ecc_binary())
        .args(["install", "--dry-run"])
        .env("ECC_ROOT", &root)
        .current_dir(&root)
        .output()
        .expect("failed to run ecc install --dry-run");

    assert!(
        output.status.success(),
        "expected success when ECC_ROOT points to workspace root.\nstderr: {}",
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
#[ignore]
fn ecc_root_env_var_invalid_installs_no_source_assets() {
    if on_github_ci() {
        eprintln!("WARN: Skipping on GitHub CI");
        return;
    }
    assert!(ensure_binary_built(), "failed to build ecc-cli");

    let tmp = std::env::temp_dir().join("ecc_test_empty_home");
    std::fs::create_dir_all(&tmp).ok();

    // With an invalid ECC_ROOT and no real HOME, the binary should still
    // run (exit 0) but produce a first-time install with no source assets.
    let output = Command::new(ecc_binary())
        .args(["install", "--dry-run", "--no-interactive"])
        .env("ECC_ROOT", "/tmp/nonexistent_ecc_root_path")
        .env("HOME", &tmp)
        .current_dir(&tmp)
        .output()
        .expect("failed to run ecc install --dry-run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "expected graceful exit even with invalid ECC_ROOT.\nstderr: {}",
        String::from_utf8_lossy(&output.stderr),
    );
    // With an invalid root, no agents/commands/skills should be found to copy
    assert!(
        stdout.contains("First-time installation"),
        "expected first-time install message with empty HOME.\nstdout: {stdout}",
    );

    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
#[ignore]
fn ecc_repo_root_detection_from_subdir() {
    if on_github_ci() {
        eprintln!("WARN: Skipping on GitHub CI");
        return;
    }
    assert!(ensure_binary_built(), "failed to build ecc-cli");

    let subdir = project_root().join("crates/ecc-cli");
    let output = Command::new(ecc_binary())
        .args(["install", "--dry-run"])
        .env_remove("ECC_ROOT")
        .current_dir(&subdir)
        .output()
        .expect("failed to run ecc install --dry-run from subdir");

    assert!(
        output.status.success(),
        "expected repo root detection to succeed from subdir.\nstderr: {}",
        String::from_utf8_lossy(&output.stderr),
    );
}
