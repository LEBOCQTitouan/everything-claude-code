//! SQLite + FTS5 adapter for [`MemoryStore`].

mod memory_queries;
mod memory_schema;

#[cfg(test)]
mod memory_tests;

use std::collections::HashMap;
use std::path::PathBuf;

use ecc_domain::memory::{MemoryEntry, MemoryId, MemoryStats, MemoryTier};
use ecc_ports::memory_store::{MemoryStore, MemoryStoreError};

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
        memory_schema::init(&path)?;
        Ok(SqliteMemoryStore { path })
    }

    fn conn(&self) -> Result<rusqlite::Connection, MemoryStoreError> {
        memory_schema::open_connection(&self.path)
    }
}

impl MemoryStore for SqliteMemoryStore {
    fn insert(&self, entry: &MemoryEntry) -> Result<MemoryId, MemoryStoreError> {
        use rusqlite::params;
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
        use rusqlite::params;
        let conn = self.conn()?;
        conn.query_row(
            "SELECT id, title, content, tier, tags, project_id, session_id,
             relevance_score, created_at, updated_at, stale, related_work_items, source_path
             FROM memories WHERE id = ?1",
            params![id.0],
            memory_queries::row_to_entry,
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => MemoryStoreError::NotFound(id),
            _ => MemoryStoreError::Database(e.to_string()),
        })
    }

    fn update(&self, entry: &MemoryEntry) -> Result<(), MemoryStoreError> {
        use rusqlite::params;
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
        use rusqlite::params;
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
        memory_queries::search_fts(&self.path, query, limit)
    }

    fn list_filtered(
        &self,
        tier: Option<MemoryTier>,
        tag: Option<&str>,
        project_id: Option<&str>,
    ) -> Result<Vec<MemoryEntry>, MemoryStoreError> {
        memory_queries::list_filtered(&self.path, tier, tag, project_id)
    }

    fn list_recent(&self, limit: usize) -> Result<Vec<MemoryEntry>, MemoryStoreError> {
        memory_queries::list_recent(&self.path, limit)
    }

    fn count_by_tier(&self) -> Result<HashMap<MemoryTier, usize>, MemoryStoreError> {
        memory_queries::count_by_tier(&self.path)
    }

    fn stats(&self) -> Result<MemoryStats, MemoryStoreError> {
        memory_queries::stats(&self.path)
    }

    fn get_by_source_path(&self, path: &str) -> Result<Option<MemoryEntry>, MemoryStoreError> {
        memory_queries::get_by_source_path(&self.path, path)
    }

    fn delete_stale_older_than(&self, days: u64) -> Result<Vec<MemoryEntry>, MemoryStoreError> {
        memory_queries::delete_stale_older_than(&self.path, days)
    }

    fn merge_entries(
        &self,
        keep_id: MemoryId,
        remove_id: MemoryId,
        merged_content: &str,
    ) -> Result<(), MemoryStoreError> {
        memory_queries::merge_entries(&self.path, keep_id, remove_id, merged_content)
    }
}
