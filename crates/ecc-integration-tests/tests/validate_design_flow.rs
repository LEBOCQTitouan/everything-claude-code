//! Integration tests for `ecc validate design` subcommand (PC-044, PC-045, PC-046).

mod common;

use common::EccTestEnv;
use std::path::Path;

fn fixtures_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// PC-044: `ecc validate design` with valid fixture exits 0.
#[test]
fn valid_design_exits_zero() {
    let env = EccTestEnv::new();
    let fixture = fixtures_dir().join("design_valid.md");

    let output = env
        .cmd()
        .arg("validate")
        .arg("design")
        .arg(&fixture)
        .output()
        .expect("failed to run ecc");

    assert_eq!(
        output.status.code(),
        Some(0),
        "expected exit code 0 for valid design, stdout: {}, stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout should be valid JSON");
    assert_eq!(parsed["valid"], true, "expected valid: true in JSON");
    assert!(
        parsed["pc_count"].as_u64().unwrap_or(0) > 0,
        "expected pc_count > 0"
    );
}

/// PC-045: `ecc validate design --spec` with coverage gap reports uncovered ACs (exit 1).
#[test]
fn coverage_gap_reported() {
    let env = EccTestEnv::new();
    let design_fixture = fixtures_dir().join("design_uncovered_ac.md");
    let spec_fixture = fixtures_dir().join("spec_valid.md");

    let output = env
        .cmd()
        .arg("validate")
        .arg("design")
        .arg(&design_fixture)
        .arg("--spec")
        .arg(&spec_fixture)
        .output()
        .expect("failed to run ecc");

    // Should exit 1 because AC-001.2 and AC-002.1 are not covered by any PC
    assert_eq!(
        output.status.code(),
        Some(1),
        "expected exit code 1 when ACs are uncovered, stdout: {}, stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout should be valid JSON");
    assert_eq!(parsed["valid"], false, "expected valid: false");
    let uncovered = parsed["uncovered_acs"]
        .as_array()
        .expect("uncovered_acs should be array");
    assert!(!uncovered.is_empty(), "expected uncovered ACs reported");
}

/// PC-046: `ecc validate design` with ordering violation reports it.
#[test]
fn ordering_violation_reported() {
    let env = EccTestEnv::new();
    let fixture = fixtures_dir().join("design_ordering_violation.md");

    let output = env
        .cmd()
        .arg("validate")
        .arg("design")
        .arg(&fixture)
        .output()
        .expect("failed to run ecc");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout should be valid JSON");

    // ordering_violations should be present (may or may not cause exit 1 depending on logic)
    let violations = parsed["ordering_violations"]
        .as_array()
        .expect("ordering_violations should be array");
    assert!(
        !violations.is_empty(),
        "expected at least one ordering violation reported, stdout: {stdout}"
    );
}
