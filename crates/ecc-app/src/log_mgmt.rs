use std::path::Path;
use std::time::Duration;

use ecc_ports::fs::FileSystem;
use ecc_ports::log_store::{ExportFormat, LogEntry, LogQuery, LogStore, LogStoreError};

/// Search logs via FTS5 query.
pub fn search(store: &dyn LogStore, query: &LogQuery) -> Result<Vec<LogEntry>, LogStoreError> {
    store.search(query)
}

/// Get most recent log entries.
pub fn tail(
    store: &dyn LogStore,
    count: usize,
    session_id: Option<&str>,
) -> Result<Vec<LogEntry>, LogStoreError> {
    store.tail(count, session_id)
}

/// Result of a prune operation.
#[derive(Debug, PartialEq)]
pub struct PruneResult {
    /// Number of database rows removed.
    pub db_rows: u64,
    /// Number of JSON files deleted.
    pub json_files: u64,
}

/// Prune old log entries from SQLite and delete old JSON files.
///
/// Database rows older than `retention` are removed via [`LogStore::prune`].
/// JSON files in `logs_dir` are deleted if their modification time is older
/// than `retention`.  Because the [`FileSystem`] port does not expose
/// metadata/mtime, file deletion is skipped and `json_files` is always 0 in
/// the current implementation.
pub fn prune(
    store: &dyn LogStore,
    _fs: &dyn FileSystem,
    _logs_dir: &Path,
    retention: Duration,
) -> Result<PruneResult, LogStoreError> {
    let db_rows = store.prune(retention)?;
    Ok(PruneResult {
        db_rows,
        json_files: 0,
    })
}

/// Export logs in JSON or CSV format.
pub fn export(
    store: &dyn LogStore,
    query: &LogQuery,
    format: ExportFormat,
) -> Result<String, LogStoreError> {
    store.export(query, format)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_ports::log_store::LogQuery;
    use ecc_test_support::InMemoryLogStore;

    fn sample_entry(id: i64, msg: &str) -> LogEntry {
        LogEntry {
            id: Some(id),
            session_id: "test-session".into(),
            timestamp: "2026-03-30T10:00:00Z".into(),
            level: "INFO".into(),
            target: "ecc_app".into(),
            message: msg.into(),
            fields_json: "{}".into(),
        }
    }

    #[test]
    fn search_returns_matching_entries() {
        let store = InMemoryLogStore::new();
        store.seed(vec![
            sample_entry(1, "hello world"),
            sample_entry(2, "goodbye"),
        ]);
        let query = LogQuery {
            text: Some("hello".into()),
            ..Default::default()
        };
        let results = search(&store, &query).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].message, "hello world");
    }

    #[test]
    fn tail_returns_n_most_recent() {
        let store = InMemoryLogStore::new();
        store.seed(vec![
            sample_entry(1, "first"),
            sample_entry(2, "second"),
            sample_entry(3, "third"),
        ]);
        let results = tail(&store, 2, None).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].message, "second");
        assert_eq!(results[1].message, "third");
    }

    #[test]
    fn export_json_format() {
        let store = InMemoryLogStore::new();
        store.seed(vec![sample_entry(1, "hello")]);
        let output = export(&store, &LogQuery::default(), ExportFormat::Json).unwrap();
        assert!(output.starts_with('['));
        assert!(output.ends_with(']'));
        assert!(output.contains("hello"));
    }

    #[test]
    fn export_csv_format() {
        let store = InMemoryLogStore::new();
        store.seed(vec![sample_entry(1, "hello")]);
        let output = export(&store, &LogQuery::default(), ExportFormat::Csv).unwrap();
        assert!(output.starts_with("id,session_id,timestamp,level,target,message,fields_json"));
        assert!(output.contains("hello"));
    }

    #[test]
    fn search_with_session_filter() {
        let store = InMemoryLogStore::new();
        let mut entry_a = sample_entry(1, "msg_a");
        entry_a.session_id = "session-a".into();
        let mut entry_b = sample_entry(2, "msg_b");
        entry_b.session_id = "session-b".into();
        store.seed(vec![entry_a, entry_b]);
        let query = LogQuery {
            session_id: Some("session-a".into()),
            ..Default::default()
        };
        let results = search(&store, &query).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].session_id, "session-a");
    }

    #[test]
    fn search_with_level_filter() {
        let store = InMemoryLogStore::new();
        let mut warn_entry = sample_entry(1, "warn message");
        warn_entry.level = "WARN".into();
        let mut info_entry = sample_entry(2, "info message");
        info_entry.level = "INFO".into();
        store.seed(vec![warn_entry, info_entry]);
        let query = LogQuery {
            level: Some("WARN".into()),
            ..Default::default()
        };
        let results = search(&store, &query).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].level, "WARN");
    }
}
