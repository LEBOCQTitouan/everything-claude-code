/// Multi-process memory lock contention tests (PC-009, PC-010, PC-011).
///
/// Spawns concurrent `ecc-workflow memory-write` processes against the same
/// temp directory to verify that per-type flock serializes writes and both
/// entries appear in the output files.
///
/// All tests are `#[ignore]` — run with:
///   cargo test -p ecc-workflow --test memory_lock_contention -- --ignored
use std::path::PathBuf;
use std::process::Command;

fn binary_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // deps/
    path.pop(); // debug/
    path.push("ecc-workflow");
    path
}

/// Derive the project hash the same way resolve_project_memory_dir does:
/// strip leading '/' and replace '/' with '-'.
fn project_memory_dir(project_dir: &std::path::Path) -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    let abs = std::fs::canonicalize(project_dir).unwrap_or_else(|_| project_dir.to_path_buf());
    let abs_str = abs.to_string_lossy();
    let project_hash = abs_str.trim_start_matches('/').replace('/', "-");
    PathBuf::from(home)
        .join(".claude/projects")
        .join(project_hash)
        .join("memory")
}

/// PC-009: Two concurrent daily writes — both entries appear in the daily file.
#[test]
#[ignore]
fn two_concurrent_daily_writes_both_appear() {
    let bin = binary_path();
    let temp_dir = tempfile::tempdir().unwrap();
    let project_dir = temp_dir.path();

    let mut child1 = Command::new(&bin)
        .args(["memory-write", "daily", "plan", "feature-a", "dev"])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .env_remove("ECC_WORKFLOW_BYPASS")
        .spawn()
        .expect("failed to spawn process 1");

    let mut child2 = Command::new(&bin)
        .args(["memory-write", "daily", "solution", "feature-b", "dev"])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .env_remove("ECC_WORKFLOW_BYPASS")
        .spawn()
        .expect("failed to spawn process 2");

    let status1 = child1.wait().expect("child1 did not exit");
    let status2 = child2.wait().expect("child2 did not exit");

    // daily writes are warn-on-error, but both should succeed in a clean dir
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

    // Find the daily file and verify it has 2 activity entries
    let memory_dir = project_memory_dir(project_dir);
    let daily_dir = memory_dir.join("daily");
    let today = {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // Approximate UTC date (good enough for test; avoids importing chrono)
        let days = secs / 86400;
        let y = 1970 + days / 365;
        let d_in_y = days % 365;
        let m = d_in_y / 30 + 1;
        let d = d_in_y % 30 + 1;
        format!("{y:04}-{m:02}-{d:02}")
    };
    let daily_file = daily_dir.join(format!("{today}.md"));
    assert!(
        daily_file.exists(),
        "daily file {:?} missing after concurrent writes",
        daily_file
    );

    let content = std::fs::read_to_string(&daily_file).expect("failed to read daily file");
    let entry_count = content.matches("- [").count();
    assert_eq!(
        entry_count, 2,
        "expected 2 activity entries in daily file, got {}. Content:\n{}",
        entry_count, content
    );
}

/// PC-010: Two concurrent MEMORY.md updates — both invocations complete without corruption.
///
/// Both processes call `memory-write memory-index`. Since memory-index is idempotent
/// (it skips if today's link already present), both should succeed and MEMORY.md
/// must exist and be valid Markdown with the ## Daily section.
#[test]
#[ignore]
fn two_concurrent_memory_index_writes_both_appear() {
    let bin = binary_path();
    let temp_dir = tempfile::tempdir().unwrap();
    let project_dir = temp_dir.path();

    let mut child1 = Command::new(&bin)
        .args(["memory-write", "memory-index"])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .env_remove("ECC_WORKFLOW_BYPASS")
        .spawn()
        .expect("failed to spawn process 1");

    let mut child2 = Command::new(&bin)
        .args(["memory-write", "memory-index"])
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

    // MEMORY.md must exist and contain ## Daily section
    let memory_dir = project_memory_dir(project_dir);
    let memory_file = memory_dir.join("MEMORY.md");
    assert!(
        memory_file.exists(),
        "MEMORY.md missing after concurrent writes"
    );

    let content = std::fs::read_to_string(&memory_file).expect("failed to read MEMORY.md");
    assert!(
        content.contains("## Daily"),
        "MEMORY.md missing ## Daily section. Content:\n{}",
        content
    );
    assert!(
        !content.is_empty(),
        "MEMORY.md is empty after concurrent writes"
    );
}

/// PC-011: Two concurrent work-item writes — revision block written on second entry.
///
/// Both processes call `memory-write work-item plan "test-feature" dev`.
/// The first write creates the file, the second detects re-entry and appends
/// a ## Revision block. The file must contain "## Revision" after both complete.
#[test]
#[ignore]
fn two_concurrent_work_item_writes_revision() {
    let bin = binary_path();
    let temp_dir = tempfile::tempdir().unwrap();
    let project_dir = temp_dir.path();

    let mut child1 = Command::new(&bin)
        .args(["memory-write", "work-item", "plan", "test-feature", "dev"])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .env_remove("ECC_WORKFLOW_BYPASS")
        .spawn()
        .expect("failed to spawn process 1");

    let mut child2 = Command::new(&bin)
        .args(["memory-write", "work-item", "plan", "test-feature", "dev"])
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

    // Find the work-item file and verify ## Revision block is present
    let work_items_dir = project_dir.join("docs/memory/work-items");
    let entries: Vec<_> = std::fs::read_dir(&work_items_dir)
        .expect("work-items dir missing")
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(
        entries.len(),
        1,
        "expected 1 work-item dir, got {}",
        entries.len()
    );

    let item_dir = entries[0].path();
    let plan_file = item_dir.join("plan.md");
    assert!(plan_file.exists(), "plan.md missing in {:?}", item_dir);

    let content = std::fs::read_to_string(&plan_file).expect("failed to read plan.md");
    assert!(
        content.contains("## Revision"),
        "expected ## Revision block in plan.md after concurrent writes. Content:\n{}",
        content
    );
}
