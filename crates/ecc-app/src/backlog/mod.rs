//! Backlog management use cases — next_id, check_duplicates, reindex, list_available.
//!
//! Orchestrates domain logic through port traits (BacklogEntryStore, BacklogLockStore,
//! BacklogIndexStore, WorktreeManager, Clock).

mod index;
mod migration;
mod status;

pub use index::{parse_index_statuses, reindex};
pub use migration::{MigrationReport, migrate_statuses};
pub use status::{list_available, update_status, update_status_with_prune_hook};
pub(crate) use status::collect_claimed_ids;

use ecc_domain::backlog::entry::BacklogError;
use ecc_domain::backlog::similarity::{DUPLICATE_THRESHOLD, DuplicateCandidate, composite_score};
use ecc_ports::backlog::BacklogEntryStore;
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

pub(crate) static BL_ID_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)bl-?(\d{3,})").unwrap());

/// Extract a BL-NNN numeric ID from a worktree path's last component.
///
/// Matches patterns like `ecc-session-20260407-bl-042-something` or
/// `ecc-bl042-feature` (case-insensitive).
pub(crate) fn extract_bl_id_from_path(path: &str) -> Option<u32> {
    let last = path.rsplit('/').next().unwrap_or(path);
    let caps = BL_ID_RE.captures(last)?;
    caps.get(1)?.as_str().parse().ok()
}

/// Compute the next sequential backlog ID from entries via the store port.
///
/// Returns `"BL-NNN"` where NNN is max existing ID + 1, zero-padded to 3 digits.
pub fn next_id(
    entries: &dyn BacklogEntryStore,
    backlog_dir: &Path,
) -> Result<String, BacklogError> {
    entries.next_id(backlog_dir)
}

/// Check for duplicate backlog entries by title similarity.
///
/// Filters to active entries (open/in-progress) only.
/// Returns candidates sorted by score descending, filtered to score >= DUPLICATE_THRESHOLD.
pub fn check_duplicates(
    entries: &dyn BacklogEntryStore,
    backlog_dir: &Path,
    query: &str,
    query_tags: &[String],
) -> Result<Vec<DuplicateCandidate>, BacklogError> {
    if query.is_empty() {
        return Err(BacklogError::EmptyQuery);
    }

    let all_entries = entries.load_entries(backlog_dir)?;
    let mut candidates = Vec::new();

    for entry in &all_entries {
        if !entry.status.is_active() {
            continue;
        }
        let score = composite_score(query, &entry.title, query_tags, &entry.tags);
        if score >= DUPLICATE_THRESHOLD {
            candidates.push(DuplicateCandidate {
                id: entry.id.clone(),
                title: entry.title.clone(),
                score: (score * 100.0).round() / 100.0,
            });
        }
    }

    candidates.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(candidates)
}

/// Extract the numeric part from a BL-NNN id string.
pub(crate) fn extract_bl_num(id: &str) -> Option<u32> {
    id.strip_prefix("BL-").and_then(|s| s.parse().ok())
}

/// Convert an [`ecc_ports::fs::FsError`] into a [`BacklogError`].
///
/// The orphan rule prevents `impl From<FsError> for BacklogError` here since neither
/// type is defined in this crate. Use this function where conversion is needed.
pub fn backlog_error_from_fs(e: ecc_ports::fs::FsError) -> BacklogError {
    BacklogError::Io {
        path: e.to_string(),
        message: e.to_string(),
    }
}

#[cfg(test)]
pub(crate) mod test_helpers {
    use ecc_domain::backlog::entry::{BacklogEntry, BacklogStatus};
    use ecc_domain::backlog::lock::LockFile;
    use ecc_test_support::MockClock;

    pub const BACKLOG_DIR: &str = "/backlog";
    pub const PROJECT_DIR: &str = "/project";

    /// Helper: 2026-04-07T10:00:00Z = 1744016400 epoch seconds (approx)
    /// Use a fixed "now" that is recent enough to not be stale.
    pub fn fresh_clock() -> MockClock {
        // A recent timestamp: 2026-04-07T12:00:00Z
        MockClock::fixed("2026-04-07T12:00:00Z", 1_744_023_600)
    }

    pub fn make_entry(id: &str, status: BacklogStatus) -> BacklogEntry {
        BacklogEntry {
            id: id.to_string(),
            title: format!("Entry {id}"),
            status,
            created: "2026-04-07".into(),
            tier: None,
            scope: None,
            target: None,
            target_command: None,
            tags: vec![],
        }
    }

    pub fn make_entry_with_tags(
        id: &str,
        status: BacklogStatus,
        tags: Vec<String>,
    ) -> BacklogEntry {
        BacklogEntry {
            id: id.to_string(),
            title: format!("Entry {id}"),
            status,
            created: "2026-04-07".into(),
            tier: None,
            scope: None,
            target: None,
            target_command: None,
            tags,
        }
    }

    pub fn make_fresh_lock(worktree_name: &str) -> LockFile {
        // Timestamp close to now (within 24h)
        LockFile::new(
            worktree_name.to_string(),
            "2026-04-07T11:00:00Z".to_string(),
        )
        .unwrap()
    }

    pub fn make_stale_lock(worktree_name: &str) -> LockFile {
        // 2026-04-06T00:00:00Z — more than 24h before fresh_clock's 2026-04-07T12:00:00Z
        LockFile::new(
            worktree_name.to_string(),
            "2026-04-06T00:00:00Z".to_string(),
        )
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::test_helpers::*;
    use ecc_domain::backlog::entry::BacklogStatus;
    use ecc_test_support::InMemoryBacklogRepository;
    use std::path::Path;

    // --- next_id tests ---

    #[test]
    fn next_id_sequential() {
        let repo = InMemoryBacklogRepository::new()
            .with_entry(make_entry("BL-001", BacklogStatus::Open))
            .with_entry(make_entry("BL-075", BacklogStatus::Implemented));
        let result = next_id(&repo, Path::new(BACKLOG_DIR)).unwrap();
        assert_eq!(result, "BL-076");
    }

    #[test]
    fn next_id_empty() {
        let repo = InMemoryBacklogRepository::new();
        let result = next_id(&repo, Path::new(BACKLOG_DIR)).unwrap();
        assert_eq!(result, "BL-001");
    }

    // --- check_duplicates tests ---

    #[test]
    fn check_duplicates_finds_similar() {
        let mut entry = make_entry_with_tags(
            "BL-052",
            BacklogStatus::Open,
            vec!["rust".into(), "hooks".into()],
        );
        entry.title = "Replace hooks with Rust binaries".into();
        let repo = InMemoryBacklogRepository::new().with_entry(entry);

        let result = check_duplicates(
            &repo,
            Path::new(BACKLOG_DIR),
            "Replace hooks with compiled Rust",
            &["rust".into(), "hooks".into()],
        )
        .unwrap();

        assert!(!result.is_empty(), "expected at least one candidate");
        assert!(
            result[0].score >= DUPLICATE_THRESHOLD,
            "score {} < {}",
            result[0].score,
            DUPLICATE_THRESHOLD
        );
    }

    // --- FsError conversion test ---

    #[test]
    fn fs_error_conversion() {
        use ecc_ports::fs::FsError;
        let fs_err = FsError::NotFound(std::path::PathBuf::from("/some/path"));
        let backlog_err = backlog_error_from_fs(fs_err);
        assert!(
            matches!(backlog_err, BacklogError::Io { .. }),
            "FsError should convert to BacklogError::Io"
        );
    }
}
