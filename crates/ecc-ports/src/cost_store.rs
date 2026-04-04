//! Cost and token tracking port traits.
//!
//! Defines the abstract boundary for persisting and querying token usage records.
//! Production adapter: `SqliteCostStore` in `ecc-infra`.
//! Test double: `InMemoryCostStore` in `ecc-test-support`.

use std::time::Duration;

use ecc_domain::cost::{
    calculator::CostSummary,
    record::TokenUsageRecord,
    value_objects::RecordId,
};

/// Query parameters for filtering token usage records.
#[derive(Debug, Default, Clone)]
pub struct CostQuery {
    /// Only return records newer than this duration ago.
    pub since: Option<Duration>,
    /// Filter by model name (substring match).
    pub model: Option<String>,
    /// Filter by agent type string.
    pub agent_type: Option<String>,
    /// Filter by session id.
    pub session_id: Option<String>,
    /// Filter by date range (ISO-8601 strings: start, end).
    pub date_range: Option<(String, String)>,
    /// Maximum number of results.
    pub limit: Option<usize>,
}

/// Output format for cost record export.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CostExportFormat {
    /// Export as JSON array.
    Json,
    /// Export as CSV.
    Csv,
}

/// Errors that can occur during cost store operations.
#[derive(Debug, thiserror::Error)]
pub enum CostStoreError {
    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(String),
    /// A query error occurred (invalid syntax, etc.).
    #[error("query error: {0}")]
    Query(String),
    /// A database error occurred.
    #[error("database error: {0}")]
    Database(String),
}

/// Port trait for persisting and querying token usage records.
///
/// Production adapter: `SqliteCostStore` in `ecc-infra`.
/// Test double: `InMemoryCostStore` in `ecc-test-support`.
pub trait CostStore: Send + Sync {
    /// Append a new record and return its assigned [`RecordId`].
    fn append(&self, record: &TokenUsageRecord) -> Result<RecordId, CostStoreError>;
    /// Query records matching the given filter.
    fn query(&self, query: &CostQuery) -> Result<Vec<TokenUsageRecord>, CostStoreError>;
    /// Return an aggregate [`CostSummary`] for records matching the filter.
    fn summary(&self, query: &CostQuery) -> Result<CostSummary, CostStoreError>;
    /// Delete records older than `older_than` and return the count of removed rows.
    fn prune(&self, older_than: Duration) -> Result<u64, CostStoreError>;
    /// Export records matching `query` in the given `format`.
    fn export(&self, query: &CostQuery, format: CostExportFormat) -> Result<String, CostStoreError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // PC-011: CostStoreError display messages match spec
    #[test]
    fn error_display() {
        let io_err = CostStoreError::Io("disk full".to_owned());
        assert_eq!(io_err.to_string(), "I/O error: disk full");

        let query_err = CostStoreError::Query("bad filter".to_owned());
        assert_eq!(query_err.to_string(), "query error: bad filter");

        let db_err = CostStoreError::Database("connection lost".to_owned());
        assert_eq!(db_err.to_string(), "database error: connection lost");
    }
}
