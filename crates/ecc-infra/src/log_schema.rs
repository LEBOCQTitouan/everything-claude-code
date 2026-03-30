use rusqlite::Connection;

/// Ensure the SQLite schema exists, creating tables and triggers if absent.
pub fn ensure_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    todo!("not yet implemented")
}

/// Run `PRAGMA integrity_check` and return true if the database is healthy.
pub fn check_integrity(conn: &Connection) -> bool {
    todo!("not yet implemented")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn open_memory() -> Connection {
        Connection::open_in_memory().expect("in-memory db")
    }

    #[test]
    fn ensure_schema_creates_schema_version_table() {
        let conn = open_memory();
        ensure_schema(&conn).unwrap();
        let version: i64 = conn
            .query_row("SELECT version FROM schema_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, 1);
    }

    #[test]
    fn ensure_schema_creates_events_table() {
        let conn = open_memory();
        ensure_schema(&conn).unwrap();
        conn.execute(
            "INSERT INTO events (session_id, timestamp, level, target, message, fields_json) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params!["s1", "2024-01-01T00:00:00Z", "INFO", "test", "hello", "{}"],
        )
        .unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn ensure_schema_is_idempotent() {
        let conn = open_memory();
        ensure_schema(&conn).unwrap();
        ensure_schema(&conn).unwrap();
    }

    #[test]
    fn check_integrity_returns_true_for_healthy_db() {
        let conn = open_memory();
        ensure_schema(&conn).unwrap();
        assert!(check_integrity(&conn));
    }

    #[test]
    fn wal_mode_is_set() {
        let conn = open_memory();
        ensure_schema(&conn).unwrap();
        let _: String = conn
            .query_row("PRAGMA journal_mode", [], |r| r.get(0))
            .unwrap();
    }
}
