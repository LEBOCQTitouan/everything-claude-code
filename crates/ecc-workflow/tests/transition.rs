mod common;

use std::process::Command;

#[test]
fn transition_updates_state() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

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
    let content =
        std::fs::read_to_string(&state_path).expect("failed to read state.json after transition");

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
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

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
        .unwrap_or_else(|| panic!("JSON missing 'status' field: {value}"))
        .as_str()
        .unwrap_or_else(|| panic!("'status' is not a string: {value}"));

    assert_eq!(
        status, "block",
        "expected status 'block' for illegal transition, got '{status}'"
    );

    let message = value
        .get("message")
        .unwrap_or_else(|| panic!("JSON missing 'message' field: {value}"))
        .as_str()
        .unwrap_or_else(|| panic!("'message' is not a string: {value}"));

    assert!(
        !message.is_empty(),
        "expected non-empty message for illegal transition block"
    );
}

/// bypass_env_var: when ECC_WORKFLOW_BYPASS=1, all subcommands exit 0 immediately with no output
/// and without creating or modifying state.json.
#[test]
fn bypass_env_var() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

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

/// bypass_env_var_ignored: when ECC_WORKFLOW_BYPASS=1, ecc-workflow still exits 0 immediately
/// without creating state.json. This characterization test captures the current early-exit behavior
/// and will be updated in PC-017 when the env var check is removed.
#[test]
fn bypass_env_var_ignored() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    let temp_dir = tempfile::tempdir().unwrap();

    let output = Command::new(&bin)
        .args(["init", "refactor", "test feature"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .env("ECC_WORKFLOW_BYPASS", "1")
        .output()
        .expect("failed to execute ecc-workflow init with ECC_WORKFLOW_BYPASS=1");

    // Current behavior: exits 0 (early exit before any state is written)
    assert_eq!(
        output.status.code(),
        Some(0),
        "expected exit 0 with ECC_WORKFLOW_BYPASS=1, got: {:?}\nstdout: {}\nstderr: {}",
        output.status.code(),
        std::str::from_utf8(&output.stdout).unwrap_or(""),
        std::str::from_utf8(&output.stderr).unwrap_or(""),
    );

    // Current behavior: no state.json is created (binary exits before doing any work)
    let state_path = temp_dir.path().join(".claude/workflow/state.json");
    assert!(
        !state_path.exists(),
        "state.json must NOT be created when ECC_WORKFLOW_BYPASS=1 (current early-exit behavior)"
    );
}

/// transition_writes_memory: after a successful transition, memory files are created as internal
/// function calls (not subprocesses). Verifies AC-004.2 and AC-004.4.
#[test]
fn transition_writes_memory() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();

    // Initialize as git repo so resolve_repo_root succeeds for daily/memory-index
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("git init failed");

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

    let action_log_content =
        std::fs::read_to_string(&action_log_path).expect("failed to read action-log.json");
    let action_log: serde_json::Value =
        serde_json::from_str(&action_log_content).unwrap_or_else(|e| {
            panic!("action-log.json is not valid JSON: {e}\ncontent: {action_log_content}")
        });
    let entries = action_log
        .as_array()
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

    let plan_md_path = work_items_dir
        .join(format!("{today}-test-feature"))
        .join("plan.md");
    assert!(
        plan_md_path.exists(),
        "work-items/{today}-test-feature/plan.md must exist after transition, checked at {:?}",
        plan_md_path
    );

    // Step 5: daily/<today>.md must exist in the project memory dir under HOME
    let repo_root = ecc_flock::resolve_repo_root(temp_dir.path());
    let repo_root = std::fs::canonicalize(&repo_root).unwrap_or_else(|_| repo_root.to_path_buf());
    let abs_str = repo_root.to_string_lossy();
    let project_hash = abs_str.trim_start_matches('/').replace('/', "-");
    let daily_dir = home_dir
        .path()
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
