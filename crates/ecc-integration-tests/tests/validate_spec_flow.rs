//! Integration tests for `ecc validate spec` subcommand (PC-041, PC-042, PC-043).

mod common;

use common::EccTestEnv;
use std::path::Path;

fn fixtures_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// PC-041: `ecc validate spec` with valid fixture exits 0 with valid JSON.
#[test]
fn valid_spec_exits_zero() {
    let env = EccTestEnv::new();
    let fixture = fixtures_dir().join("spec_valid.md");

    let output = env
        .cmd()
        .arg("validate")
        .arg("spec")
        .arg(&fixture)
        .output()
        .expect("failed to run ecc");

    assert_eq!(
        output.status.code(),
        Some(0),
        "expected exit code 0 for valid spec, stdout: {}, stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout should be valid JSON");
    assert_eq!(parsed["valid"], true, "expected valid: true in JSON");
    assert!(
        parsed["ac_count"].as_u64().unwrap_or(0) > 0,
        "expected ac_count > 0"
    );
}

/// PC-042: `ecc validate spec` with gap fixture exits 1 with errors in JSON.
#[test]
fn gap_spec_exits_one() {
    let env = EccTestEnv::new();
    let fixture = fixtures_dir().join("spec_gap.md");

    let output = env
        .cmd()
        .arg("validate")
        .arg("spec")
        .arg(&fixture)
        .output()
        .expect("failed to run ecc");

    assert_eq!(
        output.status.code(),
        Some(1),
        "expected exit code 1 for spec with gap, stdout: {}, stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout should be valid JSON");
    assert_eq!(parsed["valid"], false, "expected valid: false in JSON");
    let errors = parsed["errors"].as_array().expect("errors should be array");
    assert!(!errors.is_empty(), "expected at least one error");
}

/// PC-043: `ecc validate spec` with nonexistent path exits 1.
#[test]
fn nonexistent_path_exits_one() {
    let env = EccTestEnv::new();

    let output = env
        .cmd()
        .arg("validate")
        .arg("spec")
        .arg("/nonexistent/path/spec.md")
        .output()
        .expect("failed to run ecc");

    assert_eq!(
        output.status.code(),
        Some(1),
        "expected exit code 1 for nonexistent path"
    );
}
