mod common;

use std::process::Command;

/// transition_full_sequence: verify the complete workflow lifecycle with artifact stamping and path storage.
///
/// AC-004.2 — plan->solution->implement->done sequence:
///   - Each transition updates phase
///   - Artifact timestamps are ISO 8601 UTC
///   - Paths are stored in the correct artifact path fields
///   - done transition appends to completed array
#[test]
fn transition_full_sequence() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    let temp_dir = tempfile::tempdir().unwrap();
    let state_path = temp_dir.path().join(".claude/workflow/state.json");

    // Step 1: init → phase=plan
    let init_output = Command::new(&bin)
        .args(["init", "dev", "test feature"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .output()
        .expect("failed to execute ecc-workflow init");

    assert_eq!(
        init_output.status.code(),
        Some(0),
        "init must exit 0\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&init_output.stdout).unwrap_or(""),
        std::str::from_utf8(&init_output.stderr).unwrap_or(""),
    );

    // Step 2: transition solution --artifact plan --path "docs/specs/test/spec.md"
    let t1 = Command::new(&bin)
        .args([
            "transition",
            "solution",
            "--artifact",
            "plan",
            "--path",
            "docs/specs/test/spec.md",
        ])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .output()
        .expect("failed to execute transition plan->solution");

    assert_eq!(
        t1.status.code(),
        Some(0),
        "transition plan->solution must exit 0\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&t1.stdout).unwrap_or(""),
        std::str::from_utf8(&t1.stderr).unwrap_or(""),
    );

    let v1: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&state_path).expect("state.json must exist after t1"),
    )
    .expect("state.json must be valid JSON after t1");

    assert_eq!(
        v1["phase"].as_str(),
        Some("solution"),
        "phase must be 'solution' after plan->solution transition"
    );

    let plan_ts = v1["artifacts"]["plan"]
        .as_str()
        .expect("artifacts.plan must be a string after plan->solution");
    assert!(
        plan_ts.len() == 20 && plan_ts.ends_with('Z') && plan_ts.contains('T'),
        "artifacts.plan must be ISO 8601 UTC, got: '{plan_ts}'"
    );

    assert_eq!(
        v1["artifacts"]["spec_path"].as_str(),
        Some("docs/specs/test/spec.md"),
        "artifacts.spec_path must be set after --path passed with --artifact plan"
    );

    assert_eq!(
        v1["completed"],
        serde_json::Value::Array(vec![]),
        "completed must still be [] after plan->solution"
    );

    // Step 3: transition implement --artifact solution --path "docs/specs/test/design.md"
    let t2 = Command::new(&bin)
        .args([
            "transition",
            "implement",
            "--artifact",
            "solution",
            "--path",
            "docs/specs/test/design.md",
        ])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .output()
        .expect("failed to execute transition solution->implement");

    assert_eq!(
        t2.status.code(),
        Some(0),
        "transition solution->implement must exit 0\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&t2.stdout).unwrap_or(""),
        std::str::from_utf8(&t2.stderr).unwrap_or(""),
    );

    let v2: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&state_path).expect("state.json must exist after t2"),
    )
    .expect("state.json must be valid JSON after t2");

    assert_eq!(
        v2["phase"].as_str(),
        Some("implement"),
        "phase must be 'implement' after solution->implement transition"
    );

    let solution_ts = v2["artifacts"]["solution"]
        .as_str()
        .expect("artifacts.solution must be a string");
    assert!(
        solution_ts.len() == 20 && solution_ts.ends_with('Z') && solution_ts.contains('T'),
        "artifacts.solution must be ISO 8601 UTC, got: '{solution_ts}'"
    );

    assert_eq!(
        v2["artifacts"]["design_path"].as_str(),
        Some("docs/specs/test/design.md"),
        "artifacts.design_path must be set after --path passed with --artifact solution"
    );
    assert_eq!(
        v2["artifacts"]["spec_path"].as_str(),
        Some("docs/specs/test/spec.md"),
        "artifacts.spec_path must be preserved after solution->implement transition"
    );
    assert_eq!(
        v2["completed"],
        serde_json::Value::Array(vec![]),
        "completed must still be [] after solution->implement"
    );

    // Step 4: transition done --artifact implement
    let t3 = Command::new(&bin)
        .args(["transition", "done", "--artifact", "implement"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .output()
        .expect("failed to execute transition implement->done");

    assert_eq!(
        t3.status.code(),
        Some(0),
        "transition implement->done must exit 0\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&t3.stdout).unwrap_or(""),
        std::str::from_utf8(&t3.stderr).unwrap_or(""),
    );

    let v3: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&state_path).expect("state.json must exist after t3"),
    )
    .expect("state.json must be valid JSON after t3");

    assert_eq!(
        v3["phase"].as_str(),
        Some("done"),
        "phase must be 'done' after implement->done transition"
    );

    let implement_ts = v3["artifacts"]["implement"]
        .as_str()
        .expect("artifacts.implement must be a string");
    assert!(
        implement_ts.len() == 20 && implement_ts.ends_with('Z') && implement_ts.contains('T'),
        "artifacts.implement must be ISO 8601 UTC, got: '{implement_ts}'"
    );

    let completed = v3["completed"]
        .as_array()
        .expect("completed must be an array after implement->done");
    assert_eq!(
        completed.len(),
        1,
        "completed must have exactly one entry after implement->done"
    );
    assert_eq!(
        completed[0]["phase"].as_str(),
        Some("implement"),
        "completed[0].phase must be 'implement'"
    );
    assert_eq!(
        completed[0]["file"].as_str(),
        Some("implement-done.md"),
        "completed[0].file must be 'implement-done.md'"
    );

    let done_at = completed[0]["at"]
        .as_str()
        .expect("completed[0].at must be a string");
    assert!(
        done_at.len() == 20 && done_at.ends_with('Z') && done_at.contains('T'),
        "completed[0].at must be ISO 8601 UTC, got: '{done_at}'"
    );

    assert_eq!(
        v3["artifacts"]["spec_path"].as_str(),
        Some("docs/specs/test/spec.md"),
        "artifacts.spec_path must be preserved after implement->done"
    );
    assert_eq!(
        v3["artifacts"]["design_path"].as_str(),
        Some("docs/specs/test/design.md"),
        "artifacts.design_path must be preserved after implement->done"
    );
}
