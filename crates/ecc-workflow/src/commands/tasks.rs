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
    let project_root =
        std::fs::canonicalize(project_dir).unwrap_or_else(|_| project_dir.to_path_buf());
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
    use ecc_domain::task::TaskStatus;
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
        let last_status = entry
            .trail
            .last()
            .map(|s| s.status)
            .unwrap_or(TaskStatus::Pending);
        let current_status = last_status.to_string();

        let item = SyncItem {
            id,
            description: entry.description.clone(),
            current_status,
        };

        match last_status {
            TaskStatus::Done => completed.push(item),
            TaskStatus::Red | TaskStatus::Green => in_progress.push(item),
            TaskStatus::Failed => failed.push(item),
            TaskStatus::Pending => pending.push(item),
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
///
/// Atomically updates a single entry's status in tasks.md:
/// 1. Validates the path stays within the project directory
/// 2. Acquires an exclusive flock on "tasks"
/// 3. Reads the file, applies the domain update (FSM validation + trail append)
/// 4. Writes atomically via tempfile + rename
pub fn run_update(path: &str, id: &str, status: &str, project_dir: &Path) -> WorkflowOutput {
    match update_inner(path, id, status, project_dir) {
        Ok(output) => output,
        Err(e) => WorkflowOutput::block(format!("tasks update failed: {e}")),
    }
}

fn update_inner(
    path: &str,
    id: &str,
    status: &str,
    project_dir: &Path,
) -> Result<WorkflowOutput, anyhow::Error> {
    let tasks_path = std::path::PathBuf::from(path);
    validate_path(&tasks_path, project_dir)?;

    let new_status = status
        .parse::<ecc_domain::task::TaskStatus>()
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let timestamp = crate::time::utc_now_iso8601();

    // Acquire exclusive lock
    let _guard = ecc_flock::acquire(project_dir, "tasks")
        .map_err(|e| anyhow::anyhow!("failed to acquire tasks lock: {e}"))?;

    let content = std::fs::read_to_string(&tasks_path)
        .map_err(|e| anyhow::anyhow!("cannot read {path}: {e}"))?;

    let updated = ecc_domain::task::updater::apply_update(&content, id, new_status, &timestamp)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // Atomic write: write to .tmp then rename
    let tmp_path = tasks_path.with_extension("tmp");
    std::fs::write(&tmp_path, &updated)
        .map_err(|e| anyhow::anyhow!("failed to write temp file: {e}"))?;
    std::fs::rename(&tmp_path, &tasks_path)
        .map_err(|e| anyhow::anyhow!("failed to rename temp file: {e}"))?;

    Ok(WorkflowOutput::pass(format!("updated {id} to {status}")))
}

/// Run the `tasks init <design_path> --output <output> [--force]` subcommand.
///
/// Reads a design file, extracts the PC table, and generates tasks.md.
pub fn run_init(
    design_path: &str,
    output: &str,
    force: bool,
    project_dir: &Path,
) -> WorkflowOutput {
    match init_inner(design_path, output, force, project_dir) {
        Ok(out) => out,
        Err(e) => WorkflowOutput::block(format!("tasks init failed: {e}")),
    }
}

fn init_inner(
    design_path: &str,
    output: &str,
    force: bool,
    project_dir: &Path,
) -> Result<WorkflowOutput, anyhow::Error> {
    let design_file = Path::new(design_path);
    let output_path = std::path::PathBuf::from(output);

    validate_path(design_file, project_dir)?;
    validate_path(&output_path, project_dir)?;

    // Check if output already exists
    if output_path.exists() && !force {
        return Ok(WorkflowOutput::block(format!(
            "output file already exists: {output}. Use --force to overwrite."
        )));
    }

    // Read and parse the design file for PCs
    let design_content = std::fs::read_to_string(design_file)
        .map_err(|e| anyhow::anyhow!("cannot read {design_path}: {e}"))?;

    let pc_report = ecc_domain::spec::pc::parse_pcs(&design_content)
        .map_err(|e| anyhow::anyhow!("failed to parse design PCs: {e}"))?;

    if pc_report.pcs.is_empty() {
        return Ok(WorkflowOutput::block(
            "no PC table found in design file".to_owned(),
        ));
    }

    // Check for duplicate PC IDs
    let mut seen_ids = std::collections::HashSet::new();
    for pc in &pc_report.pcs {
        if !seen_ids.insert(pc.id.number()) {
            return Ok(WorkflowOutput::block(format!("duplicate {}", pc.id)));
        }
    }

    // Extract feature title from design file (first # heading)
    let feature_title = design_content
        .lines()
        .find(|l| l.starts_with("# "))
        .map(|l| l.trim_start_matches("# ").trim())
        .unwrap_or("Untitled");

    let timestamp = crate::time::utc_now_iso8601();
    let tasks_content =
        ecc_domain::task::renderer::render_tasks(&pc_report.pcs, feature_title, &timestamp);

    // Write atomically
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| anyhow::anyhow!("cannot create output directory: {e}"))?;
    }
    let tmp_path = output_path.with_extension("tmp");
    std::fs::write(&tmp_path, &tasks_content)
        .map_err(|e| anyhow::anyhow!("failed to write temp file: {e}"))?;
    std::fs::rename(&tmp_path, &output_path)
        .map_err(|e| anyhow::anyhow!("failed to rename temp file: {e}"))?;

    Ok(WorkflowOutput::pass(format!(
        "generated tasks.md with {} PCs at {output}",
        pc_report.pcs.len()
    )))
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
        let json: serde_json::Value =
            serde_json::from_str(&output.message).expect("message should be valid JSON");

        // AC-003.1: arrays present
        assert!(json["pending"].is_array(), "pending must be an array");
        assert!(json["completed"].is_array(), "completed must be an array");
        assert!(
            json["in_progress"].is_array(),
            "in_progress must be an array"
        );
        assert!(json["failed"].is_array(), "failed must be an array");
        assert!(json["total"].is_number(), "total must be a number");
        assert!(
            json["progress_pct"].is_number(),
            "progress_pct must be a number"
        );

        // AC-003.2: pending contains only undone items
        let pending = json["pending"].as_array().unwrap();
        let completed = json["completed"].as_array().unwrap();
        assert_eq!(
            pending.len(),
            2,
            "pending array should have 2 items (PC-001 + E2E tests)"
        );
        assert_eq!(
            completed.len(),
            1,
            "completed array should have 1 item (PC-002)"
        );
        assert_eq!(json["total"], 3, "total should be 3");

        // Check structure of a pending item
        let first_pending = &pending[0];
        assert!(first_pending["id"].is_string(), "item must have id field");
        assert!(
            first_pending["description"].is_string(),
            "item must have description field"
        );
        assert!(
            first_pending["current_status"].is_string(),
            "item must have current_status field"
        );
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

    // PC-021: tasks update performs atomic write with flock, appends trail
    #[test]
    fn update_atomic() {
        let tmp = TempDir::new().unwrap();
        let tasks_path = tmp.path().join("tasks.md");
        std::fs::write(&tasks_path, valid_tasks_md()).unwrap();

        let output = run_update(tasks_path.to_str().unwrap(), "PC-001", "red", tmp.path());

        assert!(
            matches!(output.status, Status::Pass),
            "expected pass status, got: {:?} — {}",
            output.status,
            output.message
        );

        // Verify the file was actually updated
        let content = std::fs::read_to_string(&tasks_path).unwrap();
        assert!(
            content.contains("red@"),
            "updated file should contain red@ trail segment"
        );
        // Original pending trail should still be there
        assert!(
            content.contains("pending@"),
            "updated file should still contain pending@ trail"
        );
    }

    // PC-022: tasks update rejects path traversal
    #[test]
    fn update_traversal() {
        let tmp = TempDir::new().unwrap();
        let another_tmp = TempDir::new().unwrap();
        let tasks_path = tmp.path().join("tasks.md");
        std::fs::write(&tasks_path, valid_tasks_md()).unwrap();

        let output = run_update(
            tasks_path.to_str().unwrap(),
            "PC-001",
            "red",
            another_tmp.path(),
        );

        assert!(
            matches!(output.status, Status::Block),
            "expected block status for path traversal, got: {:?} — {}",
            output.status,
            output.message
        );
    }

    /// A minimal design.md fixture with a PC table.
    fn valid_design_md() -> String {
        "# Design: Test Feature\n\
         \n\
         ## Pass Conditions\n\
         \n\
         | ID | Type | Description | Verifies AC | Command | Expected |\n\
         |----|------|-------------|-------------|---------|----------|\n\
         | PC-001 | unit | Test parser | AC-001.1 | `cargo test parser` | PASS |\n\
         | PC-002 | unit | Test updater | AC-002.1 | `cargo test updater` | PASS |\n"
            .to_owned()
    }

    // PC-023: tasks init generates tasks.md from design file's PC table
    #[test]
    fn init_generate() {
        let tmp = TempDir::new().unwrap();
        let design_path = tmp.path().join("design.md");
        let output_path = tmp.path().join("tasks.md");
        std::fs::write(&design_path, valid_design_md()).unwrap();

        let output = run_init(
            design_path.to_str().unwrap(),
            output_path.to_str().unwrap(),
            false,
            tmp.path(),
        );

        assert!(
            matches!(output.status, Status::Pass),
            "expected pass, got: {:?} — {}",
            output.status,
            output.message
        );

        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("PC-001"), "should contain PC-001");
        assert!(content.contains("PC-002"), "should contain PC-002");
        assert!(
            content.contains("## Post-TDD"),
            "should contain Post-TDD section"
        );
        assert!(
            content.contains("E2E tests"),
            "should contain E2E tests entry"
        );
        assert!(
            content.contains("pending@"),
            "should contain pending timestamp"
        );
    }

    // PC-024: tasks init blocks when output exists (no --force)
    #[test]
    fn init_exists() {
        let tmp = TempDir::new().unwrap();
        let design_path = tmp.path().join("design.md");
        let output_path = tmp.path().join("tasks.md");
        std::fs::write(&design_path, valid_design_md()).unwrap();
        std::fs::write(&output_path, "existing content").unwrap();

        let output = run_init(
            design_path.to_str().unwrap(),
            output_path.to_str().unwrap(),
            false,
            tmp.path(),
        );

        assert!(
            matches!(output.status, Status::Block),
            "expected block when output exists, got: {:?} — {}",
            output.status,
            output.message
        );
    }

    // PC-025: tasks init overwrites with --force
    #[test]
    fn init_force() {
        let tmp = TempDir::new().unwrap();
        let design_path = tmp.path().join("design.md");
        let output_path = tmp.path().join("tasks.md");
        std::fs::write(&design_path, valid_design_md()).unwrap();
        std::fs::write(&output_path, "old content").unwrap();

        let output = run_init(
            design_path.to_str().unwrap(),
            output_path.to_str().unwrap(),
            true,
            tmp.path(),
        );

        assert!(
            matches!(output.status, Status::Pass),
            "expected pass with --force, got: {:?} — {}",
            output.status,
            output.message
        );

        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("PC-001"), "should contain generated PCs");
    }

    // PC-026: tasks init blocks when design has no PC table
    #[test]
    fn init_no_pcs() {
        let tmp = TempDir::new().unwrap();
        let design_path = tmp.path().join("design.md");
        let output_path = tmp.path().join("tasks.md");
        std::fs::write(&design_path, "# Design\n\nNo PC table here.\n").unwrap();

        let output = run_init(
            design_path.to_str().unwrap(),
            output_path.to_str().unwrap(),
            false,
            tmp.path(),
        );

        assert!(
            matches!(output.status, Status::Block),
            "expected block for no PC table, got: {:?} — {}",
            output.status,
            output.message
        );
    }

    // PC-027: tasks init blocks on duplicate PC IDs
    #[test]
    fn init_dup_pcs() {
        let tmp = TempDir::new().unwrap();
        let design_path = tmp.path().join("design.md");
        let output_path = tmp.path().join("tasks.md");
        let dup_design = "# Design\n\n\
             ## Pass Conditions\n\n\
             | ID | Type | Description | Verifies AC | Command | Expected |\n\
             |----|------|-------------|-------------|---------|----------|\n\
             | PC-001 | unit | First | AC-001.1 | `cargo test` | PASS |\n\
             | PC-001 | unit | Duplicate | AC-001.2 | `cargo test` | PASS |\n";
        std::fs::write(&design_path, dup_design).unwrap();

        let output = run_init(
            design_path.to_str().unwrap(),
            output_path.to_str().unwrap(),
            false,
            tmp.path(),
        );

        assert!(
            matches!(output.status, Status::Block),
            "expected block for duplicate PCs, got: {:?} — {}",
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
