//! Backlog management use cases — next_id, check_duplicates, reindex.
//!
//! Orchestrates domain logic through the FileSystem port.

use ecc_domain::backlog::entry::{extract_id_from_filename, parse_frontmatter, BacklogEntry};
use ecc_domain::backlog::index::{extract_dependency_graph, generate_index_table, generate_stats};
use ecc_domain::backlog::similarity::{composite_score, DuplicateCandidate};
use ecc_ports::fs::FileSystem;
use std::path::Path;

/// Compute the next sequential backlog ID from existing BL-*.md files.
///
/// Returns `"BL-NNN"` where NNN is max existing ID + 1, zero-padded to 3 digits.
pub fn next_id(fs: &dyn FileSystem, backlog_dir: &Path) -> Result<String, String> {
    if !fs.is_dir(backlog_dir) {
        return Err(format!(
            "backlog directory not found: {}",
            backlog_dir.display()
        ));
    }
    let entries = fs
        .read_dir(backlog_dir)
        .map_err(|e| format!("cannot read backlog directory: {e}"))?;

    let max_id = entries
        .iter()
        .filter_map(|p| p.file_name())
        .filter_map(|name| extract_id_from_filename(&name.to_string_lossy()))
        .max()
        .unwrap_or(0);

    Ok(format!("BL-{:03}", max_id + 1))
}

/// Check for duplicate backlog entries by title similarity.
///
/// Filters to `open` and `in-progress` entries only.
/// Returns candidates sorted by score descending, filtered to score >= 0.6.
pub fn check_duplicates(
    fs: &dyn FileSystem,
    backlog_dir: &Path,
    query: &str,
    query_tags: &[String],
) -> Result<Vec<DuplicateCandidate>, String> {
    if query.is_empty() {
        return Err("query must not be empty".to_string());
    }

    let entries = fs
        .read_dir(backlog_dir)
        .map_err(|e| format!("cannot read backlog directory: {e}"))?;

    let mut candidates = Vec::new();

    for path in &entries {
        let filename = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue,
        };
        if extract_id_from_filename(&filename).is_none() {
            continue;
        }

        let content = match fs.read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                log::warn!("skipping {filename}: {e}");
                continue;
            }
        };

        let entry = match parse_frontmatter(&content) {
            Ok(e) => e,
            Err(e) => {
                log::warn!("skipping {filename}: {e}");
                continue;
            }
        };

        // Filter to open/in-progress only
        let status = entry.status.to_lowercase();
        if status != "open" && status != "in-progress" {
            continue;
        }

        let score = composite_score(query, &entry.title, query_tags, &entry.tags);
        if score >= 0.6 {
            candidates.push(DuplicateCandidate {
                id: entry.id,
                title: entry.title,
                score: (score * 100.0).round() / 100.0, // round to 2 decimal places
            });
        }
    }

    candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    Ok(candidates)
}

/// Regenerate BACKLOG.md from all BL-*.md files.
///
/// If `dry_run` is true, returns the generated content without writing.
/// Uses atomic write (tempfile + rename) for safety.
pub fn reindex(
    fs: &dyn FileSystem,
    backlog_dir: &Path,
    dry_run: bool,
) -> Result<Option<String>, String> {
    let entries_paths = fs
        .read_dir(backlog_dir)
        .map_err(|e| format!("cannot read backlog directory: {e}"))?;

    let mut entries: Vec<BacklogEntry> = Vec::new();

    for path in &entries_paths {
        let filename = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue,
        };
        if extract_id_from_filename(&filename).is_none() {
            continue;
        }

        let content = match fs.read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                log::warn!("skipping {filename}: {e}");
                continue;
            }
        };

        match parse_frontmatter(&content) {
            Ok(entry) => entries.push(entry),
            Err(e) => {
                log::warn!("skipping {filename}: {e}");
            }
        }
    }

    let table = generate_index_table(&entries);
    let stats = generate_stats(&entries);

    // Preserve dependency graph from existing BACKLOG.md
    let backlog_path = backlog_dir.join("BACKLOG.md");
    let dep_graph = if fs.exists(&backlog_path) {
        fs.read_to_string(&backlog_path)
            .ok()
            .and_then(|content| extract_dependency_graph(&content))
    } else {
        None
    };

    let mut output = String::new();
    output.push_str("# Backlog Index\n\n");
    output.push_str(&table);
    output.push_str("\n\n");
    if let Some(graph) = &dep_graph {
        output.push_str(graph);
        output.push_str("\n\n");
    }
    output.push_str(&stats);
    output.push('\n');

    if dry_run {
        return Ok(Some(output));
    }

    // Atomic write: temp file + rename
    let tmp_path = backlog_dir.join("BACKLOG.md.tmp");
    fs.write(&tmp_path, &output)
        .map_err(|e| format!("failed to write temp file: {e}"))?;
    fs.rename(&tmp_path, &backlog_path)
        .map_err(|e| format!("failed to rename temp file: {e}"))?;

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::InMemoryFileSystem;
    use std::path::Path;

    fn bl_file(id: u32, title: &str, status: &str) -> String {
        format!(
            "---\nid: BL-{id:03}\ntitle: {title}\nstatus: {status}\ncreated: 2026-03-20\nscope: MEDIUM\ntarget_command: /spec dev\ntags: []\n---\n\n# {title}\n"
        )
    }

    fn bl_file_with_tags(id: u32, title: &str, status: &str, tags: &[&str]) -> String {
        let tags_str = tags
            .iter()
            .map(|t| format!("\"{t}\""))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "---\nid: BL-{id:03}\ntitle: {title}\nstatus: {status}\ncreated: 2026-03-20\ntags: [{tags_str}]\n---\n"
        )
    }

    // --- next_id tests ---

    #[test]
    fn next_id_sequential() {
        let fs = InMemoryFileSystem::new()
            .with_file("/backlog/BL-001-first.md", &bl_file(1, "First", "open"))
            .with_file(
                "/backlog/BL-075-last.md",
                &bl_file(75, "Last", "implemented"),
            );
        let result = next_id(&fs, Path::new("/backlog")).unwrap();
        assert_eq!(result, "BL-076");
    }

    #[test]
    fn next_id_empty_dir() {
        let fs = InMemoryFileSystem::new().with_dir("/backlog");
        let result = next_id(&fs, Path::new("/backlog")).unwrap();
        assert_eq!(result, "BL-001");
    }

    #[test]
    fn next_id_with_gaps() {
        let fs = InMemoryFileSystem::new()
            .with_file("/backlog/BL-001-a.md", &bl_file(1, "A", "open"))
            .with_file("/backlog/BL-003-c.md", &bl_file(3, "C", "open"))
            .with_file("/backlog/BL-010-j.md", &bl_file(10, "J", "open"));
        let result = next_id(&fs, Path::new("/backlog")).unwrap();
        assert_eq!(result, "BL-011");
    }

    #[test]
    fn next_id_ignores_non_bl() {
        let fs = InMemoryFileSystem::new()
            .with_file("/backlog/BL-005-item.md", &bl_file(5, "Item", "open"))
            .with_file("/backlog/README.md", "# Readme")
            .with_file("/backlog/BACKLOG.md", "# Index");
        let result = next_id(&fs, Path::new("/backlog")).unwrap();
        assert_eq!(result, "BL-006");
    }

    #[test]
    fn next_id_missing_dir() {
        let fs = InMemoryFileSystem::new();
        let result = next_id(&fs, Path::new("/nonexistent"));
        assert!(result.is_err());
    }

    // --- check_duplicates tests ---

    #[test]
    fn check_duplicates_finds_similar() {
        let fs = InMemoryFileSystem::new().with_file(
            "/backlog/BL-052-replace-hooks-with-rust.md",
            &bl_file_with_tags(
                52,
                "Replace hooks with Rust binaries",
                "open",
                &["rust", "hooks"],
            ),
        );
        let result = check_duplicates(
            &fs,
            Path::new("/backlog"),
            "Replace hooks with compiled Rust",
            &["rust".into(), "hooks".into()],
        )
        .unwrap();
        assert!(!result.is_empty(), "expected at least one candidate");
        assert!(result[0].score >= 0.6, "score {} < 0.6", result[0].score);
    }

    #[test]
    fn check_duplicates_tag_boost() {
        let fs = InMemoryFileSystem::new().with_file(
            "/backlog/BL-001-test.md",
            &bl_file_with_tags(1, "Some feature title", "open", &["rust", "hooks"]),
        );
        let without_tags = check_duplicates(
            &fs,
            Path::new("/backlog"),
            "Some feature title",
            &[],
        )
        .unwrap();
        let with_tags = check_duplicates(
            &fs,
            Path::new("/backlog"),
            "Some feature title",
            &["rust".into(), "hooks".into()],
        )
        .unwrap();
        // Both should find it, but with_tags should have higher score
        assert!(!without_tags.is_empty());
        assert!(!with_tags.is_empty());
        assert!(with_tags[0].score > without_tags[0].score);
    }

    #[test]
    fn check_duplicates_no_matches() {
        let fs = InMemoryFileSystem::new().with_file(
            "/backlog/BL-001-unrelated.md",
            &bl_file(1, "Completely unrelated feature about databases", "open"),
        );
        let result = check_duplicates(
            &fs,
            Path::new("/backlog"),
            "Quantum computing integration",
            &[],
        )
        .unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn check_duplicates_status_filter() {
        let fs = InMemoryFileSystem::new()
            .with_file(
                "/backlog/BL-001-open.md",
                &bl_file(1, "Same title here", "open"),
            )
            .with_file(
                "/backlog/BL-002-implemented.md",
                &bl_file(2, "Same title here", "implemented"),
            );
        let result = check_duplicates(
            &fs,
            Path::new("/backlog"),
            "Same title here",
            &[],
        )
        .unwrap();
        // Only the open entry should be returned
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "BL-001");
    }

    #[test]
    fn check_duplicates_sorted_desc() {
        let fs = InMemoryFileSystem::new()
            .with_file(
                "/backlog/BL-001-low.md",
                &bl_file(1, "Some partially matching title", "open"),
            )
            .with_file(
                "/backlog/BL-002-high.md",
                &bl_file(2, "Exact match query title", "open"),
            );
        let result = check_duplicates(
            &fs,
            Path::new("/backlog"),
            "Exact match query title",
            &[],
        )
        .unwrap();
        if result.len() >= 2 {
            assert!(result[0].score >= result[1].score);
        }
    }

    #[test]
    fn check_duplicates_skips_malformed() {
        let fs = InMemoryFileSystem::new()
            .with_file("/backlog/BL-001-valid.md", &bl_file(1, "Valid entry", "open"))
            .with_file(
                "/backlog/BL-002-malformed.md",
                "not valid yaml frontmatter",
            );
        let result = check_duplicates(
            &fs,
            Path::new("/backlog"),
            "Valid entry",
            &[],
        )
        .unwrap();
        // Should not crash; valid entry should still be found
        assert!(!result.is_empty());
    }

    #[test]
    fn check_duplicates_empty_query() {
        let fs = InMemoryFileSystem::new().with_dir("/backlog");
        let result = check_duplicates(&fs, Path::new("/backlog"), "", &[]);
        assert!(result.is_err());
    }

    // --- reindex tests ---

    #[test]
    fn reindex_full() {
        let dep_graph = "## Dependency Graph\n\n```\nBL-001 → BL-002\n```";
        let existing_backlog = format!("# Backlog\n\n| old table |\n\n{dep_graph}\n\n## Stats\n\n- old stats\n");

        let fs = InMemoryFileSystem::new()
            .with_file("/backlog/BACKLOG.md", &existing_backlog)
            .with_file("/backlog/BL-001-first.md", &bl_file(1, "First", "open"))
            .with_file(
                "/backlog/BL-002-second.md",
                &bl_file(2, "Second", "implemented"),
            )
            .with_file("/backlog/BL-003-third.md", &bl_file(3, "Third", "open"));

        let result = reindex(&fs, Path::new("/backlog"), false).unwrap();
        assert!(result.is_none(), "non-dry-run should return None");

        let content = fs
            .read_to_string(Path::new("/backlog/BACKLOG.md"))
            .unwrap();
        assert!(content.contains("BL-001"));
        assert!(content.contains("BL-002"));
        assert!(content.contains("BL-003"));
        assert!(content.contains("## Dependency Graph"));
        assert!(content.contains("BL-001 → BL-002"));
        assert!(content.contains("**Total:** 3"));
        assert!(content.contains("**Open:** 2"));
        assert!(content.contains("**Implemented:** 1"));
    }

    #[test]
    fn reindex_dry_run() {
        let fs = InMemoryFileSystem::new()
            .with_file("/backlog/BL-001-item.md", &bl_file(1, "Item", "open"));

        let result = reindex(&fs, Path::new("/backlog"), true).unwrap();
        assert!(result.is_some(), "dry-run should return content");
        let content = result.unwrap();
        assert!(content.contains("BL-001"));
        assert!(content.contains("**Total:** 1"));

        // File should NOT have been written
        assert!(!fs.exists(Path::new("/backlog/BACKLOG.md")));
    }

    #[test]
    fn reindex_atomic_write() {
        let fs = InMemoryFileSystem::new()
            .with_file("/backlog/BL-001-item.md", &bl_file(1, "Item", "open"));

        reindex(&fs, Path::new("/backlog"), false).unwrap();

        // Temp file should not exist after rename
        assert!(!fs.exists(Path::new("/backlog/BACKLOG.md.tmp")));
        // Final file should exist
        assert!(fs.exists(Path::new("/backlog/BACKLOG.md")));
    }

    #[test]
    fn reindex_skips_malformed() {
        let fs = InMemoryFileSystem::new()
            .with_file("/backlog/BL-001-valid.md", &bl_file(1, "Valid", "open"))
            .with_file("/backlog/BL-002-bad.md", "no frontmatter at all");

        reindex(&fs, Path::new("/backlog"), false).unwrap();

        let content = fs
            .read_to_string(Path::new("/backlog/BACKLOG.md"))
            .unwrap();
        assert!(content.contains("BL-001"));
        assert!(!content.contains("BL-002")); // malformed was skipped
        assert!(content.contains("**Total:** 1"));
    }

    #[test]
    fn reindex_creates_new_file() {
        let fs = InMemoryFileSystem::new()
            .with_file("/backlog/BL-001-item.md", &bl_file(1, "Item", "open"));
        // No existing BACKLOG.md

        reindex(&fs, Path::new("/backlog"), false).unwrap();

        let content = fs
            .read_to_string(Path::new("/backlog/BACKLOG.md"))
            .unwrap();
        assert!(content.contains("# Backlog Index"));
        assert!(content.contains("BL-001"));
        assert!(!content.contains("Dependency Graph")); // no graph to preserve
    }
}
