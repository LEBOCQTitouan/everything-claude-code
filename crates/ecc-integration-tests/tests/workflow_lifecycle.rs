use assert_cmd::Command;
use std::path::Path;
use tempfile::TempDir;

/// Build an `ecc-workflow` command with project dir and bypass disabled.
fn wf_cmd(project_dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("ecc-workflow").expect("ecc-workflow binary not found");
    cmd.env("CLAUDE_PROJECT_DIR", project_dir);
    cmd.env("ECC_WORKFLOW_BYPASS", "0");
    cmd
}

/// Read state.json from `.claude/workflow/state.json` and return as JSON value.
fn read_state(project_dir: &Path) -> serde_json::Value {
    let path = project_dir.join(".claude/workflow/state.json");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read state.json at {}: {e}", path.display()));
    serde_json::from_str(&content).expect("state.json is not valid JSON")
}

/// AC-003.1: Full forward lifecycle — init → solution → implement → done.
#[test]
fn workflow_lifecycle_forward() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    // init → phase = plan
    wf_cmd(dir)
        .args(["init", "dev", "test-feature"])
        .assert()
        .success();
    assert_eq!(read_state(dir)["phase"].as_str(), Some("plan"));

    // transition to solution
    wf_cmd(dir)
        .args(["transition", "solution"])
        .assert()
        .success();
    assert_eq!(read_state(dir)["phase"].as_str(), Some("solution"));

    // transition to implement
    wf_cmd(dir)
        .args(["transition", "implement"])
        .assert()
        .success();
    assert_eq!(read_state(dir)["phase"].as_str(), Some("implement"));

    // transition to done
    wf_cmd(dir)
        .args(["transition", "done"])
        .assert()
        .success();
    assert_eq!(read_state(dir)["phase"].as_str(), Some("done"));
}

/// AC-003.2: Reset after done — state archived and state.json has phase "idle".
#[test]
fn workflow_lifecycle_reset() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    // Advance to done
    wf_cmd(dir).args(["init", "dev", "reset-test"]).assert().success();
    wf_cmd(dir).args(["transition", "solution"]).assert().success();
    wf_cmd(dir).args(["transition", "implement"]).assert().success();
    wf_cmd(dir).args(["transition", "done"]).assert().success();

    // Reset
    wf_cmd(dir).args(["reset", "--force"]).assert().success();

    // Phase must now be idle
    assert_eq!(read_state(dir)["phase"].as_str(), Some("idle"));

    // Archive directory must contain the old state
    let archive_dir = dir.join(".claude/workflow/archive");
    assert!(archive_dir.exists(), "archive directory must be created by reset");
    let entries: Vec<_> = std::fs::read_dir(&archive_dir)
        .expect("failed to read archive dir")
        .collect();
    assert!(!entries.is_empty(), "old done state must be archived on reset");
}

/// AC-003.3: Re-init after reset succeeds and phase is "plan".
#[test]
fn workflow_lifecycle_reinit() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    // Advance to done and reset
    wf_cmd(dir).args(["init", "dev", "first-feature"]).assert().success();
    wf_cmd(dir).args(["transition", "solution"]).assert().success();
    wf_cmd(dir).args(["transition", "implement"]).assert().success();
    wf_cmd(dir).args(["transition", "done"]).assert().success();
    wf_cmd(dir).args(["reset", "--force"]).assert().success();

    // Re-init with a new feature
    wf_cmd(dir)
        .args(["init", "dev", "second-feature"])
        .assert()
        .success();

    let state = read_state(dir);
    assert_eq!(state["phase"].as_str(), Some("plan"), "re-init must start at plan");
    assert_eq!(
        state["feature"].as_str(),
        Some("second-feature"),
        "re-init must use the new feature name"
    );
}

/// AC-003.4: Illegal transition from plan to implement is rejected (exit code 2).
#[test]
fn workflow_lifecycle_illegal() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    // Init → phase = plan
    wf_cmd(dir).args(["init", "dev", "illegal-test"]).assert().success();

    // Attempt plan → implement (skips solution — illegal)
    wf_cmd(dir)
        .args(["transition", "implement"])
        .assert()
        .code(2);

    // Phase must remain plan (state not mutated on illegal transition)
    assert_eq!(
        read_state(dir)["phase"].as_str(),
        Some("plan"),
        "phase must remain plan after illegal transition"
    );
}

/// AC-003.5: Artifact timestamps are recorded as ISO 8601 strings.
#[test]
fn workflow_lifecycle_artifacts() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    // Init → plan
    wf_cmd(dir).args(["init", "dev", "artifact-test"]).assert().success();

    // Transition to solution with --artifact plan
    wf_cmd(dir)
        .args(["transition", "solution", "--artifact", "plan"])
        .assert()
        .success();

    let state = read_state(dir);
    let plan_ts = state["artifacts"]["plan"]
        .as_str()
        .expect("artifacts.plan must be a non-null string after --artifact plan");

    // Validate ISO 8601 format: must contain 'T' and end with 'Z'
    assert!(
        plan_ts.contains('T') && plan_ts.ends_with('Z'),
        "artifacts.plan must be an ISO 8601 timestamp, got: {plan_ts}"
    );
}
