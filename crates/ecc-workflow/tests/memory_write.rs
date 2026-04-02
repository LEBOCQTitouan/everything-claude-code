mod common;

use std::process::Command;

/// memory_write_subcommands: verify that `ecc-workflow memory-write` subcommands produce
/// the correct file structure matching the shell memory-writer.sh behavior.
///
/// AC-004.4 — action-log.json has correct entry schema, work-item files have correct headings,
/// daily file has Activity/Insights sections, MEMORY.md has ## Daily section with link.
#[test]
fn memory_write_subcommands() {
    let bin = common::binary_path();
    assert!(bin.exists(), "ecc-workflow binary not found at {:?}", bin);

    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let project_dir = temp_dir.path();
    let home_path = home_dir.path();

    // ── Step 1: action subcommand ─────────────────────────────────────────────
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

    let action_log_content =
        std::fs::read_to_string(&action_log_path).expect("failed to read action-log.json");
    let action_log: serde_json::Value =
        serde_json::from_str(&action_log_content).unwrap_or_else(|e| {
            panic!("action-log.json is not valid JSON: {e}\ncontent: {action_log_content}")
        });
    let entries = action_log
        .as_array()
        .unwrap_or_else(|| panic!("action-log.json must be a JSON array, got: {action_log}"));
    assert_eq!(
        entries.len(),
        1,
        "action-log.json must have exactly 1 entry after one write"
    );

    let entry = &entries[0];
    let timestamp = entry
        .get("timestamp")
        .and_then(|v| v.as_str())
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
        "entry.action_type must be 'plan'"
    );
    assert_eq!(
        entry.get("description").and_then(|v| v.as_str()),
        Some("test feature"),
        "entry.description must be 'test feature'"
    );
    assert_eq!(
        entry.get("outcome").and_then(|v| v.as_str()),
        Some("success"),
        "entry.outcome must be 'success'"
    );
    assert!(
        entry.get("artifacts").is_some(),
        "entry missing 'artifacts' field: {entry}"
    );
    let tags = entry
        .get("tags")
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("entry missing 'tags' array field: {entry}"));
    assert!(
        tags.is_empty(),
        "entry.tags must be empty array [], got: {tags:?}"
    );

    // ── Step 2: work-item subcommand ──────────────────────────────────────────
    let work_item_output = Command::new(&bin)
        .args(["memory-write", "work-item", "plan", "test feature", "dev"])
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

    let work_items_dir = project_dir.join("docs/memory/work-items");
    assert!(
        work_items_dir.exists(),
        "docs/memory/work-items/ directory must exist after memory-write work-item"
    );

    let item_entries: Vec<_> = std::fs::read_dir(&work_items_dir)
        .expect("failed to read work-items dir")
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(
        item_entries.len(),
        1,
        "must have exactly one work-item subdirectory"
    );

    let item_dir = item_entries[0].path();
    let dir_name = item_dir.file_name().unwrap().to_string_lossy();
    assert!(
        dir_name.starts_with("20") && dir_name.contains('-'),
        "work-item dir must start with date, got: '{dir_name}'"
    );

    let plan_file = item_dir.join("plan.md");
    assert!(
        plan_file.exists(),
        "plan.md must exist in work-item directory {:?}",
        item_dir
    );

    let plan_content = std::fs::read_to_string(&plan_file).expect("failed to read plan.md");
    assert!(
        plan_content.contains("# Plan:"),
        "plan.md must contain '# Plan:' heading"
    );
    assert!(
        plan_content.contains("## Context"),
        "plan.md must contain '## Context' section"
    );
    assert!(
        plan_content.contains("## Decisions"),
        "plan.md must contain '## Decisions' section"
    );
    assert!(
        plan_content.contains("## Outcome"),
        "plan.md must contain '## Outcome' section"
    );

    // Initialize as git repo so resolve_repo_root succeeds for daily/memory-index
    Command::new("git")
        .args(["init"])
        .current_dir(project_dir)
        .output()
        .expect("git init failed");

    // ── Step 3: daily subcommand ──────────────────────────────────────────────
    let daily_output = Command::new(&bin)
        .args(["memory-write", "daily", "plan", "test feature", "dev"])
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

    let repo_root = ecc_flock::resolve_repo_root(project_dir);
    let repo_root = std::fs::canonicalize(&repo_root).unwrap_or_else(|_| repo_root.to_path_buf());
    let abs_str = repo_root.to_string_lossy();
    let project_hash = abs_str.trim_start_matches('/').replace('/', "-");
    let daily_dir = home_path
        .join(".claude/projects")
        .join(&project_hash)
        .join("memory/daily");
    assert!(
        daily_dir.exists(),
        "daily memory directory must exist at {:?}",
        daily_dir
    );

    let daily_files: Vec<_> = std::fs::read_dir(&daily_dir)
        .expect("failed to read daily dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
        .collect();
    assert_eq!(daily_files.len(), 1, "must have exactly one daily .md file");

    let daily_content =
        std::fs::read_to_string(daily_files[0].path()).expect("failed to read daily file");
    assert!(
        daily_content.contains("## Activity"),
        "daily file must contain '## Activity' section"
    );
    assert!(
        daily_content.contains("## Insights"),
        "daily file must contain '## Insights' section"
    );
    assert!(
        daily_content.contains("**plan**"),
        "daily file must contain '**plan**' entry"
    );

    // ── Step 4: memory-index subcommand ──────────────────────────────────────
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

    let memory_file = home_path
        .join(".claude/projects")
        .join(&project_hash)
        .join("memory/MEMORY.md");
    assert!(
        memory_file.exists(),
        "MEMORY.md must exist at {:?}",
        memory_file
    );

    let memory_content = std::fs::read_to_string(&memory_file).expect("failed to read MEMORY.md");
    assert!(
        memory_content.contains("## Daily"),
        "MEMORY.md must contain '## Daily' section"
    );
    assert!(
        memory_content.contains("daily/"),
        "MEMORY.md must contain a daily/ link"
    );
    assert!(
        memory_content.contains(".md)"),
        "MEMORY.md must contain a .md) link"
    );
}
