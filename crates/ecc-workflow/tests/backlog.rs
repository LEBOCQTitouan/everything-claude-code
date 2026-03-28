mod common;

use std::process::Command;

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

fn run_pass_condition_check(
    project_dir: &std::path::Path,
    bin: &std::path::Path,
) -> std::process::Output {
    Command::new(bin)
        .arg("pass-condition-check")
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .env_remove("ECC_WORKFLOW_BYPASS")
        .output()
        .expect("failed to execute ecc-workflow pass-condition-check")
}

/// e2e_boundary_check: verify that `ecc-workflow e2e-boundary-check` checks for "## E2E Tests"
/// section in implement-done.md at the "done" phase.
#[test]
fn e2e_boundary_check() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {bin:?}");

    fn run_e2e_boundary_check(
        project_dir: &std::path::Path,
        bin: &std::path::Path,
    ) -> std::process::Output {
        Command::new(bin)
            .args(["e2e-boundary-check"])
            .env("CLAUDE_PROJECT_DIR", project_dir)
            .env_remove("ECC_WORKFLOW_BYPASS")
            .output()
            .expect("failed to execute ecc-workflow e2e-boundary-check")
    }

    // ── Scenario 1: no state.json → exits 0, silent ───────────────────────────
    let dir_no_state = tempfile::tempdir().unwrap();
    let out1 = run_e2e_boundary_check(dir_no_state.path(), &bin);
    assert_eq!(
        out1.status.code(),
        Some(0),
        "e2e-boundary-check must exit 0 with no state.json"
    );
    assert!(
        out1.stdout.trim_ascii().is_empty() && out1.stderr.trim_ascii().is_empty(),
        "e2e-boundary-check must be silent when no state.json exists"
    );

    // ── Scenario 2: plan phase → exits 0, silent ──────────────────────────────
    let dir_plan = tempfile::tempdir().unwrap();
    init_workflow_state(dir_plan.path(), "plan", &bin);
    let out2 = run_e2e_boundary_check(dir_plan.path(), &bin);
    assert_eq!(
        out2.status.code(),
        Some(0),
        "e2e-boundary-check must exit 0 in plan phase"
    );
    assert!(
        out2.stdout.trim_ascii().is_empty() && out2.stderr.trim_ascii().is_empty(),
        "e2e-boundary-check must be silent in plan phase"
    );

    // ── Scenario 3: done phase, implement-done.md with "## E2E Tests" → exit 0
    let dir_done_with_section = tempfile::tempdir().unwrap();
    init_workflow_state(dir_done_with_section.path(), "done", &bin);
    let workflow_dir3 = dir_done_with_section.path().join(".claude/workflow");
    std::fs::write(
        workflow_dir3.join("implement-done.md"),
        "# Done\n\n## E2E Tests\n\n- Covered by existing suite\n",
    )
    .unwrap();
    let out3 = run_e2e_boundary_check(dir_done_with_section.path(), &bin);
    assert_eq!(
        out3.status.code(),
        Some(0),
        "e2e-boundary-check must exit 0 when E2E Tests section present"
    );
    let stderr3 = std::str::from_utf8(&out3.stderr).unwrap_or("");
    assert!(
        stderr3.trim().is_empty(),
        "e2e-boundary-check must be silent when E2E Tests section present"
    );

    // ── Scenario 4: done phase, implement-done.md missing "## E2E Tests" → warns
    let dir_done_missing = tempfile::tempdir().unwrap();
    init_workflow_state(dir_done_missing.path(), "done", &bin);
    let workflow_dir4 = dir_done_missing.path().join(".claude/workflow");
    std::fs::write(
        workflow_dir4.join("implement-done.md"),
        "# Done\n\n## Docs Updated\n\n- Some doc\n",
    )
    .unwrap();
    let out4 = run_e2e_boundary_check(dir_done_missing.path(), &bin);
    assert_eq!(
        out4.status.code(),
        Some(0),
        "e2e-boundary-check must always exit 0"
    );
    let stderr4 = std::str::from_utf8(&out4.stderr).unwrap_or("");
    assert!(
        !stderr4.trim().is_empty(),
        "e2e-boundary-check must warn when E2E Tests section is missing"
    );
    assert!(
        stderr4.to_lowercase().contains("e2e"),
        "warning must mention E2E, got stderr: {stderr4}"
    );
}

/// pass_condition_check: verify that `ecc-workflow pass-condition-check` checks pass condition results.
#[test]
fn pass_condition_check() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {bin:?}");

    // ── Scenario 1: done phase, all ✅ → exit 0, no warning ───────────────────
    let dir1 = tempfile::tempdir().unwrap();
    init_workflow_state(dir1.path(), "done", &bin);
    let workflow_dir1 = dir1.path().join(".claude/workflow");
    std::fs::create_dir_all(&workflow_dir1).unwrap();
    std::fs::write(
        workflow_dir1.join("implement-done.md"),
        "# Implement Done\n\n## Pass Condition Results\n\n- PC-001 ✅ passed\n- PC-002 ✅ passed\n\nAll pass conditions: 2/2 ✅\n",
    ).unwrap();
    let out1 = run_pass_condition_check(dir1.path(), &bin);
    assert_eq!(
        out1.status.code(),
        Some(0),
        "scenario 1: must exit 0 for all-pass results"
    );
    assert!(
        out1.stdout.trim_ascii().is_empty() && out1.stderr.trim_ascii().is_empty(),
        "scenario 1: must be silent for all-pass results"
    );

    // ── Scenario 2: done phase, section missing → exit 0, stderr warning ──────
    let dir2 = tempfile::tempdir().unwrap();
    init_workflow_state(dir2.path(), "done", &bin);
    let workflow_dir2 = dir2.path().join(".claude/workflow");
    std::fs::create_dir_all(&workflow_dir2).unwrap();
    std::fs::write(workflow_dir2.join("implement-done.md"),
        "# Implement Done\n\n## Docs Updated\n\n- doc.md updated\n\n## Supplemental Docs\n\n_none_\n",
    ).unwrap();
    let out2 = run_pass_condition_check(dir2.path(), &bin);
    assert_eq!(
        out2.status.code(),
        Some(0),
        "scenario 2: must exit 0 when section missing"
    );
    let stderr2 = std::str::from_utf8(&out2.stderr).unwrap_or("");
    assert!(
        !stderr2.trim().is_empty(),
        "scenario 2: must warn when '## Pass Condition Results' section is missing"
    );
    assert!(
        stderr2.to_lowercase().contains("pass condition"),
        "scenario 2: warning must mention 'pass condition'"
    );

    // ── Scenario 3: done phase, ❌ in results → exit 0, stderr warning ─────────
    let dir3 = tempfile::tempdir().unwrap();
    init_workflow_state(dir3.path(), "done", &bin);
    let workflow_dir3 = dir3.path().join(".claude/workflow");
    std::fs::create_dir_all(&workflow_dir3).unwrap();
    std::fs::write(workflow_dir3.join("implement-done.md"),
        "# Implement Done\n\n## Pass Condition Results\n\n- PC-001 ✅ passed\n- PC-002 ❌ failed\n\nAll pass conditions: 1/2\n",
    ).unwrap();
    let out3 = run_pass_condition_check(dir3.path(), &bin);
    assert_eq!(
        out3.status.code(),
        Some(0),
        "scenario 3: must exit 0 even with failures"
    );
    let stderr3 = std::str::from_utf8(&out3.stderr).unwrap_or("");
    assert!(
        !stderr3.trim().is_empty(),
        "scenario 3: must warn when ❌ failures found"
    );
    assert!(
        stderr3.contains("❌") || stderr3.to_lowercase().contains("fail"),
        "scenario 3: warning must mention failures, got stderr: {stderr3}"
    );

    // ── Scenario 4: plan phase → exit 0, silent ───────────────────────────────
    let dir4 = tempfile::tempdir().unwrap();
    init_workflow_state(dir4.path(), "plan", &bin);
    let out4 = run_pass_condition_check(dir4.path(), &bin);
    assert_eq!(
        out4.status.code(),
        Some(0),
        "scenario 4: must exit 0 in plan phase"
    );
    assert!(
        out4.stdout.trim_ascii().is_empty() && out4.stderr.trim_ascii().is_empty(),
        "scenario 4: must be silent in plan phase"
    );
}
