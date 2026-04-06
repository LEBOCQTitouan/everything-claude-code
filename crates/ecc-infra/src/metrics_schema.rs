//! Schema DDL for the metrics SQLite database.

use rusqlite::Connection;

/// Ensure the metrics schema exists, creating tables and indexes if absent.
///
/// This function is idempotent — safe to call on an already-initialised database.
pub fn ensure_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
        PRAGMA journal_mode=WAL;
        PRAGMA busy_timeout=5000;

        CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER NOT NULL
        );
        INSERT OR IGNORE INTO schema_version (rowid, version) VALUES (1, 1);

        CREATE TABLE IF NOT EXISTS metric_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            event_type TEXT NOT NULL,
            session_id TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            outcome TEXT NOT NULL,
            hook_id TEXT,
            duration_ms INTEGER,
            error_message TEXT,
            from_phase TEXT,
            to_phase TEXT,
            rejection_reason TEXT,
            agent_type TEXT,
            retry_count INTEGER,
            gates_failed TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_metric_events_timestamp ON metric_events(timestamp);
        CREATE INDEX IF NOT EXISTS idx_metric_events_session_id ON metric_events(session_id);
        CREATE INDEX IF NOT EXISTS idx_metric_events_event_type ON metric_events(event_type);
        ",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    // PC-021: schema creation idempotent
    #[test]
    fn metrics_schema_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        ensure_schema(&conn).unwrap(); // second call must not fail

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM metric_events", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    // PC-026: schema_version table has value 1
    #[test]
    fn metrics_schema_version() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();

        let version: i64 = conn
            .query_row("SELECT version FROM schema_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, 1);
    }
}
