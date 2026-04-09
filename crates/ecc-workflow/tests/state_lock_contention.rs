/// Multi-process state lock contention tests.
///
/// These tests spawn multiple `ecc-workflow` processes concurrently against the
/// same temp directory to verify that state locking serializes writes and prevents
/// data corruption.
///
/// All tests are `#[ignore]` — run with:
///   cargo test -p ecc-workflow --test state_lock_contention -- --ignored
use std::path::PathBuf;
use std::process::Command;

fn binary_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // deps/
    path.pop(); // debug/
    path.push("ecc-workflow");
    path
}

/// Initialize the workflow state in a temp directory to phase=plan.
fn init_workflow(project_dir: &std::path::Path) {
    let bin = binary_path();
    let output = Command::new(&bin)
        .args(["init", "test-concern", "test-feature"])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .output()
        .expect("failed to run ecc-workflow init");
    assert!(
        output.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// PC-005: Two concurrent transitions serialize — both complete without data loss.
///
/// Spawns two processes that both attempt to transition plan→solution.
/// At least one must succeed (exit 0). After both complete, state.json must be
/// valid JSON with a "phase" field — no corruption.
#[test]
#[ignore]
fn two_concurrent_transitions_serialize() {
    let bin = binary_path();
    let temp_dir = tempfile::tempdir().unwrap();
    let project_dir = temp_dir.path();

    init_workflow(project_dir);

    // Spawn both processes concurrently
    let mut child1 = Command::new(&bin)
        .args(["transition", "solution", "--artifact", "plan"])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .spawn()
        .expect("failed to spawn process 1");

    let mut child2 = Command::new(&bin)
        .args(["transition", "solution", "--artifact", "plan"])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .spawn()
        .expect("failed to spawn process 2");

    let status1 = child1.wait().expect("child1 did not exit");
    let status2 = child2.wait().expect("child2 did not exit");

    // At least one must have succeeded
    let both_failed = !status1.success() && !status2.success();
    assert!(
        !both_failed,
        "both transitions failed: exit1={:?}, exit2={:?}",
        status1.code(),
        status2.code()
    );

    // state.json must exist and be valid JSON with phase field
    let state_path = project_dir.join(".claude/workflow/state.json");
    assert!(
        state_path.exists(),
        "state.json missing after concurrent transitions"
    );
    let content = std::fs::read_to_string(&state_path).expect("failed to read state.json");
    let v: serde_json::Value = serde_json::from_str(&content)
        .expect("state.json is not valid JSON after concurrent transitions");
    assert!(
        v.get("phase").is_some(),
        "state.json missing 'phase' field after concurrent transitions: {content}"
    );
    // Phase must be "solution" (both agreed on the same destination)
    let phase = v["phase"].as_str().unwrap_or("");
    assert_eq!(
        phase, "solution",
        "expected phase 'solution' after concurrent transitions, got '{phase}'"
    );
}

/// PC-006: Init concurrent with transition — no data corruption.
///
/// Spawns a transition (plan→solution) and an init concurrently.
/// Verifies that state.json exists and is valid JSON after both complete —
/// no partial writes or garbled data.
#[test]
#[ignore]
fn init_concurrent_with_transition_no_loss() {
    let bin = binary_path();
    let temp_dir = tempfile::tempdir().unwrap();
    let project_dir = temp_dir.path();

    init_workflow(project_dir);

    // Spawn both processes concurrently
    let mut transition = Command::new(&bin)
        .args(["transition", "solution", "--artifact", "plan"])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .spawn()
        .expect("failed to spawn transition process");

    let mut init = Command::new(&bin)
        .args(["init", "new-concern", "new-feature"])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .spawn()
        .expect("failed to spawn init process");

    let _status_transition = transition.wait().expect("transition did not exit");
    let _status_init = init.wait().expect("init did not exit");

    // state.json must exist and be valid JSON — no corruption regardless of ordering
    let state_path = project_dir.join(".claude/workflow/state.json");
    assert!(
        state_path.exists(),
        "state.json missing after concurrent init+transition"
    );
    let content = std::fs::read_to_string(&state_path).expect("failed to read state.json");
    let v: serde_json::Value = serde_json::from_str(&content)
        .expect("state.json corrupted (not valid JSON) after concurrent init+transition");
    assert!(
        v.get("phase").is_some(),
        "state.json missing 'phase' field after concurrent init+transition: {content}"
    );
    assert!(
        v.get("concern").is_some(),
        "state.json missing 'concern' field after concurrent init+transition: {content}"
    );
    assert!(
        v.get("feature").is_some(),
        "state.json missing 'feature' field after concurrent init+transition: {content}"
    );
}

/// PC-007: Phase-gate reads post-transition state — never sees garbage.
///
/// Spawns a transition (plan→solution) and a phase-gate read concurrently.
/// Verifies that phase-gate either sees "plan" or "solution" — never corrupt data.
#[test]
#[ignore]
fn phase_gate_reads_post_transition_state() {
    let bin = binary_path();
    let temp_dir = tempfile::tempdir().unwrap();
    let project_dir = temp_dir.path();

    init_workflow(project_dir);

    // Spawn transition and phase-gate concurrently
    let mut transition = Command::new(&bin)
        .args(["transition", "solution", "--artifact", "plan"])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .spawn()
        .expect("failed to spawn transition");

    // phase-gate reads stdin, so we provide empty input via /dev/null
    let phase_gate_output = Command::new(&bin)
        .args(["phase-gate"])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .stdin(std::process::Stdio::null())
        .output()
        .expect("failed to run phase-gate");

    let _status_transition = transition.wait().expect("transition did not exit");

    // phase-gate must exit cleanly (0 or 2 are both valid — but not a crash)
    let code = phase_gate_output.status.code().unwrap_or(-1);
    assert!(
        code == 0 || code == 2,
        "phase-gate exited with unexpected code {code}"
    );

    // phase-gate output must be valid JSON
    let stdout = String::from_utf8_lossy(&phase_gate_output.stdout);
    let stderr = String::from_utf8_lossy(&phase_gate_output.stderr);
    let output_str = if !stdout.trim().is_empty() {
        stdout.trim().to_owned()
    } else {
        stderr.trim().to_owned()
    };

    if !output_str.is_empty() {
        let _v: serde_json::Value = serde_json::from_str(&output_str).unwrap_or_else(|e| {
            panic!("phase-gate produced invalid JSON: {e}\nOutput: {output_str}")
        });
    }

    // After both complete, state.json must be valid
    let state_path = project_dir.join(".claude/workflow/state.json");
    assert!(
        state_path.exists(),
        "state.json missing after phase-gate + transition"
    );
    let content = std::fs::read_to_string(&state_path).expect("failed to read state.json");
    let v: serde_json::Value =
        serde_json::from_str(&content).expect("state.json corrupted after phase-gate + transition");
    let phase = v["phase"].as_str().unwrap_or("unknown");
    assert!(
        phase == "plan" || phase == "solution",
        "unexpected phase '{phase}' — expected 'plan' or 'solution'"
    );
}
