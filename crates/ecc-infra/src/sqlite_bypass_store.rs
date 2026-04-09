//! SQLite-backed implementation of [`BypassStore`].

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use ecc_domain::hook_runtime::bypass::{
    BypassDecision, BypassSummary, BypassToken, HookBypassCount, Verdict,
};
use ecc_ports::bypass_store::{BypassStore, BypassStoreError};
use ecc_ports::clock::Clock;
use rusqlite::{Connection, params};

/// SQLite-backed bypass audit trail store.
pub struct SqliteBypassStore {
    conn: Mutex<Connection>,
    home_dir: Option<PathBuf>,
    clock: Arc<dyn Clock>,
}

impl SqliteBypassStore {
    /// Open (or create) the database at `db_path`.
    pub fn new(db_path: &Path, clock: Arc<dyn Clock>) -> Result<Self, BypassStoreError> {
        Self::new_with_home(db_path, None, clock)
    }

    /// Open (or create) the database at `db_path` with a custom home directory for token lookup.
    pub fn new_with_home(
        db_path: &Path,
        home_dir: Option<PathBuf>,
        clock: Arc<dyn Clock>,
    ) -> Result<Self, BypassStoreError> {
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
            home_dir,
            clock,
        })
    }

    /// Create an in-memory store for testing.
    pub fn in_memory(clock: Arc<dyn Clock>) -> Result<Self, BypassStoreError> {
        let conn =
            Connection::open_in_memory().map_err(|e| BypassStoreError::Database(e.to_string()))?;
        crate::bypass_schema::ensure_schema(&conn)
            .map_err(|e| BypassStoreError::Database(e.to_string()))?;
        Ok(Self {
            conn: Mutex::new(conn),
            home_dir: None,
            clock,
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
        // UTC assumption: epoch arithmetic, no DST correction.
        let now_secs = self.clock.now_epoch_secs();
        let cutoff_secs = now_secs.saturating_sub(older_than_days * 86400);
        let cutoff_dt = ecc_domain::time::datetime_from_epoch(cutoff_secs);
        // Format as ISO 8601 UTC to match stored timestamp format ("YYYY-MM-DDTHH:MM:SSZ").
        let cutoff_str = format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            cutoff_dt.year, cutoff_dt.month, cutoff_dt.day,
            cutoff_dt.hour, cutoff_dt.minute, cutoff_dt.second,
        );
        let conn = self.conn.lock().expect("lock poisoned");

        let deleted = conn
            .execute(
                "DELETE FROM bypass_decisions WHERE timestamp < ?1",
                [&cutoff_str],
            )
            .map_err(|e| BypassStoreError::Database(e.to_string()))?;
        Ok(deleted as u64)
    }

    fn check_token(&self, hook_id: &str, session_id: &str) -> Option<BypassToken> {
        let home = self.home_dir.as_ref()?;
        // Reject path traversal in session_id and hook_id
        if session_id.contains('/')
            || session_id.contains('\\')
            || session_id.contains("..")
            || hook_id.contains('/')
            || hook_id.contains('\\')
            || hook_id.contains("..")
        {
            tracing::warn!(
                session_id,
                hook_id,
                "rejecting bypass token lookup with path traversal chars"
            );
            return None;
        }
        let encoded_hook_id = hook_id.replace(':', "__");
        let token_path = home
            .join(".ecc")
            .join("bypass-tokens")
            .join(session_id)
            .join(format!("{encoded_hook_id}.json"));
        tracing::debug!("check_token: looking for token at {}", token_path.display());
        let contents = std::fs::read_to_string(&token_path).ok()?;
        let token: BypassToken = serde_json::from_str(&contents).ok()?;
        if token.hook_id == hook_id && token.session_id == session_id {
            Some(token)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use ecc_ports::clock::Clock;

    /// Fixed-time clock for deterministic prune tests.
    struct FixedClock(u64);

    impl Clock for FixedClock {
        fn now_iso8601(&self) -> String {
            // Not needed for prune tests
            String::new()
        }

        fn now_epoch_secs(&self) -> u64 {
            self.0
        }
    }

    fn make_decision(hook_id: &str, verdict: Verdict) -> BypassDecision {
        // Use far-future timestamp so prune tests don't become flaky as calendar advances
        BypassDecision::new(
            hook_id,
            "test reason",
            "session-1",
            verdict,
            "2099-01-01T00:00:00Z",
        )
        .unwrap()
    }

    fn system_clock() -> Arc<dyn Clock> {
        Arc::new(crate::system_clock::SystemClock)
    }

    #[test]
    fn bypass_record_inserts_row() {
        let store = SqliteBypassStore::in_memory(system_clock()).unwrap();
        let d = make_decision("hook-a", Verdict::Accepted);
        let id = store.record(&d).unwrap();
        assert!(id > 0);
    }

    #[test]
    fn bypass_query_by_hook_filters() {
        let store = SqliteBypassStore::in_memory(system_clock()).unwrap();
        store.record(&make_decision("hook-a", Verdict::Accepted)).unwrap();
        store.record(&make_decision("hook-b", Verdict::Refused)).unwrap();
        store.record(&make_decision("hook-a", Verdict::Applied)).unwrap();


        let results = store.query_by_hook("hook-a", 10).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|d| d.hook_id == "hook-a"));
    }

    #[test]
    fn bypass_summary_counts() {
        let store = SqliteBypassStore::in_memory(system_clock()).unwrap();
        store.record(&make_decision("hook-a", Verdict::Accepted)).unwrap();
        store.record(&make_decision("hook-a", Verdict::Refused)).unwrap();
        store.record(&make_decision("hook-b", Verdict::Accepted)).unwrap();


        let summary = store.summary().unwrap();
        assert_eq!(summary.per_hook.len(), 2);
        assert_eq!(summary.total_accepted, 2);
        assert_eq!(summary.total_refused, 1);

        let hook_a = summary
            .per_hook
            .iter()
            .find(|h| h.hook_id == "hook-a")
            .unwrap();
        assert_eq!(hook_a.accepted, 1);
        assert_eq!(hook_a.refused, 1);
    }

    #[test]
    fn sqlite_bypass_store_check_token_found() {
        let tmp = tempfile::tempdir().unwrap();
        let home_dir = tmp.path().to_path_buf();
        let session_id = "session-abc";
        let hook_id = "pre:write-edit:worktree-guard";
        let encoded_hook_id = hook_id.replace(':', "__");

        // Write a valid token JSON file at the expected path
        let token_dir = home_dir.join(".ecc").join("bypass-tokens").join(session_id);
        std::fs::create_dir_all(&token_dir).unwrap();
        let token = ecc_domain::hook_runtime::bypass::BypassToken::new(
            hook_id,
            session_id,
            "2026-04-07T10:00:00Z",
            "test bypass",
        )
        .unwrap();
        let json = serde_json::to_string(&token).unwrap();
        std::fs::write(token_dir.join(format!("{encoded_hook_id}.json")), json).unwrap();

        // Create store with home_dir, check_token should find it
        let db_path = home_dir.join("bypass.db");
        let store = SqliteBypassStore::new_with_home(&db_path, Some(home_dir)).unwrap();
        let result = ecc_ports::bypass_store::BypassStore::check_token(&store, hook_id, session_id);
        assert!(result.is_some());
        let found = result.unwrap();
        assert_eq!(found.hook_id, hook_id);
        assert_eq!(found.session_id, session_id);
    }

    #[test]
    fn sqlite_bypass_store_check_token_malformed() {
        let tmp = tempfile::tempdir().unwrap();
        let home_dir = tmp.path().to_path_buf();
        let session_id = "session-abc";
        let hook_id = "pre:write-edit:worktree-guard";
        let encoded_hook_id = hook_id.replace(':', "__");

        // Write malformed JSON to the token file
        let token_dir = home_dir.join(".ecc").join("bypass-tokens").join(session_id);
        std::fs::create_dir_all(&token_dir).unwrap();
        std::fs::write(
            token_dir.join(format!("{encoded_hook_id}.json")),
            "not valid json",
        )
        .unwrap();

        let db_path = home_dir.join("bypass.db");
        let store = SqliteBypassStore::new_with_home(&db_path, Some(home_dir)).unwrap();
        let result = ecc_ports::bypass_store::BypassStore::check_token(&store, hook_id, session_id);
        assert!(result.is_none());
    }

    #[test]
    fn sqlite_bypass_store_check_token_mismatched() {
        let tmp = tempfile::tempdir().unwrap();
        let home_dir = tmp.path().to_path_buf();
        let session_id = "session-abc";
        let hook_id = "pre:write-edit:worktree-guard";
        let encoded_hook_id = hook_id.replace(':', "__");

        // Write a token JSON with a different hook_id
        let token_dir = home_dir.join(".ecc").join("bypass-tokens").join(session_id);
        std::fs::create_dir_all(&token_dir).unwrap();
        let other_hook_id = "pre:other:hook";
        let token = ecc_domain::hook_runtime::bypass::BypassToken::new(
            other_hook_id,
            session_id,
            "2026-04-07T10:00:00Z",
            "test bypass",
        )
        .unwrap();
        let json = serde_json::to_string(&token).unwrap();
        std::fs::write(token_dir.join(format!("{encoded_hook_id}.json")), json).unwrap();

        let db_path = home_dir.join("bypass.db");
        let store = SqliteBypassStore::new_with_home(&db_path, Some(home_dir)).unwrap();
        let result = ecc_ports::bypass_store::BypassStore::check_token(&store, hook_id, session_id);
        assert!(result.is_none());
    }

    #[test]
    fn sqlite_bypass_store_check_token_no_home() {
        // Store created without home_dir — check_token must return None
        let store = SqliteBypassStore::in_memory().unwrap();
        let result = ecc_ports::bypass_store::BypassStore::check_token(
            &store,
            "pre:write-edit:worktree-guard",
            "session-abc",
        );
        assert!(result.is_none());
    }

    #[test]
    fn bypass_prune_removes_old() {
        // Use a fixed far-future clock so the old record (2020) is always pruned
        // and the recent record (2026-04-06T10:00:00Z) always survives (within 1 day of now).
        // "now" = 2026-04-06T12:00:00Z = 1743940800 secs epoch
        let store = SqliteBypassStore::in_memory(Arc::new(FixedClock(1_743_940_800))).unwrap();
        // Insert old record
        let old = BypassDecision::new(
            "hook-a",
            "old",
            "session-1",
            Verdict::Accepted,
            "2020-01-01T00:00:00Z",
        )
        .unwrap();
        store.record(&old).unwrap();
        // Insert recent record
        store
            .record(&make_decision("hook-b", Verdict::Accepted))
            .unwrap();

        let deleted = store.prune(1).unwrap();
        assert_eq!(deleted, 1);

        // hook-b should still be there
        let remaining = store.query_by_hook("hook-b", 10).unwrap();
        assert_eq!(remaining.len(), 1);
    }

    #[test]
    fn bypass_prune_with_clock() {
        // Fixed "now": 2024-04-06T12:00:00Z = epoch 1712404800
        // cutoff for 1 day back: 1712404800 - 86400 = 1712318400 = 2024-04-05T12:00:00Z
        let clock: Arc<dyn Clock> = Arc::new(FixedClock(1_712_404_800));
        let store = SqliteBypassStore::in_memory(clock).unwrap();

        // Old record: 2020-01-01T00:00:00Z — well before cutoff
        let old = BypassDecision::new(
            "hook-old", "old reason", "session-1", Verdict::Accepted, "2020-01-01T00:00:00Z",
        ).unwrap();
        store.record(&old).unwrap();

        // Recent record: 2024-04-06T10:00:00Z — 2 hours before "now", after cutoff
        let recent = BypassDecision::new(
            "hook-recent", "recent reason", "session-2", Verdict::Accepted, "2024-04-06T10:00:00Z",
        ).unwrap();
        store.record(&recent).unwrap();

        let deleted = store.prune(1).unwrap();
        assert_eq!(deleted, 1, "expected old record deleted");

        // recent record should survive
        let remaining = store.query_by_hook("hook-recent", 10).unwrap();
        assert_eq!(remaining.len(), 1, "recent record should survive");

        // old record should be gone
        let gone = store.query_by_hook("hook-old", 10).unwrap();
        assert_eq!(gone.len(), 0, "old record should be deleted");
    }
}
