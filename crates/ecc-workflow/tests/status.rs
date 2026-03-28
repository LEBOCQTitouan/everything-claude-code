mod common;

use std::process::Command;

// ── stop_gate helpers ─────────────────────────────────────────────────────────

fn run_stop_gate(project_dir: &std::path::Path) -> std::process::Output {
    let bin = common::binary_path();
    Command::new(&bin)
        .args(["stop-gate"])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .output()
        .expect("failed to execute ecc-workflow stop-gate")
}

fn write_state_with_phase(project_dir: &std::path::Path, phase: &str, feature: &str) {
    let workflow_dir = project_dir.join(".claude/workflow");
    std::fs::create_dir_all(&workflow_dir).unwrap();
    let state = serde_json::json!({
        "phase": phase,
        "concern": "dev",
        "feature": feature,
        "slug": "test-feature",
        "started_at": "2026-01-01T00:00:00Z",
        "toolchain": { "test": null, "lint": null, "build": null },
        "artifacts": {
            "plan": null, "solution": null, "implement": null,
            "campaign_path": null, "spec_path": null,
            "design_path": null, "tasks_path": null
        },
        "completed": []
    });
    std::fs::write(
        workflow_dir.join("state.json"),
        serde_json::to_string_pretty(&state).unwrap(),
    )
    .unwrap();
}

// ── stop_gate tests ───────────────────────────────────────────────────────────

#[test]
fn stop_gate_plan_phase_warns_on_stderr() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    let temp_dir = tempfile::tempdir().unwrap();
    write_state_with_phase(temp_dir.path(), "plan", "my-feature");

    let output = run_stop_gate(temp_dir.path());

    assert_eq!(
        output.status.code(),
        Some(0),
        "stop-gate must exit 0 even when phase is 'plan'\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&output.stdout).unwrap_or(""),
        std::str::from_utf8(&output.stderr).unwrap_or(""),
    );

    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");
    assert!(stderr.contains("WARNING"), "expected WARNING in stderr for phase 'plan', got: '{stderr}'");
    assert!(stderr.contains("plan"), "expected 'plan' in stderr warning, got: '{stderr}'");
}

#[test]
fn stop_gate_done_phase_is_silent() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    let temp_dir = tempfile::tempdir().unwrap();
    write_state_with_phase(temp_dir.path(), "done", "my-feature");

    let output = run_stop_gate(temp_dir.path());

    assert_eq!(output.status.code(), Some(0), "stop-gate must exit 0 in done phase");
    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");
    assert!(!stderr.contains("WARNING"), "expected no WARNING in done phase, got: '{stderr}'");
}

#[test]
fn stop_gate_no_state_is_silent() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    let temp_dir = tempfile::tempdir().unwrap();
    // No state.json

    let output = run_stop_gate(temp_dir.path());

    assert_eq!(output.status.code(), Some(0), "stop-gate must exit 0 with no state.json");
    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");
    assert!(!stderr.contains("WARNING"), "expected no WARNING with no state.json, got: '{stderr}'");
}

// ── grill_me_gate helpers ─────────────────────────────────────────────────────

fn run_grill_me_gate(project_dir: &std::path::Path) -> std::process::Output {
    let bin = common::binary_path();
    Command::new(&bin)
        .args(["grill-me-gate"])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .output()
        .expect("failed to execute ecc-workflow grill-me-gate")
}

fn write_state_with_phase_and_spec(
    project_dir: &std::path::Path,
    phase: &str,
    spec_path: Option<&str>,
) {
    let workflow_dir = project_dir.join(".claude/workflow");
    std::fs::create_dir_all(&workflow_dir).unwrap();
    let state = serde_json::json!({
        "phase": phase,
        "concern": "dev",
        "feature": "test-feature",
        "started_at": "2026-01-01T00:00:00Z",
        "toolchain": { "test": null, "lint": null, "build": null },
        "artifacts": {
            "plan": null, "solution": null, "implement": null,
            "campaign_path": null,
            "spec_path": spec_path,
            "design_path": null, "tasks_path": null
        },
        "completed": []
    });
    std::fs::write(
        workflow_dir.join("state.json"),
        serde_json::to_string_pretty(&state).unwrap(),
    )
    .unwrap();
}

// ── grill_me_gate tests ───────────────────────────────────────────────────────

#[test]
fn grill_me_gate_plan_phase_with_marker_is_silent() {
    let temp_dir = tempfile::tempdir().unwrap();

    let spec_file = temp_dir.path().join("spec.md");
    std::fs::write(&spec_file, "# Spec\n\n### Grill-Me Decisions\n\nSome grill-me content here.\n").unwrap();

    write_state_with_phase_and_spec(temp_dir.path(), "plan", Some(spec_file.to_str().unwrap()));

    let output = run_grill_me_gate(temp_dir.path());

    assert_eq!(
        output.status.code(),
        Some(0),
        "expected exit 0 when grill-me marker present\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&output.stdout).unwrap_or(""),
        std::str::from_utf8(&output.stderr).unwrap_or(""),
    );

    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");
    assert!(!stderr.contains("WARNING"), "expected no WARNING when grill-me marker present, got: '{stderr}'");
}

#[test]
fn grill_me_gate_plan_phase_without_marker_warns() {
    let temp_dir = tempfile::tempdir().unwrap();

    let spec_file = temp_dir.path().join("spec.md");
    std::fs::write(&spec_file, "# Spec\n\nSome content without grill-me section.\n").unwrap();

    write_state_with_phase_and_spec(temp_dir.path(), "plan", Some(spec_file.to_str().unwrap()));

    let output = run_grill_me_gate(temp_dir.path());

    assert_eq!(
        output.status.code(),
        Some(0),
        "grill-me-gate must always exit 0 (informational only)\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&output.stdout).unwrap_or(""),
        std::str::from_utf8(&output.stderr).unwrap_or(""),
    );

    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");
    assert!(stderr.contains("WARNING"), "expected WARNING when grill-me marker absent, got: '{stderr}'");
    assert!(stderr.to_lowercase().contains("grill"), "warning should mention grill-me, got: '{stderr}'");
}

#[test]
fn grill_me_gate_implement_phase_is_silent() {
    let temp_dir = tempfile::tempdir().unwrap();

    let spec_file = temp_dir.path().join("spec.md");
    std::fs::write(&spec_file, "# Spec\n\nNo grill-me here.\n").unwrap();

    write_state_with_phase_and_spec(temp_dir.path(), "implement", Some(spec_file.to_str().unwrap()));

    let output = run_grill_me_gate(temp_dir.path());

    assert_eq!(
        output.status.code(),
        Some(0),
        "expected exit 0 in implement phase\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&output.stdout).unwrap_or(""),
        std::str::from_utf8(&output.stderr).unwrap_or(""),
    );

    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");
    assert!(!stderr.contains("WARNING"), "expected no WARNING in implement phase, got: '{stderr}'");
}

#[test]
fn grill_me_gate_no_state_is_silent() {
    let temp_dir = tempfile::tempdir().unwrap();
    // No state.json created

    let output = run_grill_me_gate(temp_dir.path());

    assert_eq!(
        output.status.code(),
        Some(0),
        "expected exit 0 when no state.json\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&output.stdout).unwrap_or(""),
        std::str::from_utf8(&output.stderr).unwrap_or(""),
    );

    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");
    assert!(!stderr.contains("WARNING"), "expected no WARNING when no state.json, got: '{stderr}'");
}
