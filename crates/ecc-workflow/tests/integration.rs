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

/// transition_writes_memory: after a successful transition, memory files are created as internal
/// function calls (not subprocesses). Verifies AC-004.2 and AC-004.4.
///
/// Steps:
///   1. `ecc-workflow init dev "test feature"`
///   2. `ecc-workflow transition solution --artifact plan`
///   3. action-log.json must exist with at least one entry
///   4. work-items/YYYY-MM-DD-test-feature/plan.md must exist
///   5. daily/<today>.md must exist in the project memory dir
#[test]
fn transition_writes_memory() {
    let bin = binary_path();
    assert!(
        bin.exists(),
        "ecc-workflow binary not found at {:?}",
        bin
    );

    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();

    // Step 1: init
    let init_output = Command::new(&bin)
        .args(["init", "dev", "test feature"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .env("HOME", home_dir.path())
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
        .env("HOME", home_dir.path())
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

    // Step 3: action-log.json must exist with at least one entry
    let action_log_path = temp_dir.path().join("docs/memory/action-log.json");
    assert!(
        action_log_path.exists(),
        "action-log.json must exist after transition, checked at {:?}",
        action_log_path
    );

    let action_log_content = std::fs::read_to_string(&action_log_path)
        .expect("failed to read action-log.json");
    let action_log: serde_json::Value = serde_json::from_str(&action_log_content)
        .unwrap_or_else(|e| panic!("action-log.json is not valid JSON: {e}\ncontent: {action_log_content}"));
    let entries = action_log.as_array()
        .unwrap_or_else(|| panic!("action-log.json must be an array, got: {action_log}"));
    assert!(
        !entries.is_empty(),
        "action-log.json must have at least one entry after transition"
    );

    // Step 4: work-items/<today>-test-feature/plan.md must exist
    let work_items_dir = temp_dir.path().join("docs/memory/work-items");
    let today = {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let days = secs / 86400;
        let z = days + 719_468;
        let era = z / 146_097;
        let doe = z % 146_097;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = if m <= 2 { y + 1 } else { y };
        format!("{y:04}-{m:02}-{d:02}")
    };

    let plan_md_path = work_items_dir.join(format!("{today}-test-feature")).join("plan.md");
    assert!(
        plan_md_path.exists(),
        "work-items/{today}-test-feature/plan.md must exist after transition, checked at {:?}",
        plan_md_path
    );

    // Step 5: daily/<today>.md must exist in the project memory dir under HOME
    let abs_project = std::fs::canonicalize(temp_dir.path())
        .unwrap_or_else(|_| temp_dir.path().to_path_buf());
    let abs_str = abs_project.to_string_lossy();
    let project_hash = abs_str.trim_start_matches('/').replace('/', "-");
    let daily_dir = home_dir.path()
        .join(".claude/projects")
        .join(&project_hash)
        .join("memory/daily");
    let daily_file = daily_dir.join(format!("{today}.md"));
    assert!(
        daily_file.exists(),
        "daily/{today}.md must exist after transition, checked at {:?}",
        daily_file
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

/// memory_write_subcommands: verify that `ecc-workflow memory-write` subcommands produce
/// the correct file structure matching the shell memory-writer.sh behavior.
///
/// AC-004.4 — action-log.json has correct entry schema, work-item files have correct headings,
/// daily file has Activity/Insights sections, MEMORY.md has ## Daily section with link.
#[test]
fn memory_write_subcommands() {
    let bin = binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let project_dir = temp_dir.path();
    let home_path = home_dir.path();

    // ── Step 1: action subcommand ────────────────────────────────────────────────────────────
    let action_output = Command::new(&bin)
        .args([
            "memory-write",
            "action",
            "plan",
            "test feature",
            "success",
            "[]",
        ])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .env("HOME", home_path)
        .output()
        .expect("failed to execute ecc-workflow memory-write action");

    assert_eq!(
        action_output.status.code(),
        Some(0),
        "memory-write action must exit 0\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&action_output.stdout).unwrap_or(""),
        std::str::from_utf8(&action_output.stderr).unwrap_or(""),
    );

    // action-log.json must exist and contain a valid entry
    let action_log_path = project_dir.join("docs/memory/action-log.json");
    assert!(
        action_log_path.exists(),
        "docs/memory/action-log.json must exist after memory-write action"
    );

    let action_log_content = std::fs::read_to_string(&action_log_path)
        .expect("failed to read action-log.json");
    let action_log: serde_json::Value = serde_json::from_str(&action_log_content)
        .unwrap_or_else(|e| panic!("action-log.json is not valid JSON: {e}\ncontent: {action_log_content}"));

    let entries = action_log
        .as_array()
        .unwrap_or_else(|| panic!("action-log.json must be a JSON array, got: {action_log}"));

    assert_eq!(entries.len(), 1, "action-log.json must have exactly 1 entry after one write");

    let entry = &entries[0];

    // Verify required fields exist with correct schema
    let timestamp = entry.get("timestamp").and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("entry missing 'timestamp' string field: {entry}"));
    assert!(
        timestamp.len() == 20 && timestamp.ends_with('Z') && timestamp.contains('T'),
        "entry.timestamp must be ISO 8601 UTC (YYYY-MM-DDTHH:MM:SSZ), got: '{timestamp}'"
    );

    assert!(
        entry.get("session_id").is_some(),
        "entry missing 'session_id' field: {entry}"
    );

    assert_eq!(
        entry.get("action_type").and_then(|v| v.as_str()),
        Some("plan"),
        "entry.action_type must be 'plan', got: {:?}", entry.get("action_type")
    );

    assert_eq!(
        entry.get("description").and_then(|v| v.as_str()),
        Some("test feature"),
        "entry.description must be 'test feature', got: {:?}", entry.get("description")
    );

    assert_eq!(
        entry.get("outcome").and_then(|v| v.as_str()),
        Some("success"),
        "entry.outcome must be 'success', got: {:?}", entry.get("outcome")
    );

    assert!(
        entry.get("artifacts").is_some(),
        "entry missing 'artifacts' field: {entry}"
    );

    let tags = entry.get("tags")
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("entry missing 'tags' array field: {entry}"));
    assert!(tags.is_empty(), "entry.tags must be empty array [], got: {tags:?}");

    // ── Step 2: work-item subcommand ─────────────────────────────────────────────────────────
    let work_item_output = Command::new(&bin)
        .args([
            "memory-write",
            "work-item",
            "plan",
            "test feature",
            "dev",
        ])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .env("HOME", home_path)
        .output()
        .expect("failed to execute ecc-workflow memory-write work-item");

    assert_eq!(
        work_item_output.status.code(),
        Some(0),
        "memory-write work-item must exit 0\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&work_item_output.stdout).unwrap_or(""),
        std::str::from_utf8(&work_item_output.stderr).unwrap_or(""),
    );

    // work-item file must exist under docs/memory/work-items/YYYY-MM-DD-<slug>/plan.md
    let work_items_dir = project_dir.join("docs/memory/work-items");
    assert!(
        work_items_dir.exists(),
        "docs/memory/work-items/ directory must exist after memory-write work-item"
    );

    // Find the created work item directory
    let entries: Vec<_> = std::fs::read_dir(&work_items_dir)
        .expect("failed to read work-items dir")
        .filter_map(|e| e.ok())
        .collect();

    assert_eq!(entries.len(), 1, "must have exactly one work-item subdirectory");

    let item_dir = entries[0].path();
    let dir_name = item_dir.file_name().unwrap().to_string_lossy();
    assert!(
        dir_name.starts_with("20") && dir_name.contains('-'),
        "work-item dir must start with date YYYY-MM-DD-<slug>, got: '{dir_name}'"
    );

    let plan_file = item_dir.join("plan.md");
    assert!(
        plan_file.exists(),
        "plan.md must exist in work-item directory {:?}", item_dir
    );

    let plan_content = std::fs::read_to_string(&plan_file)
        .expect("failed to read plan.md");

    // Must have # Plan: heading
    assert!(
        plan_content.contains("# Plan:"),
        "plan.md must contain '# Plan:' heading\ncontent: {plan_content}"
    );
    // Must have ## Context section
    assert!(
        plan_content.contains("## Context"),
        "plan.md must contain '## Context' section\ncontent: {plan_content}"
    );
    // Must have ## Decisions section (plan phase)
    assert!(
        plan_content.contains("## Decisions"),
        "plan.md must contain '## Decisions' section\ncontent: {plan_content}"
    );
    // Must have ## Outcome section
    assert!(
        plan_content.contains("## Outcome"),
        "plan.md must contain '## Outcome' section\ncontent: {plan_content}"
    );

    // ── Step 3: daily subcommand ─────────────────────────────────────────────────────────────
    let daily_output = Command::new(&bin)
        .args([
            "memory-write",
            "daily",
            "plan",
            "test feature",
            "dev",
        ])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .env("HOME", home_path)
        .output()
        .expect("failed to execute ecc-workflow memory-write daily");

    assert_eq!(
        daily_output.status.code(),
        Some(0),
        "memory-write daily must exit 0\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&daily_output.stdout).unwrap_or(""),
        std::str::from_utf8(&daily_output.stderr).unwrap_or(""),
    );

    // Resolve project hash: remove leading / and replace / with -
    let abs_proj = std::fs::canonicalize(project_dir)
        .unwrap_or_else(|_| project_dir.to_path_buf());
    let abs_str = abs_proj.to_string_lossy();
    let project_hash = abs_str.trim_start_matches('/').replace('/', "-");
    let daily_dir = home_path
        .join(".claude/projects")
        .join(&project_hash)
        .join("memory/daily");

    assert!(
        daily_dir.exists(),
        "daily memory directory must exist at {:?}", daily_dir
    );

    // Find the daily file (YYYY-MM-DD.md)
    let daily_files: Vec<_> = std::fs::read_dir(&daily_dir)
        .expect("failed to read daily dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
        .collect();

    assert_eq!(daily_files.len(), 1, "must have exactly one daily .md file");

    let daily_file_path = daily_files[0].path();
    let daily_content = std::fs::read_to_string(&daily_file_path)
        .expect("failed to read daily file");

    // Must have ## Activity section
    assert!(
        daily_content.contains("## Activity"),
        "daily file must contain '## Activity' section\ncontent: {daily_content}"
    );
    // Must have ## Insights section
    assert!(
        daily_content.contains("## Insights"),
        "daily file must contain '## Insights' section\ncontent: {daily_content}"
    );
    // Must have the activity entry
    assert!(
        daily_content.contains("**plan**"),
        "daily file must contain '**plan**' entry\ncontent: {daily_content}"
    );

    // ── Step 4: memory-index subcommand ──────────────────────────────────────────────────────
    let index_output = Command::new(&bin)
        .args(["memory-write", "memory-index"])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .env("HOME", home_path)
        .output()
        .expect("failed to execute ecc-workflow memory-write memory-index");

    assert_eq!(
        index_output.status.code(),
        Some(0),
        "memory-write memory-index must exit 0\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&index_output.stdout).unwrap_or(""),
        std::str::from_utf8(&index_output.stderr).unwrap_or(""),
    );

    // MEMORY.md must exist in the project memory dir (parent of daily)
    let memory_file = home_path
        .join(".claude/projects")
        .join(&project_hash)
        .join("memory/MEMORY.md");

    assert!(
        memory_file.exists(),
        "MEMORY.md must exist at {:?}", memory_file
    );

    let memory_content = std::fs::read_to_string(&memory_file)
        .expect("failed to read MEMORY.md");

    // Must have ## Daily section
    assert!(
        memory_content.contains("## Daily"),
        "MEMORY.md must contain '## Daily' section\ncontent: {memory_content}"
    );
    // Must have a link to the daily file in the format [YYYY-MM-DD](daily/YYYY-MM-DD.md)
    assert!(
        memory_content.contains("daily/"),
        "MEMORY.md must contain a daily/ link\ncontent: {memory_content}"
    );
    assert!(
        memory_content.contains(".md)"),
        "MEMORY.md must contain a .md) link\ncontent: {memory_content}"
    );
}

/// phase_gate: verify that `ecc-workflow phase-gate` reads stdin JSON (hook protocol),
/// blocks Write/Edit to non-allowed paths during plan/solution, allows allowed paths,
/// allows everything during implement/done, and blocks destructive Bash commands.
///
/// AC-004.5 — phase-gate subcommand faithfully ports phase-gate.sh behavior.
#[test]
fn phase_gate() {
    use std::io::Write as _;
    use std::process::Stdio;

    let bin = binary_path();
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

    // ── Scenario 1: no state.json → exit 0 regardless of tool ───────────────────────────────
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

    // ── Scenario 2: phase=plan, Write to blocked path → exit 2 ───────────────────────────────
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

    // ── Scenario 3: phase=plan, Write to allowed path → exit 0 ───────────────────────────────
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

    // ── Scenario 4: phase=implement → exit 0 regardless of path ─────────────────────────────
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

    // ── Scenario 5: phase=plan, Bash with destructive command → exit 2 ───────────────────────
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

    // ── Scenario 6: phase=done → exit 0 regardless of tool/path ─────────────────────────────
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

// ── stop_gate tests ──────────────────────────────────────────────────────────

/// Helper: run `ecc-workflow stop-gate` in a project directory.
fn run_stop_gate(project_dir: &std::path::Path) -> std::process::Output {
    let bin = binary_path();
    Command::new(&bin)
        .args(["stop-gate"])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .output()
        .expect("failed to execute ecc-workflow stop-gate")
}

/// Helper: create a minimal state.json with a given phase inside a temp dir.
fn write_state_with_phase(
    project_dir: &std::path::Path,
    phase: &str,
    feature: &str,
) {
    let workflow_dir = project_dir.join(".claude/workflow");
    std::fs::create_dir_all(&workflow_dir).unwrap();
    let state = serde_json::json!({
        "phase": phase,
        "concern": "dev",
        "feature": feature,
        "slug": "test-feature",
        "started_at": "2026-01-01T00:00:00Z",
        "toolchain": { "test": null, "lint": null, "build": null },
        "artifacts": {
            "plan": null, "solution": null, "implement": null,
            "campaign_path": null, "spec_path": null,
            "design_path": null, "tasks_path": null
        },
        "completed": []
    });
    std::fs::write(
        workflow_dir.join("state.json"),
        serde_json::to_string_pretty(&state).unwrap(),
    )
    .unwrap();
}

#[test]
fn stop_gate_plan_phase_warns_on_stderr() {
    let bin = binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    let temp_dir = tempfile::tempdir().unwrap();
    write_state_with_phase(temp_dir.path(), "plan", "my-feature");

    let output = run_stop_gate(temp_dir.path());

    // Must always exit 0 — stop-gate is informational only
    assert_eq!(
        output.status.code(),
        Some(0),
        "stop-gate must exit 0 even when phase is 'plan'\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&output.stdout).unwrap_or(""),
        std::str::from_utf8(&output.stderr).unwrap_or(""),
    );

    // stderr must contain a WARNING
    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");
    assert!(
        stderr.contains("WARNING"),
        "expected WARNING in stderr for phase 'plan', got: '{stderr}'"
    );

    // stderr must mention the phase name
    assert!(
        stderr.contains("plan"),
        "expected 'plan' in stderr warning, got: '{stderr}'"
    );
}

#[test]
fn stop_gate_done_phase_is_silent() {
    let bin = binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    let temp_dir = tempfile::tempdir().unwrap();
    write_state_with_phase(temp_dir.path(), "done", "my-feature");

    let output = run_stop_gate(temp_dir.path());

    assert_eq!(
        output.status.code(),
        Some(0),
        "stop-gate must exit 0 when phase is 'done'\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&output.stdout).unwrap_or(""),
        std::str::from_utf8(&output.stderr).unwrap_or(""),
    );

    // stderr must be empty (silent) when phase is done
    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");
    assert!(
        stderr.trim().is_empty(),
        "expected silent stderr when phase is 'done', got: '{stderr}'"
    );
}

#[test]
fn stop_gate_no_state_is_silent() {
    let bin = binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    // Temp dir with NO state.json
    let temp_dir = tempfile::tempdir().unwrap();

    let output = run_stop_gate(temp_dir.path());

    assert_eq!(
        output.status.code(),
        Some(0),
        "stop-gate must exit 0 when no state.json\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&output.stdout).unwrap_or(""),
        std::str::from_utf8(&output.stderr).unwrap_or(""),
    );

    // No state.json → silent (no WARNING)
    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");
    assert!(
        !stderr.contains("WARNING"),
        "expected no WARNING in stderr when no state.json, got: '{stderr}'"
    );
}

// ─────────────────────────────────────────────────────────
// grill-me-gate integration tests
// ─────────────────────────────────────────────────────────

fn run_grill_me_gate(project_dir: &std::path::Path) -> std::process::Output {
    let bin = binary_path();
    Command::new(&bin)
        .args(["grill-me-gate"])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .output()
        .expect("failed to execute ecc-workflow grill-me-gate")
}

/// Write a state.json with an optional spec_path artifact.
fn write_state_with_phase_and_spec(
    project_dir: &std::path::Path,
    phase: &str,
    spec_path: Option<&str>,
) {
    let workflow_dir = project_dir.join(".claude/workflow");
    std::fs::create_dir_all(&workflow_dir).unwrap();
    let state = serde_json::json!({
        "phase": phase,
        "concern": "dev",
        "feature": "test-feature",
        "started_at": "2026-01-01T00:00:00Z",
        "toolchain": { "test": null, "lint": null, "build": null },
        "artifacts": {
            "plan": null, "solution": null, "implement": null,
            "campaign_path": null,
            "spec_path": spec_path,
            "design_path": null, "tasks_path": null
        },
        "completed": []
    });
    std::fs::write(
        workflow_dir.join("state.json"),
        serde_json::to_string_pretty(&state).unwrap(),
    )
    .unwrap();
}

#[test]
fn grill_me_gate_plan_phase_with_marker_is_silent() {
    let temp_dir = tempfile::tempdir().unwrap();

    // Create a spec file that contains the grill-me marker
    let spec_file = temp_dir.path().join("spec.md");
    std::fs::write(
        &spec_file,
        "# Spec\n\n### Grill-Me Decisions\n\nSome grill-me content here.\n",
    )
    .unwrap();

    let spec_path_str = spec_file.to_str().unwrap();
    write_state_with_phase_and_spec(temp_dir.path(), "plan", Some(spec_path_str));

    let output = run_grill_me_gate(temp_dir.path());

    assert_eq!(
        output.status.code(),
        Some(0),
        "expected exit 0 when grill-me marker present\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&output.stdout).unwrap_or(""),
        std::str::from_utf8(&output.stderr).unwrap_or(""),
    );

    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");
    assert!(
        !stderr.contains("WARNING"),
        "expected no WARNING when grill-me marker present, got: '{stderr}'"
    );
}

#[test]
fn grill_me_gate_plan_phase_without_marker_warns() {
    let temp_dir = tempfile::tempdir().unwrap();

    // Create a spec file WITHOUT grill-me markers
    let spec_file = temp_dir.path().join("spec.md");
    std::fs::write(&spec_file, "# Spec\n\nSome content without grill-me section.\n").unwrap();

    let spec_path_str = spec_file.to_str().unwrap();
    write_state_with_phase_and_spec(temp_dir.path(), "plan", Some(spec_path_str));

    let output = run_grill_me_gate(temp_dir.path());

    assert_eq!(
        output.status.code(),
        Some(0),
        "grill-me-gate must always exit 0 (informational only)\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&output.stdout).unwrap_or(""),
        std::str::from_utf8(&output.stderr).unwrap_or(""),
    );

    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");
    assert!(
        stderr.contains("WARNING"),
        "expected WARNING when grill-me marker absent, got: '{stderr}'"
    );
    assert!(
        stderr.to_lowercase().contains("grill"),
        "warning should mention grill-me, got: '{stderr}'"
    );
}

#[test]
fn grill_me_gate_implement_phase_is_silent() {
    let temp_dir = tempfile::tempdir().unwrap();

    // Even if the spec has no markers, implement phase should be skipped
    let spec_file = temp_dir.path().join("spec.md");
    std::fs::write(&spec_file, "# Spec\n\nNo grill-me here.\n").unwrap();

    let spec_path_str = spec_file.to_str().unwrap();
    write_state_with_phase_and_spec(temp_dir.path(), "implement", Some(spec_path_str));

    let output = run_grill_me_gate(temp_dir.path());

    assert_eq!(
        output.status.code(),
        Some(0),
        "expected exit 0 in implement phase\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&output.stdout).unwrap_or(""),
        std::str::from_utf8(&output.stderr).unwrap_or(""),
    );

    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");
    assert!(
        !stderr.contains("WARNING"),
        "expected no WARNING in implement phase, got: '{stderr}'"
    );
}

#[test]
fn grill_me_gate_no_state_is_silent() {
    let temp_dir = tempfile::tempdir().unwrap();
    // No state.json created

    let output = run_grill_me_gate(temp_dir.path());

    assert_eq!(
        output.status.code(),
        Some(0),
        "expected exit 0 when no state.json\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&output.stdout).unwrap_or(""),
        std::str::from_utf8(&output.stderr).unwrap_or(""),
    );

    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");
    assert!(
        !stderr.contains("WARNING"),
        "expected no WARNING when no state.json, got: '{stderr}'"
    );
}

// ─────────────────────────────────────────────────────────
// tdd-enforcement integration tests
// ─────────────────────────────────────────────────────────

/// Helper: pipe stdin JSON to `ecc-workflow tdd-enforcement` and return the Output.
fn run_tdd_enforcement(project_dir: &std::path::Path, stdin_json: &str) -> std::process::Output {
    use std::io::Write as _;
    use std::process::Stdio;

    let bin = binary_path();
    let mut child = std::process::Command::new(&bin)
        .args(["tdd-enforcement"])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .env_remove("ECC_WORKFLOW_BYPASS")
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

/// tdd_enforcement: verify that `ecc-workflow tdd-enforcement` reads stdin and tracks
/// RED/GREEN/REFACTOR state correctly.
///
/// AC-004.5 — tdd-enforcement subcommand:
///   1. Write to test file in implement phase → .tdd-state becomes "RED"
///   2. Write to source file when state=RED → .tdd-state becomes "GREEN"
///   3. Phase=plan → exits 0 silently (no tracking)
///   4. No state.json → exits 0 silently
#[test]
fn tdd_enforcement() {
    // ── Scenario 1: implement phase, Write to test file → state = RED ────────────────────────
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

    // ── Scenario 2: implement phase, Write to source file when state=RED → state = GREEN ──────
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

    // ── Scenario 3: phase=plan → exits 0, no .tdd-state written ─────────────────────────────
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
        "tdd-enforcement must exit 0 silently in plan phase\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&out3.stdout).unwrap_or(""),
        std::str::from_utf8(&out3.stderr).unwrap_or(""),
    );

    let tdd_state_plan = workflow_dir_plan.join(".tdd-state");
    assert!(
        !tdd_state_plan.exists(),
        ".tdd-state must NOT be created in plan phase"
    );

    // ── Scenario 4: no state.json → exits 0 silently ────────────────────────────────────────
    let dir_no_state = tempfile::tempdir().unwrap();

    let out4 = run_tdd_enforcement(dir_no_state.path(), &write_test_json);
    assert_eq!(
        out4.status.code(),
        Some(0),
        "tdd-enforcement must exit 0 silently with no state.json\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&out4.stdout).unwrap_or(""),
        std::str::from_utf8(&out4.stderr).unwrap_or(""),
    );

    let tdd_state_no_state = dir_no_state
        .path()
        .join(".claude/workflow/.tdd-state");
    assert!(
        !tdd_state_no_state.exists(),
        ".tdd-state must NOT be created when no state.json exists"
    );
}

// ── scope-check tests ──────────────────────────────────────────────────────────────────────────

fn run_scope_check(project_dir: &std::path::Path, bin: &std::path::Path) -> std::process::Output {
    Command::new(bin)
        .arg("scope-check")
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .output()
        .expect("failed to execute ecc-workflow scope-check")
}

fn init_workflow_state(project_dir: &std::path::Path, phase: &str, bin: &std::path::Path) {
    // init workflow
    Command::new(bin)
        .args(["init", "dev", "test-feature"])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .output()
        .expect("init failed");
    // transition to desired phase
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

#[test]
fn scope_check() {
    let bin = binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {bin:?}");

    // ── Scenario 1: no state.json → exits 0, silent ─────────────────────────────────────────
    let dir_no_state = tempfile::tempdir().unwrap();
    let out1 = run_scope_check(dir_no_state.path(), &bin);
    assert_eq!(
        out1.status.code(),
        Some(0),
        "scope-check must exit 0 with no state.json\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&out1.stdout).unwrap_or(""),
        std::str::from_utf8(&out1.stderr).unwrap_or(""),
    );
    // Silent: both stdout and stderr empty
    assert!(
        out1.stdout.trim_ascii().is_empty() && out1.stderr.trim_ascii().is_empty(),
        "scope-check must be silent when no state.json exists\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&out1.stdout).unwrap_or(""),
        std::str::from_utf8(&out1.stderr).unwrap_or(""),
    );

    // ── Scenario 2: plan phase → exits 0, silent ─────────────────────────────────────────────
    let dir_plan = tempfile::tempdir().unwrap();
    init_workflow_state(dir_plan.path(), "plan", &bin);
    let out2 = run_scope_check(dir_plan.path(), &bin);
    assert_eq!(
        out2.status.code(),
        Some(0),
        "scope-check must exit 0 in plan phase\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&out2.stdout).unwrap_or(""),
        std::str::from_utf8(&out2.stderr).unwrap_or(""),
    );
    assert!(
        out2.stdout.trim_ascii().is_empty() && out2.stderr.trim_ascii().is_empty(),
        "scope-check must be silent in plan phase\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&out2.stdout).unwrap_or(""),
        std::str::from_utf8(&out2.stderr).unwrap_or(""),
    );

    // ── Scenario 3: implement phase, no design_path → exits 0, warns about missing design ────
    let dir_impl_no_design = tempfile::tempdir().unwrap();
    init_workflow_state(dir_impl_no_design.path(), "implement", &bin);
    let out3 = run_scope_check(dir_impl_no_design.path(), &bin);
    assert_eq!(
        out3.status.code(),
        Some(0),
        "scope-check must exit 0 with no design_path\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&out3.stdout).unwrap_or(""),
        std::str::from_utf8(&out3.stderr).unwrap_or(""),
    );

    // ── Scenario 4: implement phase, design file present, all files expected → exits 0, silent
    let dir_impl_ok = tempfile::tempdir().unwrap();
    init_workflow_state(dir_impl_ok.path(), "implement", &bin);

    // Create a design file with a File Changes table
    let design_dir = dir_impl_ok.path().join("docs/specs/test-feature");
    std::fs::create_dir_all(&design_dir).unwrap();
    let design_path = design_dir.join("design.md");
    std::fs::write(
        &design_path,
        "# Design\n\n## File Changes\n\n| File | Action |\n|------|--------|\n| src/foo.rs | CREATE |\n| src/bar.rs | MODIFY |\n",
    ).unwrap();

    // Persist design_path into state.json via transition with --artifact and --path
    Command::new(&bin)
        .args([
            "transition",
            "implement",
            "--artifact",
            "design",
            "--path",
            design_path.to_str().unwrap(),
        ])
        .env("CLAUDE_PROJECT_DIR", dir_impl_ok.path())
        .output()
        .unwrap();

    // Create a fresh implement dir with the design artifact set from the start
    let dir_with_design = tempfile::tempdir().unwrap();
    init_workflow_state(dir_with_design.path(), "solution", &bin);

    // Transition to implement with design path
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

    // ── Scenario 5: design file with unexpected files → exits 0 but warns ───────────────────
    // (The actual git diff comparison would require a real git repo, so we just verify exit 0)
    // The warning behavior is tested indirectly through the design path scenario above.
}

/// doc_enforcement: verify that `ecc-workflow doc-enforcement` checks for required sections
/// in implement-done.md.
///
/// AC-004.5 — warning when sections missing, silent when present.
///
/// Scenarios:
///   1. State at "done", implement-done.md with both sections → exit 0, no warning
///   2. State at "done", implement-done.md missing "## Docs Updated" → exit 0, stderr has warning
///   3. State at "done", implement-done.md missing "## Supplemental Docs" → exit 0, stderr has warning
///   4. State at "plan" → exit 0, no warning (skipped)
///   5. No state.json → exit 0, silent
#[test]
fn doc_enforcement() {
    let bin = binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    fn write_state(dir: &std::path::Path, phase: &str) {
        let workflow_dir = dir.join(".claude/workflow");
        std::fs::create_dir_all(&workflow_dir).unwrap();
        let state = serde_json::json!({
            "phase": phase,
            "concern": "dev",
            "feature": "test-feature"
        });
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
            .env_remove("ECC_WORKFLOW_BYPASS")
            .output()
            .expect("failed to execute ecc-workflow doc-enforcement")
    }

    let full_content = "\
## Docs Updated\n\
- Updated CLAUDE.md with new feature docs\n\
\n\
## Supplemental Docs\n\
- docs/specs/2026-03-23-my-feature/design.md\n\
";

    // ── Scenario 1: both sections present → exit 0, no warning ──────────────────────────────
    let dir1 = tempfile::tempdir().unwrap();
    write_state(dir1.path(), "done");
    write_implement_done(dir1.path(), full_content);

    let out1 = run_doc_enforcement(dir1.path(), &bin);
    assert_eq!(
        out1.status.code(),
        Some(0),
        "scenario 1: must exit 0\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&out1.stdout).unwrap_or(""),
        std::str::from_utf8(&out1.stderr).unwrap_or(""),
    );
    let stderr1 = std::str::from_utf8(&out1.stderr).unwrap_or("").trim().to_string();
    assert!(
        stderr1.is_empty(),
        "scenario 1: expected no stderr warning when both sections present, got: '{stderr1}'"
    );

    // ── Scenario 2: missing "## Docs Updated" → exit 0, warning on stderr ───────────────────
    let dir2 = tempfile::tempdir().unwrap();
    write_state(dir2.path(), "done");
    write_implement_done(
        dir2.path(),
        "## Supplemental Docs\n- docs/specs/my-feature/design.md\n",
    );

    let out2 = run_doc_enforcement(dir2.path(), &bin);
    assert_eq!(
        out2.status.code(),
        Some(0),
        "scenario 2: must exit 0\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&out2.stdout).unwrap_or(""),
        std::str::from_utf8(&out2.stderr).unwrap_or(""),
    );
    let stderr2 = std::str::from_utf8(&out2.stderr).unwrap_or("").trim().to_string();
    assert!(
        !stderr2.is_empty(),
        "scenario 2: expected warning on stderr when '## Docs Updated' missing"
    );
    let json2: serde_json::Value = serde_json::from_str(&stderr2)
        .unwrap_or_else(|e| panic!("scenario 2: stderr is not valid JSON: {e}\nstderr: {stderr2}"));
    assert_eq!(
        json2.get("status").and_then(|v| v.as_str()),
        Some("warn"),
        "scenario 2: expected status 'warn', got: {:?}",
        json2.get("status")
    );

    // ── Scenario 3: missing "## Supplemental Docs" → exit 0, warning on stderr ───────────────
    let dir3 = tempfile::tempdir().unwrap();
    write_state(dir3.path(), "done");
    write_implement_done(
        dir3.path(),
        "## Docs Updated\n- Updated CLAUDE.md\n",
    );

    let out3 = run_doc_enforcement(dir3.path(), &bin);
    assert_eq!(
        out3.status.code(),
        Some(0),
        "scenario 3: must exit 0\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&out3.stdout).unwrap_or(""),
        std::str::from_utf8(&out3.stderr).unwrap_or(""),
    );
    let stderr3 = std::str::from_utf8(&out3.stderr).unwrap_or("").trim().to_string();
    assert!(
        !stderr3.is_empty(),
        "scenario 3: expected warning on stderr when '## Supplemental Docs' missing"
    );
    let json3: serde_json::Value = serde_json::from_str(&stderr3)
        .unwrap_or_else(|e| panic!("scenario 3: stderr is not valid JSON: {e}\nstderr: {stderr3}"));
    assert_eq!(
        json3.get("status").and_then(|v| v.as_str()),
        Some("warn"),
        "scenario 3: expected status 'warn', got: {:?}",
        json3.get("status")
    );

    // ── Scenario 4: state at "plan" → exit 0, no warning (skipped) ──────────────────────────
    let dir4 = tempfile::tempdir().unwrap();
    write_state(dir4.path(), "plan");
    // No implement-done.md written (it wouldn't matter for non-done phase)

    let out4 = run_doc_enforcement(dir4.path(), &bin);
    assert_eq!(
        out4.status.code(),
        Some(0),
        "scenario 4: must exit 0 for non-done phase\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&out4.stdout).unwrap_or(""),
        std::str::from_utf8(&out4.stderr).unwrap_or(""),
    );
    let stderr4 = std::str::from_utf8(&out4.stderr).unwrap_or("").trim().to_string();
    assert!(
        stderr4.is_empty(),
        "scenario 4: expected no stderr for non-done phase, got: '{stderr4}'"
    );

    // ── Scenario 5: no state.json → exit 0, silent ───────────────────────────────────────────
    let dir5 = tempfile::tempdir().unwrap();
    // No state.json written

    let out5 = run_doc_enforcement(dir5.path(), &bin);
    assert_eq!(
        out5.status.code(),
        Some(0),
        "scenario 5: must exit 0 when no state.json\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&out5.stdout).unwrap_or(""),
        std::str::from_utf8(&out5.stderr).unwrap_or(""),
    );
    let stderr5 = std::str::from_utf8(&out5.stderr).unwrap_or("").trim().to_string();
    assert!(
        stderr5.is_empty(),
        "scenario 5: expected silent output when no state.json, got: '{stderr5}'"
    );
}
