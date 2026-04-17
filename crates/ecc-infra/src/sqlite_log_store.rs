use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ecc_domain::time::is_leap_year;
use ecc_ports::log_store::{ExportFormat, LogEntry, LogQuery, LogStore, LogStoreError};
use rusqlite::{Connection, params};

/// SQLite-backed implementation of [`LogStore`].
pub struct SqliteLogStore {
    conn: Mutex<Connection>,
}

impl SqliteLogStore {
    /// Open (or create) the SQLite database at `db_path`.
    pub fn new(db_path: &std::path::Path) -> Result<Self, LogStoreError> {
        let conn = Connection::open(db_path).map_err(|e| LogStoreError::Database(e.to_string()))?;
        crate::log_schema::ensure_schema(&conn)
            .map_err(|e| LogStoreError::Database(e.to_string()))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

/// Map a rusqlite row to a [`LogEntry`].
fn row_to_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<LogEntry> {
    Ok(LogEntry {
        id: row.get(0)?,
        session_id: row.get(1)?,
        timestamp: row.get(2)?,
        level: row.get(3)?,
        target: row.get(4)?,
        message: row.get(5)?,
        fields_json: row.get(6)?,
    })
}

/// Compute an ISO-8601 cutoff string from a duration ago.
fn cutoff_timestamp(older_than: Duration) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let cutoff_secs = now.as_secs().saturating_sub(older_than.as_secs());
    format_iso8601(cutoff_secs)
}

/// Format Unix seconds as `YYYY-MM-DDTHH:MM:SSZ` (UTC, approximate).
fn format_iso8601(secs: u64) -> String {
    // Days since Unix epoch
    let mut days = secs / 86400;
    let day_secs = secs % 86400;
    let h = day_secs / 3600;
    let m = (day_secs % 3600) / 60;
    let s = day_secs % 60;

    // Gregorian calendar conversion
    let mut year = 1970u64;
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }
    let months = [
        31u64,
        if is_leap_year(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1u64;
    for &dim in &months {
        if days < dim {
            break;
        }
        days -= dim;
        month += 1;
    }
    let day = days + 1;
    format!("{year:04}-{month:02}-{day:02}T{h:02}:{m:02}:{s:02}Z")
}

impl LogStore for SqliteLogStore {
    fn search(&self, query: &LogQuery) -> Result<Vec<LogEntry>, LogStoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LogStoreError::Database(e.to_string()))?;

        let limit = query.limit as i64;

        // Build base SQL depending on whether FTS is needed
        let mut filters: Vec<String> = Vec::new();
        let mut positional_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut param_idx = 1usize;

        let base_sql = if query.text.is_some() {
            // FTS path
            let sql = format!(
                "SELECT e.id, e.session_id, e.timestamp, e.level, e.target, e.message, e.fields_json \
                 FROM events e JOIN events_fts ON e.id = events_fts.rowid \
                 WHERE events_fts MATCH ?{param_idx}"
            );
            positional_params.push(Box::new(query.text.clone().unwrap()));
            param_idx += 1;
            sql
        } else {
            "SELECT id, session_id, timestamp, level, target, message, fields_json FROM events"
                .to_string()
        };

        if let Some(ref sid) = query.session_id {
            filters.push(format!("session_id = ?{param_idx}"));
            positional_params.push(Box::new(sid.clone()));
            param_idx += 1;
        }

        if let Some(since) = query.since {
            let cutoff = cutoff_timestamp(since);
            filters.push(format!("timestamp >= ?{param_idx}"));
            positional_params.push(Box::new(cutoff));
            param_idx += 1;
        }

        if let Some(ref level) = query.level {
            filters.push(format!("level = ?{param_idx}"));
            positional_params.push(Box::new(level.to_uppercase()));
            param_idx += 1;
        }

        // Combine
        let where_clause = if !filters.is_empty() {
            let prefix = if query.text.is_some() {
                " AND "
            } else {
                " WHERE "
            };
            format!("{prefix}{}", filters.join(" AND "))
        } else {
            String::new()
        };

        let sql = format!("{base_sql}{where_clause} ORDER BY timestamp DESC LIMIT ?{param_idx}");
        positional_params.push(Box::new(limit));

        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            positional_params.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| LogStoreError::Query(e.to_string()))?;

        let entries = stmt
            .query_map(params_refs.as_slice(), row_to_entry)
            .map_err(|e| LogStoreError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| LogStoreError::Database(e.to_string()))?;

        Ok(entries)
    }

    fn tail(&self, count: usize, session_id: Option<&str>) -> Result<Vec<LogEntry>, LogStoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LogStoreError::Database(e.to_string()))?;

        let sql = if session_id.is_some() {
            "SELECT id, session_id, timestamp, level, target, message, fields_json \
             FROM events WHERE session_id = ?1 ORDER BY id DESC LIMIT ?2"
        } else {
            "SELECT id, session_id, timestamp, level, target, message, fields_json \
             FROM events ORDER BY id DESC LIMIT ?1"
        };

        let entries = if let Some(sid) = session_id {
            let mut stmt = conn
                .prepare(sql)
                .map_err(|e| LogStoreError::Query(e.to_string()))?;
            stmt.query_map(params![sid, count as i64], row_to_entry)
                .map_err(|e| LogStoreError::Database(e.to_string()))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| LogStoreError::Database(e.to_string()))?
        } else {
            let mut stmt = conn
                .prepare(sql)
                .map_err(|e| LogStoreError::Query(e.to_string()))?;
            stmt.query_map(params![count as i64], row_to_entry)
                .map_err(|e| LogStoreError::Database(e.to_string()))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| LogStoreError::Database(e.to_string()))?
        };

        Ok(entries)
    }

    fn prune(&self, older_than: Duration) -> Result<u64, LogStoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LogStoreError::Database(e.to_string()))?;
        let cutoff = cutoff_timestamp(older_than);

        let removed = conn
            .execute("DELETE FROM events WHERE timestamp < ?1", params![cutoff])
            .map_err(|e| LogStoreError::Database(e.to_string()))? as u64;

        // Optimize FTS index after bulk delete
        conn.execute("INSERT INTO events_fts(events_fts) VALUES('optimize')", [])
            .map_err(|e| LogStoreError::Database(e.to_string()))?;

        Ok(removed)
    }

    fn export(&self, query: &LogQuery, format: ExportFormat) -> Result<String, LogStoreError> {
        let entries = self.search(query)?;
        match format {
            ExportFormat::Json => {
                let items: Vec<String> = entries
                    .iter()
                    .map(|e| {
                        format!(
                            r#"{{"id":{id},"session_id":{sid},"timestamp":{ts},"level":{lvl},"target":{tgt},"message":{msg},"fields_json":{fj}}}"#,
                            id = e.id.map_or_else(|| "null".to_string(), |v| v.to_string()),
                            sid = json_str(&e.session_id),
                            ts = json_str(&e.timestamp),
                            lvl = json_str(&e.level),
                            tgt = json_str(&e.target),
                            msg = json_str(&e.message),
                            fj = e.fields_json.clone(),
                        )
                    })
                    .collect();
                Ok(format!("[{}]", items.join(",")))
            }
            ExportFormat::Csv => {
                let mut rows =
                    vec!["id,session_id,timestamp,level,target,message,fields_json".to_string()];
                for e in &entries {
                    rows.push(format!(
                        "{},{},{},{},{},{},{}",
                        e.id.map_or_else(String::new, |v| v.to_string()),
                        csv_escape(&e.session_id),
                        csv_escape(&e.timestamp),
                        csv_escape(&e.level),
                        csv_escape(&e.target),
                        csv_escape(&e.message),
                        csv_escape(&e.fields_json),
                    ));
                }
                Ok(rows.join("\n"))
            }
        }
    }
}

fn json_str(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
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
            params![session_id, timestamp, level, "test", message, "{}"],
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
        assert!(results.iter().any(|e| e.message == "third"));
        assert!(results.iter().any(|e| e.message == "second"));
    }

    #[test]
    fn prune_removes_old_events() {
        let (store, _dir) = make_store();
        insert_entry(&store, "s1", "INFO", "old", "2020-01-01T00:00:00Z");
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let recent_ts = format_iso8601(now_secs);
        insert_entry(&store, "s1", "INFO", "new", &recent_ts);

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
        let output = store
            .export(&LogQuery::default(), ExportFormat::Json)
            .unwrap();
        assert!(output.starts_with('['));
        assert!(output.ends_with(']'));
        assert!(output.contains("hello"));
    }

    #[test]
    fn export_csv_format() {
        let (store, _dir) = make_store();
        insert_entry(&store, "s1", "INFO", "hello", "2024-01-01T00:00:00Z");
        let output = store
            .export(&LogQuery::default(), ExportFormat::Csv)
            .unwrap();
        assert!(output.contains("id,session_id,timestamp,level,target,message,fields_json"));
        assert!(output.contains("hello"));
    }
}
