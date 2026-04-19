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
pub fn gc_trash(fs: &dyn FileSystem, root: &SafePath, today: &str, retention_days: u32) -> u32 {
    let trash_root = root.full().join(".trash");
    let entries = match fs.read_dir(&trash_root) {
        Ok(e) => e,
        Err(_) => return 0,
    };

    let today_days = match date_to_days(today) {
        Some(d) => d,
        None => return 0,
    };

    let mut deleted = 0u32;
    for entry in entries {
        let dir_name = match entry.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_owned(),
            None => continue,
        };
        let entry_days = match date_to_days(&dir_name) {
            Some(d) => d,
            None => continue,
        };
        let age = today_days.saturating_sub(entry_days);
        if age > retention_days && fs.remove_dir_all(&entry).is_ok() {
            deleted += 1;
        }
    }
    deleted
}

/// Convert a `YYYY-MM-DD` string to a day count (days since year 0).
///
/// Returns `None` if the string cannot be parsed as a valid date.
fn date_to_days(date: &str) -> Option<u32> {
    let parts: Vec<&str> = date.splitn(3, '-').collect();
    if parts.len() != 3 {
        return None;
    }
    let year: u32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;
    let day: u32 = parts[2].parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    // Days in each month (non-leap year base; February adjusted below)
    let days_in_month = [0u32, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let is_leap = (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400);
    let feb_days = if is_leap { 29u32 } else { 28u32 };

    // Days from year 0 (approximate; sufficient for age comparison)
    let year_days = year * 365 + year / 4 - year / 100 + year / 400;
    let mut month_days = 0u32;
    for m in 1..month {
        month_days += if m == 2 {
            feb_days
        } else {
            days_in_month[m as usize]
        };
    }
    Some(year_days + month_days + day)
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
