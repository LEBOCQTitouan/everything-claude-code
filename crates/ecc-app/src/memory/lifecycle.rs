//! Lifecycle use cases: gc, stats, promote.

use ecc_domain::memory::{MemoryEntry, MemoryId, MemoryTier};
use ecc_ports::memory_store::{MemoryStore, MemoryStoreError};

use crate::memory::crud::MemoryAppError;

/// Result of a GC run.
#[derive(Debug)]
pub struct GcResult {
    pub deleted_count: usize,
    pub dry_run: bool,
    pub entries: Vec<MemoryEntry>,
}

/// Run garbage collection: deletes stale entries older than 180 days.
///
/// If `dry_run=true`, reports without deleting.
pub fn gc(store: &dyn MemoryStore, dry_run: bool) -> Result<GcResult, MemoryAppError> {
    if dry_run {
        // For dry-run: query what would be deleted without deleting.
        // We use delete_stale_older_than in a temp store — but since we can't
        // partially preview, we list all entries and filter manually.
        let all = store
            .list_filtered(None, None, None)
            .map_err(MemoryAppError::Store)?;
        let candidates: Vec<MemoryEntry> = all.into_iter().filter(|e| e.stale).collect();
        let count = candidates.len();
        return Ok(GcResult {
            deleted_count: count,
            dry_run: true,
            entries: candidates,
        });
    }

    let deleted = store
        .delete_stale_older_than(180)
        .map_err(MemoryAppError::Store)?;
    let count = deleted.len();
    Ok(GcResult {
        deleted_count: count,
        dry_run: false,
        entries: deleted,
    })
}

/// Get memory store statistics.
pub fn stats(store: &dyn MemoryStore) -> Result<ecc_domain::memory::MemoryStats, MemoryAppError> {
    store.stats().map_err(MemoryAppError::Store)
}

/// Delete all [`MemoryStore`] entries whose `source_path` equals `backlog_id`.
///
/// Returns the number of entries deleted.
pub fn prune_by_backlog(store: &dyn MemoryStore, backlog_id: &str) -> Result<u32, MemoryAppError> {
    let all = store
        .list_filtered(None, None, None)
        .map_err(MemoryAppError::Store)?;

    let to_delete: Vec<_> = all
        .into_iter()
        .filter(|e| e.source_path.as_deref() == Some(backlog_id))
        .collect();

    let mut count = 0u32;
    for entry in to_delete {
        store.delete(entry.id).map_err(MemoryAppError::Store)?;
        count += 1;
    }
    Ok(count)
}

/// Promote an entry from episodic to semantic, boosting relevance 2x.
///
/// Returns `AlreadySemantic` error if already at semantic tier.
/// Returns `NotFound` error if entry doesn't exist.
pub fn promote(store: &dyn MemoryStore, id: MemoryId) -> Result<MemoryEntry, MemoryAppError> {
    let entry = store.get(id).map_err(|e| match e {
        MemoryStoreError::NotFound(id) => MemoryAppError::NotFound(id),
        other => MemoryAppError::Store(other),
    })?;

    if entry.tier == MemoryTier::Semantic {
        return Err(MemoryAppError::AlreadySemantic);
    }

    let promoted = MemoryEntry::new(
        entry.id,
        MemoryTier::Semantic,
        entry.title.clone(),
        entry.content.clone(),
        entry.tags.clone(),
        entry.project_id.clone(),
        entry.session_id.clone(),
        entry.relevance_score * 2.0,
        entry.created_at.clone(),
        crate::memory::crud::current_timestamp(),
        entry.stale,
        entry.related_work_items.clone(),
        entry.source_path.clone(),
    );

    store.update(&promoted).map_err(MemoryAppError::Store)?;
    Ok(promoted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_domain::memory::MemoryTier;
    use ecc_test_support::InMemoryMemoryStore;

    fn make_store() -> InMemoryMemoryStore {
        InMemoryMemoryStore::new()
    }

    fn insert_entry(store: &InMemoryMemoryStore, tier: MemoryTier, stale: bool) -> MemoryId {
        let entry = ecc_domain::memory::MemoryEntry::new(
            MemoryId(0),
            tier,
            "Title",
            "content for this entry",
            vec![],
            None,
            None,
            1.0,
            "2024-01-01T00:00:00Z",
            "2024-01-01T00:00:00Z",
            stale,
            vec![],
            None,
        );
        store.insert(&entry).unwrap()
    }

    fn create_entry_with_source(
        backlog_id: &str,
        content: &str,
    ) -> ecc_domain::memory::MemoryEntry {
        ecc_domain::memory::MemoryEntry::new(
            MemoryId(0),
            MemoryTier::Episodic,
            "Title",
            content,
            vec![],
            None,
            None,
            1.0,
            "2026-01-01T00:00:00Z",
            "2026-01-01T00:00:00Z",
            false,
            vec![],
            Some(backlog_id.to_owned()),
        )
    }

    // PC-047: prune_by_backlog on an empty store returns Ok(0)
    #[test]
    fn prune_by_backlog_empty_store() {
        let store = InMemoryMemoryStore::new();
        let count = prune_by_backlog(&store, "BL-001").unwrap();
        assert_eq!(count, 0);
    }

    // PC-044: prune_by_backlog deletes all entries tagged with BL-ID and returns count
    #[test]
    fn prune_by_backlog_returns_count() {
        let store = InMemoryMemoryStore::new();
        store
            .insert(&create_entry_with_source("BL-001", "content1"))
            .unwrap();
        store
            .insert(&create_entry_with_source("BL-001", "content2"))
            .unwrap();
        store
            .insert(&create_entry_with_source("BL-001", "content3"))
            .unwrap();
        store
            .insert(&create_entry_with_source("BL-002", "other1"))
            .unwrap();
        store
            .insert(&create_entry_with_source("BL-002", "other2"))
            .unwrap();

        let count = prune_by_backlog(&store, "BL-001").unwrap();
        assert_eq!(count, 3, "returns count of deleted BL-001 entries");
    }

    // PC-038: App `gc` deletes stale entries >180 days
    #[test]
    fn test_gc_deletes_stale_entries() {
        let store = make_store();
        // Insert a stale entry with an old date (2024, which is ~365+ days before 2026)
        insert_entry(&store, MemoryTier::Episodic, true);
        // Insert a non-stale entry
        insert_entry(&store, MemoryTier::Episodic, false);

        let result = gc(&store, false).unwrap();
        // The stale entry with year 2024 (approx 730 days old) should be deleted
        assert!(result.deleted_count >= 1);
        assert!(!result.dry_run);
    }

    // PC-039: App `gc --dry-run` reports without deleting
    #[test]
    fn test_gc_dry_run_does_not_delete() {
        let store = make_store();
        insert_entry(&store, MemoryTier::Episodic, true);
        insert_entry(&store, MemoryTier::Episodic, false);

        let result = gc(&store, true).unwrap();
        assert!(result.dry_run);
        // Non-stale entry remains in store
        let all = store.list_filtered(None, None, None).unwrap();
        assert_eq!(all.len(), 2); // Nothing deleted
        assert!(result.deleted_count >= 1); // Reported what would be deleted
    }

    // PC-040: App `stats` returns counts by type, stale count, db size, dates
    #[test]
    fn test_stats_returns_counts_by_tier() {
        let store = make_store();
        insert_entry(&store, MemoryTier::Working, false);
        insert_entry(&store, MemoryTier::Semantic, false);
        insert_entry(&store, MemoryTier::Semantic, true);

        let s = stats(&store).unwrap();
        assert_eq!(
            s.counts_by_tier
                .get(&MemoryTier::Working)
                .copied()
                .unwrap_or(0),
            1
        );
        assert_eq!(
            s.counts_by_tier
                .get(&MemoryTier::Semantic)
                .copied()
                .unwrap_or(0),
            2
        );
        assert_eq!(s.stale_count, 1);
    }

    #[test]
    fn test_promote_episodic_to_semantic() {
        let store = make_store();
        let entry = ecc_domain::memory::MemoryEntry::new(
            MemoryId(0),
            MemoryTier::Episodic,
            "Promote Me",
            "content here",
            vec![],
            None,
            None,
            1.0,
            "2026-01-01T00:00:00Z",
            "2026-01-01T00:00:00Z",
            false,
            vec![],
            None,
        );
        let id = store.insert(&entry).unwrap();
        let promoted = promote(&store, id).unwrap();
        assert_eq!(promoted.tier, MemoryTier::Semantic);
        assert!((promoted.relevance_score - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_promote_already_semantic_returns_error() {
        let store = make_store();
        let entry = ecc_domain::memory::MemoryEntry::new(
            MemoryId(0),
            MemoryTier::Semantic,
            "Already Semantic",
            "content here",
            vec![],
            None,
            None,
            1.0,
            "2026-01-01T00:00:00Z",
            "2026-01-01T00:00:00Z",
            false,
            vec![],
            None,
        );
        let id = store.insert(&entry).unwrap();
        let result = promote(&store, id);
        assert!(matches!(result, Err(MemoryAppError::AlreadySemantic)));
    }

    #[test]
    fn test_promote_nonexistent_returns_not_found() {
        let store = make_store();
        let result = promote(&store, MemoryId(999));
        assert!(matches!(result, Err(MemoryAppError::NotFound(_))));
    }

    // PC-045: prune_by_backlog reuses existing MemoryStore trait methods only
    #[test]
    fn prune_by_backlog_no_new_port() {
        const PORT_SOURCE: &str = include_str!("../../../ecc-ports/src/memory_store.rs");

        let forbidden = [
            concat!("fn prune_", "by_backlog"),
            concat!("fn delete_", "by_backlog"),
            concat!("fn list_", "by_backlog"),
        ];
        for pat in forbidden {
            assert!(
                !PORT_SOURCE.contains(pat),
                "MemoryStore port has new backlog-specific method: {pat}"
            );
        }

        // Sanity: the lifecycle function itself still works via existing methods
        let store = InMemoryMemoryStore::new();
        let entry = create_entry_with_source("BL-045", "sanity");
        store.insert(&entry).unwrap();
        let deleted = prune_by_backlog(&store, "BL-045").unwrap();
        assert_eq!(deleted, 1);
    }
}
