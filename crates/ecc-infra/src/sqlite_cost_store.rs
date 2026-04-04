//! SQLite-backed implementation of [`CostStore`].

use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use ecc_domain::cost::{
    calculator::{CostCalculator, CostSummary},
    record::TokenUsageRecord,
    value_objects::{ModelId, Money, RecordId, TokenCount},
};
use ecc_ports::cost_store::{CostExportFormat, CostQuery, CostStore, CostStoreError};
use rusqlite::{Connection, params};

/// SQLite-backed implementation of [`CostStore`].
pub struct SqliteCostStore {
    conn: Mutex<Connection>,
}

impl SqliteCostStore {
    /// Open (or create) the SQLite database at `db_path`, setting WAL mode and ensuring the schema.
    pub fn new(db_path: &Path) -> Result<Self, CostStoreError> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| CostStoreError::Io(e.to_string()))?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))
                    .map_err(|e| CostStoreError::Io(e.to_string()))?;
            }
        }
        let conn = Connection::open(db_path)
            .map_err(|e| CostStoreError::Database(e.to_string()))?;
        crate::cost_schema::ensure_schema(&conn)
            .map_err(|e| CostStoreError::Database(e.to_string()))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Create an in-memory store for testing.
    pub fn in_memory() -> Result<Self, CostStoreError> {
        let conn = Connection::open_in_memory()
            .map_err(|e| CostStoreError::Database(e.to_string()))?;
        crate::cost_schema::ensure_schema(&conn)
            .map_err(|e| CostStoreError::Database(e.to_string()))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

/// Map a SQLite row to a [`TokenUsageRecord`].
fn row_to_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<TokenUsageRecord> {
    let model_str: String = row.get(3)?;
    Ok(TokenUsageRecord {
        record_id: {
            let id: i64 = row.get(0)?;
            Some(RecordId(id))
        },
        session_id: row.get(1)?,
        timestamp: row.get(2)?,
        model: ModelId::new(&model_str).unwrap_or_else(|_| ModelId::new("unknown").unwrap()),
        input_tokens: TokenCount::new(row.get::<_, i64>(4)? as u64),
        output_tokens: TokenCount::new(row.get::<_, i64>(5)? as u64),
        thinking_tokens: TokenCount::new(row.get::<_, i64>(6)? as u64),
        estimated_cost: Money::usd(row.get(7)?),
        agent_type: row.get(8)?,
        parent_session_id: row.get(9)?,
    })
}

/// Compute an ISO-8601 cutoff string from a duration ago.
fn cutoff_timestamp(older_than: Duration) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let cutoff_secs = now.as_secs().saturating_sub(older_than.as_secs());
    format_iso8601(cutoff_secs)
}

/// Format Unix seconds as `YYYY-MM-DDTHH:MM:SSZ`.
fn format_iso8601(secs: u64) -> String {
    let mut days = secs / 86400;
    let day_secs = secs % 86400;
    let h = day_secs / 3600;
    let m = (day_secs % 3600) / 60;
    let s = day_secs % 60;

    let mut year = 1970u64;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }
    let months = [
        31u64,
        if is_leap(year) { 29 } else { 28 },
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

fn is_leap(year: u64) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

impl CostStore for SqliteCostStore {
    fn append(&self, record: &TokenUsageRecord) -> Result<RecordId, CostStoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CostStoreError::Database(e.to_string()))?;

        conn.execute(
            "INSERT OR IGNORE INTO token_usage \
             (session_id, timestamp, model, input_tokens, output_tokens, \
              thinking_tokens, estimated_cost_usd, agent_type, parent_session_id) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                record.session_id,
                record.timestamp,
                record.model.as_str(),
                record.input_tokens.value() as i64,
                record.output_tokens.value() as i64,
                record.thinking_tokens.value() as i64,
                record.estimated_cost.value(),
                record.agent_type,
                record.parent_session_id,
            ],
        )
        .map_err(|e| CostStoreError::Database(e.to_string()))?;

        let row_id = conn.last_insert_rowid();
        Ok(RecordId(row_id))
    }

    fn query(&self, query: &CostQuery) -> Result<Vec<TokenUsageRecord>, CostStoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CostStoreError::Database(e.to_string()))?;

        let mut filters: Vec<String> = Vec::new();
        let mut positional_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut param_idx = 1usize;

        if let Some(ref sid) = query.session_id {
            filters.push(format!("session_id = ?{param_idx}"));
            positional_params.push(Box::new(sid.clone()));
            param_idx += 1;
        }

        if let Some(ref model) = query.model {
            filters.push(format!("model LIKE ?{param_idx}"));
            positional_params.push(Box::new(format!("%{model}%")));
            param_idx += 1;
        }

        if let Some(ref agent_type) = query.agent_type {
            filters.push(format!("agent_type = ?{param_idx}"));
            positional_params.push(Box::new(agent_type.clone()));
            param_idx += 1;
        }

        if let Some(since) = query.since {
            let cutoff = cutoff_timestamp(since);
            filters.push(format!("timestamp >= ?{param_idx}"));
            positional_params.push(Box::new(cutoff));
            param_idx += 1;
        }

        if let Some((ref start, ref end)) = query.date_range {
            filters.push(format!("timestamp >= ?{param_idx}"));
            positional_params.push(Box::new(start.clone()));
            param_idx += 1;
            filters.push(format!("timestamp <= ?{param_idx}"));
            positional_params.push(Box::new(end.clone()));
            param_idx += 1;
        }

        let where_clause = if filters.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", filters.join(" AND "))
        };

        let limit_clause = if let Some(limit) = query.limit {
            positional_params.push(Box::new(limit as i64));
            format!(" LIMIT ?{param_idx}")
        } else {
            String::new()
        };

        let sql = format!(
            "SELECT id, session_id, timestamp, model, input_tokens, output_tokens, \
             thinking_tokens, estimated_cost_usd, agent_type, parent_session_id \
             FROM token_usage{where_clause} ORDER BY timestamp ASC{limit_clause}"
        );

        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            positional_params.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| CostStoreError::Query(e.to_string()))?;

        let records = stmt
            .query_map(params_refs.as_slice(), row_to_record)
            .map_err(|e| CostStoreError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CostStoreError::Database(e.to_string()))?;

        Ok(records)
    }

    fn summary(&self, query: &CostQuery) -> Result<CostSummary, CostStoreError> {
        let records = self.query(query)?;
        Ok(CostCalculator::summarize(&records))
    }

    fn prune(&self, older_than: Duration) -> Result<u64, CostStoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CostStoreError::Database(e.to_string()))?;
        let cutoff = cutoff_timestamp(older_than);

        let removed = conn
            .execute(
                "DELETE FROM token_usage WHERE timestamp < ?1",
                params![cutoff],
            )
            .map_err(|e| CostStoreError::Database(e.to_string()))? as u64;

        Ok(removed)
    }

    fn export(&self, query: &CostQuery, format: CostExportFormat) -> Result<String, CostStoreError> {
        let records = self.query(query)?;
        match format {
            CostExportFormat::Json => {
                let items: Vec<String> = records
                    .iter()
                    .map(|r| {
                        format!(
                            r#"{{"record_id":{record_id},"session_id":{session_id},"timestamp":{timestamp},"model":{model},"input_tokens":{input_tokens},"output_tokens":{output_tokens},"thinking_tokens":{thinking_tokens},"estimated_cost":{estimated_cost},"agent_type":{agent_type}}}"#,
                            record_id = r.record_id.map_or("null".to_string(), |v| v.0.to_string()),
                            session_id = json_str(&r.session_id),
                            timestamp = json_str(&r.timestamp),
                            model = json_str(r.model.as_str()),
                            input_tokens = r.input_tokens.value(),
                            output_tokens = r.output_tokens.value(),
                            thinking_tokens = r.thinking_tokens.value(),
                            estimated_cost = r.estimated_cost.value(),
                            agent_type = json_str(&r.agent_type),
                        )
                    })
                    .collect();
                Ok(format!("[{}]", items.join(",")))
            }
            CostExportFormat::Csv => {
                let mut rows = vec!["record_id,session_id,timestamp,model,input_tokens,output_tokens,thinking_tokens,estimated_cost,agent_type".to_string()];
                for r in &records {
                    rows.push(format!(
                        "{},{},{},{},{},{},{},{},{}",
                        r.record_id.map_or("".to_string(), |v| v.0.to_string()),
                        csv_escape(&r.session_id),
                        csv_escape(&r.timestamp),
                        csv_escape(r.model.as_str()),
                        r.input_tokens.value(),
                        r.output_tokens.value(),
                        r.thinking_tokens.value(),
                        r.estimated_cost.value(),
                        csv_escape(&r.agent_type),
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
    use ecc_domain::cost::value_objects::{ModelId, Money, TokenCount};

    fn make_record(
        session_id: &str,
        timestamp: &str,
        model: &str,
        input: u64,
        output: u64,
        cost: f64,
    ) -> TokenUsageRecord {
        TokenUsageRecord {
            record_id: None,
            session_id: session_id.to_string(),
            timestamp: timestamp.to_string(),
            model: ModelId::new(model).unwrap(),
            input_tokens: TokenCount::new(input),
            output_tokens: TokenCount::new(output),
            thinking_tokens: TokenCount::new(0),
            estimated_cost: Money::usd(cost),
            agent_type: "main".to_string(),
            parent_session_id: None,
        }
    }

    // PC-014: SqliteCostStore schema idempotent
    #[test]
    fn schema_creation_idempotent() {
        let store1 = SqliteCostStore::in_memory().expect("first init should succeed");
        // Calling ensure_schema again on the same connection should be idempotent
        {
            let conn = store1.conn.lock().unwrap();
            crate::cost_schema::ensure_schema(&conn).expect("second schema call should succeed");
        }
        // Also verify the table exists and is usable
        let record = make_record("s1", "2026-04-04T10:00:00Z", "claude-sonnet-4-6", 1000, 500, 0.0105);
        let id = store1.append(&record).expect("append should succeed");
        assert!(id.0 > 0);
    }

    // PC-015: Query with date range filter
    #[test]
    fn query_date_range_filter() {
        let store = SqliteCostStore::in_memory().unwrap();

        // Insert records at different timestamps
        store.append(&make_record("s1", "2026-01-01T00:00:00Z", "claude-haiku-4-5", 100, 50, 0.001)).unwrap();
        store.append(&make_record("s1", "2026-03-01T00:00:00Z", "claude-haiku-4-5", 200, 100, 0.002)).unwrap();
        store.append(&make_record("s1", "2026-04-01T00:00:00Z", "claude-haiku-4-5", 300, 150, 0.003)).unwrap();
        store.append(&make_record("s1", "2026-05-01T00:00:00Z", "claude-haiku-4-5", 400, 200, 0.004)).unwrap();

        // Query with date range: only March and April records
        let query = CostQuery {
            date_range: Some(("2026-02-01T00:00:00Z".to_string(), "2026-04-30T00:00:00Z".to_string())),
            ..CostQuery::default()
        };
        let results = store.query(&query).expect("query should succeed");
        assert_eq!(results.len(), 2, "should return only March and April records");
        assert_eq!(results[0].timestamp, "2026-03-01T00:00:00Z");
        assert_eq!(results[1].timestamp, "2026-04-01T00:00:00Z");
    }

    // PC-016: Prune removes old records
    #[test]
    fn prune_removes_old_records() {
        let store = SqliteCostStore::in_memory().unwrap();

        // Insert a record with a very old timestamp
        store.append(&make_record("s1", "2020-01-01T00:00:00Z", "claude-sonnet-4-6", 100, 50, 0.001)).unwrap();
        // Insert a recent record
        use std::time::{SystemTime, UNIX_EPOCH};
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let recent_ts = format_iso8601(now_secs);
        store.append(&make_record("s1", &recent_ts, "claude-sonnet-4-6", 200, 100, 0.002)).unwrap();

        // Prune records older than 30 days
        let removed = store.prune(Duration::from_secs(30 * 86400)).expect("prune should succeed");
        assert_eq!(removed, 1, "should remove only the old record");

        let results = store.query(&CostQuery::default()).expect("query should succeed");
        assert_eq!(results.len(), 1, "only recent record should remain");
    }

    // PC-017: Export JSON format
    #[test]
    fn export_json_format() {
        let store = SqliteCostStore::in_memory().unwrap();
        store.append(&make_record("sess-123", "2026-04-04T10:00:00Z", "claude-sonnet-4-6", 1000, 500, 0.0105)).unwrap();

        let output = store
            .export(&CostQuery::default(), CostExportFormat::Json)
            .expect("export should succeed");

        assert!(output.starts_with('['), "JSON should start with [");
        assert!(output.ends_with(']'), "JSON should end with ]");
        assert!(output.contains("session_id"), "JSON should contain session_id key");
        assert!(output.contains("sess-123"), "JSON should contain the session_id value");
        assert!(output.contains("claude-sonnet-4-6"), "JSON should contain model name");
        assert!(output.contains("estimated_cost"), "JSON should contain estimated_cost key");
    }

    // PC-018: 10 threads x 100 appends WAL (stress test — ignored by default)
    #[test]
    #[ignore]
    fn concurrent_writes_wal() {
        use std::sync::Arc;
        use tempfile::TempDir;

        let dir = TempDir::new().expect("temp dir");
        let db_path = dir.path().join("stress_test.db");

        // Create the store with a real file (WAL mode requires file-backed DB)
        let store = Arc::new(SqliteCostStore::new(&db_path).expect("store creation"));

        let thread_count = 10usize;
        let records_per_thread = 100usize;

        let handles: Vec<_> = (0..thread_count)
            .map(|thread_id| {
                let store = Arc::clone(&store);
                std::thread::spawn(move || {
                    for record_idx in 0..records_per_thread {
                        // Each record has a unique (timestamp, session_id, model) triple
                        let timestamp = format!(
                            "2026-04-04T{:02}:{:02}:{:02}Z",
                            thread_id % 24,
                            record_idx / 60,
                            record_idx % 60
                        );
                        let session_id = format!("thread-{thread_id}-record-{record_idx}");
                        let record = TokenUsageRecord {
                            record_id: None,
                            session_id,
                            timestamp,
                            model: ModelId::new("claude-sonnet-4-6").unwrap(),
                            input_tokens: TokenCount::new(1000),
                            output_tokens: TokenCount::new(500),
                            thinking_tokens: TokenCount::new(0),
                            estimated_cost: Money::usd(0.0105),
                            agent_type: "main".to_string(),
                            parent_session_id: None,
                        };
                        store.append(&record).expect("concurrent append should succeed");
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("thread should not panic");
        }

        // Verify all 1000 records were written
        let conn = store.conn.lock().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM token_usage", [], |r| r.get(0))
            .expect("count query");
        assert_eq!(count, (thread_count * records_per_thread) as i64, "all 1000 records should be present");

        // Verify records deserialize correctly by querying all
        drop(conn);
        let records = store.query(&CostQuery::default()).expect("query all");
        assert_eq!(records.len(), thread_count * records_per_thread);
        for r in &records {
            assert_eq!(r.model.as_str(), "claude-sonnet-4-6");
            assert_eq!(r.input_tokens.value(), 1000);
        }
    }
}
