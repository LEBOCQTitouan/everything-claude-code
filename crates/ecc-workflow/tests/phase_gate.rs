mod common;

use std::process::Command;

/// phase_gate: verify that `ecc-workflow phase-gate` reads stdin JSON (hook protocol),
/// blocks Write/Edit to non-allowed paths during plan/solution, allows allowed paths,
/// allows everything during implement/done, and blocks destructive Bash commands.
///
/// AC-004.5 — phase-gate subcommand faithfully ports phase-gate.sh behavior.
#[test]
fn phase_gate() {
    use std::io::Write as _;
    use std::process::Stdio;

    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    // Helper: pipe stdin JSON to `ecc-workflow phase-gate` and return the Output.
    let run_phase_gate = |project_dir: &std::path::Path, stdin_json: &str| -> std::process::Output {
        let mut child = Command::new(&bin)
            .args(["phase-gate"])
            .env("CLAUDE_PROJECT_DIR", project_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to spawn ecc-workflow phase-gate");

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(stdin_json.as_bytes()).ok();
        }
        child.wait_with_output().expect("failed to wait for ecc-workflow phase-gate")
    };

    // ── Scenario 1: no state.json → exit 0 regardless of tool ────────────────
    let dir_no_state = tempfile::tempdir().unwrap();
    let blocked_path_json = serde_json::json!({
        "tool_name": "Write",
        "tool_input": { "file_path": "src/main.rs" }
    })
    .to_string();

    let out = run_phase_gate(dir_no_state.path(), &blocked_path_json);
    assert_eq!(
        out.status.code(),
        Some(0),
        "no state.json must exit 0\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&out.stdout).unwrap_or(""),
        std::str::from_utf8(&out.stderr).unwrap_or(""),
    );

    // ── Scenario 2: phase=plan, Write to blocked path → exit 2 ───────────────
    let dir_plan = tempfile::tempdir().unwrap();
    let workflow_dir = dir_plan.path().join(".claude/workflow");
    std::fs::create_dir_all(&workflow_dir).unwrap();
    let state_plan = serde_json::json!({
        "concern": "dev", "phase": "plan", "feature": "test",
        "started_at": "2026-01-01T00:00:00Z",
        "toolchain": { "test": null, "lint": null, "build": null },
        "artifacts": { "plan": null, "solution": null, "implement": null,
                       "campaign_path": null, "spec_path": null,
                       "design_path": null, "tasks_path": null },
        "completed": []
    });
    std::fs::write(
        workflow_dir.join("state.json"),
        serde_json::to_string_pretty(&state_plan).unwrap(),
    )
    .unwrap();

    let out_blocked = run_phase_gate(dir_plan.path(), &blocked_path_json);
    assert_eq!(
        out_blocked.status.code(),
        Some(2),
        "Write to blocked path during plan must exit 2\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&out_blocked.stdout).unwrap_or(""),
        std::str::from_utf8(&out_blocked.stderr).unwrap_or(""),
    );

    // ── Scenario 3: phase=plan, Write to allowed path → exit 0 ───────────────
    let allowed_path_json = serde_json::json!({
        "tool_name": "Write",
        "tool_input": { "file_path": "docs/specs/my-feature/spec.md" }
    })
    .to_string();

    let out_allowed = run_phase_gate(dir_plan.path(), &allowed_path_json);
    assert_eq!(
        out_allowed.status.code(),
        Some(0),
        "Write to allowed path during plan must exit 0\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&out_allowed.stdout).unwrap_or(""),
        std::str::from_utf8(&out_allowed.stderr).unwrap_or(""),
    );

    // ── Scenario 4: phase=implement → exit 0 regardless of path ──────────────
    let dir_impl = tempfile::tempdir().unwrap();
    let workflow_dir_impl = dir_impl.path().join(".claude/workflow");
    std::fs::create_dir_all(&workflow_dir_impl).unwrap();
    let state_impl = serde_json::json!({
        "concern": "dev", "phase": "implement", "feature": "test",
        "started_at": "2026-01-01T00:00:00Z",
        "toolchain": { "test": null, "lint": null, "build": null },
        "artifacts": { "plan": null, "solution": null, "implement": null,
                       "campaign_path": null, "spec_path": null,
                       "design_path": null, "tasks_path": null },
        "completed": []
    });
    std::fs::write(
        workflow_dir_impl.join("state.json"),
        serde_json::to_string_pretty(&state_impl).unwrap(),
    )
    .unwrap();

    let out_impl = run_phase_gate(dir_impl.path(), &blocked_path_json);
    assert_eq!(
        out_impl.status.code(),
        Some(0),
        "Write to any path during implement must exit 0\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&out_impl.stdout).unwrap_or(""),
        std::str::from_utf8(&out_impl.stderr).unwrap_or(""),
    );

    // ── Scenario 5: phase=plan, Bash with destructive command → exit 2 ────────
    let destructive_bash_json = serde_json::json!({
        "tool_name": "Bash",
        "tool_input": { "command": "rm -rf /tmp/test" }
    })
    .to_string();

    let out_destructive = run_phase_gate(dir_plan.path(), &destructive_bash_json);
    assert_eq!(
        out_destructive.status.code(),
        Some(2),
        "Bash with rm -rf during plan must exit 2\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&out_destructive.stdout).unwrap_or(""),
        std::str::from_utf8(&out_destructive.stderr).unwrap_or(""),
    );

    // ── Scenario 6: phase=done → exit 0 regardless of tool/path ──────────────
    let dir_done = tempfile::tempdir().unwrap();
    let workflow_dir_done = dir_done.path().join(".claude/workflow");
    std::fs::create_dir_all(&workflow_dir_done).unwrap();
    let state_done = serde_json::json!({
        "concern": "dev", "phase": "done", "feature": "test",
        "started_at": "2026-01-01T00:00:00Z",
        "toolchain": { "test": null, "lint": null, "build": null },
        "artifacts": { "plan": null, "solution": null, "implement": null,
                       "campaign_path": null, "spec_path": null,
                       "design_path": null, "tasks_path": null },
        "completed": []
    });
    std::fs::write(
        workflow_dir_done.join("state.json"),
        serde_json::to_string_pretty(&state_done).unwrap(),
    )
    .unwrap();

    let out_done = run_phase_gate(dir_done.path(), &blocked_path_json);
    assert_eq!(
        out_done.status.code(),
        Some(0),
        "Write to any path during done must exit 0\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&out_done.stdout).unwrap_or(""),
        std::str::from_utf8(&out_done.stderr).unwrap_or(""),
    );
}
