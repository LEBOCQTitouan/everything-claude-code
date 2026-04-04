//! Schema DDL for the cost SQLite database.

use rusqlite::Connection;

/// Ensure the cost schema exists, creating tables and indexes if absent.
///
/// This function is idempotent — safe to call on an already-initialised database.
pub fn ensure_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
        PRAGMA journal_mode=WAL;
        CREATE TABLE IF NOT EXISTS token_usage (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            model TEXT NOT NULL,
            input_tokens INTEGER NOT NULL,
            output_tokens INTEGER NOT NULL,
            thinking_tokens INTEGER NOT NULL,
            estimated_cost_usd REAL NOT NULL,
            agent_type TEXT NOT NULL DEFAULT 'main',
            parent_session_id TEXT,
            UNIQUE(timestamp, session_id, model)
        );
        CREATE INDEX IF NOT EXISTS idx_token_usage_timestamp ON token_usage(timestamp);
        CREATE INDEX IF NOT EXISTS idx_token_usage_session_id ON token_usage(session_id);
        CREATE INDEX IF NOT EXISTS idx_token_usage_model ON token_usage(model);
        ",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn ensure_schema_creates_token_usage_table() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM token_usage", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn ensure_schema_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        ensure_schema(&conn).unwrap();
    }
}
