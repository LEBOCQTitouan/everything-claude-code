//! SQLite-backed implementation of [`BypassStore`].

use std::path::Path;
use std::sync::Mutex;

use ecc_domain::hook_runtime::bypass::{
    BypassDecision, BypassSummary, HookBypassCount, Verdict,
};
use ecc_ports::bypass_store::{BypassStore, BypassStoreError};
use rusqlite::{Connection, params};

/// SQLite-backed bypass audit trail store.
pub struct SqliteBypassStore {
    conn: Mutex<Connection>,
}

impl SqliteBypassStore {
    /// Open (or create) the database at `db_path`.
    pub fn new(db_path: &Path) -> Result<Self, BypassStoreError> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| BypassStoreError::Io(e.to_string()))?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))
                    .map_err(|e| BypassStoreError::Io(e.to_string()))?;
            }
        }
        let conn =
            Connection::open(db_path).map_err(|e| BypassStoreError::Database(e.to_string()))?;
        crate::bypass_schema::ensure_schema(&conn)
            .map_err(|e| BypassStoreError::Database(e.to_string()))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Create an in-memory store for testing.
    pub fn in_memory() -> Result<Self, BypassStoreError> {
        let conn = Connection::open_in_memory()
            .map_err(|e| BypassStoreError::Database(e.to_string()))?;
        crate::bypass_schema::ensure_schema(&conn)
            .map_err(|e| BypassStoreError::Database(e.to_string()))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

fn verdict_to_str(v: Verdict) -> &'static str {
    match v {
        Verdict::Accepted => "accepted",
        Verdict::Refused => "refused",
        Verdict::Applied => "applied",
    }
}

fn str_to_verdict(s: &str) -> Verdict {
    match s {
        "accepted" => Verdict::Accepted,
        "refused" => Verdict::Refused,
        "applied" => Verdict::Applied,
        _ => Verdict::Refused, // safe default
    }
}

fn row_to_decision(row: &rusqlite::Row<'_>) -> rusqlite::Result<BypassDecision> {
    let id: i64 = row.get(0)?;
    let hook_id: String = row.get(1)?;
    let reason: String = row.get(2)?;
    let session_id: String = row.get(3)?;
    let verdict_str: String = row.get(4)?;
    let timestamp: String = row.get(5)?;
    Ok(BypassDecision::from_stored(
        id,
        hook_id,
        reason,
        session_id,
        str_to_verdict(&verdict_str),
        timestamp,
    ))
}

impl BypassStore for SqliteBypassStore {
    fn record(&self, decision: &BypassDecision) -> Result<i64, BypassStoreError> {
        let conn = self.conn.lock().expect("lock poisoned");
        conn.execute(
            "INSERT INTO bypass_decisions (hook_id, reason, session_id, verdict, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                decision.hook_id,
                decision.reason,
                decision.session_id,
                verdict_to_str(decision.verdict),
                decision.timestamp,
            ],
        )
        .map_err(|e| BypassStoreError::Database(e.to_string()))?;
        Ok(conn.last_insert_rowid())
    }

    fn query_by_hook(
        &self,
        hook_id: &str,
        limit: usize,
    ) -> Result<Vec<BypassDecision>, BypassStoreError> {
        let conn = self.conn.lock().expect("lock poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, hook_id, reason, session_id, verdict, timestamp FROM bypass_decisions WHERE hook_id = ?1 ORDER BY id DESC LIMIT ?2",
            )
            .map_err(|e| BypassStoreError::Query(e.to_string()))?;
        let results = stmt
            .query_map(params![hook_id, limit as i64], row_to_decision)
            .map_err(|e| BypassStoreError::Query(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| BypassStoreError::Query(e.to_string()))?;
        Ok(results)
    }

    fn summary(&self) -> Result<BypassSummary, BypassStoreError> {
        let conn = self.conn.lock().expect("lock poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT hook_id, \
                 SUM(CASE WHEN verdict IN ('accepted', 'applied') THEN 1 ELSE 0 END) as accepted, \
                 SUM(CASE WHEN verdict = 'refused' THEN 1 ELSE 0 END) as refused \
                 FROM bypass_decisions GROUP BY hook_id ORDER BY hook_id",
            )
            .map_err(|e| BypassStoreError::Query(e.to_string()))?;
        let per_hook: Vec<HookBypassCount> = stmt
            .query_map([], |row| {
                Ok(HookBypassCount {
                    hook_id: row.get(0)?,
                    accepted: row.get::<_, i64>(1)? as u64,
                    refused: row.get::<_, i64>(2)? as u64,
                })
            })
            .map_err(|e| BypassStoreError::Query(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| BypassStoreError::Query(e.to_string()))?;

        let total_accepted = per_hook.iter().map(|h| h.accepted).sum();
        let total_refused = per_hook.iter().map(|h| h.refused).sum();

        Ok(BypassSummary {
            per_hook,
            total_accepted,
            total_refused,
        })
    }

    fn prune(&self, older_than_days: u64) -> Result<u64, BypassStoreError> {
        let conn = self.conn.lock().expect("lock poisoned");
        let cutoff = format!(
            "datetime('now', '-{} days')",
            older_than_days
        );
        let deleted = conn
            .execute(
                &format!("DELETE FROM bypass_decisions WHERE timestamp < {cutoff}"),
                [],
            )
            .map_err(|e| BypassStoreError::Database(e.to_string()))?;
        Ok(deleted as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_decision(hook_id: &str, verdict: Verdict) -> BypassDecision {
        BypassDecision::new(
            hook_id,
            "test reason",
            "session-1",
            verdict,
            "2026-04-06T10:00:00Z",
        )
        .unwrap()
    }

    #[test]
    fn bypass_record_inserts_row() {
        let store = SqliteBypassStore::in_memory().unwrap();
        let d = make_decision("hook-a", Verdict::Accepted);
        let id = store.record(&d).unwrap();
        assert!(id > 0);
    }

    #[test]
    fn bypass_query_by_hook_filters() {
        let store = SqliteBypassStore::in_memory().unwrap();
        store.record(&make_decision("hook-a", Verdict::Accepted)).unwrap();
        store.record(&make_decision("hook-b", Verdict::Refused)).unwrap();
        store.record(&make_decision("hook-a", Verdict::Applied)).unwrap();

        let results = store.query_by_hook("hook-a", 10).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|d| d.hook_id == "hook-a"));
    }

    #[test]
    fn bypass_summary_counts() {
        let store = SqliteBypassStore::in_memory().unwrap();
        store.record(&make_decision("hook-a", Verdict::Accepted)).unwrap();
        store.record(&make_decision("hook-a", Verdict::Refused)).unwrap();
        store.record(&make_decision("hook-b", Verdict::Accepted)).unwrap();

        let summary = store.summary().unwrap();
        assert_eq!(summary.per_hook.len(), 2);
        assert_eq!(summary.total_accepted, 2);
        assert_eq!(summary.total_refused, 1);

        let hook_a = summary.per_hook.iter().find(|h| h.hook_id == "hook-a").unwrap();
        assert_eq!(hook_a.accepted, 1);
        assert_eq!(hook_a.refused, 1);
    }

    #[test]
    fn bypass_prune_removes_old() {
        let store = SqliteBypassStore::in_memory().unwrap();
        // Insert old record
        let old = BypassDecision::new(
            "hook-a", "old", "session-1", Verdict::Accepted, "2020-01-01T00:00:00Z",
        ).unwrap();
        store.record(&old).unwrap();
        // Insert recent record
        store.record(&make_decision("hook-b", Verdict::Accepted)).unwrap();

        let deleted = store.prune(1).unwrap();
        assert_eq!(deleted, 1);

        // hook-b should still be there
        let remaining = store.query_by_hook("hook-b", 10).unwrap();
        assert_eq!(remaining.len(), 1);
    }
}
