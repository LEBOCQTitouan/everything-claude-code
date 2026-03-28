mod common;

use std::process::Command;

/// dual_invocation: verify both CLI args mode and stdin JSON mode produce equivalent results.
#[test]
fn dual_invocation() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    // --- Test 1: CLI mode (no stdin) ---
    let temp_dir_cli = tempfile::tempdir().unwrap();

    let cli_output = Command::new(&bin)
        .args(["init", "dev", "test feature"])
        .env("CLAUDE_PROJECT_DIR", temp_dir_cli.path())
        .stdin(std::process::Stdio::null())
        .output()
        .expect("failed to execute ecc-workflow init (CLI mode)");

    assert_eq!(
        cli_output.status.code(),
        Some(0),
        "CLI mode must exit 0, got: {:?}\nstdout: {}\nstderr: {}",
        cli_output.status.code(),
        std::str::from_utf8(&cli_output.stdout).unwrap_or(""),
        std::str::from_utf8(&cli_output.stderr).unwrap_or(""),
    );

    common::assert_structured_json_output(&cli_output);

    let cli_stdout = std::str::from_utf8(&cli_output.stdout)
        .unwrap_or("")
        .trim()
        .to_string();
    let cli_value: serde_json::Value = serde_json::from_str(&cli_stdout)
        .unwrap_or_else(|e| panic!("CLI mode stdout is not valid JSON: {e}\nstdout: {cli_stdout}"));
    assert_eq!(
        cli_value.get("status").and_then(|v| v.as_str()),
        Some("pass"),
        "CLI mode expected status 'pass'"
    );

    // --- Test 2: Stdin JSON mode ---
    let temp_dir_stdin = tempfile::tempdir().unwrap();
    let stdin_json = serde_json::json!({
        "tool_name": "Bash",
        "tool_input": { "command": "ecc-workflow init dev \"test feature\"" }
    })
    .to_string();

    let stdin_output = Command::new(&bin)
        .args(["init", "dev", "test feature"])
        .env("CLAUDE_PROJECT_DIR", temp_dir_stdin.path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(stdin_json.as_bytes()).ok();
            }
            child.wait_with_output()
        })
        .expect("failed to execute ecc-workflow init (stdin JSON mode)");

    assert_eq!(
        stdin_output.status.code(),
        Some(0),
        "stdin JSON mode must exit 0, got: {:?}\nstdout: {}\nstderr: {}",
        stdin_output.status.code(),
        std::str::from_utf8(&stdin_output.stdout).unwrap_or(""),
        std::str::from_utf8(&stdin_output.stderr).unwrap_or(""),
    );

    common::assert_structured_json_output(&stdin_output);

    let stdin_stdout = std::str::from_utf8(&stdin_output.stdout)
        .unwrap_or("")
        .trim()
        .to_string();
    let stdin_value: serde_json::Value = serde_json::from_str(&stdin_stdout).unwrap_or_else(|e| {
        panic!("Stdin JSON mode stdout is not valid JSON: {e}\nstdout: {stdin_stdout}")
    });
    assert_eq!(
        stdin_value.get("status").and_then(|v| v.as_str()),
        Some("pass"),
        "Stdin JSON mode expected status 'pass'"
    );

    assert_eq!(
        cli_value.get("status"),
        stdin_value.get("status"),
        "CLI and stdin JSON modes must produce equivalent status"
    );

    let cli_state: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(temp_dir_cli.path().join(".claude/workflow/state.json"))
            .expect("CLI mode state.json not found"),
    )
    .expect("CLI mode state.json is not valid JSON");

    let stdin_state: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(temp_dir_stdin.path().join(".claude/workflow/state.json"))
            .expect("Stdin JSON mode state.json not found"),
    )
    .expect("Stdin JSON mode state.json is not valid JSON");

    assert_eq!(
        cli_state.get("phase"),
        stdin_state.get("phase"),
        "Both modes must produce the same phase"
    );
    assert_eq!(
        cli_state.get("concern"),
        stdin_state.get("concern"),
        "Both modes must produce the same concern"
    );
    assert_eq!(
        cli_state.get("feature"),
        stdin_state.get("feature"),
        "Both modes must produce the same feature"
    );
}

/// init_matches_shell: verify that `ecc-workflow init` produces semantically equivalent
/// state.json to the shell version, including stale workflow archiving and artifact cleanup.
#[test]
fn init_matches_shell() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    // ── Part 1: fixture JSON comparison ──────────────────────────────────────
    let temp_dir = tempfile::tempdir().unwrap();
    let output = Command::new(&bin)
        .args(["init", "dev", "my feature"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .output()
        .expect("failed to execute ecc-workflow init");

    assert_eq!(
        output.status.code(),
        Some(0),
        "init must exit 0\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&output.stdout).unwrap_or(""),
        std::str::from_utf8(&output.stderr).unwrap_or(""),
    );

    let state_path = temp_dir.path().join(".claude/workflow/state.json");
    assert!(state_path.exists(), "state.json must exist after init");

    let content = std::fs::read_to_string(&state_path).expect("failed to read state.json");
    let v: serde_json::Value =
        serde_json::from_str(&content).expect("state.json must be valid JSON");

    assert_eq!(v["concern"].as_str(), Some("dev"), "concern must be 'dev'");
    assert_eq!(v["phase"].as_str(), Some("plan"), "phase must be 'plan'");
    assert_eq!(
        v["feature"].as_str(),
        Some("my feature"),
        "feature must be 'my feature'"
    );
    let started_at = v["started_at"]
        .as_str()
        .expect("started_at must be a string");
    assert!(
        started_at.len() == 20 && started_at.ends_with('Z') && started_at.contains('T'),
        "started_at must be YYYY-MM-DDTHH:MM:SSZ, got: '{started_at}'"
    );
    assert_eq!(
        v["toolchain"]["test"],
        serde_json::Value::Null,
        "toolchain.test must be null"
    );
    assert_eq!(
        v["toolchain"]["lint"],
        serde_json::Value::Null,
        "toolchain.lint must be null"
    );
    assert_eq!(
        v["toolchain"]["build"],
        serde_json::Value::Null,
        "toolchain.build must be null"
    );
    for key in &[
        "plan",
        "solution",
        "implement",
        "campaign_path",
        "spec_path",
        "design_path",
        "tasks_path",
    ] {
        assert_eq!(
            v["artifacts"][key],
            serde_json::Value::Null,
            "artifacts.{key} must be null"
        );
    }
    assert_eq!(
        v["completed"],
        serde_json::Value::Array(vec![]),
        "completed must be []"
    );

    // ── Part 2: stale workflow archiving ──────────────────────────────────────
    let temp2 = tempfile::tempdir().unwrap();
    let workflow_dir2 = temp2.path().join(".claude/workflow");
    std::fs::create_dir_all(&workflow_dir2).unwrap();

    let stale_json = serde_json::json!({
        "concern": "old-concern", "phase": "solution", "feature": "old feature",
        "started_at": "2026-01-01T00:00:00Z",
        "toolchain": { "test": null, "lint": null, "build": null },
        "artifacts": { "plan": null, "solution": null, "implement": null,
                       "campaign_path": null, "spec_path": null, "design_path": null, "tasks_path": null },
        "completed": []
    });
    std::fs::write(
        workflow_dir2.join("state.json"),
        serde_json::to_string_pretty(&stale_json).unwrap(),
    )
    .unwrap();
    std::fs::write(workflow_dir2.join("implement-done.md"), "done").unwrap();
    std::fs::write(workflow_dir2.join(".tdd-state"), "state").unwrap();

    let output2 = Command::new(&bin)
        .args(["init", "dev", "new feature"])
        .env("CLAUDE_PROJECT_DIR", temp2.path())
        .output()
        .expect("failed to execute ecc-workflow init (stale archiving)");

    assert_eq!(
        output2.status.code(),
        Some(0),
        "init must exit 0 even when overwriting stale state"
    );

    let archive_dir = workflow_dir2.join("archive");
    assert!(
        archive_dir.exists(),
        "archive/ dir must be created when archiving stale state"
    );

    let archived: Vec<_> = std::fs::read_dir(&archive_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(
        !archived.is_empty(),
        "at least one archived state file must exist in archive/"
    );
    assert!(
        !workflow_dir2.join("implement-done.md").exists(),
        "implement-done.md must be cleaned up during init"
    );
    assert!(
        !workflow_dir2.join(".tdd-state").exists(),
        ".tdd-state must be cleaned up during init"
    );

    let new_content = std::fs::read_to_string(workflow_dir2.join("state.json"))
        .expect("new state.json must exist");
    let new_v: serde_json::Value = serde_json::from_str(&new_content).expect("valid JSON");
    assert_eq!(
        new_v["feature"].as_str(),
        Some("new feature"),
        "new state must have new feature"
    );
    assert_eq!(
        new_v["phase"].as_str(),
        Some("plan"),
        "new state must start at plan phase"
    );
}

/// toolchain_persist: verify that `ecc-workflow toolchain-persist` writes toolchain fields to state.json.
#[test]
fn toolchain_persist() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    let temp_dir = tempfile::tempdir().unwrap();

    let init_output = Command::new(&bin)
        .args(["init", "dev", "test feature"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .output()
        .expect("failed to execute ecc-workflow init");

    assert_eq!(init_output.status.code(), Some(0), "init must exit 0");

    let persist_output = Command::new(&bin)
        .args([
            "toolchain-persist",
            "cargo test",
            "cargo clippy -- -D warnings",
            "cargo build",
        ])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .output()
        .expect("failed to execute ecc-workflow toolchain-persist");

    assert_eq!(
        persist_output.status.code(),
        Some(0),
        "toolchain-persist must exit 0"
    );

    let state_path = temp_dir.path().join(".claude/workflow/state.json");
    let content = std::fs::read_to_string(&state_path)
        .expect("failed to read state.json after toolchain-persist");
    let value: serde_json::Value =
        serde_json::from_str(&content).expect("state.json must be valid JSON");
    let toolchain = value
        .get("toolchain")
        .expect("missing 'toolchain' field in state.json");

    assert_eq!(
        toolchain.get("test").and_then(|v| v.as_str()),
        Some("cargo test"),
        "toolchain.test must be 'cargo test'"
    );
    assert_eq!(
        toolchain.get("lint").and_then(|v| v.as_str()),
        Some("cargo clippy -- -D warnings"),
        "toolchain.lint must match"
    );
    assert_eq!(
        toolchain.get("build").and_then(|v| v.as_str()),
        Some("cargo build"),
        "toolchain.build must be 'cargo build'"
    );
}
