//! In-memory test double for [`BypassStore`].

use std::sync::Mutex;

use ecc_domain::hook_runtime::bypass::{
    BypassDecision, BypassSummary, HookBypassCount, Verdict,
};
use ecc_ports::bypass_store::{BypassStore, BypassStoreError};

/// In-memory test double for [`BypassStore`].
///
/// All writes held in `Mutex<Vec<_>>`. Thread-safe and deterministic.
pub struct InMemoryBypassStore {
    records: Mutex<Vec<BypassDecision>>,
    next_id: Mutex<i64>,
}

impl InMemoryBypassStore {
    pub fn new() -> Self {
        Self {
            records: Mutex::new(Vec::new()),
            next_id: Mutex::new(1),
        }
    }

    pub fn snapshot(&self) -> Vec<BypassDecision> {
        self.records.lock().expect("lock poisoned").clone()
    }
}

impl Default for InMemoryBypassStore {
    fn default() -> Self {
        Self::new()
    }
}

impl BypassStore for InMemoryBypassStore {
    fn record(&self, decision: &BypassDecision) -> Result<i64, BypassStoreError> {
        let mut id_guard = self.next_id.lock().expect("lock poisoned");
        let id = *id_guard;
        *id_guard += 1;

        let stored = BypassDecision::from_stored(
            id,
            decision.hook_id.clone(),
            decision.reason.clone(),
            decision.session_id.clone(),
            decision.verdict,
            decision.timestamp.clone(),
        );
        self.records.lock().expect("lock poisoned").push(stored);
        Ok(id)
    }

    fn query_by_hook(
        &self,
        hook_id: &str,
        limit: usize,
    ) -> Result<Vec<BypassDecision>, BypassStoreError> {
        let guard = self.records.lock().expect("lock poisoned");
        let mut results: Vec<_> = guard
            .iter()
            .filter(|d| d.hook_id == hook_id)
            .cloned()
            .collect();
        results.reverse(); // most recent first
        results.truncate(limit);
        Ok(results)
    }

    fn summary(&self) -> Result<BypassSummary, BypassStoreError> {
        let guard = self.records.lock().expect("lock poisoned");
        let mut counts: std::collections::HashMap<String, (u64, u64)> =
            std::collections::HashMap::new();

        for d in guard.iter() {
            let entry = counts.entry(d.hook_id.clone()).or_insert((0, 0));
            match d.verdict {
                Verdict::Accepted | Verdict::Applied => entry.0 += 1,
                Verdict::Refused => entry.1 += 1,
            }
        }

        let mut per_hook: Vec<HookBypassCount> = counts
            .into_iter()
            .map(|(hook_id, (accepted, refused))| HookBypassCount {
                hook_id,
                accepted,
                refused,
            })
            .collect();
        per_hook.sort_by(|a, b| a.hook_id.cmp(&b.hook_id));

        let total_accepted = per_hook.iter().map(|h| h.accepted).sum();
        let total_refused = per_hook.iter().map(|h| h.refused).sum();

        Ok(BypassSummary {
            per_hook,
            total_accepted,
            total_refused,
        })
    }

    fn prune(&self, _older_than_days: u64) -> Result<u64, BypassStoreError> {
        // InMemory doesn't track timestamps for pruning; just return 0
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_decision(hook_id: &str, verdict: Verdict) -> BypassDecision {
        BypassDecision::new(hook_id, "test reason", "session-1", verdict, "2026-04-06T10:00:00Z")
            .unwrap()
    }

    #[test]
    fn bypass_round_trip() {
        let store = InMemoryBypassStore::new();
        let d = make_decision("hook-a", Verdict::Accepted);
        let id = store.record(&d).unwrap();
        assert!(id > 0);

        let results = store.query_by_hook("hook-a", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].hook_id, "hook-a");
        assert_eq!(results[0].id, Some(1));
    }

    #[test]
    fn check_token_returns_matching_token() {
        use ecc_domain::hook_runtime::bypass::BypassToken;
        use ecc_ports::bypass_store::BypassStore;

        let token = BypassToken::new(
            "pre:write-edit:worktree-guard",
            "session-xyz",
            "2026-04-07T10:00:00Z",
            "test reason",
        )
        .unwrap();

        let store = InMemoryBypassStore::new().with_token(token.clone());

        let result = store.check_token("pre:write-edit:worktree-guard", "session-xyz");
        assert!(result.is_some());
        let found = result.unwrap();
        assert_eq!(found.hook_id, token.hook_id);
        assert_eq!(found.session_id, token.session_id);
    }

    #[test]
    fn bypass_summary_aggregates() {
        let store = InMemoryBypassStore::new();
        store.record(&make_decision("hook-a", Verdict::Accepted)).unwrap();
        store.record(&make_decision("hook-a", Verdict::Refused)).unwrap();
        store.record(&make_decision("hook-b", Verdict::Accepted)).unwrap();

        let summary = store.summary().unwrap();
        assert_eq!(summary.total_accepted, 2);
        assert_eq!(summary.total_refused, 1);
        assert_eq!(summary.per_hook.len(), 2);
    }
}
