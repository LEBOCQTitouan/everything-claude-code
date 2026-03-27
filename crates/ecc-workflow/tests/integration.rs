use std::path::PathBuf;
use std::process::Command;

fn binary_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // remove test binary name
    path.pop(); // remove deps/
    path.push("ecc-workflow");
    path
}

fn valid_statuses() -> &'static [&'static str] {
    &["pass", "block", "warn"]
}

/// Parse stdout or stderr as JSON and verify the "status" field is in ["pass","block","warn"]
/// and a "message" field is present.
fn assert_structured_json_output(output: &std::process::Output) {
    // At least one of stdout or stderr should be non-empty JSON
    let stdout = std::str::from_utf8(&output.stdout).unwrap_or("");
    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");

    // Try stdout first, then stderr
    let json_str = if !stdout.trim().is_empty() {
        stdout.trim()
    } else if !stderr.trim().is_empty() {
        stderr.trim()
    } else {
        panic!(
            "Both stdout and stderr are empty. exit code: {:?}",
            output.status.code()
        );
    };

    let value: serde_json::Value = serde_json::from_str(json_str)
        .unwrap_or_else(|e| panic!("Output is not valid JSON: {e}\nOutput was: {json_str}"));

    let status = value
        .get("status")
        .unwrap_or_else(|| panic!("JSON missing 'status' field: {value}"))
        .as_str()
        .unwrap_or_else(|| panic!("'status' field is not a string: {value}"));

    assert!(
        valid_statuses().contains(&status),
        "'status' value '{status}' is not in {:?}",
        valid_statuses()
    );

    assert!(
        value.get("message").is_some(),
        "JSON missing 'message' field: {value}"
    );
}

#[test]
fn missing_state_exits_zero_with_warning() {
    let bin = binary_path();
    assert!(
        bin.exists(),
        "ecc-workflow binary not found at {:?}",
        bin
    );

    // Create a temp dir with NO state.json inside
    let temp_dir = tempfile::tempdir().unwrap();

    let output = std::process::Command::new(&bin)
        .args(["transition", "solution"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .output()
        .expect("failed to execute ecc-workflow transition");

    // Exit code must be 0
    assert_eq!(
        output.status.code(),
        Some(0),
        "expected exit 0 when state.json is missing, got: {:?}",
        output.status.code()
    );

    // stderr must contain JSON with status "warn"
    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");
    assert!(
        !stderr.trim().is_empty(),
        "expected non-empty stderr when state.json is missing"
    );

    let value: serde_json::Value = serde_json::from_str(stderr.trim())
        .unwrap_or_else(|e| panic!("stderr is not valid JSON: {e}\nstderr was: {stderr}"));

    let status = value
        .get("status")
        .unwrap_or_else(|| panic!("JSON missing 'status' field: {value}"))
        .as_str()
        .unwrap_or_else(|| panic!("'status' is not a string: {value}"));

    assert_eq!(
        status, "warn",
        "expected status 'warn' when state.json missing, got '{status}'"
    );

    let message = value
        .get("message")
        .unwrap_or_else(|| panic!("JSON missing 'message' field: {value}"))
        .as_str()
        .unwrap_or_else(|| panic!("'message' is not a string: {value}"));

    // The message must indicate the state is missing / workflow not initialized,
    // not just "not yet implemented"
    assert!(
        message.to_lowercase().contains("state")
            || message.to_lowercase().contains("not initialized")
            || message.to_lowercase().contains("workflow"),
        "warn message should indicate missing state, got: '{message}'"
    );
}

#[test]
fn output_is_structured_json() {
    let bin = binary_path();
    assert!(
        bin.exists(),
        "ecc-workflow binary not found at {:?}",
        bin
    );

    let temp_dir = tempfile::tempdir().unwrap();

    // Test: init subcommand produces structured JSON
    let output = Command::new(&bin)
        .args(["init", "dev", "test-feature"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .output()
        .expect("failed to execute ecc-workflow init");

    assert_structured_json_output(&output);
}

#[test]
fn init_creates_state_json() {
    let bin = binary_path();
    assert!(
        bin.exists(),
        "ecc-workflow binary not found at {:?}",
        bin
    );

    let temp_dir = tempfile::tempdir().unwrap();

    // Run init
    let output = Command::new(&bin)
        .args(["init", "dev", "test feature"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .output()
        .expect("failed to execute ecc-workflow init");

    assert_eq!(
        output.status.code(),
        Some(0),
        "expected exit 0, got: {:?}\nstdout: {}\nstderr: {}",
        output.status.code(),
        std::str::from_utf8(&output.stdout).unwrap_or(""),
        std::str::from_utf8(&output.stderr).unwrap_or(""),
    );

    // Read state.json from the temp dir
    let state_path = temp_dir.path().join(".claude/workflow/state.json");
    assert!(
        state_path.exists(),
        "state.json was not created at {:?}",
        state_path
    );

    let content = std::fs::read_to_string(&state_path)
        .expect("failed to read state.json");

    let value: serde_json::Value = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("state.json is not valid JSON: {e}\ncontent: {content}"));

    // concern == "dev"
    assert_eq!(
        value.get("concern").and_then(|v| v.as_str()),
        Some("dev"),
        "expected concern 'dev', got: {:?}",
        value.get("concern")
    );

    // phase == "plan"
    assert_eq!(
        value.get("phase").and_then(|v| v.as_str()),
        Some("plan"),
        "expected phase 'plan', got: {:?}",
        value.get("phase")
    );

    // feature == "test feature"
    assert_eq!(
        value.get("feature").and_then(|v| v.as_str()),
        Some("test feature"),
        "expected feature 'test feature', got: {:?}",
        value.get("feature")
    );

    // started_at is ISO 8601 format: YYYY-MM-DDTHH:MM:SSZ
    let started_at = value
        .get("started_at")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("missing 'started_at' field in state.json"));
    assert!(
        started_at.len() == 20 && started_at.ends_with('Z') && started_at.contains('T'),
        "started_at must be ISO 8601 UTC (YYYY-MM-DDTHH:MM:SSZ), got: '{started_at}'"
    );

    // toolchain fields are all null
    let toolchain = value
        .get("toolchain")
        .unwrap_or_else(|| panic!("missing 'toolchain' field in state.json"));
    assert_eq!(
        toolchain.get("test"),
        Some(&serde_json::Value::Null),
        "expected toolchain.test == null"
    );
    assert_eq!(
        toolchain.get("lint"),
        Some(&serde_json::Value::Null),
        "expected toolchain.lint == null"
    );
    assert_eq!(
        toolchain.get("build"),
        Some(&serde_json::Value::Null),
        "expected toolchain.build == null"
    );

    // artifacts fields are all null
    let artifacts = value
        .get("artifacts")
        .unwrap_or_else(|| panic!("missing 'artifacts' field in state.json"));
    for key in &["plan", "solution", "implement", "campaign_path", "spec_path", "design_path", "tasks_path"] {
        assert_eq!(
            artifacts.get(*key),
            Some(&serde_json::Value::Null),
            "expected artifacts.{key} == null"
        );
    }

    // completed == []
    let completed = value
        .get("completed")
        .unwrap_or_else(|| panic!("missing 'completed' field in state.json"));
    assert_eq!(
        completed,
        &serde_json::Value::Array(vec![]),
        "expected completed == []"
    );
}

#[test]
fn transition_updates_state() {
    let bin = binary_path();
    assert!(
        bin.exists(),
        "ecc-workflow binary not found at {:?}",
        bin
    );

    let temp_dir = tempfile::tempdir().unwrap();

    // Step 1: init to create state.json with phase=plan
    let init_output = Command::new(&bin)
        .args(["init", "dev", "test feature"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .output()
        .expect("failed to execute ecc-workflow init");

    assert_eq!(
        init_output.status.code(),
        Some(0),
        "init must exit 0, got: {:?}\nstdout: {}\nstderr: {}",
        init_output.status.code(),
        std::str::from_utf8(&init_output.stdout).unwrap_or(""),
        std::str::from_utf8(&init_output.stderr).unwrap_or(""),
    );

    // Step 2: transition to solution with --artifact plan
    let transition_output = Command::new(&bin)
        .args(["transition", "solution", "--artifact", "plan"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .output()
        .expect("failed to execute ecc-workflow transition");

    assert_eq!(
        transition_output.status.code(),
        Some(0),
        "transition must exit 0, got: {:?}\nstdout: {}\nstderr: {}",
        transition_output.status.code(),
        std::str::from_utf8(&transition_output.stdout).unwrap_or(""),
        std::str::from_utf8(&transition_output.stderr).unwrap_or(""),
    );

    // Step 3: read state.json and verify
    let state_path = temp_dir.path().join(".claude/workflow/state.json");
    let content = std::fs::read_to_string(&state_path)
        .expect("failed to read state.json after transition");

    let value: serde_json::Value = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("state.json is not valid JSON: {e}\ncontent: {content}"));

    // phase == "solution"
    assert_eq!(
        value.get("phase").and_then(|v| v.as_str()),
        Some("solution"),
        "expected phase 'solution' after transition, got: {:?}",
        value.get("phase")
    );

    // artifacts.plan has an ISO 8601 timestamp (not null)
    let artifacts = value
        .get("artifacts")
        .unwrap_or_else(|| panic!("missing 'artifacts' field in state.json"));

    let plan_ts = artifacts
        .get("plan")
        .unwrap_or_else(|| panic!("missing 'artifacts.plan' field"));

    assert!(
        !plan_ts.is_null(),
        "artifacts.plan must not be null after transition with --artifact plan"
    );

    let plan_str = plan_ts
        .as_str()
        .unwrap_or_else(|| panic!("artifacts.plan must be a string, got: {plan_ts}"));

    // ISO 8601 UTC: YYYY-MM-DDTHH:MM:SSZ — length 20, ends with Z, contains T
    assert!(
        plan_str.len() == 20 && plan_str.ends_with('Z') && plan_str.contains('T'),
        "artifacts.plan must be ISO 8601 UTC (YYYY-MM-DDTHH:MM:SSZ), got: '{plan_str}'"
    );
}

#[test]
fn transition_illegal_exits_nonzero() {
    let bin = binary_path();
    assert!(
        bin.exists(),
        "ecc-workflow binary not found at {:?}",
        bin
    );

    let temp_dir = tempfile::tempdir().unwrap();

    // Step 1: init to create state.json with phase=plan
    let init_output = Command::new(&bin)
        .args(["init", "dev", "test feature"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .output()
        .expect("failed to execute ecc-workflow init");

    assert_eq!(
        init_output.status.code(),
        Some(0),
        "init must exit 0, got: {:?}\nstdout: {}\nstderr: {}",
        init_output.status.code(),
        std::str::from_utf8(&init_output.stdout).unwrap_or(""),
        std::str::from_utf8(&init_output.stderr).unwrap_or(""),
    );

    // Step 2: attempt illegal transition plan->done (plan phase only allows solution)
    let transition_output = Command::new(&bin)
        .args(["transition", "done"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .output()
        .expect("failed to execute ecc-workflow transition done");

    // Exit code must be non-zero (expected: 2 for block)
    let code = transition_output.status.code();
    assert!(
        code != Some(0),
        "expected non-zero exit for illegal transition plan->done, got exit 0"
    );

    // stderr must contain JSON with status "block"
    let stderr = std::str::from_utf8(&transition_output.stderr).unwrap_or("");
    assert!(
        !stderr.trim().is_empty(),
        "expected non-empty stderr for illegal transition"
    );

    let value: serde_json::Value = serde_json::from_str(stderr.trim())
        .unwrap_or_else(|e| panic!("stderr is not valid JSON: {e}\nstderr was: {stderr}"));

    let status = value
        .get("status")
        .unwrap_or_else(|| panic!("JSON missing \'status\' field: {value}"))
        .as_str()
        .unwrap_or_else(|| panic!("\'status\' is not a string: {value}"));

    assert_eq!(
        status, "block",
        "expected status \'block\' for illegal transition, got \'{status}\'"
    );

    let message = value
        .get("message")
        .unwrap_or_else(|| panic!("JSON missing \'message\' field: {value}"))
        .as_str()
        .unwrap_or_else(|| panic!("\'message\' is not a string: {value}"));

    assert!(
        !message.is_empty(),
        "expected non-empty message for illegal transition block"
    );
}

/// bypass_env_var: when ECC_WORKFLOW_BYPASS=1, all subcommands exit 0 immediately with no output
/// and without creating or modifying state.json.
#[test]
fn bypass_env_var() {
    let bin = binary_path();
    assert!(
        bin.exists(),
        "ecc-workflow binary not found at {:?}",
        bin
    );

    // --- Test 1: init with bypass ---
    let temp_dir = tempfile::tempdir().unwrap();

    let output = Command::new(&bin)
        .args(["init", "dev", "test"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .env("ECC_WORKFLOW_BYPASS", "1")
        .output()
        .expect("failed to execute ecc-workflow init with bypass");

    assert_eq!(
        output.status.code(),
        Some(0),
        "expected exit 0 with ECC_WORKFLOW_BYPASS=1, got: {:?}",
        output.status.code()
    );

    let stdout = std::str::from_utf8(&output.stdout).unwrap_or("");
    assert!(
        stdout.trim().is_empty(),
        "expected empty stdout with ECC_WORKFLOW_BYPASS=1, got: '{stdout}'"
    );

    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");
    assert!(
        stderr.trim().is_empty(),
        "expected empty stderr with ECC_WORKFLOW_BYPASS=1, got: '{stderr}'"
    );

    let state_path = temp_dir.path().join(".claude/workflow/state.json");
    assert!(
        !state_path.exists(),
        "state.json must NOT be created when ECC_WORKFLOW_BYPASS=1"
    );

    // --- Test 2: transition with bypass ---
    let temp_dir2 = tempfile::tempdir().unwrap();

    let output2 = Command::new(&bin)
        .args(["transition", "solution"])
        .env("CLAUDE_PROJECT_DIR", temp_dir2.path())
        .env("ECC_WORKFLOW_BYPASS", "1")
        .output()
        .expect("failed to execute ecc-workflow transition with bypass");

    assert_eq!(
        output2.status.code(),
        Some(0),
        "expected exit 0 for transition with ECC_WORKFLOW_BYPASS=1, got: {:?}",
        output2.status.code()
    );

    let stdout2 = std::str::from_utf8(&output2.stdout).unwrap_or("");
    assert!(
        stdout2.trim().is_empty(),
        "expected empty stdout for transition with ECC_WORKFLOW_BYPASS=1, got: '{stdout2}'"
    );

    let stderr2 = std::str::from_utf8(&output2.stderr).unwrap_or("");
    assert!(
        stderr2.trim().is_empty(),
        "expected empty stderr for transition with ECC_WORKFLOW_BYPASS=1, got: '{stderr2}'"
    );
}

/// dual_invocation: verify both CLI args mode and stdin JSON mode produce equivalent results.
///
/// Test 1 — CLI mode: `ecc-workflow init dev "test feature"` with no stdin piped.
/// Test 2 — Stdin JSON mode: pipe JSON context on stdin while running `ecc-workflow init dev "test feature"`.
///
/// Both modes must exit 0 and produce structured JSON with status "pass".
#[test]
fn dual_invocation() {
    let bin = binary_path();
    assert!(
        bin.exists(),
        "ecc-workflow binary not found at {:?}",
        bin
    );

    // --- Test 1: CLI mode (no stdin) ---
    let temp_dir_cli = tempfile::tempdir().unwrap();

    let cli_output = Command::new(&bin)
        .args(["init", "dev", "test feature"])
        .env("CLAUDE_PROJECT_DIR", temp_dir_cli.path())
        // Explicitly set stdin to null/empty to ensure no piped data.
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

    assert_structured_json_output(&cli_output);

    let cli_stdout = std::str::from_utf8(&cli_output.stdout).unwrap_or("").trim().to_string();
    let cli_value: serde_json::Value = serde_json::from_str(&cli_stdout)
        .unwrap_or_else(|e| panic!("CLI mode stdout is not valid JSON: {e}\nstdout: {cli_stdout}"));

    assert_eq!(
        cli_value.get("status").and_then(|v| v.as_str()),
        Some("pass"),
        "CLI mode expected status 'pass', got: {:?}",
        cli_value.get("status")
    );

    // --- Test 2: Stdin JSON mode ---
    // Pipe a hooks.json-style JSON context on stdin. The binary should read and
    // process the stdin context without crashing, and still complete the init command
    // using the CLI args.
    let temp_dir_stdin = tempfile::tempdir().unwrap();

    let stdin_json = serde_json::json!({
        "tool_name": "Bash",
        "tool_input": {
            "command": "ecc-workflow init dev \"test feature\""
        }
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
                // Drop stdin to signal EOF
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

    assert_structured_json_output(&stdin_output);

    let stdin_stdout = std::str::from_utf8(&stdin_output.stdout).unwrap_or("").trim().to_string();
    let stdin_value: serde_json::Value = serde_json::from_str(&stdin_stdout)
        .unwrap_or_else(|e| panic!("Stdin JSON mode stdout is not valid JSON: {e}\nstdout: {stdin_stdout}"));

    assert_eq!(
        stdin_value.get("status").and_then(|v| v.as_str()),
        Some("pass"),
        "Stdin JSON mode expected status 'pass', got: {:?}",
        stdin_value.get("status")
    );

    // Both modes must produce the same status
    assert_eq!(
        cli_value.get("status"),
        stdin_value.get("status"),
        "CLI and stdin JSON modes must produce equivalent status"
    );

    // Both modes must create state.json with identical phase
    let cli_state_path = temp_dir_cli.path().join(".claude/workflow/state.json");
    let stdin_state_path = temp_dir_stdin.path().join(".claude/workflow/state.json");

    let cli_state: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&cli_state_path)
            .unwrap_or_else(|_| panic!("CLI mode state.json not found at {:?}", cli_state_path)),
    )
    .expect("CLI mode state.json is not valid JSON");

    let stdin_state: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&stdin_state_path)
            .unwrap_or_else(|_| panic!("Stdin JSON mode state.json not found at {:?}", stdin_state_path)),
    )
    .expect("Stdin JSON mode state.json is not valid JSON");

    assert_eq!(
        cli_state.get("phase"),
        stdin_state.get("phase"),
        "Both modes must produce the same phase in state.json"
    );

    assert_eq!(
        cli_state.get("concern"),
        stdin_state.get("concern"),
        "Both modes must produce the same concern in state.json"
    );

    assert_eq!(
        cli_state.get("feature"),
        stdin_state.get("feature"),
        "Both modes must produce the same feature in state.json"
    );
}

/// init_matches_shell: verify that `ecc-workflow init` produces semantically equivalent
/// state.json to the shell version, including stale workflow archiving and artifact cleanup.
///
/// AC-004.1 — same keys, same value types, same default values; stale archiving preserved.
#[test]
fn init_matches_shell() {
    let bin = binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    // ── Part 1: fixture JSON comparison ──────────────────────────────────────────────────────
    // Run init and verify every field matches the shell version's schema.
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

    // concern == "dev"
    assert_eq!(v["concern"].as_str(), Some("dev"), "concern must be 'dev'");
    // phase == "plan"
    assert_eq!(v["phase"].as_str(), Some("plan"), "phase must be 'plan'");
    // feature == "my feature"
    assert_eq!(v["feature"].as_str(), Some("my feature"), "feature must be 'my feature'");
    // started_at: ISO 8601 UTC, length 20, ends with Z
    let started_at = v["started_at"].as_str().expect("started_at must be a string");
    assert!(
        started_at.len() == 20 && started_at.ends_with('Z') && started_at.contains('T'),
        "started_at must be YYYY-MM-DDTHH:MM:SSZ, got: '{started_at}'"
    );
    // toolchain: test/lint/build all null
    assert_eq!(v["toolchain"]["test"], serde_json::Value::Null, "toolchain.test must be null");
    assert_eq!(v["toolchain"]["lint"], serde_json::Value::Null, "toolchain.lint must be null");
    assert_eq!(v["toolchain"]["build"], serde_json::Value::Null, "toolchain.build must be null");
    // artifacts: all expected keys null
    for key in &["plan", "solution", "implement", "campaign_path", "spec_path", "design_path", "tasks_path"] {
        assert_eq!(
            v["artifacts"][key],
            serde_json::Value::Null,
            "artifacts.{key} must be null"
        );
    }
    // completed == []
    assert_eq!(
        v["completed"],
        serde_json::Value::Array(vec![]),
        "completed must be []"
    );

    // ── Part 2: stale workflow archiving ─────────────────────────────────────────────────────
    // If state.json exists and phase != "done", it must be archived before writing the new one.
    let temp2 = tempfile::tempdir().unwrap();
    let workflow_dir2 = temp2.path().join(".claude/workflow");
    std::fs::create_dir_all(&workflow_dir2).unwrap();

    // Write a stale state.json at phase "solution"
    let stale_json = serde_json::json!({
        "concern": "old-concern",
        "phase": "solution",
        "feature": "old feature",
        "started_at": "2026-01-01T00:00:00Z",
        "toolchain": { "test": null, "lint": null, "build": null },
        "artifacts": { "plan": null, "solution": null, "implement": null, "campaign_path": null,
                       "spec_path": null, "design_path": null, "tasks_path": null },
        "completed": []
    });
    std::fs::write(
        workflow_dir2.join("state.json"),
        serde_json::to_string_pretty(&stale_json).unwrap(),
    )
    .unwrap();

    // Also create implement-done.md and .tdd-state to test cleanup
    std::fs::write(workflow_dir2.join("implement-done.md"), "done").unwrap();
    std::fs::write(workflow_dir2.join(".tdd-state"), "state").unwrap();

    // Run init over the stale state
    let output2 = Command::new(&bin)
        .args(["init", "dev", "new feature"])
        .env("CLAUDE_PROJECT_DIR", temp2.path())
        .output()
        .expect("failed to execute ecc-workflow init (stale archiving)");

    assert_eq!(
        output2.status.code(),
        Some(0),
        "init must exit 0 even when overwriting stale state\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&output2.stdout).unwrap_or(""),
        std::str::from_utf8(&output2.stderr).unwrap_or(""),
    );

    // Archive dir must exist
    let archive_dir = workflow_dir2.join("archive");
    assert!(archive_dir.exists(), "archive/ dir must be created when archiving stale state");

    // At least one archived state file must exist
    let archived: Vec<_> = std::fs::read_dir(&archive_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(
        !archived.is_empty(),
        "at least one archived state file must exist in archive/"
    );

    // implement-done.md must be deleted
    assert!(
        !workflow_dir2.join("implement-done.md").exists(),
        "implement-done.md must be cleaned up during init"
    );

    // .tdd-state must be deleted
    assert!(
        !workflow_dir2.join(".tdd-state").exists(),
        ".tdd-state must be cleaned up during init"
    );

    // New state.json has the new feature
    let new_content =
        std::fs::read_to_string(workflow_dir2.join("state.json")).expect("new state.json must exist");
    let new_v: serde_json::Value = serde_json::from_str(&new_content).expect("valid JSON");
    assert_eq!(new_v["feature"].as_str(), Some("new feature"), "new state must have new feature");
    assert_eq!(new_v["phase"].as_str(), Some("plan"), "new state must start at plan phase");
}

/// toolchain_persist: verify that `ecc-workflow toolchain-persist` writes toolchain fields to state.json.
#[test]
fn toolchain_persist() {
    let bin = binary_path();
    assert!(
        bin.exists(),
        "ecc-workflow binary not found at {:?}",
        bin
    );

    let temp_dir = tempfile::tempdir().unwrap();

    // Step 1: init to create state.json
    let init_output = Command::new(&bin)
        .args(["init", "dev", "test feature"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .output()
        .expect("failed to execute ecc-workflow init");

    assert_eq!(
        init_output.status.code(),
        Some(0),
        "init must exit 0, got: {:?}\nstdout: {}\nstderr: {}",
        init_output.status.code(),
        std::str::from_utf8(&init_output.stdout).unwrap_or(""),
        std::str::from_utf8(&init_output.stderr).unwrap_or(""),
    );

    // Step 2: run toolchain-persist with the three commands
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
        "toolchain-persist must exit 0, got: {:?}\nstdout: {}\nstderr: {}",
        persist_output.status.code(),
        std::str::from_utf8(&persist_output.stdout).unwrap_or(""),
        std::str::from_utf8(&persist_output.stderr).unwrap_or(""),
    );

    // Step 3: read state.json and verify toolchain fields
    let state_path = temp_dir.path().join(".claude/workflow/state.json");
    let content = std::fs::read_to_string(&state_path)
        .expect("failed to read state.json after toolchain-persist");

    let value: serde_json::Value = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("state.json is not valid JSON: {e}\ncontent: {content}"));

    let toolchain = value
        .get("toolchain")
        .unwrap_or_else(|| panic!("missing 'toolchain' field in state.json"));

    assert_eq!(
        toolchain.get("test").and_then(|v| v.as_str()),
        Some("cargo test"),
        "expected toolchain.test == 'cargo test', got: {:?}",
        toolchain.get("test")
    );

    assert_eq!(
        toolchain.get("lint").and_then(|v| v.as_str()),
        Some("cargo clippy -- -D warnings"),
        "expected toolchain.lint == 'cargo clippy -- -D warnings', got: {:?}",
        toolchain.get("lint")
    );

    assert_eq!(
        toolchain.get("build").and_then(|v| v.as_str()),
        Some("cargo build"),
        "expected toolchain.build == 'cargo build', got: {:?}",
        toolchain.get("build")
    );
}

/// transition_full_sequence: verify the complete workflow lifecycle with artifact stamping and path storage.
///
/// AC-004.2 — plan->solution->implement->done sequence:
///   - Each transition updates phase
///   - Artifact timestamps are ISO 8601 UTC
///   - Paths are stored in the correct artifact path fields
///   - done transition appends to completed array
#[test]
fn transition_full_sequence() {
    let bin = binary_path();
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
    // Expected: phase=solution, artifacts.plan stamped (ISO 8601), artifacts.spec_path set
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

    // phase == "solution"
    assert_eq!(
        v1["phase"].as_str(),
        Some("solution"),
        "phase must be 'solution' after plan->solution transition"
    );

    // artifacts.plan is ISO 8601 timestamp
    let plan_ts = v1["artifacts"]["plan"]
        .as_str()
        .expect("artifacts.plan must be a string after plan->solution");
    assert!(
        plan_ts.len() == 20 && plan_ts.ends_with('Z') && plan_ts.contains('T'),
        "artifacts.plan must be ISO 8601 UTC (YYYY-MM-DDTHH:MM:SSZ), got: '{plan_ts}'"
    );

    // artifacts.spec_path == "docs/specs/test/spec.md"
    assert_eq!(
        v1["artifacts"]["spec_path"].as_str(),
        Some("docs/specs/test/spec.md"),
        "artifacts.spec_path must be set after --path passed with --artifact plan"
    );

    // completed still empty
    assert_eq!(
        v1["completed"],
        serde_json::Value::Array(vec![]),
        "completed must still be [] after plan->solution"
    );

    // Step 3: transition implement --artifact solution --path "docs/specs/test/design.md"
    // Expected: phase=implement, artifacts.solution stamped, artifacts.design_path set
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

    // phase == "implement"
    assert_eq!(
        v2["phase"].as_str(),
        Some("implement"),
        "phase must be 'implement' after solution->implement transition"
    );

    // artifacts.solution is ISO 8601 timestamp
    let solution_ts = v2["artifacts"]["solution"]
        .as_str()
        .expect("artifacts.solution must be a string after solution->implement");
    assert!(
        solution_ts.len() == 20 && solution_ts.ends_with('Z') && solution_ts.contains('T'),
        "artifacts.solution must be ISO 8601 UTC (YYYY-MM-DDTHH:MM:SSZ), got: '{solution_ts}'"
    );

    // artifacts.design_path == "docs/specs/test/design.md"
    assert_eq!(
        v2["artifacts"]["design_path"].as_str(),
        Some("docs/specs/test/design.md"),
        "artifacts.design_path must be set after --path passed with --artifact solution"
    );

    // spec_path preserved from previous step
    assert_eq!(
        v2["artifacts"]["spec_path"].as_str(),
        Some("docs/specs/test/spec.md"),
        "artifacts.spec_path must be preserved after solution->implement transition"
    );

    // completed still empty
    assert_eq!(
        v2["completed"],
        serde_json::Value::Array(vec![]),
        "completed must still be [] after solution->implement"
    );

    // Step 4: transition done --artifact implement
    // Expected: phase=done, artifacts.implement stamped, completed has one entry
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

    // phase == "done"
    assert_eq!(
        v3["phase"].as_str(),
        Some("done"),
        "phase must be 'done' after implement->done transition"
    );

    // artifacts.implement is ISO 8601 timestamp
    let implement_ts = v3["artifacts"]["implement"]
        .as_str()
        .expect("artifacts.implement must be a string after implement->done");
    assert!(
        implement_ts.len() == 20 && implement_ts.ends_with('Z') && implement_ts.contains('T'),
        "artifacts.implement must be ISO 8601 UTC (YYYY-MM-DDTHH:MM:SSZ), got: '{implement_ts}'"
    );

    // completed has exactly one entry
    let completed = v3["completed"]
        .as_array()
        .expect("completed must be an array after implement->done");
    assert_eq!(
        completed.len(),
        1,
        "completed must have exactly one entry after implement->done, got: {completed:?}"
    );

    // completed[0].phase == "implement"
    assert_eq!(
        completed[0]["phase"].as_str(),
        Some("implement"),
        "completed[0].phase must be 'implement'"
    );

    // completed[0].file == "implement-done.md"
    assert_eq!(
        completed[0]["file"].as_str(),
        Some("implement-done.md"),
        "completed[0].file must be 'implement-done.md'"
    );

    // completed[0].at is ISO 8601 timestamp
    let done_at = completed[0]["at"]
        .as_str()
        .expect("completed[0].at must be a string");
    assert!(
        done_at.len() == 20 && done_at.ends_with('Z') && done_at.contains('T'),
        "completed[0].at must be ISO 8601 UTC (YYYY-MM-DDTHH:MM:SSZ), got: '{done_at}'"
    );

    // All previously stored paths are preserved
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
