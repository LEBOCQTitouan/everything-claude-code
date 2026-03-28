/// Multi-process action-log contention test (PC-008).
///
/// Spawns two concurrent `ecc-workflow memory-write action` processes against
/// the same temp directory to verify that flock serializes writes and both
/// entries end up in action-log.json.
///
/// Run with:
///   cargo test -p ecc-workflow --test action_log_contention -- --ignored
use std::path::PathBuf;
use std::process::Command;

fn binary_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // deps/
    path.pop(); // debug/
    path.push("ecc-workflow");
    path
}

/// PC-008: Two concurrent action-log writes — both appear.
///
/// Spawns two `memory-write action` processes concurrently.
/// After both complete, action-log.json must contain exactly 2 entries.
#[test]
#[ignore]
fn two_concurrent_action_log_writes_both_appear() {
    let bin = binary_path();
    let temp_dir = tempfile::tempdir().unwrap();
    let project_dir = temp_dir.path();

    // Initialize action-log.json with empty array
    let memory_dir = project_dir.join("docs/memory");
    std::fs::create_dir_all(&memory_dir).unwrap();
    std::fs::write(memory_dir.join("action-log.json"), b"[]").unwrap();

    // Spawn two concurrent action-log write processes
    let mut child1 = Command::new(&bin)
        .args([
            "memory-write",
            "action",
            "plan",
            "test entry A",
            "success",
            "[]",
        ])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .env_remove("ECC_WORKFLOW_BYPASS")
        .spawn()
        .expect("failed to spawn process 1");

    let mut child2 = Command::new(&bin)
        .args([
            "memory-write",
            "action",
            "plan",
            "test entry B",
            "success",
            "[]",
        ])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .env_remove("ECC_WORKFLOW_BYPASS")
        .spawn()
        .expect("failed to spawn process 2");

    let status1 = child1.wait().expect("child1 did not exit");
    let status2 = child2.wait().expect("child2 did not exit");

    assert!(
        status1.success(),
        "process 1 failed with exit code {:?}",
        status1.code()
    );
    assert!(
        status2.success(),
        "process 2 failed with exit code {:?}",
        status2.code()
    );

    // Read action-log.json and verify both entries are present
    let action_log_path = memory_dir.join("action-log.json");
    assert!(
        action_log_path.exists(),
        "action-log.json missing after concurrent writes"
    );

    let content =
        std::fs::read_to_string(&action_log_path).expect("failed to read action-log.json");
    let log: serde_json::Value = serde_json::from_str(&content)
        .expect("action-log.json is not valid JSON after concurrent writes");

    let entries = log
        .as_array()
        .expect("action-log.json root is not an array");
    assert_eq!(
        entries.len(),
        2,
        "expected 2 entries in action-log.json, got {}. Content:\n{}",
        entries.len(),
        content
    );
}
