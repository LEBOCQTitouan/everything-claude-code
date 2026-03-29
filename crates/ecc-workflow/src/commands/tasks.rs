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

/// A single item in the sync output arrays.
#[derive(serde::Serialize)]
struct SyncItem {
    id: String,
    description: String,
    current_status: String,
}

/// The sync JSON output schema.
#[derive(serde::Serialize)]
struct SyncOutput {
    pending: Vec<SyncItem>,
    completed: Vec<SyncItem>,
    in_progress: Vec<SyncItem>,
    failed: Vec<SyncItem>,
    total: usize,
    progress_pct: f64,
}

/// Build the sync JSON output from a parsed `TaskReport`.
fn build_sync_output(report: &ecc_domain::task::TaskReport) -> SyncOutput {
    use ecc_domain::task::entry::EntryKind;

    let mut pending = Vec::new();
    let mut completed = Vec::new();
    let mut in_progress = Vec::new();
    let mut failed = Vec::new();

    for entry in &report.entries {
        let id = match &entry.kind {
            EntryKind::Pc(pc_id) => pc_id.to_string(),
            EntryKind::PostTdd(label) => label.clone(),
        };
        let current_status = entry
            .trail
            .last()
            .map(|s| s.status.to_string())
            .unwrap_or_else(|| "pending".to_owned());

        let item = SyncItem {
            id,
            description: entry.description.clone(),
            current_status: current_status.clone(),
        };

        match current_status.as_str() {
            "done" => completed.push(item),
            "red" | "green" => in_progress.push(item),
            "failed" => failed.push(item),
            _ => pending.push(item),
        }
    }

    SyncOutput {
        total: report.total,
        progress_pct: report.progress_pct,
        pending,
        completed,
        in_progress,
        failed,
    }
}

/// Run the `tasks sync <path>` subcommand.
///
/// - File I/O errors (missing file, permission denied) → `block`
/// - Parse errors (malformed tasks.md) → `warn`
/// - Success → `pass` with JSON message
pub fn run_sync(path: &str, project_dir: &Path) -> WorkflowOutput {
    let tasks_path = Path::new(path);

    // Validate path before any I/O
    if let Err(e) = validate_path(tasks_path, project_dir) {
        return WorkflowOutput::block(format!("tasks sync failed: {e}"));
    }

    // File I/O error → block
    let content = match std::fs::read_to_string(tasks_path) {
        Ok(c) => c,
        Err(e) => return WorkflowOutput::block(format!("cannot read {path}: {e}")),
    };

    // Parse error → warn
    let report = match ecc_domain::task::parser::parse_tasks(&content) {
        Ok(r) => r,
        Err(e) => return WorkflowOutput::warn(format!("malformed tasks.md: {e}")),
    };

    let sync_output = build_sync_output(&report);
    let json = match serde_json::to_string(&sync_output) {
        Ok(j) => j,
        Err(e) => return WorkflowOutput::block(format!("failed to serialize sync output: {e}")),
    };

    WorkflowOutput::pass(json)
}

/// Run the `tasks update <path> <id> <status>` subcommand.
pub fn run_update(path: &str, id: &str, status: &str, project_dir: &Path) -> WorkflowOutput {
    let _ = (path, id, status, project_dir);
    WorkflowOutput::block("not implemented")
}

/// Run the `tasks init <design_path> --output <output> [--force]` subcommand.
pub fn run_init(
    design_path: &str,
    output: &str,
    force: bool,
    project_dir: &Path,
) -> WorkflowOutput {
    let _ = (design_path, output, force, project_dir);
    WorkflowOutput::block("not implemented")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::Status;
    use tempfile::TempDir;

    /// A minimal valid tasks.md fixture with one pending PC and one done PC.
    fn valid_tasks_md() -> String {
        // Uses the parser's expected format: `PC-NNN: <desc> | \`cmd\` | <trail>`
        "# Tasks\n\
         \n\
         ## Pass Conditions\n\
         \n\
         - [ ] PC-001: Implement feature A | `cargo test pc001` | pending@2026-03-29T10:00:00Z\n\
         - [x] PC-002: Implement feature B | `cargo test pc002` | pending@2026-03-29T10:00:00Z \u{2192} done@2026-03-29T11:00:00Z\n\
         \n\
         ## Post-TDD\n\
         \n\
         - [ ] E2E tests | pending@2026-03-29T10:00:00Z\n"
            .to_owned()
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
        // Write a line that looks like a PC entry but has a malformed trail segment
        std::fs::write(
            &tasks_path,
            "## Pass Conditions\n\
             - [ ] PC-001: description | `cmd` | BADTRAIL_NO_AT_SIGN\n",
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
