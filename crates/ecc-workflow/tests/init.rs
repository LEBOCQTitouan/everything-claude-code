mod common;

use std::process::Command;

#[test]
fn missing_state_exits_zero_with_warning() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

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
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    let temp_dir = tempfile::tempdir().unwrap();

    // Test: init subcommand produces structured JSON
    let output = Command::new(&bin)
        .args(["init", "dev", "test-feature"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .output()
        .expect("failed to execute ecc-workflow init");

    common::assert_structured_json_output(&output);
}

#[test]
fn init_creates_state_json() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

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

    let content = std::fs::read_to_string(&state_path).expect("failed to read state.json");

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
