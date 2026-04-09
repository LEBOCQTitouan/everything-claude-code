//! Bypass audit trail storage port.
//!
//! Defines the abstract boundary for persisting and querying bypass decisions.
//! Production adapter: `SqliteBypassStore` in `ecc-infra`.
//! Test double: `InMemoryBypassStore` in `ecc-test-support`.

use ecc_domain::hook_runtime::bypass::{BypassDecision, BypassSummary, BypassToken};

/// Errors that can occur during bypass store operations.
#[derive(Debug, thiserror::Error)]
pub enum BypassStoreError {
    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(String),
    /// A query error occurred.
    #[error("query error: {0}")]
    Query(String),
    /// A database error occurred.
    #[error("database error: {0}")]
    Database(String),
}

/// Port trait for persisting and querying bypass decisions.
///
/// Production adapter: `SqliteBypassStore` in `ecc-infra`.
/// Test double: `InMemoryBypassStore` in `ecc-test-support`.
pub trait BypassStore: Send + Sync {
    /// Record a bypass decision (append-only audit trail).
    fn record(&self, decision: &BypassDecision) -> Result<i64, BypassStoreError>;

    /// Query bypass decisions by hook ID, most recent first.
    fn query_by_hook(
        &self,
        hook_id: &str,
        limit: usize,
    ) -> Result<Vec<BypassDecision>, BypassStoreError>;

    /// Get aggregate bypass summary (per-hook accepted/refused counts).
    fn summary(&self) -> Result<BypassSummary, BypassStoreError>;

    /// Delete records older than the given number of days. Returns count deleted.
    fn prune(&self, older_than_days: u64) -> Result<u64, BypassStoreError>;

    /// Look up a valid bypass token for the given hook and session.
    ///
    /// Returns `Some(token)` if a token file exists, is valid JSON, and matches
    /// both `hook_id` and `session_id`. Returns `None` on any failure or mismatch.
    fn check_token(&self, hook_id: &str, session_id: &str) -> Option<BypassToken>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bypass_store_error_display() {
        let e = BypassStoreError::Io("disk full".into());
        assert_eq!(e.to_string(), "I/O error: disk full");
        let e = BypassStoreError::Query("bad filter".into());
        assert_eq!(e.to_string(), "query error: bad filter");
        let e = BypassStoreError::Database("locked".into());
        assert_eq!(e.to_string(), "database error: locked");
    }

    // Compile-time assertion: BypassStore is Send + Sync
    fn _assert_send_sync<T: Send + Sync>() {}
    #[test]
    fn bypass_store_is_send_sync() {
        // If this compiles, the trait requires Send + Sync
        fn _check<T: BypassStore>() {
            _assert_send_sync::<T>();
        }
    }

    // Verify trait has query_by_hook, summary, prune methods (compile-time)
    #[test]
    fn bypass_store_has_required_methods() {
        // This test verifies the trait has all methods by referencing them.
        // If any method is missing, this won't compile.
        fn _use_all<S: BypassStore>(s: &S) {
            let _ = s.query_by_hook("hook", 10);
            let _ = s.summary();
            let _ = s.prune(90);
        }
    }

    /// Compile-time assertion: BypassStore trait has check_token method.
    #[test]
    fn bypass_store_has_check_token() {
        fn _use_check_token<S: BypassStore>(s: &S) {
            let _: Option<ecc_domain::hook_runtime::bypass::BypassToken> =
                s.check_token("hook-id", "session-id");
        }
    }
}
