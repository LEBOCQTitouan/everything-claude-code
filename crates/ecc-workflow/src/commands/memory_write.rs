use std::path::{Path, PathBuf};

use crate::output::WorkflowOutput;
use crate::slug::make_slug;
use crate::time::{utc_hhmm, utc_now_iso8601, utc_today};

/// Dispatch `memory-write` subcommands.
///
/// Supported kinds:
/// - `action <action_type> <description> <outcome> <artifacts_json>`
/// - `work-item <phase> <description> <concern>`
/// - `daily <phase> <feature> <concern>`
/// - `memory-index`
pub fn run(kind: &str, args: &[String], project_dir: &Path) -> WorkflowOutput {
    tracing::info!(kind = kind, "memory-write: processing {kind} write");
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
/// Uses `ecc_flock::resolve_repo_root` to find the main repo root (even from
/// a worktree), ensuring all sessions produce the same project hash.
/// Returns an error if the resolved root is not a git repository.
fn resolve_project_memory_dir(project_dir: &Path) -> Result<PathBuf, anyhow::Error> {
    let home = std::env::var("HOME").map_err(|_| anyhow::anyhow!("HOME env var not set"))?;

    let repo_root = ecc_flock::resolve_repo_root(project_dir);
    // Canonicalize to resolve macOS symlinks (/var → /private/var) so the
    // hash is identical whether called from a worktree or the main repo.
    let repo_root =
        std::fs::canonicalize(&repo_root).unwrap_or_else(|_| repo_root.to_path_buf());
    if !repo_root.join(".git").exists() {
        return Err(anyhow::anyhow!(
            "not a git repository: {} (resolved from {})",
            repo_root.display(),
            project_dir.display(),
        ));
    }
    let abs_str = repo_root.to_string_lossy();
    let project_hash = abs_str.trim_start_matches('/').replace('/', "-");

    Ok(PathBuf::from(home)
        .join(".claude/projects")
        .join(project_hash)
        .join("memory"))
}

// ── pure content builders (no I/O) ───────────────────────────────────────────

/// Build a JSON action log entry from its constituent parts.
///
/// `artifacts_json` is parsed as JSON; on parse failure it defaults to `[]`.
pub(crate) fn build_action_entry(
    action_type: &str,
    description: &str,
    outcome: &str,
    artifacts_json: &str,
    session_id: &str,
    timestamp: &str,
) -> serde_json::Value {
    let artifacts: serde_json::Value =
        serde_json::from_str(artifacts_json).unwrap_or(serde_json::Value::Array(vec![]));
    serde_json::json!({
        "timestamp": timestamp,
        "session_id": session_id,
        "action_type": action_type,
        "description": description,
        "artifacts": artifacts,
        "outcome": outcome,
        "tags": []
    })
}

/// Build the markdown content for a work item phase file.
///
/// Returns `Err` for unknown phases.
pub(crate) fn try_build_work_item_content(
    phase: &str,
    description: &str,
    concern: &str,
    timestamp: &str,
) -> Result<String, anyhow::Error> {
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
    Ok(content)
}

/// Build the markdown content for a work item file (panics on unknown phase, for internal use).
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn build_work_item_content(
    phase: &str,
    description: &str,
    concern: &str,
    timestamp: &str,
) -> String {
    try_build_work_item_content(phase, description, concern, timestamp)
        .expect("valid phase required")
}

/// Build a single activity log entry line for the daily memory file.
pub(crate) fn build_daily_entry(phase: &str, feature: &str, concern: &str, time: &str) -> String {
    format!("- [{time}] **{phase}** {feature} — {concern}")
}

/// Build the markdown link line for a daily entry in MEMORY.md.
pub(crate) fn build_memory_index_link(today: &str) -> String {
    format!("- [{today}](daily/{today}.md)")
}

// ── action ────────────────────────────────────────────────────────────────────

pub fn write_action(
    action_type: &str,
    description: &str,
    outcome: &str,
    artifacts_json: &str,
    project_dir: &Path,
) -> Result<(), anyhow::Error> {
    let memory_dir = project_dir.join("docs/memory");
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

    let entry = build_action_entry(
        action_type,
        description,
        outcome,
        artifacts_json,
        &session_id,
        &timestamp,
    );

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
    let memory_dir = project_dir.join("docs/memory");
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

    let content = try_build_work_item_content(phase, description, concern, &timestamp)?;

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
    let entry = build_daily_entry(phase, feature, concern, &now);

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

/// Insert `entry` immediately after the given `heading` line (and its following blank line).
pub(crate) fn insert_after_heading(content: &str, heading: &str, entry: &str) -> String {
    let mut result = String::with_capacity(content.len() + entry.len() + 2);
    let mut inserted = false;

    let mut lines = content.lines().peekable();
    while let Some(line) = lines.next() {
        result.push_str(line);
        result.push('\n');

        if !inserted && line.trim() == heading {
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
        result.push_str(&format!("\n{heading}\n\n"));
        result.push_str(entry);
        result.push('\n');
    }

    result
}

/// Insert `entry` immediately after the `## Activity` heading line (and its following blank line).
fn insert_after_activity(content: &str, entry: &str) -> String {
    insert_after_heading(content, "## Activity", entry)
}

// ── memory-index ──────────────────────────────────────────────────────────────

pub fn write_memory_index(project_dir: &Path) -> Result<(), anyhow::Error> {
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
    let link = build_memory_index_link(&today);

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
    insert_after_heading(content, "## Daily", link)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── build_action_entry tests ──────────────────────────────────────────────

    #[test]
    fn build_action_entry_happy_path() {
        let entry = build_action_entry(
            "feat",
            "add feature X",
            "success",
            "[]",
            "session-1",
            "2026-01-01T00:00:00Z",
        );
        assert_eq!(entry["action_type"], "feat");
        assert_eq!(entry["description"], "add feature X");
        assert_eq!(entry["outcome"], "success");
        assert_eq!(entry["session_id"], "session-1");
        assert_eq!(entry["timestamp"], "2026-01-01T00:00:00Z");
    }

    #[test]
    fn build_action_entry_invalid_artifacts_json_defaults_to_empty_array() {
        let entry = build_action_entry("fix", "desc", "done", "not-valid-json", "sess", "ts");
        assert_eq!(entry["artifacts"], serde_json::json!([]));
    }

    #[test]
    fn build_action_entry_empty_strings_are_preserved() {
        let entry = build_action_entry("", "", "", "[]", "", "");
        assert_eq!(entry["action_type"], "");
        assert_eq!(entry["description"], "");
        assert_eq!(entry["outcome"], "");
    }

    // ── build_work_item_content tests ─────────────────────────────────────────

    #[test]
    fn build_work_item_content_plan_contains_concern() {
        let content =
            build_work_item_content("plan", "Test Feature", "auth", "2026-01-01T00:00:00Z");
        assert!(content.contains("Concern: auth"));
        assert!(content.contains("# Plan: Test Feature"));
    }

    #[test]
    fn build_work_item_content_solution_phase() {
        let content =
            build_work_item_content("solution", "My Solution", "infra", "2026-01-01T00:00:00Z");
        assert!(content.contains("# Solution: My Solution"));
        assert!(content.contains("## File Changes"));
    }

    #[test]
    fn build_work_item_content_implementation_phase() {
        let content = build_work_item_content(
            "implementation",
            "Impl desc",
            "concern",
            "2026-01-01T00:00:00Z",
        );
        assert!(content.contains("# Implementation: Impl desc"));
        assert!(content.contains("## Changes Made"));
    }

    #[test]
    fn build_work_item_content_unknown_phase_returns_error() {
        let result = try_build_work_item_content("unknown", "desc", "concern", "ts");
        assert!(result.is_err());
    }

    // ── build_daily_content tests ─────────────────────────────────────────────

    #[test]
    fn build_daily_entry_contains_phase_feature_concern() {
        let entry = build_daily_entry("spec", "auth feature", "security", "09:30");
        assert!(entry.contains("**spec**"));
        assert!(entry.contains("auth feature"));
        assert!(entry.contains("security"));
        assert!(entry.contains("09:30"));
    }

    #[test]
    fn build_daily_entry_empty_phase_produces_entry() {
        let entry = build_daily_entry("", "feature", "concern", "10:00");
        assert!(entry.contains("****"));
    }

    #[test]
    fn build_daily_entry_special_chars_preserved() {
        let entry = build_daily_entry("plan", "feat: add & fix", "a/b", "12:00");
        assert!(entry.contains("feat: add & fix"));
        assert!(entry.contains("a/b"));
    }

    // ── build_memory_index_link tests ─────────────────────────────────────────

    #[test]
    fn build_memory_index_link_correct_format() {
        let link = build_memory_index_link("2026-01-01");
        assert_eq!(link, "- [2026-01-01](daily/2026-01-01.md)");
    }

    #[test]
    fn build_memory_index_link_different_date() {
        let link = build_memory_index_link("2025-12-31");
        assert!(link.contains("2025-12-31"));
        assert!(link.contains("daily/2025-12-31.md"));
    }

    #[test]
    fn build_memory_index_link_is_markdown_list_item() {
        let link = build_memory_index_link("2026-03-28");
        assert!(link.starts_with("- ["));
    }

    // ── resolve_project_memory_dir tests ─────────────────────────────────────

    #[test]
    fn resolve_project_memory_dir_errors_on_non_git() {
        let tmp = tempfile::tempdir().unwrap();
        let result = resolve_project_memory_dir(tmp.path());
        assert!(result.is_err(), "should error on non-git directory");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not a git repository"),
            "error should mention 'not a git repository', got: {err_msg}"
        );
    }

    #[test]
    fn resolve_project_memory_dir_succeeds_for_git_repo() {
        let tmp = tempfile::tempdir().unwrap();
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(tmp.path())
            .output()
            .expect("git init failed");

        let result = resolve_project_memory_dir(tmp.path());
        assert!(result.is_ok(), "should succeed for git-initialized dir");
        let path = result.unwrap();
        let path_str = path.to_string_lossy();
        assert!(
            path_str.contains(".claude/projects/") && path_str.ends_with("/memory"),
            "path should contain .claude/projects/<hash>/memory, got: {path_str}"
        );
    }

    // ── insert_after_heading generic tests ────────────────────────────────────

    #[test]
    fn insert_after_heading_works_for_activity() {
        let content = "# Daily: 2026-01-01\n\n## Activity\n\n## Insights\n\n";
        let entry = "- [09:00] **spec** auth — security";
        let result = insert_after_heading(content, "## Activity", entry);
        let lines: Vec<&str> = result.lines().collect();
        let activity_pos = lines.iter().position(|l| *l == "## Activity").unwrap();
        let entry_pos = lines.iter().position(|l| *l == entry).unwrap();
        assert!(
            entry_pos > activity_pos,
            "entry should appear after ## Activity"
        );
        assert!(
            entry_pos <= activity_pos + 2,
            "entry should be inserted right after ## Activity (with optional blank)"
        );
    }

    #[test]
    fn insert_after_heading_works_for_daily() {
        let content = "# Memory Index\n\n## Daily\n\n";
        let entry = "- [2026-01-01](daily/2026-01-01.md)";
        let result = insert_after_heading(content, "## Daily", entry);
        let lines: Vec<&str> = result.lines().collect();
        let daily_pos = lines.iter().position(|l| *l == "## Daily").unwrap();
        let entry_pos = lines.iter().position(|l| *l == entry).unwrap();
        assert!(
            entry_pos > daily_pos,
            "entry should appear after ## Daily"
        );
        assert!(
            entry_pos <= daily_pos + 2,
            "entry should be inserted right after ## Daily (with optional blank)"
        );
    }
}
