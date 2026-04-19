
use tempfile::TempDir;

use ecc_domain::memory::{MemoryEntry, MemoryId, MemoryTier};
use ecc_ports::memory_store::{MemoryStore, MemoryStoreError};
use rusqlite::Connection;

use super::SqliteMemoryStore;

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

fn temp_store(dir: &TempDir) -> SqliteMemoryStore {
    let db_path = dir.path().join("memory.db");
    SqliteMemoryStore::new(db_path).unwrap()
}

// PC-026: SqliteMemoryStore::new creates DB file + FTS5 table if missing (auto-migration)
#[test]
fn test_new_creates_db_and_fts_table() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("subdir").join("memory.db");
    assert!(!db_path.exists());
    let store = SqliteMemoryStore::new(&db_path).unwrap();
    assert!(db_path.exists());

    // Verify FTS5 table exists by inserting and searching
    let e = make_entry(MemoryTier::Episodic, "test title", "content", vec![]);
    store.insert(&e).unwrap();
    let results = store.search_fts("test", 10).unwrap();
    assert!(!results.is_empty());
}

// PC-027: SqliteMemoryStore insert + search_fts returns BM25-ranked results for "warn block"
#[test]
fn test_insert_and_search_fts() {
    let dir = TempDir::new().unwrap();
    let store = temp_store(&dir);

    let e1 = make_entry(
        MemoryTier::Semantic,
        "warn not block",
        "prefer warn over block for lint rules",
        vec!["rust"],
    );
    let e2 = make_entry(
        MemoryTier::Episodic,
        "unrelated topic",
        "something else entirely",
        vec![],
    );
    store.insert(&e1).unwrap();
    store.insert(&e2).unwrap();

    // Both "warn" and "block" appear in e1 but not e2
    let results = store.search_fts("warn", 10).unwrap();
    assert!(!results.is_empty());
    assert!(results.iter().any(|e| e.title == "warn not block"));
}

// PC-028: SqliteMemoryStore list_filtered with type=semantic, tag="rust" returns only matching
#[test]
fn test_list_filtered_semantic_rust() {
    let dir = TempDir::new().unwrap();
    let store = temp_store(&dir);

    let e1 = make_entry(MemoryTier::Semantic, "Rust tip", "content", vec!["rust"]);
    let e2 = make_entry(
        MemoryTier::Episodic,
        "Episodic rust",
        "content",
        vec!["rust"],
    );
    let e3 = make_entry(
        MemoryTier::Semantic,
        "Semantic python",
        "content",
        vec!["python"],
    );
    store.insert(&e1).unwrap();
    store.insert(&e2).unwrap();
    store.insert(&e3).unwrap();

    let results = store
        .list_filtered(Some(MemoryTier::Semantic), Some("rust"), None)
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Rust tip");
}

// PC-029: SqliteMemoryStore enables WAL mode; PRAGMA journal_mode returns "wal"
#[test]
fn test_wal_mode_enabled() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("memory.db");
    let _store = SqliteMemoryStore::new(&db_path).unwrap();

    let conn = Connection::open(&db_path).unwrap();
    let mode: String = conn
        .query_row("PRAGMA journal_mode", [], |r| r.get(0))
        .unwrap();
    assert_eq!(mode, "wal");
}

// PC-030: SqliteMemoryStore detects corruption, backs up as `.corrupt`, recreates empty DB
#[test]
fn test_corruption_detection_and_recovery() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("memory.db");

    // Write garbage to simulate corruption
    std::fs::write(&db_path, b"this is not a valid sqlite database!!! garbage").unwrap();

    // SqliteMemoryStore::new should detect corruption and recover
    let store = SqliteMemoryStore::new(&db_path).unwrap();

    // Corrupt backup should exist
    let corrupt_path = db_path.with_extension("db.corrupt");
    assert!(corrupt_path.exists(), "corrupt backup not found");

    // New DB should be functional
    let e = make_entry(MemoryTier::Working, "after recovery", "content", vec![]);
    let id = store.insert(&e).unwrap();
    let fetched = store.get(id).unwrap();
    assert_eq!(fetched.title, "after recovery");
}

// PC-031: SqliteMemoryStore search with no results returns empty vec (not error)
#[test]
fn test_search_no_results_returns_empty() {
    let dir = TempDir::new().unwrap();
    let store = temp_store(&dir);

    let results = store.search_fts("nonexistent_xyz_abc", 10).unwrap();
    assert!(results.is_empty());
}

// PC-032: SqliteMemoryStore stores and retrieves Unicode content (emoji, CJK) via FTS5
#[test]
fn test_unicode_content_stored_and_retrieved() {
    let dir = TempDir::new().unwrap();
    let store = temp_store(&dir);

    let unicode_content = "Hello 世界 🦀 Привет мир";
    let e = make_entry(
        MemoryTier::Episodic,
        "Unicode entry",
        unicode_content,
        vec!["unicode"],
    );
    let id = store.insert(&e).unwrap();
    let fetched = store.get(id).unwrap();
    assert_eq!(fetched.content, unicode_content);
}

#[test]
fn test_unicode_fts_search() {
    let dir = TempDir::new().unwrap();
    let store = temp_store(&dir);

    let e = make_entry(
        MemoryTier::Semantic,
        "CJK title 世界",
        "content with emoji 🦀",
        vec![],
    );
    store.insert(&e).unwrap();

    // Search with ASCII should still match (FTS5 unicode61)
    let results = store.search_fts("CJK", 10).unwrap();
    assert!(!results.is_empty());
}

// PC-033: SqliteMemoryStore::delete removes from both main table and FTS index
#[test]
fn test_delete_removes_from_both_tables() {
    let dir = TempDir::new().unwrap();
    let store = temp_store(&dir);

    let e = make_entry(MemoryTier::Episodic, "to delete", "content", vec![]);
    let id = store.insert(&e).unwrap();

    store.delete(id).unwrap();

    // Main table: not found
    assert!(matches!(store.get(id), Err(MemoryStoreError::NotFound(_))));

    // FTS index: search should return no results
    let results = store.search_fts("to delete", 10).unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_crud_round_trip() {
    let dir = TempDir::new().unwrap();
    let store = temp_store(&dir);

    let e = make_entry(
        MemoryTier::Semantic,
        "Original",
        "original content",
        vec!["tag1"],
    );
    let id = store.insert(&e).unwrap();

    let mut fetched = store.get(id).unwrap();
    assert_eq!(fetched.title, "Original");

    fetched.title = "Updated".to_owned();
    store.update(&fetched).unwrap();

    let refetched = store.get(id).unwrap();
    assert_eq!(refetched.title, "Updated");

    store.delete(id).unwrap();
    assert!(matches!(store.get(id), Err(MemoryStoreError::NotFound(_))));
}

#[test]
fn test_count_by_tier() {
    let dir = TempDir::new().unwrap();
    let store = temp_store(&dir);

    store
        .insert(&make_entry(MemoryTier::Working, "w", "c", vec![]))
        .unwrap();
    store
        .insert(&make_entry(MemoryTier::Semantic, "s", "c", vec![]))
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
    let dir = TempDir::new().unwrap();
    let store = temp_store(&dir);

    let mut e = make_entry(MemoryTier::Episodic, "T", "C", vec![]);
    e.source_path = Some("/path/to/file.md".to_owned());
    store.insert(&e).unwrap();

    let found = store.get_by_source_path("/path/to/file.md").unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().title, "T");

    let not_found = store.get_by_source_path("/nonexistent").unwrap();
    assert!(not_found.is_none());
}

#[test]
fn test_merge_entries_transaction() {
    let dir = TempDir::new().unwrap();
    let store = temp_store(&dir);

    let e1 = make_entry(MemoryTier::Episodic, "Entry1", "content1", vec![]);
    let e2 = make_entry(MemoryTier::Episodic, "Entry2", "content2", vec![]);
    let id1 = store.insert(&e1).unwrap();
    let id2 = store.insert(&e2).unwrap();

    store.merge_entries(id1, id2, "merged").unwrap();

    let kept = store.get(id1).unwrap();
    assert_eq!(kept.content, "merged");
    assert!(matches!(store.get(id2), Err(MemoryStoreError::NotFound(_))));
}

#[test]
fn test_stats_returns_aggregates() {
    let dir = TempDir::new().unwrap();
    let store = temp_store(&dir);

    store
        .insert(&make_entry(MemoryTier::Working, "w", "c", vec![]))
        .unwrap();
    store
        .insert(&make_entry(MemoryTier::Semantic, "s", "c", vec![]))
        .unwrap();

    let stats = store.stats().unwrap();
    assert_eq!(stats.total_count(), 2);
    assert!(stats.db_size_bytes > 0);
}

#[test]
fn test_list_recent_limit() {
    let dir = TempDir::new().unwrap();
    let store = temp_store(&dir);

    for i in 0..5 {
        let mut e = make_entry(MemoryTier::Episodic, "T", "C", vec![]);
        e.updated_at = format!("2026-01-0{}T00:00:00Z", i + 1);
        store.insert(&e).unwrap();
    }

    let results = store.list_recent(3).unwrap();
    assert_eq!(results.len(), 3);
}

#[test]
fn test_delete_stale_older_than() {
    let dir = TempDir::new().unwrap();
    let store = temp_store(&dir);

    // Insert a stale entry with an old date
    let mut old_stale = make_entry(MemoryTier::Episodic, "Old stale", "c", vec![]);
    old_stale.stale = true;
    old_stale.created_at = "2020-01-01T00:00:00Z".to_owned();
    store.insert(&old_stale).unwrap();

    // Insert a fresh entry
    let fresh = make_entry(MemoryTier::Episodic, "Fresh", "c", vec![]);
    store.insert(&fresh).unwrap();

    let deleted = store.delete_stale_older_than(30).unwrap();
    assert!(!deleted.is_empty());
    assert!(deleted.iter().any(|e| e.title == "Old stale"));
}

// PC-090: DB dir has 0700 perms, file has 0600 perms
#[cfg(unix)]
#[test]
fn test_file_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().unwrap();
    let db_dir = dir.path().join("mem");
    let db_path = db_dir.join("memory.db");
    let _store = SqliteMemoryStore::new(&db_path).unwrap();

    let dir_perms = std::fs::metadata(&db_dir).unwrap().permissions().mode() & 0o777;
    let file_perms = std::fs::metadata(&db_path).unwrap().permissions().mode() & 0o777;

    assert_eq!(dir_perms, 0o700, "directory should have 0700 perms");
    assert_eq!(file_perms, 0o600, "db file should have 0600 perms");
}
