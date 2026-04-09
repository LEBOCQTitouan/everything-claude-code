//! CRUD and query implementations for [`SqliteMemoryStore`].

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ecc_domain::memory::{MemoryEntry, MemoryId, MemoryStats, MemoryTier};
use ecc_ports::memory_store::MemoryStoreError;
use rusqlite::{Connection, params};

use super::memory_schema::open_connection;

/// Map a SQLite row to a [`MemoryEntry`].
pub(super) fn row_to_entry(row: &rusqlite::Row) -> rusqlite::Result<MemoryEntry> {
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
pub(super) fn sanitize_fts_query(query: &str) -> String {
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

/// Full-text search using the FTS5 index.
pub(super) fn search_fts(
    path: &Path,
    query: &str,
    limit: usize,
) -> Result<Vec<MemoryEntry>, MemoryStoreError> {
    let conn = open_connection(path)?;
    let safe_query = sanitize_fts_query(query);
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
        .query_map(params![safe_query, limit as i64], row_to_entry)
        .map_err(|e| MemoryStoreError::Database(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

    Ok(entries)
}

/// List entries filtered by tier, tag, and/or project_id.
pub(super) fn list_filtered(
    path: &Path,
    tier: Option<MemoryTier>,
    tag: Option<&str>,
    project_id: Option<&str>,
) -> Result<Vec<MemoryEntry>, MemoryStoreError> {
    let conn = open_connection(path)?;

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
        .query_map(params![tier_str, tag, project_id], row_to_entry)
        .map_err(|e| MemoryStoreError::Database(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

    Ok(entries)
}

/// List the most recently updated entries up to `limit`.
pub(super) fn list_recent(
    path: &Path,
    limit: usize,
) -> Result<Vec<MemoryEntry>, MemoryStoreError> {
    let conn = open_connection(path)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, title, content, tier, tags, project_id, session_id,
             relevance_score, created_at, updated_at, stale, related_work_items, source_path
             FROM memories ORDER BY updated_at DESC LIMIT ?1",
        )
        .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

    let entries = stmt
        .query_map(params![limit as i64], row_to_entry)
        .map_err(|e| MemoryStoreError::Database(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

    Ok(entries)
}

/// Count entries grouped by tier.
pub(super) fn count_by_tier(
    path: &Path,
) -> Result<HashMap<MemoryTier, usize>, MemoryStoreError> {
    let conn = open_connection(path)?;
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
        let (tier_str, count) = row.map_err(|e| MemoryStoreError::Database(e.to_string()))?;
        if let Ok(tier) = tier_str.parse::<MemoryTier>() {
            counts.insert(tier, count);
        }
    }
    Ok(counts)
}

/// Aggregate statistics about the memory store.
pub(super) fn stats(path: &PathBuf) -> Result<MemoryStats, MemoryStoreError> {
    let counts = count_by_tier(path)?;

    let conn = open_connection(path)?;

    let stale_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM memories WHERE stale = 1", [], |r| {
            r.get(0)
        })
        .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

    let oldest: Option<String> = conn
        .query_row("SELECT MIN(created_at) FROM memories", [], |r| r.get(0))
        .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

    let newest: Option<String> = conn
        .query_row("SELECT MAX(created_at) FROM memories", [], |r| r.get(0))
        .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

    let db_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    Ok(MemoryStats::new(
        counts,
        stale_count as usize,
        db_size,
        oldest,
        newest,
    ))
}

/// Find an entry by its source path.
pub(super) fn get_by_source_path(
    path: &Path,
    source_path: &str,
) -> Result<Option<MemoryEntry>, MemoryStoreError> {
    let conn = open_connection(path)?;
    let result = conn.query_row(
        "SELECT id, title, content, tier, tags, project_id, session_id,
         relevance_score, created_at, updated_at, stale, related_work_items, source_path
         FROM memories WHERE source_path = ?1",
        params![source_path],
        row_to_entry,
    );

    match result {
        Ok(entry) => Ok(Some(entry)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(MemoryStoreError::Database(e.to_string())),
    }
}

/// Delete all stale entries older than `days` days and return them.
pub(super) fn delete_stale_older_than(
    path: &Path,
    days: u64,
) -> Result<Vec<MemoryEntry>, MemoryStoreError> {
    let conn = open_connection(path)?;

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
        .query_map(params![cutoff], row_to_entry)
        .map_err(|e| MemoryStoreError::Database(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

    for entry in &to_delete {
        conn.execute("DELETE FROM memories WHERE id=?1", params![entry.id.0])
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?;
    }

    Ok(to_delete)
}

/// Merge two entries: update `keep_id` with `merged_content`, delete `remove_id`.
pub(super) fn merge_entries(
    path: &Path,
    keep_id: MemoryId,
    remove_id: MemoryId,
    merged_content: &str,
) -> Result<(), MemoryStoreError> {
    let conn = open_connection(path)?;

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

    run_merge_transaction(&conn, keep_id, remove_id, merged_content)
}

fn run_merge_transaction(
    conn: &Connection,
    keep_id: MemoryId,
    remove_id: MemoryId,
    merged_content: &str,
) -> Result<(), MemoryStoreError> {
    conn.execute_batch("BEGIN;")
        .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

    let result = (|| -> Result<(), MemoryStoreError> {
        conn.execute(
            "UPDATE memories SET content=?1 WHERE id=?2",
            params![merged_content, keep_id.0],
        )
        .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

        conn.execute("DELETE FROM memories WHERE id=?1", params![remove_id.0])
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
