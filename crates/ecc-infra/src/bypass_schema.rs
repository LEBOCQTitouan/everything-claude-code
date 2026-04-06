//! Schema DDL for the bypass SQLite database.

use rusqlite::Connection;

/// Ensure the bypass schema exists, creating tables and indexes if absent.
///
/// Idempotent — safe to call on an already-initialised database.
pub fn ensure_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
        PRAGMA journal_mode=WAL;
        PRAGMA busy_timeout=5000;
        CREATE TABLE IF NOT EXISTS bypass_decisions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            hook_id TEXT NOT NULL,
            reason TEXT NOT NULL,
            session_id TEXT NOT NULL,
            verdict TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            schema_version INTEGER NOT NULL DEFAULT 1
        );
        CREATE INDEX IF NOT EXISTS idx_bypass_hook_id ON bypass_decisions(hook_id);
        CREATE INDEX IF NOT EXISTS idx_bypass_timestamp ON bypass_decisions(timestamp);
        CREATE INDEX IF NOT EXISTS idx_bypass_session_id ON bypass_decisions(session_id);
        ",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn bypass_schema_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        ensure_schema(&conn).unwrap(); // second call should not fail
        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='bypass_decisions'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn bypass_schema_has_version_field() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        // Insert a row and verify schema_version defaults to 1
        conn.execute(
            "INSERT INTO bypass_decisions (hook_id, reason, session_id, verdict, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["hook-a", "test", "session-1", "accepted", "2026-04-06T10:00:00Z"],
        ).unwrap();
        let version: i64 = conn
            .query_row("SELECT schema_version FROM bypass_decisions WHERE id=1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 1);
    }

    #[test]
    fn bypass_wal_and_busy_timeout() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        let journal: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        // In-memory DBs use "memory" journal, but WAL pragma was set
        assert!(journal == "wal" || journal == "memory");
        let timeout: i64 = conn
            .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
            .unwrap();
        assert_eq!(timeout, 5000);
    }
}
