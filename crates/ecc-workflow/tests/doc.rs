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

fn run_doc_level_check(
    project_dir: &std::path::Path,
    bin: &std::path::Path,
) -> std::process::Output {
    Command::new(bin)
        .args(["doc-level-check"])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .output()
        .expect("failed to execute ecc-workflow doc-level-check")
}

/// doc_enforcement: verify that `ecc-workflow doc-enforcement` checks for required sections.
#[test]
fn doc_enforcement() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    fn write_state(dir: &std::path::Path, phase: &str) {
        let workflow_dir = dir.join(".claude/workflow");
        std::fs::create_dir_all(&workflow_dir).unwrap();
        let state =
            serde_json::json!({"phase": phase, "concern": "dev", "feature": "test-feature"});
        std::fs::write(
            workflow_dir.join("state.json"),
            serde_json::to_string(&state).unwrap(),
        )
        .unwrap();
    }

    fn write_implement_done(dir: &std::path::Path, content: &str) {
        let workflow_dir = dir.join(".claude/workflow");
        std::fs::create_dir_all(&workflow_dir).unwrap();
        std::fs::write(workflow_dir.join("implement-done.md"), content).unwrap();
    }

    fn run_doc_enforcement(dir: &std::path::Path, bin: &std::path::Path) -> std::process::Output {
        Command::new(bin)
            .args(["doc-enforcement"])
            .env("CLAUDE_PROJECT_DIR", dir)
            .output()
            .expect("failed to execute ecc-workflow doc-enforcement")
    }

    let full_content = "## Docs Updated\n- Updated CLAUDE.md with new feature docs\n\n## Supplemental Docs\n- docs/specs/2026-03-23-my-feature/design.md\n";

    // ── Scenario 1: both sections present → exit 0, no warning ───────────────
    let dir1 = tempfile::tempdir().unwrap();
    write_state(dir1.path(), "done");
    write_implement_done(dir1.path(), full_content);
    let out1 = run_doc_enforcement(dir1.path(), &bin);
    assert_eq!(out1.status.code(), Some(0), "scenario 1: must exit 0");
    let stderr1 = std::str::from_utf8(&out1.stderr)
        .unwrap_or("")
        .trim()
        .to_string();
    assert!(
        stderr1.is_empty(),
        "scenario 1: expected no stderr warning, got: '{stderr1}'"
    );

    // ── Scenario 2: missing "## Docs Updated" → exit 0, warning on stderr ────
    let dir2 = tempfile::tempdir().unwrap();
    write_state(dir2.path(), "done");
    write_implement_done(
        dir2.path(),
        "## Supplemental Docs\n- docs/specs/my-feature/design.md\n",
    );
    let out2 = run_doc_enforcement(dir2.path(), &bin);
    assert_eq!(out2.status.code(), Some(0), "scenario 2: must exit 0");
    let stderr2 = std::str::from_utf8(&out2.stderr)
        .unwrap_or("")
        .trim()
        .to_string();
    assert!(
        !stderr2.is_empty(),
        "scenario 2: expected warning on stderr when '## Docs Updated' missing"
    );
    let json2: serde_json::Value = serde_json::from_str(&stderr2)
        .unwrap_or_else(|e| panic!("scenario 2: stderr is not valid JSON: {e}\nstderr: {stderr2}"));
    assert_eq!(
        json2.get("status").and_then(|v| v.as_str()),
        Some("warn"),
        "scenario 2: expected status 'warn'"
    );

    // ── Scenario 3: missing "## Supplemental Docs" → exit 0, warning on stderr
    let dir3 = tempfile::tempdir().unwrap();
    write_state(dir3.path(), "done");
    write_implement_done(dir3.path(), "## Docs Updated\n- Updated CLAUDE.md\n");
    let out3 = run_doc_enforcement(dir3.path(), &bin);
    assert_eq!(out3.status.code(), Some(0), "scenario 3: must exit 0");
    let stderr3 = std::str::from_utf8(&out3.stderr)
        .unwrap_or("")
        .trim()
        .to_string();
    assert!(
        !stderr3.is_empty(),
        "scenario 3: expected warning on stderr when '## Supplemental Docs' missing"
    );
    let json3: serde_json::Value = serde_json::from_str(&stderr3)
        .unwrap_or_else(|e| panic!("scenario 3: stderr is not valid JSON: {e}\nstderr: {stderr3}"));
    assert_eq!(
        json3.get("status").and_then(|v| v.as_str()),
        Some("warn"),
        "scenario 3: expected status 'warn'"
    );

    // ── Scenario 4: state at "plan" → exit 0, no warning ─────────────────────
    let dir4 = tempfile::tempdir().unwrap();
    write_state(dir4.path(), "plan");
    let out4 = run_doc_enforcement(dir4.path(), &bin);
    assert_eq!(
        out4.status.code(),
        Some(0),
        "scenario 4: must exit 0 for non-done phase"
    );
    let stderr4 = std::str::from_utf8(&out4.stderr)
        .unwrap_or("")
        .trim()
        .to_string();
    assert!(
        stderr4.is_empty(),
        "scenario 4: expected no stderr for non-done phase, got: '{stderr4}'"
    );

    // ── Scenario 5: no state.json → exit 0, silent ───────────────────────────
    let dir5 = tempfile::tempdir().unwrap();
    let out5 = run_doc_enforcement(dir5.path(), &bin);
    assert_eq!(
        out5.status.code(),
        Some(0),
        "scenario 5: must exit 0 when no state.json"
    );
    let stderr5 = std::str::from_utf8(&out5.stderr)
        .unwrap_or("")
        .trim()
        .to_string();
    assert!(
        stderr5.is_empty(),
        "scenario 5: expected silent output when no state.json, got: '{stderr5}'"
    );
}

/// doc_level_check: verify that `ecc-workflow doc-level-check` warns about oversized CLAUDE.md.
#[test]
fn doc_level_check() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {bin:?}");

    // ── Scenario 1: no state.json → exits 0, silent ───────────────────────────
    let dir_no_state = tempfile::tempdir().unwrap();
    let out1 = run_doc_level_check(dir_no_state.path(), &bin);
    assert_eq!(
        out1.status.code(),
        Some(0),
        "doc-level-check must exit 0 with no state.json"
    );
    assert!(
        out1.stdout.trim_ascii().is_empty() && out1.stderr.trim_ascii().is_empty(),
        "doc-level-check must be silent when no state.json"
    );

    // ── Scenario 2: plan phase → exits 0, silent ──────────────────────────────
    let dir_plan = tempfile::tempdir().unwrap();
    init_workflow_state(dir_plan.path(), "plan", &bin);
    let out2 = run_doc_level_check(dir_plan.path(), &bin);
    assert_eq!(
        out2.status.code(),
        Some(0),
        "doc-level-check must exit 0 in plan phase"
    );
    assert!(
        out2.stdout.trim_ascii().is_empty() && out2.stderr.trim_ascii().is_empty(),
        "doc-level-check must be silent in plan phase"
    );

    // ── Scenario 3: done phase, small CLAUDE.md → exits 0, no warning ─────────
    let dir_done_small = tempfile::tempdir().unwrap();
    init_workflow_state(dir_done_small.path(), "done", &bin);
    let small_content: String = (1..=10).map(|i| format!("line {i}\n")).collect();
    std::fs::write(dir_done_small.path().join("CLAUDE.md"), &small_content).unwrap();
    let out3 = run_doc_level_check(dir_done_small.path(), &bin);
    assert_eq!(
        out3.status.code(),
        Some(0),
        "doc-level-check must exit 0 for small CLAUDE.md"
    );
    let stderr3 = std::str::from_utf8(&out3.stderr).unwrap_or("");
    assert!(
        !stderr3.to_lowercase().contains("claude.md"),
        "doc-level-check must not warn for small CLAUDE.md"
    );

    // ── Scenario 4: done phase, oversized CLAUDE.md → exits 0, warns ──────────
    let dir_done_big = tempfile::tempdir().unwrap();
    init_workflow_state(dir_done_big.path(), "done", &bin);
    let big_content: String = (1..=201).map(|i| format!("line {i}\n")).collect();
    std::fs::write(dir_done_big.path().join("CLAUDE.md"), &big_content).unwrap();
    let out4 = run_doc_level_check(dir_done_big.path(), &bin);
    assert_eq!(
        out4.status.code(),
        Some(0),
        "doc-level-check must always exit 0"
    );
    let stderr4 = std::str::from_utf8(&out4.stderr).unwrap_or("");
    assert!(
        stderr4.to_lowercase().contains("claude.md"),
        "doc-level-check must warn about oversized CLAUDE.md"
    );
}
