//! `ecc-workflow tasks` subcommand handlers.
//!
//! Handles I/O for all tasks subcommands: path validation, file reads,
//! atomic writes, and flock locking. Calls domain functions for parsing,
//! validation, and rendering.

use std::path::Path;

use crate::output::WorkflowOutput;

/// Validate that `path` does not escape `project_dir`.
///
/// Uses `canonicalize` + `starts_with` to prevent symlink-based traversals.
/// For non-existent paths, canonicalizes the parent directory instead.
fn validate_path(path: &Path, project_dir: &Path) -> Result<(), anyhow::Error> {
    let resolved = if path.exists() {
        std::fs::canonicalize(path)?
    } else {
        let parent = path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("invalid path: no parent directory"))?;
        std::fs::canonicalize(parent)?.join(path.file_name().unwrap_or_default())
    };
    let project_root = std::fs::canonicalize(project_dir)
        .unwrap_or_else(|_| project_dir.to_path_buf());
    if !resolved.starts_with(&project_root) {
        anyhow::bail!("path escapes project directory: {}", path.display());
    }
    Ok(())
}

/// Run the `tasks sync <path>` subcommand.
pub fn run_sync(path: &str, project_dir: &Path) -> WorkflowOutput {
    // stub — RED phase
    WorkflowOutput::block("not implemented")
}

/// Run the `tasks update <path> <id> <status>` subcommand.
pub fn run_update(path: &str, id: &str, status: &str, project_dir: &Path) -> WorkflowOutput {
    WorkflowOutput::block("not implemented")
}

/// Run the `tasks init <design_path> --output <output> [--force]` subcommand.
pub fn run_init(
    design_path: &str,
    output: &str,
    force: bool,
    project_dir: &Path,
) -> WorkflowOutput {
    WorkflowOutput::block("not implemented")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::Status;
    use tempfile::TempDir;

    /// A minimal valid tasks.md fixture with one pending PC and one done PC.
    fn valid_tasks_md() -> &'static str {
        "# Tasks\n\
         \n\
         ## Pass Conditions\n\
         \n\
         - [ ] PC-001 — Implement feature A — `cargo test pc001` — pending@2026-03-29T10:00:00Z\n\
         - [x] PC-002 — Implement feature B — `cargo test pc002` — pending@2026-03-29T10:00:00Z → done@2026-03-29T11:00:00Z\n\
         \n\
         ## Post-TDD\n\
         \n\
         - [ ] E2E tests — pending@2026-03-29T10:00:00Z\n"
    }

    // PC-017: tasks sync outputs JSON with correct arrays and counters for valid tasks.md
    #[test]
    fn sync_valid_tasks() {
        let tmp = TempDir::new().unwrap();
        let tasks_path = tmp.path().join("tasks.md");
        std::fs::write(&tasks_path, valid_tasks_md()).unwrap();

        let output = run_sync(tasks_path.to_str().unwrap(), tmp.path());

        assert!(
            matches!(output.status, Status::Pass),
            "expected pass status, got: {:?} — {}",
            output.status,
            output.message
        );

        // Parse the JSON in the message
        let json: serde_json::Value = serde_json::from_str(&output.message)
            .expect("message should be valid JSON");

        // AC-003.1: arrays present
        assert!(json["pending"].is_array(), "pending must be an array");
        assert!(json["completed"].is_array(), "completed must be an array");
        assert!(json["in_progress"].is_array(), "in_progress must be an array");
        assert!(json["failed"].is_array(), "failed must be an array");
        assert!(json["total"].is_number(), "total must be a number");
        assert!(json["progress_pct"].is_number(), "progress_pct must be a number");

        // AC-003.2: pending contains only undone items
        let pending = json["pending"].as_array().unwrap();
        let completed = json["completed"].as_array().unwrap();
        assert_eq!(pending.len(), 2, "pending array should have 2 items (PC-001 + E2E tests)");
        assert_eq!(completed.len(), 1, "completed array should have 1 item (PC-002)");
        assert_eq!(json["total"], 3, "total should be 3");

        // Check structure of a pending item
        let first_pending = &pending[0];
        assert!(first_pending["id"].is_string(), "item must have id field");
        assert!(first_pending["description"].is_string(), "item must have description field");
        assert!(first_pending["current_status"].is_string(), "item must have current_status field");
    }

    // PC-018: tasks sync returns block error for nonexistent path
    #[test]
    fn sync_missing() {
        let tmp = TempDir::new().unwrap();
        let nonexistent = tmp.path().join("does_not_exist.md");

        let output = run_sync(nonexistent.to_str().unwrap(), tmp.path());

        assert!(
            matches!(output.status, Status::Block),
            "expected block status for missing file, got: {:?} — {}",
            output.status,
            output.message
        );
    }

    // PC-019: tasks sync returns warn for malformed tasks.md
    #[test]
    fn sync_malformed() {
        let tmp = TempDir::new().unwrap();
        let tasks_path = tmp.path().join("tasks.md");
        // Write garbage that looks like a checklist item but has an unparseable status trail
        std::fs::write(
            &tasks_path,
            "## Pass Conditions\n\
             - [ ] NOTAPC — description — `cmd` — !!!INVALID_STATUS@timestamp\n",
        )
        .unwrap();

        let output = run_sync(tasks_path.to_str().unwrap(), tmp.path());

        assert!(
            matches!(output.status, Status::Warn),
            "expected warn status for malformed tasks.md, got: {:?} — {}",
            output.status,
            output.message
        );
    }

    // PC-020: tasks sync rejects path traversal
    #[test]
    fn sync_traversal() {
        let tmp = TempDir::new().unwrap();
        // Use a path that is outside the project dir (tmp.path())
        // by providing a different tmpdir as project_dir
        let another_tmp = TempDir::new().unwrap();
        let tasks_path = tmp.path().join("tasks.md");
        std::fs::write(&tasks_path, valid_tasks_md()).unwrap();

        // project_dir is another_tmp — tasks_path is in tmp, outside project_dir
        let output = run_sync(tasks_path.to_str().unwrap(), another_tmp.path());

        assert!(
            matches!(output.status, Status::Block),
            "expected block status for path traversal, got: {:?} — {}",
            output.status,
            output.message
        );
    }
}
