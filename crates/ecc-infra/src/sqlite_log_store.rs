use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ecc_ports::log_store::{ExportFormat, LogEntry, LogQuery, LogStore, LogStoreError};
use rusqlite::Connection;

/// SQLite-backed implementation of [`LogStore`].
pub struct SqliteLogStore {
    conn: Mutex<Connection>,
}

impl SqliteLogStore {
    /// Open (or create) the SQLite database at `db_path`.
    pub fn new(db_path: &std::path::Path) -> Result<Self, LogStoreError> {
        let conn = Connection::open(db_path)
            .map_err(|e| LogStoreError::Database(e.to_string()))?;
        crate::log_schema::ensure_schema(&conn)
            .map_err(|e| LogStoreError::Database(e.to_string()))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

impl LogStore for SqliteLogStore {
    fn search(&self, _query: &LogQuery) -> Result<Vec<LogEntry>, LogStoreError> {
        todo!("not yet implemented")
    }

    fn tail(&self, _count: usize, _session_id: Option<&str>) -> Result<Vec<LogEntry>, LogStoreError> {
        todo!("not yet implemented")
    }

    fn prune(&self, _older_than: Duration) -> Result<u64, LogStoreError> {
        todo!("not yet implemented")
    }

    fn export(&self, _query: &LogQuery, _format: ExportFormat) -> Result<String, LogStoreError> {
        todo!("not yet implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_store() -> (SqliteLogStore, TempDir) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SqliteLogStore::new(&db_path).unwrap();
        (store, dir)
    }

    fn insert_entry(
        store: &SqliteLogStore,
        session_id: &str,
        level: &str,
        message: &str,
        timestamp: &str,
    ) {
        let conn = store.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO events (session_id, timestamp, level, target, message, fields_json) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![session_id, timestamp, level, "test", message, "{}"],
        )
        .unwrap();
    }

    #[test]
    fn search_empty_db_returns_empty() {
        let (store, _dir) = make_store();
        let results = store.search(&LogQuery::default()).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn search_with_fts_match() {
        let (store, _dir) = make_store();
        insert_entry(&store, "s1", "INFO", "hello world", "2024-01-01T00:00:00Z");
        insert_entry(&store, "s1", "INFO", "goodbye moon", "2024-01-01T00:01:00Z");
        let query = LogQuery {
            text: Some("hello".to_string()),
            ..LogQuery::default()
        };
        let results = store.search(&query).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].message, "hello world");
    }

    #[test]
    fn search_with_session_filter() {
        let (store, _dir) = make_store();
        insert_entry(&store, "s1", "INFO", "msg1", "2024-01-01T00:00:00Z");
        insert_entry(&store, "s2", "INFO", "msg2", "2024-01-01T00:01:00Z");
        let query = LogQuery {
            session_id: Some("s1".to_string()),
            ..LogQuery::default()
        };
        let results = store.search(&query).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].session_id, "s1");
    }

    #[test]
    fn tail_returns_latest() {
        let (store, _dir) = make_store();
        insert_entry(&store, "s1", "INFO", "first", "2024-01-01T00:00:00Z");
        insert_entry(&store, "s1", "INFO", "second", "2024-01-01T00:01:00Z");
        insert_entry(&store, "s1", "INFO", "third", "2024-01-01T00:02:00Z");
        let results = store.tail(2, None).unwrap();
        assert_eq!(results.len(), 2);
        // Should be the 2 most recent
        assert!(results.iter().any(|e| e.message == "third"));
        assert!(results.iter().any(|e| e.message == "second"));
    }

    #[test]
    fn prune_removes_old_events() {
        let (store, _dir) = make_store();
        // Insert an old entry
        insert_entry(&store, "s1", "INFO", "old", "2020-01-01T00:00:00Z");
        // Insert a recent entry
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let recent_ts = format_timestamp(now_secs);
        insert_entry(&store, "s1", "INFO", "new", &recent_ts);

        // Prune entries older than 1 day
        let removed = store.prune(Duration::from_secs(86400)).unwrap();
        assert_eq!(removed, 1);

        let results = store.search(&LogQuery::default()).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].message, "new");
    }

    #[test]
    fn export_json_format() {
        let (store, _dir) = make_store();
        insert_entry(&store, "s1", "INFO", "hello", "2024-01-01T00:00:00Z");
        let output = store.export(&LogQuery::default(), ExportFormat::Json).unwrap();
        assert!(output.starts_with('['));
        assert!(output.ends_with(']'));
        assert!(output.contains("hello"));
    }

    #[test]
    fn export_csv_format() {
        let (store, _dir) = make_store();
        insert_entry(&store, "s1", "INFO", "hello", "2024-01-01T00:00:00Z");
        let output = store.export(&LogQuery::default(), ExportFormat::Csv).unwrap();
        assert!(output.contains("id,session_id,timestamp,level,target,message,fields_json"));
        assert!(output.contains("hello"));
    }

    /// Format Unix seconds as ISO-8601 (approximate, for test purposes).
    fn format_timestamp(secs: u64) -> String {
        let days = secs / 86400;
        let rem = secs % 86400;
        let year = 1970 + days / 365;
        let month = (days % 365) / 30 + 1;
        let day = (days % 365) % 30 + 1;
        let h = rem / 3600;
        let m = (rem % 3600) / 60;
        let s = rem % 60;
        format!("{year:04}-{month:02}-{day:02}T{h:02}:{m:02}:{s:02}Z")
    }
}
