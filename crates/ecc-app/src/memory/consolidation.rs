//! Consolidation use cases: dedup, stale marking, relevance scoring,
//! CONTEXT.md generation, and working-memory expiry.

use std::path::Path;

use ecc_domain::memory::{MemoryEntry, MemoryId, MemoryTier};
use ecc_ports::memory_store::MemoryStore;

use crate::memory::crud::{MemoryAppError, current_timestamp};
use ecc_domain::time::is_leap_year;

/// Result of a consolidation run.
#[derive(Debug)]
pub struct ConsolidationResult {
    pub merged_count: usize,
    pub stale_marked_count: usize,
    pub scores_updated_count: usize,
    /// True when consolidation was skipped because another session holds the lock.
    pub skipped: bool,
}

/// Result of a working-memory expiry run.
#[derive(Debug)]
pub struct ExpireResult {
    pub promoted_count: usize,
    pub deleted_count: usize,
}

/// Parse an ISO-8601 UTC timestamp string ("YYYY-MM-DDTHH:MM:SSZ") into
/// approximate days since the Unix epoch.
///
/// Returns 0 on parse failure.
pub fn iso8601_to_epoch_days(ts: &str) -> u64 {
    if ts.len() < 10 {
        return 0;
    }
    let date_part = &ts[..10]; // "YYYY-MM-DD"
    let parts: Vec<&str> = date_part.split('-').collect();
    if parts.len() != 3 {
        return 0;
    }
    let year: u64 = parts[0].parse().unwrap_or(1970);
    let month: u64 = parts[1].parse().unwrap_or(1);
    let day: u64 = parts[2].parse().unwrap_or(1);
    ymd_to_epoch_days(year, month, day)
}


fn ymd_to_epoch_days(year: u64, month: u64, day: u64) -> u64 {
    let mut days = 0u64;
    for y in 1970..year {
        days += if is_leap_year(y) { 366 } else { 365 };
    }
    let month_days: [u64; 12] = [
        31,
        if is_leap_year(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    days += month_days.iter().take((month - 1) as usize).sum::<u64>();
    days += day - 1;
    days
}

/// Compute age in days between two ISO-8601 timestamps.
///
/// Returns 0 if `created_at` is after `now` or parse fails.
pub fn age_days(created_at: &str, now: &str) -> u64 {
    let created = iso8601_to_epoch_days(created_at);
    let now_days = iso8601_to_epoch_days(now);
    now_days.saturating_sub(created)
}

/// Run consolidation on recent entries:
/// 1. Fetch the `max_entries` most recent entries.
/// 2. For each pair, merge if Jaccard similarity > 0.8 (skip short entries).
/// 3. Mark entries >90 days with zero work-item refs as stale.
/// 4. Recompute relevance scores.
///
/// If `lock_held` is true, returns immediately with `skipped=true`.
pub fn consolidate(
    store: &dyn MemoryStore,
    max_entries: usize,
    now: &str,
    lock_held: bool,
) -> Result<ConsolidationResult, MemoryAppError> {
    if lock_held {
        return Ok(ConsolidationResult {
            merged_count: 0,
            stale_marked_count: 0,
            scores_updated_count: 0,
            skipped: true,
        });
    }

    let entries = store
        .list_recent(max_entries)
        .map_err(MemoryAppError::Store)?;

    let mut merged_count = 0usize;
    let mut stale_marked_count = 0usize;
    let mut scores_updated_count = 0usize;

    let mut removed_ids: std::collections::HashSet<MemoryId> = std::collections::HashSet::new();

    // --- Pass 1: dedup/merge similar entries ---
    let n = entries.len();
    for i in 0..n {
        if removed_ids.contains(&entries[i].id) {
            continue;
        }
        if ecc_domain::memory::consolidation::is_short_entry(&entries[i].content) {
            continue;
        }
        for j in (i + 1)..n {
            if removed_ids.contains(&entries[j].id) {
                continue;
            }
            if ecc_domain::memory::consolidation::is_short_entry(&entries[j].content) {
                continue;
            }
            if ecc_domain::memory::consolidation::should_merge(
                &entries[i].content,
                &entries[j].content,
            ) {
                let (keep_id, remove_id) = if entries[i].updated_at >= entries[j].updated_at {
                    (entries[i].id, entries[j].id)
                } else {
                    (entries[j].id, entries[i].id)
                };
                store
                    .merge_entries(keep_id, remove_id, "merged: similar content")
                    .map_err(MemoryAppError::Store)?;
                removed_ids.insert(remove_id);
                merged_count += 1;
            }
        }
    }

    // Re-fetch after merges to get updated state.
    let current_entries = store
        .list_recent(max_entries)
        .map_err(MemoryAppError::Store)?;

    // --- Pass 2: mark stale + recompute scores ---
    for entry in &current_entries {
        if removed_ids.contains(&entry.id) {
            continue;
        }

        let entry_age = age_days(&entry.created_at, now);
        let ref_count = entry.related_work_items.len() as u32;

        let should_stale =
            ecc_domain::memory::consolidation::should_mark_stale(entry_age, ref_count);
        let needs_stale_update = should_stale && !entry.stale;

        let recency = ecc_domain::memory::consolidation::recency_factor(entry_age);
        let new_score =
            ecc_domain::memory::consolidation::compute_relevance_score(recency, ref_count);
        let score_changed = (new_score - entry.relevance_score).abs() > 1e-9;

        if needs_stale_update || score_changed {
            let updated = MemoryEntry::new(
                entry.id,
                entry.tier.clone(),
                entry.title.clone(),
                entry.content.clone(),
                entry.tags.clone(),
                entry.project_id.clone(),
                entry.session_id.clone(),
                new_score,
                entry.created_at.clone(),
                current_timestamp(),
                entry.stale || needs_stale_update,
                entry.related_work_items.clone(),
                entry.source_path.clone(),
            );
            store.update(&updated).map_err(MemoryAppError::Store)?;

            if needs_stale_update {
                stale_marked_count += 1;
            }
            if score_changed {
                scores_updated_count += 1;
            }
        }
    }

    Ok(ConsolidationResult {
        merged_count,
        stale_marked_count,
        scores_updated_count,
        skipped: false,
    })
}

/// Generate CONTEXT.md from the top-N entries by relevance, writing to `context_path`.
///
/// Does NOT modify MEMORY.md.
pub fn generate_context_md(
    store: &dyn MemoryStore,
    fs: &dyn ecc_ports::fs::FileSystem,
    context_path: &Path,
    top_n: usize,
    max_lines: usize,
) -> Result<(), MemoryAppError> {
    let mut entries = store.list_recent(top_n).map_err(MemoryAppError::Store)?;

    // Sort by relevance_score descending
    entries.sort_by(|a, b| {
        b.relevance_score
            .partial_cmp(&a.relevance_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let content = ecc_domain::memory::context::format_context_md(&entries, max_lines);

    if let Some(parent) = context_path.parent()
        && !fs.exists(parent)
    {
        fs.create_dir_all(parent)
            .map_err(|e| MemoryAppError::InvalidInput(e.to_string()))?;
    }

    fs.write(context_path, &content)
        .map_err(|e| MemoryAppError::InvalidInput(e.to_string()))?;

    Ok(())
}

/// Expire working-tier memories older than `max_age_hours`.
///
/// - If content.len() > 50: promote to episodic tier.
/// - Otherwise: delete entirely.
pub fn expire_working_memories(
    store: &dyn MemoryStore,
    max_age_hours: u64,
    now: &str,
) -> Result<ExpireResult, MemoryAppError> {
    let working_entries = store
        .list_filtered(Some(MemoryTier::Working), None, None)
        .map_err(MemoryAppError::Store)?;

    let max_age_days = max_age_hours / 24;

    let mut promoted_count = 0usize;
    let mut deleted_count = 0usize;

    for entry in &working_entries {
        let entry_age_days = age_days(&entry.created_at, now);

        if entry_age_days < max_age_days {
            continue;
        }

        if entry.content.len() > 50 {
            let promoted = MemoryEntry::new(
                entry.id,
                MemoryTier::Episodic,
                entry.title.clone(),
                entry.content.clone(),
                entry.tags.clone(),
                entry.project_id.clone(),
                entry.session_id.clone(),
                entry.relevance_score,
                entry.created_at.clone(),
                current_timestamp(),
                entry.stale,
                entry.related_work_items.clone(),
                entry.source_path.clone(),
            );
            store.update(&promoted).map_err(MemoryAppError::Store)?;
            promoted_count += 1;
        } else {
            store.delete(entry.id).map_err(MemoryAppError::Store)?;
            deleted_count += 1;
        }
    }

    Ok(ExpireResult {
        promoted_count,
        deleted_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_domain::memory::{MemoryEntry, MemoryId, MemoryTier};
    use ecc_ports::fs::FileSystem;
    use ecc_test_support::{InMemoryFileSystem, InMemoryMemoryStore};

    fn make_store() -> InMemoryMemoryStore {
        InMemoryMemoryStore::new()
    }

    fn insert_entry_with_content(
        store: &InMemoryMemoryStore,
        tier: MemoryTier,
        content: &str,
        created_at: &str,
        updated_at: &str,
        stale: bool,
    ) -> MemoryId {
        let entry = MemoryEntry::new(
            MemoryId(0),
            tier,
            "Test Entry",
            content,
            vec![],
            None,
            None,
            1.0,
            created_at,
            updated_at,
            stale,
            vec![],
            None,
        );
        store.insert(&entry).unwrap()
    }

    fn long_content(prefix: &str) -> String {
        format!(
            "{} one two three four five six seven eight nine ten eleven twelve words here for long",
            prefix
        )
    }

    // PC-061: consolidate_merges_similar_entries
    #[test]
    fn consolidate_merges_similar_entries() {
        let store = make_store();
        let content_a = "The quick brown fox jumps over the lazy dog in the forest today";
        let content_b = "The quick brown fox jumps over the lazy dog in the forest today"; // identical
        insert_entry_with_content(
            &store,
            MemoryTier::Episodic,
            content_a,
            "2026-01-01T00:00:00Z",
            "2026-01-01T00:00:00Z",
            false,
        );
        insert_entry_with_content(
            &store,
            MemoryTier::Episodic,
            content_b,
            "2026-01-02T00:00:00Z",
            "2026-01-02T00:00:00Z",
            false,
        );

        let result = consolidate(&store, 100, "2026-03-01T00:00:00Z", false).unwrap();
        assert_eq!(result.merged_count, 1);
        assert!(!result.skipped);

        let remaining = store.list_filtered(None, None, None).unwrap();
        assert_eq!(remaining.len(), 1);
    }

    // PC-066: consolidate_skips_short_entries
    #[test]
    fn consolidate_skips_short_entries() {
        let store = make_store();
        insert_entry_with_content(
            &store,
            MemoryTier::Episodic,
            "short content",
            "2026-01-01T00:00:00Z",
            "2026-01-01T00:00:00Z",
            false,
        );
        insert_entry_with_content(
            &store,
            MemoryTier::Episodic,
            "short content",
            "2026-01-02T00:00:00Z",
            "2026-01-02T00:00:00Z",
            false,
        );

        let result = consolidate(&store, 100, "2026-03-01T00:00:00Z", false).unwrap();
        assert_eq!(result.merged_count, 0, "short entries should not be merged");

        let remaining = store.list_filtered(None, None, None).unwrap();
        assert_eq!(remaining.len(), 2);
    }

    // PC-062: consolidate_marks_stale (entries older than 90 days, 0 refs)
    #[test]
    fn consolidate_marks_stale() {
        let store = make_store();
        // 2025-11-01 to 2026-03-01 = 120 days (> 90, should be stale)
        insert_entry_with_content(
            &store,
            MemoryTier::Episodic,
            &long_content("old"),
            "2025-11-01T00:00:00Z",
            "2025-11-01T00:00:00Z",
            false,
        );

        let result = consolidate(&store, 100, "2026-03-01T00:00:00Z", false).unwrap();
        assert!(
            result.stale_marked_count >= 1,
            "old entry should be marked stale"
        );
    }

    // PC-064: consolidate_caps_at_max_entries
    #[test]
    fn consolidate_caps_at_max_entries() {
        let store = make_store();
        for i in 0..150u32 {
            let content = format!("entry number {} with many unique words here alpha beta", i);
            insert_entry_with_content(
                &store,
                MemoryTier::Episodic,
                &content,
                "2026-01-01T00:00:00Z",
                "2026-01-01T00:00:00Z",
                false,
            );
        }

        let result = consolidate(&store, 100, "2026-03-01T00:00:00Z", false).unwrap();
        assert!(!result.skipped);
        let remaining = store.list_filtered(None, None, None).unwrap();
        assert!(
            remaining.len() >= 50,
            "unprocessed entries should still be in store"
        );
    }

    // PC-063: consolidate_updates_relevance_scores
    #[test]
    fn consolidate_updates_relevance_scores() {
        let store = make_store();
        let content = long_content("test");
        let id = insert_entry_with_content(
            &store,
            MemoryTier::Episodic,
            &content,
            "2026-01-01T00:00:00Z",
            "2026-01-01T00:00:00Z",
            false,
        );

        let before = store.get(id).unwrap().relevance_score;
        let result = consolidate(&store, 100, "2026-03-01T00:00:00Z", false).unwrap();
        let after = store.get(id).unwrap().relevance_score;

        assert!(result.scores_updated_count >= 1);
        assert!(
            after < before,
            "score should decay with age: before={before}, after={after}"
        );
    }

    // PC-065: consolidate acquires try-lock; if held, returns Ok(skipped)
    #[test]
    fn consolidate_skips_when_lock_held() {
        let store = make_store();
        let result = consolidate(&store, 100, "2026-03-01T00:00:00Z", true).unwrap();
        assert!(result.skipped);
        assert_eq!(result.merged_count, 0);
    }

    // expire_working_promotes_long_content (PC-079 content)
    #[test]
    fn expire_working_promotes_long_content() {
        let store = make_store();
        let long =
            "This is a working memory entry that has more than fifty characters in total length.";
        assert!(long.len() > 50);
        insert_entry_with_content(
            &store,
            MemoryTier::Working,
            long,
            "2026-01-01T00:00:00Z",
            "2026-01-01T00:00:00Z",
            false,
        );

        let result = expire_working_memories(&store, 24, "2026-01-03T00:00:00Z").unwrap();
        assert_eq!(result.promoted_count, 1);
        assert_eq!(result.deleted_count, 0);

        let entries = store
            .list_filtered(Some(MemoryTier::Episodic), None, None)
            .unwrap();
        assert_eq!(entries.len(), 1);
        let working = store
            .list_filtered(Some(MemoryTier::Working), None, None)
            .unwrap();
        assert_eq!(working.len(), 0);
    }

    // expire_working_deletes_short_content (PC-079 content)
    #[test]
    fn expire_working_deletes_short_content() {
        let store = make_store();
        let short = "short content here";
        assert!(short.len() <= 50);
        insert_entry_with_content(
            &store,
            MemoryTier::Working,
            short,
            "2026-01-01T00:00:00Z",
            "2026-01-01T00:00:00Z",
            false,
        );

        let result = expire_working_memories(&store, 24, "2026-01-03T00:00:00Z").unwrap();
        assert_eq!(result.deleted_count, 1);
        assert_eq!(result.promoted_count, 0);

        let all = store.list_filtered(None, None, None).unwrap();
        assert_eq!(all.len(), 0);
    }

    // generates_context_file (PC-068)
    #[test]
    fn generates_context_file() {
        let store = make_store();
        let entry = MemoryEntry::new(
            MemoryId(0),
            MemoryTier::Episodic,
            "Test Entry",
            "Some content that is more than minimal",
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
        store.insert(&entry).unwrap();

        let fs = InMemoryFileSystem::new();
        let path = std::path::PathBuf::from("/tmp/context/CONTEXT.md");
        generate_context_md(&store, &fs, &path, 10, 200).unwrap();

        assert!(fs.exists(&path));
        let content = fs.read_to_string(&path).unwrap();
        assert!(content.contains("Test Entry"));
    }

    // respects_max_lines (PC-069)
    #[test]
    fn respects_max_lines() {
        let store = make_store();
        for i in 0..5 {
            let entry = MemoryEntry::new(
                MemoryId(0),
                MemoryTier::Episodic,
                &format!("Entry {i}"),
                "Content that goes here for the entry",
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
            store.insert(&entry).unwrap();
        }

        let fs = InMemoryFileSystem::new();
        let path = std::path::PathBuf::from("/tmp/context/CONTEXT.md");
        generate_context_md(&store, &fs, &path, 10, 5).unwrap();

        let content = fs.read_to_string(&path).unwrap();
        let line_count = content.lines().count();
        assert!(
            line_count <= 5,
            "expected at most 5 lines, got {line_count}"
        );
    }

    // generates_empty_message_when_no_entries (PC-071)
    #[test]
    fn generates_empty_message_when_no_entries() {
        let store = make_store();
        let fs = InMemoryFileSystem::new();
        let path = std::path::PathBuf::from("/tmp/context/CONTEXT.md");
        generate_context_md(&store, &fs, &path, 10, 200).unwrap();

        let content = fs.read_to_string(&path).unwrap();
        assert!(content.contains("No memories stored"));
    }

    // PC-070: generate_context does NOT modify MEMORY.md
    #[test]
    fn generate_context_does_not_modify_memory_md() {
        let store = make_store();
        let memory_md_path = std::path::PathBuf::from("/tmp/MEMORY.md");
        let fs =
            InMemoryFileSystem::new().with_file("/tmp/MEMORY.md", "original MEMORY.md content");

        let context_path = std::path::PathBuf::from("/tmp/context/CONTEXT.md");
        generate_context_md(&store, &fs, &context_path, 10, 200).unwrap();

        let memory_content = fs.read_to_string(&memory_md_path).unwrap();
        assert_eq!(
            memory_content, "original MEMORY.md content",
            "MEMORY.md must not be modified"
        );
    }
}
