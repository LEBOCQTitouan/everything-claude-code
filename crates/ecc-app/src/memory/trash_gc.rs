//! Trash garbage collection for the memory system.
//!
//! Provides time-based retention cleanup for `.trash/` subdirectories,
//! independent of BL-ID content (SOLID-001 split from file_prune).

use ecc_domain::memory::SafePath;
use ecc_ports::fs::FileSystem;

/// Delete trash directories under `<root>/.trash/` that are older than
/// `retention_days` days relative to `today`.
///
/// Each subdirectory name must be a `YYYY-MM-DD` date string. Directories
/// whose age (in days) exceeds `retention_days` are removed recursively.
///
/// Returns the count of directories deleted.
pub fn gc_trash(
    _fs: &dyn FileSystem,
    _root: &SafePath,
    _today: &str,
    _retention_days: u32,
) -> u32 {
    unimplemented!("gc_trash not yet implemented")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_domain::memory::SafePath;
    use ecc_test_support::InMemoryFileSystem;
    use std::path::PathBuf;

    #[test]
    fn gc_by_date_only() {
        // Seed: root with .trash/2026-04-05/foo.md (14 days old) and .trash/2026-04-18/bar.md (1 day old)
        // today = "2026-04-19", retention_days = 7
        // gc_trash should delete the 2026-04-05 dir but not 2026-04-18

        let root_path = PathBuf::from("/root/memory");
        let fs = InMemoryFileSystem::new()
            .with_dir(&root_path)
            .with_file(root_path.join(".trash/2026-04-05/foo.md"), "stale")
            .with_file(root_path.join(".trash/2026-04-18/bar.md"), "recent");
        let safe = SafePath::from_canonical(root_path.clone(), root_path.clone()).unwrap();

        let deleted = gc_trash(&fs, &safe, "2026-04-19", 7);
        assert_eq!(deleted, 1);
        assert!(
            !fs.exists(&root_path.join(".trash/2026-04-05")),
            "stale dir removed"
        );
        assert!(
            fs.exists(&root_path.join(".trash/2026-04-18")),
            "recent dir kept"
        );
    }
}
