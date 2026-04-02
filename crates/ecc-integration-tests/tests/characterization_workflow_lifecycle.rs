//! Characterization test for full workflow lifecycle (PC-034).
//!
//! Exercises the complete init -> plan -> solution -> implement -> done cycle
//! using the ecc-workflow binary, verifying exit codes and state.json at each step.

use assert_cmd::Command;
use std::path::Path;
use tempfile::TempDir;

fn wf_cmd(project_dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("ecc-workflow").expect("ecc-workflow binary not found");
    cmd.env("CLAUDE_PROJECT_DIR", project_dir);
    cmd.env("ECC_WORKFLOW_BYPASS", "0");
    cmd
}

fn read_phase(project_dir: &Path) -> String {
    let path = project_dir.join(".claude/workflow/state.json");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read state.json: {e}"));
    let v: serde_json::Value = serde_json::from_str(&content).expect("invalid JSON");
    v["phase"].as_str().unwrap_or("unknown").to_owned()
}

/// PC-034: Full lifecycle characterization: init -> solution -> implement -> done.
#[test]
fn full_lifecycle() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    // Init (creates state at plan phase)
    wf_cmd(dir)
        .args(["init", "dev", "characterization-test"])
        .assert()
        .success();
    assert_eq!(read_phase(dir), "plan");

    // Transition plan -> solution
    wf_cmd(dir)
        .args(["transition", "solution", "--artifact", "plan"])
        .assert()
        .success();
    assert_eq!(read_phase(dir), "solution");

    // Transition solution -> implement
    wf_cmd(dir)
        .args(["transition", "implement", "--artifact", "solution"])
        .assert()
        .success();
    assert_eq!(read_phase(dir), "implement");

    // Transition implement -> done
    wf_cmd(dir)
        .args(["transition", "done", "--artifact", "implement"])
        .assert()
        .success();
    assert_eq!(read_phase(dir), "done");
}

/// Characterize: illegal transition is rejected.
#[test]
fn illegal_transition_rejected() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    wf_cmd(dir)
        .args(["init", "dev", "test"])
        .assert()
        .success();

    // plan -> done is illegal (must go through solution, implement)
    let output = wf_cmd(dir)
        .args(["transition", "done"])
        .output()
        .expect("failed to run");

    assert_eq!(
        output.status.code(),
        Some(2),
        "illegal transition should exit 2"
    );
}
