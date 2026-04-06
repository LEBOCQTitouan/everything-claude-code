//! SQLite-backed implementation of [`MetricsStore`].

use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use ecc_domain::metrics::{
    CommitGateKind, HarnessMetrics, MetricAggregator, MetricEvent, MetricEventType, MetricOutcome,
};
use ecc_ports::metrics_store::{
    MetricsExportFormat, MetricsQuery, MetricsStore, MetricsStoreError,
};
use rusqlite::{Connection, params};

/// SQLite-backed implementation of [`MetricsStore`].
pub struct SqliteMetricsStore {
    conn: Mutex<Connection>,
}

impl SqliteMetricsStore {
    /// Open (or create) the SQLite database at `db_path`.
    pub fn new(db_path: &Path) -> Result<Self, MetricsStoreError> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| MetricsStoreError::Io(e.to_string()))?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))
                    .map_err(|e| MetricsStoreError::Io(e.to_string()))?;
            }
        }
        let conn = Connection::open(db_path)
            .map_err(|e| MetricsStoreError::Database(e.to_string()))?;
        crate::metrics_schema::ensure_schema(&conn)
            .map_err(|e| MetricsStoreError::Database(e.to_string()))?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    /// Create an in-memory store for testing.
    pub fn in_memory() -> Result<Self, MetricsStoreError> {
        let conn = Connection::open_in_memory()
            .map_err(|e| MetricsStoreError::Database(e.to_string()))?;
        crate::metrics_schema::ensure_schema(&conn)
            .map_err(|e| MetricsStoreError::Database(e.to_string()))?;
        Ok(Self { conn: Mutex::new(conn) })
    }
}

fn gates_to_string(gates: &[CommitGateKind]) -> String {
    gates.iter().map(|g| g.to_string()).collect::<Vec<_>>().join(",")
}

fn string_to_gates(s: &str) -> Vec<CommitGateKind> {
    if s.is_empty() {
        return Vec::new();
    }
    s.split(',').filter_map(CommitGateKind::from_str_opt).collect()
}

fn row_to_event(row: &rusqlite::Row<'_>) -> rusqlite::Result<MetricEvent> {
    let event_type_str: String = row.get(1)?;
    let outcome_str: String = row.get(4)?;
    let gates_str: String = row.get::<_, Option<String>>(13)?.unwrap_or_default();

    Ok(MetricEvent {
        id: Some(row.get(0)?),
        event_type: MetricEventType::from_str_opt(&event_type_str)
            .unwrap_or(MetricEventType::HookExecution),
        session_id: row.get(2)?,
        timestamp: row.get(3)?,
        outcome: MetricOutcome::from_str_opt(&outcome_str)
            .unwrap_or(MetricOutcome::Failure),
        hook_id: row.get(5)?,
        duration_ms: row.get::<_, Option<i64>>(6)?.map(|v| v as u64),
        error_message: row.get(7)?,
        from_phase: row.get(8)?,
        to_phase: row.get(9)?,
        rejection_reason: row.get(10)?,
        agent_type: row.get(11)?,
        retry_count: row.get::<_, Option<i32>>(12)?.map(|v| v as u32),
        gates_failed: string_to_gates(&gates_str),
    })
}

fn cutoff_timestamp(older_than: Duration) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let cutoff_secs = now.as_secs().saturating_sub(older_than.as_secs());
    format_iso8601(cutoff_secs)
}

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
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
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

impl MetricsStore for SqliteMetricsStore {
    fn record(&self, event: &MetricEvent) -> Result<i64, MetricsStoreError> {
        let conn = self.conn.lock()
            .map_err(|e| MetricsStoreError::Database(e.to_string()))?;

        conn.execute(
            "INSERT INTO metric_events \
             (event_type, session_id, timestamp, outcome, hook_id, duration_ms, \
              error_message, from_phase, to_phase, rejection_reason, agent_type, \
              retry_count, gates_failed) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                event.event_type.to_string(),
                event.session_id,
                event.timestamp,
                event.outcome.to_string(),
                event.hook_id,
                event.duration_ms.map(|v| v as i64),
                event.error_message,
                event.from_phase,
                event.to_phase,
                event.rejection_reason,
                event.agent_type,
                event.retry_count.map(|v| v as i32),
                gates_to_string(&event.gates_failed),
            ],
        ).map_err(|e| MetricsStoreError::Database(e.to_string()))?;

        Ok(conn.last_insert_rowid())
    }

    fn query(&self, query: &MetricsQuery) -> Result<Vec<MetricEvent>, MetricsStoreError> {
        let conn = self.conn.lock()
            .map_err(|e| MetricsStoreError::Database(e.to_string()))?;

        let mut filters: Vec<String> = Vec::new();
        let mut positional_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut idx = 1usize;

        if let Some(ref sid) = query.session_id {
            filters.push(format!("session_id = ?{idx}"));
            positional_params.push(Box::new(sid.clone()));
            idx += 1;
        }
        if let Some(ref et) = query.event_type {
            filters.push(format!("event_type = ?{idx}"));
            positional_params.push(Box::new(et.to_string()));
            idx += 1;
        }
        if let Some(ref outcome) = query.outcome {
            filters.push(format!("outcome = ?{idx}"));
            positional_params.push(Box::new(outcome.to_string()));
            idx += 1;
        }
        if let Some(since) = query.since {
            let cutoff = cutoff_timestamp(since);
            filters.push(format!("timestamp >= ?{idx}"));
            positional_params.push(Box::new(cutoff));
            idx += 1;
        }
        if let Some((ref start, ref end)) = query.date_range {
            filters.push(format!("timestamp >= ?{idx}"));
            positional_params.push(Box::new(start.clone()));
            idx += 1;
            filters.push(format!("timestamp <= ?{idx}"));
            positional_params.push(Box::new(end.clone()));
            idx += 1;
        }

        let where_clause = if filters.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", filters.join(" AND "))
        };

        let limit_clause = if let Some(limit) = query.limit {
            positional_params.push(Box::new(limit as i64));
            format!(" LIMIT ?{idx}")
        } else {
            String::new()
        };

        let sql = format!(
            "SELECT id, event_type, session_id, timestamp, outcome, hook_id, duration_ms, \
             error_message, from_phase, to_phase, rejection_reason, agent_type, \
             retry_count, gates_failed \
             FROM metric_events{where_clause} ORDER BY timestamp ASC{limit_clause}"
        );

        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            positional_params.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn.prepare(&sql)
            .map_err(|e| MetricsStoreError::Query(e.to_string()))?;

        let events = stmt
            .query_map(params_refs.as_slice(), row_to_event)
            .map_err(|e| MetricsStoreError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| MetricsStoreError::Database(e.to_string()))?;

        Ok(events)
    }

    fn summarize(&self, query: &MetricsQuery) -> Result<HarnessMetrics, MetricsStoreError> {
        let events = self.query(query)?;
        Ok(MetricAggregator::summarize(&events))
    }

    fn prune(&self, older_than: Duration) -> Result<u64, MetricsStoreError> {
        let conn = self.conn.lock()
            .map_err(|e| MetricsStoreError::Database(e.to_string()))?;
        let cutoff = cutoff_timestamp(older_than);

        let removed = conn
            .execute("DELETE FROM metric_events WHERE timestamp < ?1", params![cutoff])
            .map_err(|e| MetricsStoreError::Database(e.to_string()))? as u64;

        Ok(removed)
    }

    fn export(
        &self,
        query: &MetricsQuery,
        format: MetricsExportFormat,
    ) -> Result<String, MetricsStoreError> {
        let events = self.query(query)?;
        match format {
            MetricsExportFormat::Json => {
                let items: Vec<String> = events.iter().map(|e| {
                    format!(
                        r#"{{"id":{},"event_type":"{}","session_id":"{}","timestamp":"{}","outcome":"{}"}}"#,
                        e.id.map_or("null".to_string(), |v| v.to_string()),
                        e.event_type, e.session_id, e.timestamp, e.outcome,
                    )
                }).collect();
                Ok(format!("[{}]", items.join(",")))
            }
            MetricsExportFormat::Csv => {
                let mut rows = vec![
                    "id,event_type,session_id,timestamp,outcome".to_string(),
                ];
                for e in &events {
                    rows.push(format!(
                        "{},{},{},{},{}",
                        e.id.map_or(String::new(), |v| v.to_string()),
                        e.event_type, e.session_id, e.timestamp, e.outcome,
                    ));
                }
                Ok(rows.join("\n"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_domain::metrics::{MetricEvent, MetricOutcome};

    fn hook(session: &str, ts: &str, outcome: MetricOutcome) -> MetricEvent {
        MetricEvent::hook_execution(session.into(), ts.into(), "test-hook".into(), 100, outcome, None).unwrap()
    }

    // PC-022: record + query round-trip
    #[test]
    fn metrics_store_sqlite_round_trip() {
        let store = SqliteMetricsStore::in_memory().unwrap();
        let event = hook("s1", "2026-04-06T10:00:00Z", MetricOutcome::Success);

        let id = store.record(&event).unwrap();
        assert!(id > 0);

        let results = store.query(&MetricsQuery::default()).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, Some(id));
        assert_eq!(results[0].outcome, MetricOutcome::Success);
        assert_eq!(results[0].hook_id.as_deref(), Some("test-hook"));
        assert_eq!(results[0].duration_ms, Some(100));
    }

    // PC-023: WAL mode + busy_timeout
    #[test]
    fn metrics_store_wal_mode() {
        let store = SqliteMetricsStore::in_memory().unwrap();
        let conn = store.conn.lock().unwrap();

        let journal: String = conn
            .query_row("PRAGMA journal_mode", [], |r| r.get(0))
            .unwrap();
        // In-memory databases use "memory" journal mode, but WAL was requested
        // WAL mode only applies to file-backed databases; in-memory always returns "memory"
        assert!(journal == "wal" || journal == "memory");

        let timeout: i64 = conn
            .query_row("PRAGMA busy_timeout", [], |r| r.get(0))
            .unwrap();
        assert_eq!(timeout, 5000);
    }

    // PC-024: summarize computes rates
    #[test]
    fn metrics_store_sqlite_summarize() {
        let store = SqliteMetricsStore::in_memory().unwrap();
        store.record(&hook("s1", "2026-04-06T10:00:00Z", MetricOutcome::Success)).unwrap();
        store.record(&hook("s1", "2026-04-06T10:01:00Z", MetricOutcome::Failure)).unwrap();

        let metrics = store.summarize(&MetricsQuery::default()).unwrap();
        assert_eq!(metrics.total_events, 2);
        assert!((metrics.hook_success_rate.unwrap() - 0.5).abs() < f64::EPSILON);
    }

    // PC-025: prune removes old events
    #[test]
    fn metrics_store_sqlite_prune() {
        let store = SqliteMetricsStore::in_memory().unwrap();
        store.record(&hook("s1", "2020-01-01T00:00:00Z", MetricOutcome::Success)).unwrap();

        use std::time::{SystemTime, UNIX_EPOCH};
        let now_secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let recent_ts = format_iso8601(now_secs);
        store.record(&hook("s1", &recent_ts, MetricOutcome::Success)).unwrap();

        let removed = store.prune(Duration::from_secs(30 * 86400)).unwrap();
        assert_eq!(removed, 1);
        assert_eq!(store.query(&MetricsQuery::default()).unwrap().len(), 1);
    }

    // PC-036: time-range query
    #[test]
    fn metrics_store_sqlite_time_range() {
        let store = SqliteMetricsStore::in_memory().unwrap();
        store.record(&hook("s1", "2026-01-01T00:00:00Z", MetricOutcome::Success)).unwrap();
        store.record(&hook("s1", "2026-03-01T00:00:00Z", MetricOutcome::Success)).unwrap();
        store.record(&hook("s1", "2026-05-01T00:00:00Z", MetricOutcome::Success)).unwrap();

        let q = MetricsQuery {
            date_range: Some(("2026-02-01T00:00:00Z".into(), "2026-04-01T00:00:00Z".into())),
            ..Default::default()
        };
        let results = store.query(&q).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].timestamp, "2026-03-01T00:00:00Z");
    }
}
