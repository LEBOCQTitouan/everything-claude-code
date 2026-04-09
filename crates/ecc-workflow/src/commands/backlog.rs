use std::path::Path;

use crate::output::WorkflowOutput;
use crate::slug::make_slug;
use crate::time::utc_today;
use ecc_infra::fs_backlog::FsBacklogRepository;
use ecc_infra::os_fs::OsFileSystem;
use ecc_ports::backlog::{BacklogEntryStore, BacklogIndexStore};
use ecc_ports::fs::FileSystem;

/// Run the `backlog add-entry` subcommand.
///
/// Atomically:
/// 1. Acquires `ecc_flock::acquire(project_dir, "backlog")`
/// 2. Scans `docs/backlog/` for `BL-NNN-*.md` files and computes the next ID
/// 3. Creates the new entry file with frontmatter
/// 4. Appends a new row to `docs/backlog/BACKLOG.md`
/// 5. Releases the lock (guard drops)
/// 6. Outputs JSON: `{"status":"pass","message":"Created BL-NNN: <title>"}`
pub fn run(
    title: &str,
    scope: &str,
    target: &str,
    tags: &str,
    project_dir: &Path,
) -> WorkflowOutput {
    match add_entry(title, scope, target, tags, project_dir) {
        Ok(output) => output,
        Err(e) => WorkflowOutput::block(format!("backlog add-entry failed: {e}")),
    }
}

/// Inner implementation — returns `Err` on any I/O or lock failure.
fn add_entry(
    title: &str,
    scope: &str,
    target: &str,
    tags: &str,
    project_dir: &Path,
) -> Result<WorkflowOutput, anyhow::Error> {
    let backlog_dir = project_dir.join("docs/backlog");
    let fs = OsFileSystem;

    // Acquire exclusive lock for the backlog directory
    let _guard = ecc_flock::acquire(project_dir, "backlog")
        .map_err(|e| anyhow::anyhow!("Failed to acquire backlog lock: {e}"))?;

    // Ensure backlog dir exists
    fs.create_dir_all(&backlog_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create docs/backlog: {e}"))?;

    // Compute next ID via the BacklogEntryStore port
    let repo = FsBacklogRepository::new(&fs);
    let next_id_str = repo
        .next_id(&backlog_dir)
        .map_err(|e| anyhow::anyhow!("Failed to compute next ID: {e}"))?;

    // Parse numeric part from "BL-NNN"
    let next_id: u32 = next_id_str
        .strip_prefix("BL-")
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| anyhow::anyhow!("Invalid next_id format: {next_id_str}"))?;

    // Build slug and filename
    let slug = make_slug(title);
    let filename = format!("BL-{next_id:03}-{slug}.md");
    let entry_path = backlog_dir.join(&filename);

    // Build frontmatter content
    let today = utc_today();
    let tags_list = format_tags(tags);

    let content = format!(
        "---\n\
         id: BL-{next_id:03}\n\
         title: {title}\n\
         status: open\n\
         created: {today}\n\
         tags: {tags_list}\n\
         scope: {scope}\n\
         target_command: {target}\n\
         ---\n\
         \n\
         ## Optimized Prompt\n\
         \n\
         (To be filled by /backlog add workflow)\n\
         \n\
         ## Original Input\n\
         \n\
         Created via ecc-workflow backlog add-entry\n"
    );

    fs.write(&entry_path, &content)
        .map_err(|e| anyhow::anyhow!("Failed to write entry file {filename}: {e}"))?;

    // Update BACKLOG.md — append a new row via BacklogIndexStore port
    append_backlog_index(&repo, &backlog_dir, next_id, title, scope, target, &today)?;

    Ok(WorkflowOutput::pass(format!(
        "Created BL-{next_id:03}: {title}"
    )))
}

/// Format a comma-separated tags string as a YAML array literal.
fn format_tags(tags: &str) -> String {
    if tags.is_empty() {
        "[]".to_string()
    } else {
        let items: Vec<String> = tags
            .split(',')
            .map(|t| format!("\"{}\"", t.trim()))
            .collect();
        format!("[{}]", items.join(", "))
    }
}

/// Append a new row to `docs/backlog/BACKLOG.md` via the BacklogIndexStore port.
fn append_backlog_index(
    repo: &FsBacklogRepository<'_>,
    backlog_dir: &Path,
    id: u32,
    title: &str,
    scope: &str,
    target: &str,
    today: &str,
) -> Result<(), anyhow::Error> {
    let existing = repo
        .read_index(backlog_dir)
        .map_err(|e| anyhow::anyhow!("Failed to read BACKLOG.md: {e}"))?
        .unwrap_or_else(|| {
            "# Backlog\n\n| ID | Title | Body | Scope | Target | Status | Created |\n|---|---|---|---|---|---|---|\n"
                .to_string()
        });

    let new_row = format!("| BL-{id:03} | {title} | — | {scope} | {target} | open | {today} |\n");

    // Append the new row after the last table line (or at end of file)
    let output_str = if existing.trim_end().ends_with('|') {
        format!("{}\n{}", existing.trim_end(), new_row)
    } else {
        format!("{}{}", existing, new_row)
    };

    repo.write_index(backlog_dir, &output_str)
        .map_err(|e| anyhow::anyhow!("Failed to write BACKLOG.md: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_backlog(tmp: &TempDir) {
        let backlog_dir = tmp.path().join("docs/backlog");
        let fs = OsFileSystem;
        fs.create_dir_all(&backlog_dir).unwrap();
        fs.write(
            &backlog_dir.join("BACKLOG.md"),
            "# Backlog\n\n| ID | Title | Body | Scope | Target | Status | Created |\n|---|---|---|---|---|---|---|\n",
        )
        .unwrap();
    }

    #[test]
    fn single_add_entry_creates_file() {
        let tmp = TempDir::new().unwrap();
        setup_backlog(&tmp);

        let result = run("My Feature", "MEDIUM", "/spec-dev", "", tmp.path());
        assert!(
            matches!(result.status, crate::output::Status::Pass),
            "expected pass: {}",
            result.message
        );

        let backlog_dir = tmp.path().join("docs/backlog");
        let fs = OsFileSystem;
        let files: Vec<_> = fs
            .read_dir(&backlog_dir)
            .unwrap()
            .into_iter()
            .filter(|p| {
                let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                name.starts_with("BL-") && name.ends_with(".md")
            })
            .collect();
        assert_eq!(files.len(), 1, "expected 1 entry file");
    }

    #[test]
    fn sequential_adds_get_incrementing_ids() {
        let tmp = TempDir::new().unwrap();
        setup_backlog(&tmp);

        run("First", "LOW", "/spec-dev", "", tmp.path());
        run("Second", "HIGH", "/spec-dev", "", tmp.path());

        let backlog_dir = tmp.path().join("docs/backlog");
        let fs = OsFileSystem;
        let mut ids: Vec<u32> = fs
            .read_dir(&backlog_dir)
            .unwrap()
            .into_iter()
            .filter_map(|p| {
                let name = p.file_name()?.to_string_lossy().into_owned();
                if name.starts_with("BL-") && name.ends_with(".md") {
                    let after = name.strip_prefix("BL-")?;
                    let id_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
                    id_str.parse::<u32>().ok()
                } else {
                    None
                }
            })
            .collect();
        ids.sort_unstable();
        assert_eq!(ids, vec![1, 2]);
    }

    #[test]
    fn backlog_md_updated_with_both_entries() {
        let tmp = TempDir::new().unwrap();
        setup_backlog(&tmp);

        run("Alpha", "LOW", "/spec-dev", "", tmp.path());
        run("Beta", "MEDIUM", "/spec-dev", "", tmp.path());

        let fs = OsFileSystem;
        let content = fs
            .read_to_string(&tmp.path().join("docs/backlog/BACKLOG.md"))
            .unwrap();
        assert!(content.contains("Alpha"), "missing Alpha");
        assert!(content.contains("Beta"), "missing Beta");
    }
}
