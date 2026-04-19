//! File pruning utilities for the memory system.
//!
//! Provides BL-ID pattern matching for memory file cleanup.

use regex::Regex;

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

        let safe =
            SafePath::from_canonical(root_path.clone(), root_path.clone()).unwrap();

        let report = prune_file_memories_for_backlog(&fs, &safe, "BL-001", "2026-04-19");

        assert_eq!(report.trashed_files.len(), 2, "both BL-001 files trashed");

        assert!(!fs.exists(&root_path.join("project_bl001_foo.md")));
        assert!(!fs.exists(&root_path.join("project_bl001_bar.md")));

        assert!(fs.exists(&root_path.join(".trash/2026-04-19/project_bl001_foo.md")));
        assert!(fs.exists(&root_path.join(".trash/2026-04-19/project_bl001_bar.md")));

        assert!(fs.exists(&root_path.join("project_bl002_other.md")));

        let memory_md = fs
            .read_to_string(&root_path.join("MEMORY.md"))
            .unwrap();
        assert!(!memory_md.contains("project_bl001_foo.md"));
        assert!(!memory_md.contains("project_bl001_bar.md"));
        assert!(memory_md.contains("project_bl002_other.md"));

        assert!(report.index_updated);
        assert_eq!(report.errors, Vec::<String>::new());
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
        assert!(matches_bl_id("project_bl31.md", "BL-031"), "zero-pad allowed via 0*");

        // Invalid BL ID returns false
        assert!(!matches_bl_id("project_bl001.md", "invalid"));
    }
}
