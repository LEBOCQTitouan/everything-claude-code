//! In-memory test double for [`MemoryStore`].

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use ecc_domain::memory::{MemoryEntry, MemoryId, MemoryStats, MemoryTier};
use ecc_ports::memory_store::{MemoryStore, MemoryStoreError};

/// State shared across clones.
#[derive(Debug, Default)]
struct State {
    entries: Vec<MemoryEntry>,
    next_id: i64,
}

/// In-memory implementation of [`MemoryStore`] for deterministic unit tests.
///
/// Uses a `Vec<MemoryEntry>` protected by a `Mutex`; auto-increments IDs.
/// FTS search is a case-insensitive substring match on title + content + tags.
#[derive(Debug, Clone, Default)]
pub struct InMemoryMemoryStore {
    state: Arc<Mutex<State>>,
}

impl InMemoryMemoryStore {
    /// Create an empty store.
    pub fn new() -> Self {
        Self::default()
    }
}

impl MemoryStore for InMemoryMemoryStore {
    fn insert(&self, entry: &MemoryEntry) -> Result<MemoryId, MemoryStoreError> {
        let mut s = self.state.lock().unwrap();
        s.next_id += 1;
        let id = MemoryId(s.next_id);
        let mut stored = entry.clone();
        stored.id = id;
        s.entries.push(stored);
        Ok(id)
    }

    fn get(&self, id: MemoryId) -> Result<MemoryEntry, MemoryStoreError> {
        let s = self.state.lock().unwrap();
        s.entries
            .iter()
            .find(|e| e.id == id)
            .cloned()
            .ok_or(MemoryStoreError::NotFound(id))
    }

    fn update(&self, entry: &MemoryEntry) -> Result<(), MemoryStoreError> {
        let mut s = self.state.lock().unwrap();
        let pos = s
            .entries
            .iter()
            .position(|e| e.id == entry.id)
            .ok_or(MemoryStoreError::NotFound(entry.id))?;
        s.entries[pos] = entry.clone();
        Ok(())
    }

    fn delete(&self, id: MemoryId) -> Result<(), MemoryStoreError> {
        let mut s = self.state.lock().unwrap();
        let pos = s
            .entries
            .iter()
            .position(|e| e.id == id)
            .ok_or(MemoryStoreError::NotFound(id))?;
        s.entries.remove(pos);
        Ok(())
    }

    fn search_fts(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>, MemoryStoreError> {
        let s = self.state.lock().unwrap();
        let q = query.to_lowercase();
        let results: Vec<MemoryEntry> = s
            .entries
            .iter()
            .filter(|e| {
                let haystack = format!(
                    "{} {} {}",
                    e.title.to_lowercase(),
                    e.content.to_lowercase(),
                    e.tags.join(" ").to_lowercase()
                );
                haystack.contains(&q)
            })
            .take(limit)
            .cloned()
            .collect();
        Ok(results)
    }

    fn list_filtered(
        &self,
        tier: Option<MemoryTier>,
        tag: Option<&str>,
        project_id: Option<&str>,
    ) -> Result<Vec<MemoryEntry>, MemoryStoreError> {
        let s = self.state.lock().unwrap();
        let results: Vec<MemoryEntry> = s
            .entries
            .iter()
            .filter(|e| {
                if let Some(ref t) = tier
                    && &e.tier != t
                {
                    return false;
                }
                if let Some(tag_str) = tag
                    && !e.tags.iter().any(|t| t == tag_str)
                {
                    return false;
                }
                if let Some(pid) = project_id
                    && e.project_id.as_deref() != Some(pid)
                {
                    return false;
                }
                true
            })
            .cloned()
            .collect();
        Ok(results)
    }

    fn list_recent(&self, limit: usize) -> Result<Vec<MemoryEntry>, MemoryStoreError> {
        let s = self.state.lock().unwrap();
        let mut results: Vec<MemoryEntry> = s.entries.clone();
        // Sort by updated_at descending (lexicographic on ISO-8601 strings)
        results.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        results.truncate(limit);
        Ok(results)
    }

    fn count_by_tier(&self) -> Result<HashMap<MemoryTier, usize>, MemoryStoreError> {
        let s = self.state.lock().unwrap();
        let mut counts: HashMap<MemoryTier, usize> = HashMap::new();
        for e in &s.entries {
            *counts.entry(e.tier.clone()).or_insert(0) += 1;
        }
        Ok(counts)
    }

    fn stats(&self) -> Result<MemoryStats, MemoryStoreError> {
        let s = self.state.lock().unwrap();
        let mut counts: HashMap<MemoryTier, usize> = HashMap::new();
        let mut stale_count = 0usize;
        let mut oldest: Option<String> = None;
        let mut newest: Option<String> = None;

        for e in &s.entries {
            *counts.entry(e.tier.clone()).or_insert(0) += 1;
            if e.stale {
                stale_count += 1;
            }
            match &oldest {
                None => oldest = Some(e.created_at.clone()),
                Some(o) if e.created_at < *o => oldest = Some(e.created_at.clone()),
                _ => {}
            }
            match &newest {
                None => newest = Some(e.created_at.clone()),
                Some(n) if e.created_at > *n => newest = Some(e.created_at.clone()),
                _ => {}
            }
        }

        Ok(MemoryStats::new(counts, stale_count, 0, oldest, newest))
    }

    fn get_by_source_path(&self, path: &str) -> Result<Option<MemoryEntry>, MemoryStoreError> {
        let s = self.state.lock().unwrap();
        Ok(s.entries
            .iter()
            .find(|e| e.source_path.as_deref() == Some(path))
            .cloned())
    }

    fn delete_stale_older_than(&self, days: u64) -> Result<Vec<MemoryEntry>, MemoryStoreError> {
        let mut s = self.state.lock().unwrap();
        // Simple heuristic: parse ISO-8601 dates as strings and compare by year cutoff
        // In tests, we use days=0 to delete all stale or specific date strings.
        // For proper date arithmetic, we use a simple string comparison approach.
        // Format: "2026-01-01T00:00:00Z" — cutoff computed from "now" approximation.
        // In tests, entries have fixed dates, so we accept any stale entry as older.
        let cutoff_year = 2026u64;
        let cutoff_days_threshold = days;

        let (to_delete, keep): (Vec<MemoryEntry>, Vec<MemoryEntry>) =
            s.entries.iter().cloned().partition(|e| {
                if !e.stale {
                    return false;
                }
                // Approximate age check: if days=0, delete all stale
                if cutoff_days_threshold == 0 {
                    return true;
                }
                // Parse year from created_at (ISO-8601 prefix)
                let year_str = &e.created_at[..4];
                let year: u64 = year_str.parse().unwrap_or(cutoff_year);
                // Age in days (rough): (2026 - year) * 365
                let approx_age = (cutoff_year.saturating_sub(year)) * 365;
                approx_age >= cutoff_days_threshold
            });

        s.entries = keep;
        Ok(to_delete)
    }

    fn merge_entries(
        &self,
        keep_id: MemoryId,
        remove_id: MemoryId,
        merged_content: &str,
    ) -> Result<(), MemoryStoreError> {
        let mut s = self.state.lock().unwrap();

        // Check both exist
        let keep_pos = s
            .entries
            .iter()
            .position(|e| e.id == keep_id)
            .ok_or(MemoryStoreError::NotFound(keep_id))?;
        let remove_pos = s
            .entries
            .iter()
            .position(|e| e.id == remove_id)
            .ok_or(MemoryStoreError::NotFound(remove_id))?;

        s.entries[keep_pos].content = merged_content.to_owned();

        // Remove the higher index first to avoid shifting
        if remove_pos > keep_pos {
            s.entries.remove(remove_pos);
        } else {
            s.entries.remove(remove_pos);
            // keep_pos shifted down by 1 — already updated so no issue
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_domain::memory::{MemoryEntry, MemoryId, MemoryTier};

    fn make_entry(tier: MemoryTier, title: &str, content: &str, tags: Vec<&str>) -> MemoryEntry {
        MemoryEntry::new(
            MemoryId(0),
            tier,
            title,
            content,
            tags.into_iter().map(str::to_owned).collect(),
            None,
            None,
            1.0,
            "2026-01-01T00:00:00Z",
            "2026-01-01T00:00:00Z",
            false,
            vec![],
            None,
        )
    }

    // PC-023: InMemoryMemoryStore implements all MemoryStore methods; insert + get round-trip
    #[test]
    fn test_insert_and_get_round_trip() {
        let store = InMemoryMemoryStore::new();
        let entry = make_entry(MemoryTier::Episodic, "Title", "Content", vec!["tag1"]);
        let id = store.insert(&entry).unwrap();
        let fetched = store.get(id).unwrap();
        assert_eq!(fetched.id, id);
        assert_eq!(fetched.title, "Title");
        assert_eq!(fetched.content, "Content");
    }

    #[test]
    fn test_insert_auto_increments_id() {
        let store = InMemoryMemoryStore::new();
        let e1 = make_entry(MemoryTier::Working, "A", "a", vec![]);
        let e2 = make_entry(MemoryTier::Episodic, "B", "b", vec![]);
        let id1 = store.insert(&e1).unwrap();
        let id2 = store.insert(&e2).unwrap();
        assert_ne!(id1, id2);
        assert_eq!(id1.0 + 1, id2.0);
    }

    #[test]
    fn test_get_not_found_returns_error() {
        let store = InMemoryMemoryStore::new();
        let result = store.get(MemoryId(999));
        assert!(matches!(result, Err(MemoryStoreError::NotFound(_))));
    }

    #[test]
    fn test_update_modifies_entry() {
        let store = InMemoryMemoryStore::new();
        let entry = make_entry(MemoryTier::Semantic, "Old Title", "Old", vec![]);
        let id = store.insert(&entry).unwrap();
        let mut updated = store.get(id).unwrap();
        updated.title = "New Title".to_owned();
        store.update(&updated).unwrap();
        let fetched = store.get(id).unwrap();
        assert_eq!(fetched.title, "New Title");
    }

    #[test]
    fn test_delete_removes_entry() {
        let store = InMemoryMemoryStore::new();
        let entry = make_entry(MemoryTier::Working, "T", "C", vec![]);
        let id = store.insert(&entry).unwrap();
        store.delete(id).unwrap();
        assert!(matches!(store.get(id), Err(MemoryStoreError::NotFound(_))));
    }

    #[test]
    fn test_delete_nonexistent_returns_not_found() {
        let store = InMemoryMemoryStore::new();
        let result = store.delete(MemoryId(100));
        assert!(matches!(result, Err(MemoryStoreError::NotFound(_))));
    }

    // PC-024: InMemoryMemoryStore::search_fts does substring match as FTS5 approximation
    #[test]
    fn test_search_fts_matches_title() {
        let store = InMemoryMemoryStore::new();
        let e1 = make_entry(MemoryTier::Episodic, "warn not block", "details", vec![]);
        let e2 = make_entry(MemoryTier::Episodic, "unrelated", "nothing", vec![]);
        store.insert(&e1).unwrap();
        store.insert(&e2).unwrap();
        let results = store.search_fts("warn", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "warn not block");
    }

    #[test]
    fn test_search_fts_matches_content() {
        let store = InMemoryMemoryStore::new();
        let e = make_entry(MemoryTier::Episodic, "title", "FTS5 is great", vec![]);
        store.insert(&e).unwrap();
        let results = store.search_fts("fts5", 10).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_fts_matches_tags() {
        let store = InMemoryMemoryStore::new();
        let e = make_entry(
            MemoryTier::Semantic,
            "title",
            "content",
            vec!["rust", "ddd"],
        );
        store.insert(&e).unwrap();
        let results = store.search_fts("rust", 10).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_fts_case_insensitive() {
        let store = InMemoryMemoryStore::new();
        let e = make_entry(MemoryTier::Episodic, "UPPER TITLE", "lower content", vec![]);
        store.insert(&e).unwrap();
        let results = store.search_fts("upper", 10).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_fts_no_match_returns_empty() {
        let store = InMemoryMemoryStore::new();
        let e = make_entry(MemoryTier::Episodic, "something", "else", vec![]);
        store.insert(&e).unwrap();
        let results = store.search_fts("nonexistent_xyz", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_fts_respects_limit() {
        let store = InMemoryMemoryStore::new();
        for i in 0..5 {
            let e = make_entry(
                MemoryTier::Episodic,
                "common",
                &format!("entry {i}"),
                vec![],
            );
            store.insert(&e).unwrap();
        }
        let results = store.search_fts("common", 3).unwrap();
        assert_eq!(results.len(), 3);
    }

    // PC-025: InMemoryMemoryStore::list_filtered filters by tier and tag
    #[test]
    fn test_list_filtered_by_tier() {
        let store = InMemoryMemoryStore::new();
        let e1 = make_entry(MemoryTier::Semantic, "S1", "c1", vec![]);
        let e2 = make_entry(MemoryTier::Episodic, "E1", "c2", vec![]);
        let e3 = make_entry(MemoryTier::Semantic, "S2", "c3", vec![]);
        store.insert(&e1).unwrap();
        store.insert(&e2).unwrap();
        store.insert(&e3).unwrap();
        let results = store
            .list_filtered(Some(MemoryTier::Semantic), None, None)
            .unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|e| e.tier == MemoryTier::Semantic));
    }

    #[test]
    fn test_list_filtered_by_tag() {
        let store = InMemoryMemoryStore::new();
        let e1 = make_entry(MemoryTier::Episodic, "T1", "c1", vec!["rust"]);
        let e2 = make_entry(MemoryTier::Episodic, "T2", "c2", vec!["python"]);
        store.insert(&e1).unwrap();
        store.insert(&e2).unwrap();
        let results = store.list_filtered(None, Some("rust"), None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "T1");
    }

    #[test]
    fn test_list_filtered_no_filters_returns_all() {
        let store = InMemoryMemoryStore::new();
        store
            .insert(&make_entry(MemoryTier::Working, "A", "a", vec![]))
            .unwrap();
        store
            .insert(&make_entry(MemoryTier::Episodic, "B", "b", vec![]))
            .unwrap();
        let results = store.list_filtered(None, None, None).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_list_filtered_combined_tier_and_tag() {
        let store = InMemoryMemoryStore::new();
        let e1 = make_entry(MemoryTier::Semantic, "S-rust", "c1", vec!["rust"]);
        let e2 = make_entry(MemoryTier::Semantic, "S-python", "c2", vec!["python"]);
        let e3 = make_entry(MemoryTier::Episodic, "E-rust", "c3", vec!["rust"]);
        store.insert(&e1).unwrap();
        store.insert(&e2).unwrap();
        store.insert(&e3).unwrap();
        let results = store
            .list_filtered(Some(MemoryTier::Semantic), Some("rust"), None)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "S-rust");
    }

    #[test]
    fn test_list_recent_returns_sorted() {
        let store = InMemoryMemoryStore::new();
        let mut e1 = make_entry(MemoryTier::Episodic, "old", "c1", vec![]);
        let mut e2 = make_entry(MemoryTier::Episodic, "new", "c2", vec![]);
        e1.updated_at = "2026-01-01T00:00:00Z".to_owned();
        e2.updated_at = "2026-03-01T00:00:00Z".to_owned();
        store.insert(&e1).unwrap();
        store.insert(&e2).unwrap();
        let results = store.list_recent(2).unwrap();
        assert_eq!(results[0].title, "new");
    }

    #[test]
    fn test_count_by_tier() {
        let store = InMemoryMemoryStore::new();
        store
            .insert(&make_entry(MemoryTier::Working, "w1", "c", vec![]))
            .unwrap();
        store
            .insert(&make_entry(MemoryTier::Semantic, "s1", "c", vec![]))
            .unwrap();
        store
            .insert(&make_entry(MemoryTier::Semantic, "s2", "c", vec![]))
            .unwrap();
        let counts = store.count_by_tier().unwrap();
        assert_eq!(counts[&MemoryTier::Working], 1);
        assert_eq!(counts[&MemoryTier::Semantic], 2);
    }

    #[test]
    fn test_get_by_source_path() {
        let store = InMemoryMemoryStore::new();
        let mut e = make_entry(MemoryTier::Episodic, "T", "C", vec![]);
        e.source_path = Some("/path/to/file.md".to_owned());
        store.insert(&e).unwrap();
        let found = store.get_by_source_path("/path/to/file.md").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "T");
    }

    #[test]
    fn test_get_by_source_path_not_found_returns_none() {
        let store = InMemoryMemoryStore::new();
        let result = store.get_by_source_path("/nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_merge_entries() {
        let store = InMemoryMemoryStore::new();
        let e1 = make_entry(MemoryTier::Episodic, "Entry1", "content1", vec![]);
        let e2 = make_entry(MemoryTier::Episodic, "Entry2", "content2", vec![]);
        let id1 = store.insert(&e1).unwrap();
        let id2 = store.insert(&e2).unwrap();
        store.merge_entries(id1, id2, "merged content").unwrap();
        let kept = store.get(id1).unwrap();
        assert_eq!(kept.content, "merged content");
        assert!(matches!(store.get(id2), Err(MemoryStoreError::NotFound(_))));
    }

    #[test]
    fn test_stats_returns_counts() {
        let store = InMemoryMemoryStore::new();
        store
            .insert(&make_entry(MemoryTier::Working, "w", "c", vec![]))
            .unwrap();
        let stats = store.stats().unwrap();
        assert_eq!(stats.counts_by_tier[&MemoryTier::Working], 1);
    }
}
