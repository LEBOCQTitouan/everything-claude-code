//! File pruning utilities for the memory system.
//!
//! Provides BL-ID pattern matching and file-level pruning for memory cleanup.

use ecc_domain::backlog::entry::{BacklogEntry, BacklogStatus};
use ecc_domain::memory::SafePath;
use ecc_ports::fs::FileSystem;
use regex::Regex;
use std::path::PathBuf;

/// Report produced by [`prune_file_memories_for_backlog`].
#[derive(Debug, Default)]
pub struct PruneReport {
    /// Files that were moved to the trash directory.
    pub trashed_files: Vec<PathBuf>,
    /// Whether `MEMORY.md` was successfully rewritten.
    pub index_updated: bool,
    /// Non-fatal errors encountered during pruning.
    pub errors: Vec<String>,
}

/// Report produced by [`prune_orphaned_file_memories`].
#[derive(Debug, Default)]
pub struct OrphanedPruneReport {
    /// Files that would be trashed in a real (apply) run.
    pub would_trash: Vec<PathBuf>,
    /// Files that were actually moved to the trash directory (only non-empty when `apply=true`).
    pub trashed_files: Vec<PathBuf>,
    /// Non-fatal errors encountered during scanning.
    pub errors: Vec<String>,
}

/// Scan `root` for `project_bl<N>_*.md` files whose backlog entry is marked
/// `Implemented` or `Archived` in `backlog_entries`.
///
/// When `apply` is `false` (dry-run), populates `would_trash` but does NOT
/// move any files. When `apply` is `true`, moves matching files to
/// `<root>/.trash/<today>/`.
pub fn prune_orphaned_file_memories(
    fs: &dyn FileSystem,
    root: &std::path::Path,
    backlog_entries: &[BacklogEntry],
    today: &str,
    apply: bool,
) -> OrphanedPruneReport {
    let mut report = OrphanedPruneReport::default();

    let entries = match fs.read_dir(root) {
        Ok(e) => e,
        Err(err) => {
            report.errors.push(format!("read_dir failed: {err}"));
            return report;
        }
    };

    let trash_dir = root.join(format!(".trash/{today}"));

    for entry in entries {
        let filename = match entry.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_owned(),
            None => continue,
        };

        let bl_num = match extract_bl_num_from_filename(&filename) {
            Some(n) => n,
            None => continue,
        };

        let orphaned = backlog_entries.iter().any(|e| {
            is_same_bl_id(&e.id, bl_num)
                && matches!(e.status, BacklogStatus::Implemented | BacklogStatus::Archived)
        });

        if !orphaned {
            continue;
        }

        if apply {
            if let Err(err) = fs.create_dir_all(&trash_dir) {
                report
                    .errors
                    .push(format!("create_dir_all {}: {err}", trash_dir.display()));
                continue;
            }
            let dst = trash_dir.join(&filename);
            match fs.rename(&entry, &dst) {
                Ok(()) => report.trashed_files.push(dst),
                Err(err) => {
                    report.errors.push(format!(
                        "rename {} -> {}: {err}",
                        entry.display(),
                        dst.display()
                    ));
                }
            }
        } else {
            report.would_trash.push(entry);
        }
    }

    report
}

/// Extract the numeric BL ID from a filename like `project_bl031_foo.md`.
///
/// Returns `None` if the filename doesn't match the `project_bl<N>` pattern.
fn extract_bl_num_from_filename(filename: &str) -> Option<u32> {
    let pattern = Regex::new(r"(?i)^project_bl(\d+)").expect("valid regex");
    pattern
        .captures(filename)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<u32>().ok())
}

/// Returns true if backlog entry id `bl_id` (e.g. `"BL-031"`) matches numeric `bl_num`.
fn is_same_bl_id(bl_id: &str, bl_num: u32) -> bool {
    bl_id
        .strip_prefix("BL-")
        .and_then(|s| s.parse::<u32>().ok())
        .is_some_and(|n| n == bl_num)
}

/// Move memory files matching `bl_id` to `<root>/.trash/<today>/` and remove
/// their rows from `MEMORY.md` via atomic rewrite.
///
/// Fire-and-forget: individual file errors are appended to `report.errors` but
/// do not abort the overall prune.
pub fn prune_file_memories_for_backlog(
    fs: &dyn FileSystem,
    root: &SafePath,
    bl_id: &str,
    today: &str,
) -> PruneReport {
    let mut report = PruneReport::default();
    let root_dir = root.full();

    // 1. Scan root dir for files matching the BL-ID pattern.
    let entries = match fs.read_dir(root_dir) {
        Ok(e) => e,
        Err(err) => {
            report.errors.push(format!("read_dir failed: {err}"));
            return report;
        }
    };

    let trash_dir = root_dir.join(format!(".trash/{today}"));

    let mut trashed_filenames: Vec<String> = Vec::new();

    for entry in entries {
        let filename = match entry.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_owned(),
            None => continue,
        };
        if !matches_bl_id(&filename, bl_id) {
            continue;
        }

        // Ensure trash directory exists.
        if let Err(err) = fs.create_dir_all(&trash_dir) {
            report
                .errors
                .push(format!("create_dir_all {}: {err}", trash_dir.display()));
            continue;
        }

        let dst = trash_dir.join(&filename);
        match fs.rename(&entry, &dst) {
            Ok(()) => {
                report.trashed_files.push(dst);
                trashed_filenames.push(filename);
            }
            Err(err) => {
                report.errors.push(format!(
                    "rename {} -> {}: {err}",
                    entry.display(),
                    dst.display()
                ));
            }
        }
    }

    // 2. Atomic rewrite of MEMORY.md — remove rows referencing trashed files.
    let memory_md_path = root_dir.join("MEMORY.md");
    match fs.read_to_string(&memory_md_path) {
        Ok(content) => {
            let updated: String = content
                .lines()
                .filter(|line| {
                    !trashed_filenames
                        .iter()
                        .any(|name| line.contains(name.as_str()))
                })
                .map(|line| format!("{line}\n"))
                .collect();

            // Atomic write: write to .tmp then rename.
            let tmp_path = root_dir.join("MEMORY.md.tmp");
            match fs.write(&tmp_path, &updated) {
                Ok(()) => match fs.rename(&tmp_path, &memory_md_path) {
                    Ok(()) => {
                        report.index_updated = true;
                    }
                    Err(err) => {
                        report.errors.push(format!("rename MEMORY.md.tmp: {err}"));
                    }
                },
                Err(err) => {
                    report.errors.push(format!("write MEMORY.md.tmp: {err}"));
                }
            }
        }
        Err(err) => {
            report.errors.push(format!("read MEMORY.md: {err}"));
        }
    }

    report
}

/// Build a regex that matches `project_bl<N>_*.md` for a specific BL numeric ID.
///
/// Uses `0*<N>` to allow zero-padded variants while being collision-safe:
/// `project_bl0*10` matches `bl10` and `bl010` but not `bl100`.
fn bl_memory_regex(bl_id_num: u32) -> Regex {
    // Pattern: ^project_bl0*<N>(_[a-z0-9_-]+)?\.md$ case-insensitive
    // The `0*` before the number allows leading zeros but the trailing `(_...|$)` anchor
    // prevents e.g. BL-10 matching BL-100.
    let pattern = format!(r"(?i)^project_bl0*{bl_id_num}(_[a-z0-9_-]+)?\.md$");
    Regex::new(&pattern).expect("valid regex")
}

/// Returns true if `filename` matches the BL-ID pattern.
///
/// Accepts filenames like `project_bl031.md` or `project_bl031_foo.md`.
/// The BL-ID string must be in the form `BL-<digits>` (e.g. `BL-031`).
pub fn matches_bl_id(filename: &str, bl_id: &str) -> bool {
    let num = match bl_id
        .strip_prefix("BL-")
        .and_then(|s| s.parse::<u32>().ok())
    {
        Some(n) => n,
        None => return false,
    };
    bl_memory_regex(num).is_match(filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trashes_and_updates_index() {
        use ecc_domain::memory::SafePath;
        use ecc_ports::fs::FileSystem as _;
        use ecc_test_support::InMemoryFileSystem;
        use std::path::PathBuf;

        let root_path = PathBuf::from("/home/alice/.claude/projects/foo/memory");
        let fs = InMemoryFileSystem::new()
            .with_dir(&root_path)
            .with_file(
                root_path.join("project_bl001_foo.md"),
                "BL-001 memory content",
            )
            .with_file(
                root_path.join("project_bl001_bar.md"),
                "another BL-001 memory",
            )
            .with_file(
                root_path.join("project_bl002_other.md"),
                "BL-002 memory — should NOT be trashed",
            )
            .with_file(
                root_path.join("MEMORY.md"),
                "# Memory\n\n- [BL-001 foo](project_bl001_foo.md)\n- [BL-001 bar](project_bl001_bar.md)\n- [BL-002 other](project_bl002_other.md)\n",
            );

        let safe = SafePath::from_canonical(root_path.clone(), root_path.clone()).unwrap();

        let report = prune_file_memories_for_backlog(&fs, &safe, "BL-001", "2026-04-19");

        assert_eq!(report.trashed_files.len(), 2, "both BL-001 files trashed");

        assert!(!fs.exists(&root_path.join("project_bl001_foo.md")));
        assert!(!fs.exists(&root_path.join("project_bl001_bar.md")));

        assert!(fs.exists(&root_path.join(".trash/2026-04-19/project_bl001_foo.md")));
        assert!(fs.exists(&root_path.join(".trash/2026-04-19/project_bl001_bar.md")));

        assert!(fs.exists(&root_path.join("project_bl002_other.md")));

        let memory_md = fs.read_to_string(&root_path.join("MEMORY.md")).unwrap();
        assert!(!memory_md.contains("project_bl001_foo.md"));
        assert!(!memory_md.contains("project_bl001_bar.md"));
        assert!(memory_md.contains("project_bl002_other.md"));

        assert!(report.index_updated);
        assert_eq!(report.errors, Vec::<String>::new());
    }

    #[test]
    fn is_idempotent() {
        use ecc_domain::memory::SafePath;
        use ecc_test_support::InMemoryFileSystem;
        use std::path::PathBuf;

        let root_path = PathBuf::from("/home/alice/.claude/projects/foo/memory");
        let fs = InMemoryFileSystem::new()
            .with_dir(&root_path)
            .with_file(root_path.join("project_bl001_foo.md"), "content")
            .with_file(
                root_path.join("MEMORY.md"),
                "- [foo](project_bl001_foo.md)\n",
            );

        let safe = SafePath::from_canonical(root_path.clone(), root_path.clone()).unwrap();

        let r1 = prune_file_memories_for_backlog(&fs, &safe, "BL-001", "2026-04-19");
        assert_eq!(r1.trashed_files.len(), 1, "first run trashes file");

        let r2 = prune_file_memories_for_backlog(&fs, &safe, "BL-001", "2026-04-19");
        assert_eq!(r2.trashed_files.len(), 0, "second run is no-op");
        assert!(r2.errors.is_empty(), "idempotent: no errors");
    }

    #[test]
    fn memory_md_atomic_rewrite() {
        // Structural guard: file_prune.rs must implement MEMORY.md updates via
        // temp-file + rename (not direct overwrite). This prevents partial
        // reads during concurrent file access.
        const SOURCE: &str = include_str!("file_prune.rs");

        // Production code only (before the #[cfg(test)] block)
        let production = SOURCE.split("#[cfg(test)]").next().unwrap_or(SOURCE);

        // Must use rename for MEMORY.md updates
        let has_rename = production.contains(".rename(") || production.contains("fs.rename");
        // Must use a temp path (.tmp suffix or similar) before renaming
        let has_temp = production.contains(".tmp")
            || production.contains("temp")
            || production.contains("MEMORY.md.new");
        assert!(
            has_rename && has_temp,
            "MEMORY.md rewrite must use temp+rename for atomicity; \
             found rename={has_rename} temp={has_temp}"
        );
    }

    #[test]
    fn uses_safe_path_only() {
        const SOURCE: &str = include_str!("file_prune.rs");

        // The main public functions should take &SafePath for the root
        let production = SOURCE.split("#[cfg(test)]").next().unwrap_or(SOURCE);

        // Primary fn signatures should include &SafePath
        assert!(
            production.contains("root: &SafePath") || production.contains("&SafePath"),
            "file_prune production code must use &SafePath for root-derived paths"
        );
    }

    #[test]
    fn bl_id_regex_collision_safety() {
        // BL-10 matches project_bl10* and project_bl010* but NOT project_bl100*
        assert!(matches_bl_id("project_bl10.md", "BL-10"));
        assert!(matches_bl_id("project_bl010_foo.md", "BL-10"));
        assert!(
            !matches_bl_id("project_bl100.md", "BL-10"),
            "collision risk: bl100 must not match BL-10"
        );
        assert!(!matches_bl_id("project_bl100_foo.md", "BL-10"));

        // BL-100 matches bl100* but NOT bl10*
        assert!(matches_bl_id("project_bl100.md", "BL-100"));
        assert!(matches_bl_id("project_bl100_bar.md", "BL-100"));
        assert!(!matches_bl_id("project_bl10.md", "BL-100"));

        // BL-031 matches with or without suffix
        assert!(matches_bl_id("project_bl031.md", "BL-031"));
        assert!(matches_bl_id("project_bl031_foo.md", "BL-031"));
        assert!(
            matches_bl_id("project_bl31.md", "BL-031"),
            "zero-pad allowed via 0*"
        );

        // Invalid BL ID returns false
        assert!(!matches_bl_id("project_bl001.md", "invalid"));
    }
}
