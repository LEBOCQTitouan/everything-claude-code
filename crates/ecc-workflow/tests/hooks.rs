mod common;

use std::process::Command;

// ── shared helpers ────────────────────────────────────────────────────────────

fn init_workflow_state(project_dir: &std::path::Path, phase: &str, bin: &std::path::Path) {
    Command::new(bin)
        .args(["init", "dev", "test-feature"])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .output()
        .expect("init failed");
    let phases: &[&str] = match phase {
        "plan" => &[],
        "solution" => &["solution"],
        "implement" => &["solution", "implement"],
        "done" => &["solution", "implement", "done"],
        _ => panic!("unknown phase: {phase}"),
    };
    for p in phases {
        Command::new(bin)
            .args(["transition", p])
            .env("CLAUDE_PROJECT_DIR", project_dir)
            .output()
            .unwrap_or_else(|_| panic!("transition to {p} failed"));
    }
}

// ── tdd_enforcement ───────────────────────────────────────────────────────────

fn run_tdd_enforcement(project_dir: &std::path::Path, stdin_json: &str) -> std::process::Output {
    use std::io::Write as _;
    use std::process::Stdio;

    let bin = common::binary_path();
    let mut child = std::process::Command::new(&bin)
        .args(["tdd-enforcement"])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn ecc-workflow tdd-enforcement");

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(stdin_json.as_bytes()).ok();
    }
    child
        .wait_with_output()
        .expect("failed to wait for ecc-workflow tdd-enforcement")
}

#[test]
fn tdd_enforcement() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    // ── Scenario 1: implement phase, Write to test file → state = RED ─────────
    let dir_impl = tempfile::tempdir().unwrap();
    let workflow_dir = dir_impl.path().join(".claude/workflow");
    std::fs::create_dir_all(&workflow_dir).unwrap();

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
        workflow_dir.join("state.json"),
        serde_json::to_string_pretty(&state_impl).unwrap(),
    )
    .unwrap();

    let write_test_json = serde_json::json!({
        "tool_name": "Write",
        "tool_input": { "file_path": "crates/mylib/tests/integration.rs" }
    })
    .to_string();

    let out1 = run_tdd_enforcement(dir_impl.path(), &write_test_json);
    assert_eq!(
        out1.status.code(),
        Some(0),
        "tdd-enforcement must always exit 0\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&out1.stdout).unwrap_or(""),
        std::str::from_utf8(&out1.stderr).unwrap_or(""),
    );

    let tdd_state_path = workflow_dir.join(".tdd-state");
    assert!(
        tdd_state_path.exists(),
        ".tdd-state must be created after Write to test file in implement phase"
    );
    let tdd_state = std::fs::read_to_string(&tdd_state_path).expect("failed to read .tdd-state");
    assert_eq!(
        tdd_state.trim(),
        "RED",
        ".tdd-state must be RED after Write to test file, got: {}",
        tdd_state.trim()
    );

    // ── Scenario 2: implement phase, Write to source file when state=RED → GREEN
    let write_src_json = serde_json::json!({
        "tool_name": "Write",
        "tool_input": { "file_path": "crates/mylib/src/lib.rs" }
    })
    .to_string();

    let out2 = run_tdd_enforcement(dir_impl.path(), &write_src_json);
    assert_eq!(
        out2.status.code(),
        Some(0),
        "tdd-enforcement must always exit 0\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&out2.stdout).unwrap_or(""),
        std::str::from_utf8(&out2.stderr).unwrap_or(""),
    );

    let tdd_state2 = std::fs::read_to_string(&tdd_state_path).expect("failed to read .tdd-state");
    assert_eq!(
        tdd_state2.trim(),
        "GREEN",
        ".tdd-state must be GREEN after Write to src file when state=RED, got: {}",
        tdd_state2.trim()
    );

    // ── Scenario 3: phase=plan → exits 0 silently, no .tdd-state ─────────────
    let dir_plan = tempfile::tempdir().unwrap();
    let workflow_dir_plan = dir_plan.path().join(".claude/workflow");
    std::fs::create_dir_all(&workflow_dir_plan).unwrap();

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
        workflow_dir_plan.join("state.json"),
        serde_json::to_string_pretty(&state_plan).unwrap(),
    )
    .unwrap();

    let out3 = run_tdd_enforcement(dir_plan.path(), &write_test_json);
    assert_eq!(
        out3.status.code(),
        Some(0),
        "tdd-enforcement must exit 0 silently in plan phase"
    );
    assert!(
        !workflow_dir_plan.join(".tdd-state").exists(),
        ".tdd-state must NOT be created in plan phase"
    );

    // ── Scenario 4: no state.json → exits 0 silently ─────────────────────────
    let dir_no_state = tempfile::tempdir().unwrap();
    let out4 = run_tdd_enforcement(dir_no_state.path(), &write_test_json);
    assert_eq!(
        out4.status.code(),
        Some(0),
        "tdd-enforcement must exit 0 silently with no state.json"
    );
    assert!(
        !dir_no_state
            .path()
            .join(".claude/workflow/.tdd-state")
            .exists(),
        ".tdd-state must NOT be created when no state.json exists"
    );
}

// ── scope_check ───────────────────────────────────────────────────────────────

fn run_scope_check(project_dir: &std::path::Path, bin: &std::path::Path) -> std::process::Output {
    Command::new(bin)
        .arg("scope-check")
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .output()
        .expect("failed to execute ecc-workflow scope-check")
}

#[test]
fn scope_check() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {bin:?}");

    // ── Scenario 1: no state.json → exits 0, silent ───────────────────────────
    let dir_no_state = tempfile::tempdir().unwrap();
    let out1 = run_scope_check(dir_no_state.path(), &bin);
    assert_eq!(
        out1.status.code(),
        Some(0),
        "scope-check must exit 0 with no state.json\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&out1.stdout).unwrap_or(""),
        std::str::from_utf8(&out1.stderr).unwrap_or(""),
    );
    assert!(
        out1.stdout.trim_ascii().is_empty() && out1.stderr.trim_ascii().is_empty(),
        "scope-check must be silent when no state.json exists"
    );

    // ── Scenario 2: plan phase → exits 0, silent ──────────────────────────────
    let dir_plan = tempfile::tempdir().unwrap();
    init_workflow_state(dir_plan.path(), "plan", &bin);
    let out2 = run_scope_check(dir_plan.path(), &bin);
    assert_eq!(
        out2.status.code(),
        Some(0),
        "scope-check must exit 0 in plan phase"
    );
    assert!(
        out2.stdout.trim_ascii().is_empty() && out2.stderr.trim_ascii().is_empty(),
        "scope-check must be silent in plan phase"
    );

    // ── Scenario 3: implement phase, no design_path → exits 0, warns ──────────
    let dir_impl_no_design = tempfile::tempdir().unwrap();
    init_workflow_state(dir_impl_no_design.path(), "implement", &bin);
    let out3 = run_scope_check(dir_impl_no_design.path(), &bin);
    assert_eq!(
        out3.status.code(),
        Some(0),
        "scope-check must exit 0 with no design_path"
    );

    // ── Scenario 4: implement phase with design file present → exits 0 ────────
    let dir_with_design = tempfile::tempdir().unwrap();
    init_workflow_state(dir_with_design.path(), "solution", &bin);

    let design_dir = dir_with_design.path().join("docs/specs/test-feature");
    std::fs::create_dir_all(&design_dir).unwrap();
    let design_path = design_dir.join("design.md");
    std::fs::write(&design_path,
        "# Design\n\n## File Changes\n\n| File | Action |\n|------|--------|\n| src/foo.rs | CREATE |\n",
    ).unwrap();

    Command::new(&bin)
        .args([
            "transition",
            "implement",
            "--artifact",
            "design",
            "--path",
            design_path.to_str().unwrap(),
        ])
        .env("CLAUDE_PROJECT_DIR", dir_with_design.path())
        .output()
        .unwrap();

    let out4 = run_scope_check(dir_with_design.path(), &bin);
    assert_eq!(
        out4.status.code(),
        Some(0),
        "scope-check must always exit 0\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&out4.stdout).unwrap_or(""),
        std::str::from_utf8(&out4.stderr).unwrap_or(""),
    );
}
