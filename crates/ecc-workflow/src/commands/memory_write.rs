use std::path::{Path, PathBuf};

use crate::output::WorkflowOutput;
use crate::slug::make_slug;
use crate::time::{utc_hhmm, utc_now_iso8601, utc_today};

#[cfg(test)]
mod memory_write {
    pub(super) mod tests {
        use tempfile::TempDir;

        /// PC-005: write_action and write_work_item resolve to repo root.
        ///
        /// When `project_dir` is inside a worktree, the memory dir should resolve
        /// to the repo root's `docs/memory`, not the worktree-local path.
        /// We verify this by checking that the memory dir path is derived from
        /// `ecc_flock::resolve_repo_root(project_dir)`.
        #[test]
        fn resolves_repo_root() {
            let tmp = TempDir::new().unwrap();
            let project_dir = tmp.path();

            // resolve_repo_root falls back to project_dir when not in a git repo
            let expected_root = ecc_flock::resolve_repo_root(project_dir);
            let expected_memory_dir = expected_root.join("docs/memory");

            // write_action should create memory dir under resolve_repo_root
            let _ = super::super::write_action("plan", "test", "success", "[]", project_dir);
            assert!(
                expected_memory_dir.exists(),
                "expected memory dir at {:?}",
                expected_memory_dir
            );

            // write_work_item should also use the same resolved root
            let _ = super::super::write_work_item("plan", "test feature", "dev", project_dir);
            let work_items = expected_memory_dir.join("work-items");
            assert!(
                work_items.exists(),
                "expected work-items dir at {:?}",
                work_items
            );
        }
    }
}

#[cfg(test)]
mod uses_correct_lock_tests {
    use tempfile::TempDir;

    /// PC-012: Each memory type creates its dedicated lock file.
    #[test]
    fn uses_correct_lock() {
        let tmp = TempDir::new().unwrap();
        let project_dir = tmp.path();

        // write_action → action-log.lock
        let _ = super::write_action("plan", "test", "success", "[]", project_dir);
        let action_lock = ecc_flock::lock_dir(project_dir).join("action-log.lock");
        assert!(
            action_lock.exists(),
            "action-log.lock not created at {:?}",
            action_lock
        );

        // write_work_item → work-item.lock
        let _ = super::write_work_item("plan", "test feature", "dev", project_dir);
        let work_item_lock = ecc_flock::lock_dir(project_dir).join("work-item.lock");
        assert!(
            work_item_lock.exists(),
            "work-item.lock not created at {:?}",
            work_item_lock
        );

        // write_daily → daily.lock
        let _ = super::write_daily("plan", "feature", "dev", project_dir);
        let daily_lock = ecc_flock::lock_dir(project_dir).join("daily.lock");
        assert!(
            daily_lock.exists(),
            "daily.lock not created at {:?}",
            daily_lock
        );

        // write_memory_index → memory-index.lock
        let _ = super::write_memory_index(project_dir);
        let memory_index_lock = ecc_flock::lock_dir(project_dir).join("memory-index.lock");
        assert!(
            memory_index_lock.exists(),
            "memory-index.lock not created at {:?}",
            memory_index_lock
        );
    }
}

/// Dispatch `memory-write` subcommands.
///
/// Supported kinds:
/// - `action <action_type> <description> <outcome> <artifacts_json>`
/// - `work-item <phase> <description> <concern>`
/// - `daily <phase> <feature> <concern>`
/// - `memory-index`
pub fn run(kind: &str, args: &[String], project_dir: &Path) -> WorkflowOutput {
    match kind {
        "action" => {
            if args.len() < 4 {
                return WorkflowOutput::block(
                    "memory-write action requires: <action_type> <description> <outcome> <artifacts_json>",
                );
            }
            match write_action(&args[0], &args[1], &args[2], &args[3], project_dir) {
                Ok(()) => WorkflowOutput::pass("action log entry written"),
                Err(e) => WorkflowOutput::block(format!("Failed to write action log: {e}")),
            }
        }
        "work-item" => {
            if args.len() < 3 {
                return WorkflowOutput::block(
                    "memory-write work-item requires: <phase> <description> <concern>",
                );
            }
            match write_work_item(&args[0], &args[1], &args[2], project_dir) {
                Ok(()) => WorkflowOutput::pass("work item file written"),
                Err(e) => WorkflowOutput::block(format!("Failed to write work item: {e}")),
            }
        }
        "daily" => {
            if args.len() < 3 {
                return WorkflowOutput::block(
                    "memory-write daily requires: <phase> <feature> <concern>",
                );
            }
            match write_daily(&args[0], &args[1], &args[2], project_dir) {
                Ok(()) => WorkflowOutput::pass("daily memory entry written"),
                Err(e) => WorkflowOutput::warn(format!("Failed to write daily memory: {e}")),
            }
        }
        "memory-index" => match write_memory_index(project_dir) {
            Ok(()) => WorkflowOutput::pass("MEMORY.md index updated"),
            Err(e) => WorkflowOutput::warn(format!("Failed to update MEMORY.md: {e}")),
        },
        other => WorkflowOutput::block(format!(
            "Unknown memory-write kind '{other}'. Use: action, work-item, daily, memory-index"
        )),
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

/// Resolve `~/.claude/projects/<project-hash>/memory/` for a given project dir.
///
/// The project hash is the absolute path with the leading `/` stripped and
/// remaining `/` replaced with `-`.
fn resolve_project_memory_dir(project_dir: &Path) -> Result<PathBuf, anyhow::Error> {
    let home = std::env::var("HOME").map_err(|_| anyhow::anyhow!("HOME env var not set"))?;

    // Canonicalize to get absolute path; fall back to as-is if that fails.
    let abs = std::fs::canonicalize(project_dir).unwrap_or_else(|_| project_dir.to_path_buf());
    let abs_str = abs.to_string_lossy();
    let project_hash = abs_str.trim_start_matches('/').replace('/', "-");

    Ok(PathBuf::from(home)
        .join(".claude/projects")
        .join(project_hash)
        .join("memory"))
}

// ── action ────────────────────────────────────────────────────────────────────

pub fn write_action(
    action_type: &str,
    description: &str,
    outcome: &str,
    artifacts_json: &str,
    project_dir: &Path,
) -> Result<(), anyhow::Error> {
    let _guard = ecc_flock::acquire(project_dir, "action-log")
        .map_err(|e| anyhow::anyhow!("Failed to acquire action-log lock: {e}"))?;
    let memory_dir = ecc_flock::resolve_repo_root(project_dir).join("docs/memory");
    let work_items_dir = memory_dir.join("work-items");
    std::fs::create_dir_all(&memory_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create memory dir: {e}"))?;
    std::fs::create_dir_all(&work_items_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create work-items dir: {e}"))?;

    let action_log = memory_dir.join("action-log.json");
    if !action_log.exists() {
        std::fs::write(&action_log, b"[]")
            .map_err(|e| anyhow::anyhow!("Failed to init action-log.json: {e}"))?;
    }

    let timestamp = utc_now_iso8601();
    let session_id = std::env::var("CLAUDE_SESSION_ID").unwrap_or_else(|_| "unknown".to_string());

    // Parse artifacts_json to ensure it's valid JSON
    let artifacts: serde_json::Value =
        serde_json::from_str(artifacts_json).unwrap_or(serde_json::Value::Array(vec![]));

    let entry = serde_json::json!({
        "timestamp": timestamp,
        "session_id": session_id,
        "action_type": action_type,
        "description": description,
        "artifacts": artifacts,
        "outcome": outcome,
        "tags": []
    });

    // Atomic append: read current array, push entry, write atomically
    let current_content = std::fs::read_to_string(&action_log)
        .map_err(|e| anyhow::anyhow!("Failed to read action-log.json: {e}"))?;
    let mut log: serde_json::Value =
        serde_json::from_str(&current_content).unwrap_or(serde_json::Value::Array(vec![]));

    if let Some(arr) = log.as_array_mut() {
        arr.push(entry);
    }

    let new_content = serde_json::to_string_pretty(&log)
        .map_err(|e| anyhow::anyhow!("Failed to serialize action log: {e}"))?;

    let tmp_path = memory_dir.join(".action-log.tmp");
    std::fs::write(&tmp_path, new_content)
        .map_err(|e| anyhow::anyhow!("Failed to write temp action log: {e}"))?;
    std::fs::rename(&tmp_path, &action_log)
        .map_err(|e| anyhow::anyhow!("Failed to atomically rename action log: {e}"))?;

    Ok(())
}

// ── work-item ─────────────────────────────────────────────────────────────────

pub fn write_work_item(
    phase: &str,
    description: &str,
    concern: &str,
    project_dir: &Path,
) -> Result<(), anyhow::Error> {
    let _guard = ecc_flock::acquire(project_dir, "work-item")
        .map_err(|e| anyhow::anyhow!("Failed to acquire work-item lock: {e}"))?;
    let memory_dir = ecc_flock::resolve_repo_root(project_dir).join("docs/memory");
    let work_items_dir = memory_dir.join("work-items");
    std::fs::create_dir_all(&work_items_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create work-items dir: {e}"))?;

    // Ensure action-log.json exists
    let action_log = memory_dir.join("action-log.json");
    if !action_log.exists() {
        std::fs::write(&action_log, b"[]")
            .map_err(|e| anyhow::anyhow!("Failed to init action-log.json: {e}"))?;
    }

    let today = utc_today();
    let slug = make_slug(description);
    let item_dir = work_items_dir.join(format!("{today}-{slug}"));
    std::fs::create_dir_all(&item_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create item dir: {e}"))?;

    let target_file = item_dir.join(format!("{phase}.md"));
    let timestamp = utc_now_iso8601();

    // Re-entry: append revision block
    if target_file.exists() {
        let revision = format!(
            "\n## Revision\n\nDate: {timestamp}\n\nRe-entry detected. This phase was re-executed.\n"
        );
        let mut content = std::fs::read_to_string(&target_file)
            .map_err(|e| anyhow::anyhow!("Failed to read existing work item: {e}"))?;
        content.push_str(&revision);
        std::fs::write(&target_file, content)
            .map_err(|e| anyhow::anyhow!("Failed to append revision: {e}"))?;
        return Ok(());
    }

    let content = match phase {
        "plan" => format!(
            "# Plan: {description}\n\
             \n\
             ## Context\n\
             \n\
             Concern: {concern}\n\
             Created: {timestamp}\n\
             \n\
             ## Decisions\n\
             \n\
             (Populated by /spec-* command output)\n\
             \n\
             ## User Stories\n\
             \n\
             (Populated by /spec-* command output)\n\
             \n\
             ## Outcome\n\
             \n\
             Phase completed at {timestamp}\n"
        ),
        "solution" => format!(
            "# Solution: {description}\n\
             \n\
             ## Context\n\
             \n\
             Concern: {concern}\n\
             Created: {timestamp}\n\
             \n\
             ## File Changes\n\
             \n\
             (Populated by /design command output)\n\
             \n\
             ## Pass Conditions\n\
             \n\
             (Populated by /design command output)\n\
             \n\
             ## Outcome\n\
             \n\
             Phase completed at {timestamp}\n"
        ),
        "implementation" => format!(
            "# Implementation: {description}\n\
             \n\
             ## Context\n\
             \n\
             Concern: {concern}\n\
             Created: {timestamp}\n\
             \n\
             ## Changes Made\n\
             \n\
             (Populated from implement-done.md)\n\
             \n\
             ## Test Results\n\
             \n\
             (Populated from implement-done.md)\n\
             \n\
             ## Outcome\n\
             \n\
             Phase completed at {timestamp}\n"
        ),
        other => {
            return Err(anyhow::anyhow!(
                "Unknown phase '{other}'. Use: plan, solution, implementation"
            ));
        }
    };

    std::fs::write(&target_file, content)
        .map_err(|e| anyhow::anyhow!("Failed to write work item file: {e}"))?;

    Ok(())
}

// ── daily ─────────────────────────────────────────────────────────────────────

pub fn write_daily(
    phase: &str,
    feature: &str,
    concern: &str,
    project_dir: &Path,
) -> Result<(), anyhow::Error> {
    let _guard = ecc_flock::acquire(project_dir, "daily")
        .map_err(|e| anyhow::anyhow!("Failed to acquire daily lock: {e}"))?;
    let memory_dir = resolve_project_memory_dir(project_dir).map_err(|e| anyhow::anyhow!("{e}"))?;
    let daily_dir = memory_dir.join("daily");
    std::fs::create_dir_all(&daily_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create daily dir: {e}"))?;

    let today = utc_today();
    let daily_file = daily_dir.join(format!("{today}.md"));

    // Init file if missing
    if !daily_file.exists() {
        let init_content = build_daily_init_content(&today, &daily_dir);
        let tmp = daily_dir.join(".daily.tmp");
        std::fs::write(&tmp, &init_content)
            .map_err(|e| anyhow::anyhow!("Failed to write daily temp: {e}"))?;
        std::fs::rename(&tmp, &daily_file)
            .map_err(|e| anyhow::anyhow!("Failed to rename daily file: {e}"))?;
    }

    // Read current content
    let mut content = std::fs::read_to_string(&daily_file)
        .map_err(|e| anyhow::anyhow!("Failed to read daily file: {e}"))?;

    // Ensure ## Activity and ## Insights sections exist
    if !content.contains("## Activity") {
        content.push_str("\n## Activity\n\n");
    }
    if !content.contains("## Insights") {
        content.push_str("\n## Insights\n\n");
    }

    // Append entry under ## Activity
    let now = utc_hhmm();
    let entry = format!("- [{now}] **{phase}** {feature} — {concern}");

    // Insert after ## Activity line (and any immediately following blank line)
    let new_content = insert_after_activity(&content, &entry);

    let tmp = daily_dir.join(".daily.tmp");
    std::fs::write(&tmp, new_content)
        .map_err(|e| anyhow::anyhow!("Failed to write daily temp: {e}"))?;
    std::fs::rename(&tmp, &daily_file)
        .map_err(|e| anyhow::anyhow!("Failed to rename daily file: {e}"))?;

    Ok(())
}

fn build_daily_init_content(today: &str, daily_dir: &Path) -> String {
    let mut content = format!("# Daily: {today}\n\n");

    // Link to recent previous sessions (last 3 .md files)
    let mut prev_files: Vec<_> = std::fs::read_dir(daily_dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
                .map(|e| e.path())
                .collect()
        })
        .unwrap_or_default();
    prev_files.sort();
    prev_files.reverse();

    if !prev_files.is_empty() {
        content.push_str("## Context from previous sessions\n\n");
        for f in prev_files.iter().take(3) {
            if let Some(name) = f.file_name() {
                let n = name.to_string_lossy();
                content.push_str(&format!("- [{n}]({n})\n"));
            }
        }
        content.push('\n');
    }

    content.push_str("## Activity\n\n## Insights\n\n");
    content
}

/// Insert `entry` immediately after the `## Activity` heading line (and its following blank line).
fn insert_after_activity(content: &str, entry: &str) -> String {
    let mut result = String::with_capacity(content.len() + entry.len() + 2);
    let mut inserted = false;

    let mut lines = content.lines().peekable();
    while let Some(line) = lines.next() {
        result.push_str(line);
        result.push('\n');

        if !inserted && line.trim() == "## Activity" {
            // Skip one blank line if present, then insert entry
            if lines.peek().is_some_and(|next| next.trim().is_empty())
                && let Some(blank) = lines.next()
            {
                result.push_str(blank);
                result.push('\n');
            }
            result.push_str(entry);
            result.push('\n');
            inserted = true;
        }
    }

    if !inserted {
        result.push_str("\n## Activity\n\n");
        result.push_str(entry);
        result.push('\n');
    }

    result
}

// ── memory-index ──────────────────────────────────────────────────────────────

pub fn write_memory_index(project_dir: &Path) -> Result<(), anyhow::Error> {
    let _guard = ecc_flock::acquire(project_dir, "memory-index")
        .map_err(|e| anyhow::anyhow!("Failed to acquire memory-index lock: {e}"))?;
    let memory_dir = resolve_project_memory_dir(project_dir).map_err(|e| anyhow::anyhow!("{e}"))?;
    std::fs::create_dir_all(&memory_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create memory dir: {e}"))?;

    let memory_file = memory_dir.join("MEMORY.md");

    // Create if missing
    if !memory_file.exists() {
        std::fs::write(&memory_file, "# Memory Index\n")
            .map_err(|e| anyhow::anyhow!("Failed to create MEMORY.md: {e}"))?;
    }

    let mut content = std::fs::read_to_string(&memory_file)
        .map_err(|e| anyhow::anyhow!("Failed to read MEMORY.md: {e}"))?;

    // Ensure ## Daily section exists
    if !content.contains("## Daily") {
        content.push_str("\n## Daily\n\n");
    }

    let today = utc_today();
    let link = format!("- [{today}](daily/{today}.md)");

    // Skip if already present
    if content.contains(&link) {
        return Ok(());
    }

    // Insert link after ## Daily heading (and its following blank line)
    let new_content = insert_after_daily_heading(&content, &link);

    let tmp = memory_dir.join(".memory.tmp");
    std::fs::write(&tmp, new_content)
        .map_err(|e| anyhow::anyhow!("Failed to write MEMORY.md temp: {e}"))?;
    std::fs::rename(&tmp, &memory_file)
        .map_err(|e| anyhow::anyhow!("Failed to rename MEMORY.md: {e}"))?;

    Ok(())
}

fn insert_after_daily_heading(content: &str, link: &str) -> String {
    let mut result = String::with_capacity(content.len() + link.len() + 2);
    let mut inserted = false;

    let mut lines = content.lines().peekable();
    while let Some(line) = lines.next() {
        result.push_str(line);
        result.push('\n');

        if !inserted && line.trim() == "## Daily" {
            // Skip one blank line if present
            if lines.peek().is_some_and(|next| next.trim().is_empty())
                && let Some(blank) = lines.next()
            {
                result.push_str(blank);
                result.push('\n');
            }
            result.push_str(link);
            result.push('\n');
            inserted = true;
        }
    }

    if !inserted {
        result.push_str("\n## Daily\n\n");
        result.push_str(link);
        result.push('\n');
    }

    result
}
