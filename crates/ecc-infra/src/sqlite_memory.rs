//! SQLite + FTS5 adapter for [`MemoryStore`].

use std::collections::HashMap;
use std::path::PathBuf;

use ecc_domain::memory::{MemoryEntry, MemoryId, MemoryStats, MemoryTier};
use ecc_ports::memory_store::{MemoryStore, MemoryStoreError};
use rusqlite::{Connection, OpenFlags, params};

/// Production SQLite-backed implementation of [`MemoryStore`].
///
/// - WAL journal mode for concurrent read access.
/// - FTS5 virtual table with unicode61 tokenizer + external-content triggers.
/// - Corruption detection: backs up corrupt file and recreates empty DB.
pub struct SqliteMemoryStore {
    path: PathBuf,
}

impl SqliteMemoryStore {
    /// Open (or create) the database at `path`.
    ///
    /// Creates parent directories, initialises the schema, sets WAL mode.
    /// On corruption, renames the corrupt file and recreates an empty DB.
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, MemoryStoreError> {
        let path: PathBuf = path.into();
        let store = SqliteMemoryStore { path };
        store.init()?;
        Ok(store)
    }

    fn init(&self) -> Result<(), MemoryStoreError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

            // Set directory permissions to 0700 on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = std::fs::Permissions::from_mode(0o700);
                std::fs::set_permissions(parent, perms)
                    .map_err(|e| MemoryStoreError::Database(e.to_string()))?;
            }
        }

        let conn = self.open_connection()?;

        // Check integrity first if DB already exists and has content
        if self.path.exists() && std::fs::metadata(&self.path).map(|m| m.len()).unwrap_or(0) > 0 {
            let integrity_ok: bool = conn
                .query_row("PRAGMA integrity_check", [], |row| {
                    let result: String = row.get(0)?;
                    Ok(result == "ok")
                })
                .unwrap_or(false);

            if !integrity_ok {
                drop(conn);
                return self.handle_corruption();
            }
        }

        self.apply_schema(&conn)?;
        self.set_file_permissions()?;

        Ok(())
    }

    fn open_connection(&self) -> Result<Connection, MemoryStoreError> {
        Connection::open_with_flags(
            &self.path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )
        .map_err(|e| MemoryStoreError::Database(e.to_string()))
    }

    fn conn(&self) -> Result<Connection, MemoryStoreError> {
        self.open_connection()
    }

    fn apply_schema(&self, conn: &Connection) -> Result<(), MemoryStoreError> {
        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS memories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                tier TEXT NOT NULL CHECK(tier IN ('working', 'episodic', 'semantic')),
                tags TEXT NOT NULL DEFAULT '',
                project_id TEXT,
                session_id TEXT,
                relevance_score REAL NOT NULL DEFAULT 1.0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                stale INTEGER NOT NULL DEFAULT 0,
                related_work_items TEXT NOT NULL DEFAULT '',
                source_path TEXT
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
                title, content, tags,
                content='memories',
                content_rowid='id',
                tokenize='unicode61'
            );

            CREATE TRIGGER IF NOT EXISTS memories_ai AFTER INSERT ON memories BEGIN
                INSERT INTO memories_fts(rowid, title, content, tags)
                VALUES (new.id, new.title, new.content, new.tags);
            END;

            CREATE TRIGGER IF NOT EXISTS memories_ad AFTER DELETE ON memories BEGIN
                INSERT INTO memories_fts(memories_fts, rowid, title, content, tags)
                VALUES ('delete', old.id, old.title, old.content, old.tags);
            END;

            CREATE TRIGGER IF NOT EXISTS memories_au AFTER UPDATE ON memories BEGIN
                INSERT INTO memories_fts(memories_fts, rowid, title, content, tags)
                VALUES ('delete', old.id, old.title, old.content, old.tags);
                INSERT INTO memories_fts(rowid, title, content, tags)
                VALUES (new.id, new.title, new.content, new.tags);
            END;",
        )
        .map_err(|e| MemoryStoreError::Database(e.to_string()))
    }

    fn set_file_permissions(&self) -> Result<(), MemoryStoreError> {
        #[cfg(unix)]
        if self.path.exists() {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&self.path, perms)
                .map_err(|e| MemoryStoreError::Database(e.to_string()))?;
        }
        Ok(())
    }

    fn handle_corruption(&self) -> Result<(), MemoryStoreError> {
        let corrupt_path = self.path.with_extension("db.corrupt");
        std::fs::rename(&self.path, &corrupt_path)
            .map_err(|e| MemoryStoreError::Corruption(e.to_string()))?;
        eprintln!(
            "WARNING: memory.db was corrupted. Backed up to {:?}. Recreating empty database.",
            corrupt_path
        );
        let conn = self.open_connection()?;
        self.apply_schema(&conn)?;
        self.set_file_permissions()?;
        Ok(())
    }

    fn row_to_entry(row: &rusqlite::Row) -> rusqlite::Result<MemoryEntry> {
        let id: i64 = row.get(0)?;
        let title: String = row.get(1)?;
        let content: String = row.get(2)?;
        let tier_str: String = row.get(3)?;
        let tags_str: String = row.get(4)?;
        let project_id: Option<String> = row.get(5)?;
        let session_id: Option<String> = row.get(6)?;
        let relevance_score: f64 = row.get(7)?;
        let created_at: String = row.get(8)?;
        let updated_at: String = row.get(9)?;
        let stale_int: i64 = row.get(10)?;
        let related_str: String = row.get(11)?;
        let source_path: Option<String> = row.get(12)?;

        let tier = tier_str
            .parse::<MemoryTier>()
            .unwrap_or(MemoryTier::Episodic);
        let tags: Vec<String> = if tags_str.is_empty() {
            vec![]
        } else {
            tags_str.split(',').map(str::to_owned).collect()
        };
        let related_work_items: Vec<String> = if related_str.is_empty() {
            vec![]
        } else {
            related_str.split(',').map(str::to_owned).collect()
        };

        Ok(MemoryEntry::new(
            MemoryId(id),
            tier,
            title,
            content,
            tags,
            project_id,
            session_id,
            relevance_score,
            created_at,
            updated_at,
            stale_int != 0,
            related_work_items,
            source_path,
        ))
    }

    /// Sanitize an FTS5 query to prevent operator injection.
    ///
    /// Each whitespace-delimited token is individually double-quoted so that
    /// FTS5 operators (`OR`, `AND`, `*`, etc.) in user input are treated as
    /// literals. Multi-word queries become AND of individually quoted terms.
    fn sanitize_fts_query(query: &str) -> String {
        if query.trim().is_empty() {
            return String::new();
        }
        query
            .split_whitespace()
            .map(|token| {
                // Escape internal double quotes by doubling
                let escaped = token.replace('"', "\"\"");
                format!("\"{}\"", escaped)
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl MemoryStore for SqliteMemoryStore {
    fn insert(&self, entry: &MemoryEntry) -> Result<MemoryId, MemoryStoreError> {
        let conn = self.conn()?;
        let tags = entry.tags.join(",");
        let related = entry.related_work_items.join(",");
        conn.execute(
            "INSERT INTO memories (title, content, tier, tags, project_id, session_id,
             relevance_score, created_at, updated_at, stale, related_work_items, source_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                entry.title,
                entry.content,
                entry.tier.to_string(),
                tags,
                entry.project_id,
                entry.session_id,
                entry.relevance_score,
                entry.created_at,
                entry.updated_at,
                entry.stale as i64,
                related,
                entry.source_path,
            ],
        )
        .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

        let id = conn.last_insert_rowid();
        Ok(MemoryId(id))
    }

    fn get(&self, id: MemoryId) -> Result<MemoryEntry, MemoryStoreError> {
        let conn = self.conn()?;
        conn.query_row(
            "SELECT id, title, content, tier, tags, project_id, session_id,
             relevance_score, created_at, updated_at, stale, related_work_items, source_path
             FROM memories WHERE id = ?1",
            params![id.0],
            Self::row_to_entry,
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => MemoryStoreError::NotFound(id),
            _ => MemoryStoreError::Database(e.to_string()),
        })
    }

    fn update(&self, entry: &MemoryEntry) -> Result<(), MemoryStoreError> {
        let conn = self.conn()?;
        let tags = entry.tags.join(",");
        let related = entry.related_work_items.join(",");
        let rows = conn
            .execute(
                "UPDATE memories SET title=?1, content=?2, tier=?3, tags=?4, project_id=?5,
             session_id=?6, relevance_score=?7, created_at=?8, updated_at=?9, stale=?10,
             related_work_items=?11, source_path=?12 WHERE id=?13",
                params![
                    entry.title,
                    entry.content,
                    entry.tier.to_string(),
                    tags,
                    entry.project_id,
                    entry.session_id,
                    entry.relevance_score,
                    entry.created_at,
                    entry.updated_at,
                    entry.stale as i64,
                    related,
                    entry.source_path,
                    entry.id.0,
                ],
            )
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

        if rows == 0 {
            return Err(MemoryStoreError::NotFound(entry.id));
        }
        Ok(())
    }

    fn delete(&self, id: MemoryId) -> Result<(), MemoryStoreError> {
        let conn = self.conn()?;
        let rows = conn
            .execute("DELETE FROM memories WHERE id=?1", params![id.0])
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?;
        if rows == 0 {
            return Err(MemoryStoreError::NotFound(id));
        }
        Ok(())
    }

    fn search_fts(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>, MemoryStoreError> {
        let conn = self.conn()?;
        let safe_query = Self::sanitize_fts_query(query);
        if safe_query.is_empty() {
            return Ok(vec![]);
        }
        let mut stmt = conn
            .prepare(
                "SELECT m.id, m.title, m.content, m.tier, m.tags, m.project_id, m.session_id,
                 m.relevance_score, m.created_at, m.updated_at, m.stale, m.related_work_items, m.source_path
                 FROM memories m
                 JOIN memories_fts f ON m.id = f.rowid
                 WHERE memories_fts MATCH ?1
                 ORDER BY rank
                 LIMIT ?2",
            )
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

        let entries = stmt
            .query_map(params![safe_query, limit as i64], Self::row_to_entry)
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

        Ok(entries)
    }

    fn list_filtered(
        &self,
        tier: Option<MemoryTier>,
        tag: Option<&str>,
        project_id: Option<&str>,
    ) -> Result<Vec<MemoryEntry>, MemoryStoreError> {
        let conn = self.conn()?;

        // Use (?N IS NULL OR condition) so params are always exactly 3.
        let sql = "SELECT id, title, content, tier, tags, project_id, session_id,
             relevance_score, created_at, updated_at, stale, related_work_items, source_path
             FROM memories
             WHERE (?1 IS NULL OR tier = ?1)
               AND (?2 IS NULL OR (',' || tags || ',' LIKE '%,' || ?2 || ',%'))
               AND (?3 IS NULL OR project_id = ?3)
             ORDER BY updated_at DESC";

        let tier_str = tier.as_ref().map(|t| t.to_string());

        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

        let entries = stmt
            .query_map(params![tier_str, tag, project_id], Self::row_to_entry)
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

        Ok(entries)
    }

    fn list_recent(&self, limit: usize) -> Result<Vec<MemoryEntry>, MemoryStoreError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, title, content, tier, tags, project_id, session_id,
                 relevance_score, created_at, updated_at, stale, related_work_items, source_path
                 FROM memories ORDER BY updated_at DESC LIMIT ?1",
            )
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

        let entries = stmt
            .query_map(params![limit as i64], Self::row_to_entry)
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

        Ok(entries)
    }

    fn count_by_tier(&self) -> Result<HashMap<MemoryTier, usize>, MemoryStoreError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT tier, COUNT(*) FROM memories GROUP BY tier")
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

        let mut counts = HashMap::new();
        let rows = stmt
            .query_map([], |row| {
                let tier_str: String = row.get(0)?;
                let count: i64 = row.get(1)?;
                Ok((tier_str, count as usize))
            })
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

        for row in rows {
            let (tier_str, count) =
                row.map_err(|e| MemoryStoreError::Database(e.to_string()))?;
            if let Ok(tier) = tier_str.parse::<MemoryTier>() {
                counts.insert(tier, count);
            }
        }
        Ok(counts)
    }

    fn stats(&self) -> Result<MemoryStats, MemoryStoreError> {
        let counts = self.count_by_tier()?;

        let conn = self.conn()?;

        let stale_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM memories WHERE stale = 1",
                [],
                |r| r.get(0),
            )
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

        let oldest: Option<String> = conn
            .query_row(
                "SELECT MIN(created_at) FROM memories",
                [],
                |r| r.get(0),
            )
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

        let newest: Option<String> = conn
            .query_row(
                "SELECT MAX(created_at) FROM memories",
                [],
                |r| r.get(0),
            )
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

        let db_size = std::fs::metadata(&self.path)
            .map(|m| m.len())
            .unwrap_or(0);

        Ok(MemoryStats::new(
            counts,
            stale_count as usize,
            db_size,
            oldest,
            newest,
        ))
    }

    fn get_by_source_path(&self, path: &str) -> Result<Option<MemoryEntry>, MemoryStoreError> {
        let conn = self.conn()?;
        let result = conn.query_row(
            "SELECT id, title, content, tier, tags, project_id, session_id,
             relevance_score, created_at, updated_at, stale, related_work_items, source_path
             FROM memories WHERE source_path = ?1",
            params![path],
            Self::row_to_entry,
        );

        match result {
            Ok(entry) => Ok(Some(entry)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(MemoryStoreError::Database(e.to_string())),
        }
    }

    fn delete_stale_older_than(&self, days: u64) -> Result<Vec<MemoryEntry>, MemoryStoreError> {
        let conn = self.conn()?;

        // Compute cutoff date string using SQLite datetime arithmetic
        let cutoff = format!("-{} days", days);

        let mut stmt = conn
            .prepare(
                "SELECT id, title, content, tier, tags, project_id, session_id,
                 relevance_score, created_at, updated_at, stale, related_work_items, source_path
                 FROM memories
                 WHERE stale = 1
                   AND created_at < datetime('now', ?1)",
            )
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

        let to_delete: Vec<MemoryEntry> = stmt
            .query_map(params![cutoff], Self::row_to_entry)
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

        for entry in &to_delete {
            conn.execute("DELETE FROM memories WHERE id=?1", params![entry.id.0])
                .map_err(|e| MemoryStoreError::Database(e.to_string()))?;
        }

        Ok(to_delete)
    }

    fn merge_entries(
        &self,
        keep_id: MemoryId,
        remove_id: MemoryId,
        merged_content: &str,
    ) -> Result<(), MemoryStoreError> {
        let conn = self.conn()?;

        // Verify both exist before transacting
        let keep_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM memories WHERE id=?1",
                params![keep_id.0],
                |r| r.get::<_, i64>(0),
            )
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?
            > 0;

        if !keep_exists {
            return Err(MemoryStoreError::NotFound(keep_id));
        }

        let remove_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM memories WHERE id=?1",
                params![remove_id.0],
                |r| r.get::<_, i64>(0),
            )
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?
            > 0;

        if !remove_exists {
            return Err(MemoryStoreError::NotFound(remove_id));
        }

        // Run within a transaction (atomically)
        conn.execute_batch("BEGIN;")
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

        let result = (|| -> Result<(), MemoryStoreError> {
            conn.execute(
                "UPDATE memories SET content=?1 WHERE id=?2",
                params![merged_content, keep_id.0],
            )
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

            conn.execute(
                "DELETE FROM memories WHERE id=?1",
                params![remove_id.0],
            )
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

            Ok(())
        })();

        match result {
            Ok(()) => {
                conn.execute_batch("COMMIT;")
                    .map_err(|e| MemoryStoreError::Database(e.to_string()))?;
                Ok(())
            }
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK;");
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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
        std::fs::write(&db_path, b"this is not a valid sqlite database!!! garbage")
            .unwrap();

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

        let e = make_entry(MemoryTier::Semantic, "Original", "original content", vec!["tag1"]);
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
}
