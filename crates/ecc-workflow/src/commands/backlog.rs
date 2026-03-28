use std::path::Path;

use crate::output::WorkflowOutput;
use crate::time::utc_today;
use crate::slug::make_slug;

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
    let backlog_dir = project_dir.join("docs/backlog");

    let result = (|| -> Result<WorkflowOutput, anyhow::Error> {
        // Acquire exclusive lock for the backlog directory
        let _guard = ecc_flock::acquire(project_dir, "backlog")
            .map_err(|e| anyhow::anyhow!("Failed to acquire backlog lock: {e}"))?;

        // Ensure backlog dir exists
        std::fs::create_dir_all(&backlog_dir)
            .map_err(|e| anyhow::anyhow!("Failed to create docs/backlog: {e}"))?;

        // Scan for existing BL-NNN-*.md files and find max ID
        let next_id = compute_next_id(&backlog_dir)?;

        // Build slug and filename
        let slug = make_slug(title);
        let filename = format!("BL-{next_id:03}-{slug}.md");
        let entry_path = backlog_dir.join(&filename);

        // Build frontmatter content
        let today = utc_today();
        let tags_list = if tags.is_empty() {
            "[]".to_string()
        } else {
            let tag_items: Vec<String> = tags
                .split(',')
                .map(|t| format!("\"{}\"", t.trim()))
                .collect();
            format!("[{}]", tag_items.join(", "))
        };

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

        std::fs::write(&entry_path, &content)
            .map_err(|e| anyhow::anyhow!("Failed to write entry file {filename}: {e}"))?;

        // Update BACKLOG.md — append a new row
        update_backlog_index(&backlog_dir, next_id, title, scope, target, &today)?;

        Ok(WorkflowOutput::pass(format!(
            "Created BL-{next_id:03}: {title}"
        )))
    })();

    match result {
        Ok(output) => output,
        Err(e) => WorkflowOutput::block(format!("backlog add-entry failed: {e}")),
    }
}

/// Scan `backlog_dir` for `BL-NNN-*.md` files and return max ID + 1 (or 1 if none).
fn compute_next_id(backlog_dir: &Path) -> Result<u32, anyhow::Error> {
    let read_dir = std::fs::read_dir(backlog_dir)
        .map_err(|e| anyhow::anyhow!("Failed to read {}: {e}", backlog_dir.display()))?;

    let max_id = read_dir
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let name = entry.file_name();
            let name_str = name.to_string_lossy().into_owned();
            if name_str.starts_with("BL-") && name_str.ends_with(".md") {
                let after_bl = name_str.strip_prefix("BL-")?;
                let id_str: String = after_bl.chars().take_while(|c| c.is_ascii_digit()).collect();
                id_str.parse::<u32>().ok()
            } else {
                None
            }
        })
        .max();

    Ok(max_id.map_or(1, |m| m + 1))
}

/// Append a new row to `docs/backlog/BACKLOG.md`.
///
/// Reads the file, appends the row at the end of the table (or after the last `|` line),
/// and writes the file back atomically.
fn update_backlog_index(
    backlog_dir: &Path,
    id: u32,
    title: &str,
    scope: &str,
    target: &str,
    today: &str,
) -> Result<(), anyhow::Error> {
    let backlog_md = backlog_dir.join("BACKLOG.md");

    let existing = if backlog_md.exists() {
        std::fs::read_to_string(&backlog_md)
            .map_err(|e| anyhow::anyhow!("Failed to read BACKLOG.md: {e}"))?
    } else {
        "# Backlog\n\n| ID | Title | Body | Scope | Target | Status | Created |\n|---|---|---|---|---|---|---|\n".to_string()
    };

    let new_row = format!(
        "| BL-{id:03} | {title} | — | {scope} | {target} | open | {today} |\n"
    );

    // Append the new row after the last table line (or at end of file)
    let output_str = if existing.trim_end().ends_with('|') {
        format!("{}\n{}", existing.trim_end(), new_row)
    } else {
        format!("{}{}", existing, new_row)
    };

    // Atomic write via temp file + rename
    let tmp_path = backlog_dir.join(".BACKLOG.md.tmp");
    std::fs::write(&tmp_path, &output_str)
        .map_err(|e| anyhow::anyhow!("Failed to write temp BACKLOG.md: {e}"))?;
    std::fs::rename(&tmp_path, &backlog_md)
        .map_err(|e| anyhow::anyhow!("Failed to rename BACKLOG.md: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_backlog(tmp: &TempDir) {
        let backlog_dir = tmp.path().join("docs/backlog");
        std::fs::create_dir_all(&backlog_dir).unwrap();
        std::fs::write(
            backlog_dir.join("BACKLOG.md"),
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
        let files: Vec<_> = std::fs::read_dir(&backlog_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let n = e.file_name();
                let s = n.to_string_lossy();
                s.starts_with("BL-") && s.ends_with(".md")
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
        let mut ids: Vec<u32> = std::fs::read_dir(&backlog_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let name = e.file_name();
                let s = name.to_string_lossy().into_owned();
                if s.starts_with("BL-") && s.ends_with(".md") {
                    let after = s.strip_prefix("BL-")?;
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

        let content =
            std::fs::read_to_string(tmp.path().join("docs/backlog/BACKLOG.md")).unwrap();
        assert!(content.contains("Alpha"), "missing Alpha");
        assert!(content.contains("Beta"), "missing Beta");
    }
}
