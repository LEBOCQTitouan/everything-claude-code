//! Harness reliability metrics store port.
//!
//! Defines the abstract boundary for persisting and querying harness metric events.
//! Production adapter: `SqliteMetricsStore` in `ecc-infra`.
//! Test double: `InMemoryMetricsStore` in `ecc-test-support`.

use std::time::Duration;

use ecc_domain::metrics::{HarnessMetrics, MetricEvent, MetricEventType, MetricOutcome};

/// Query parameters for filtering metric events.
#[derive(Debug, Default, Clone)]
pub struct MetricsQuery {
    /// Filter by session id.
    pub session_id: Option<String>,
    /// Filter by event type.
    pub event_type: Option<MetricEventType>,
    /// Only return events newer than this duration ago.
    pub since: Option<Duration>,
    /// Filter by outcome.
    pub outcome: Option<MetricOutcome>,
    /// Filter by date range (ISO-8601 strings: start, end).
    pub date_range: Option<(String, String)>,
    /// Maximum number of results.
    pub limit: Option<usize>,
}

/// Output format for metric event export.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MetricsExportFormat {
    /// Export as JSON array.
    Json,
    /// Export as CSV.
    Csv,
}

/// Errors that can occur during metrics store operations.
#[derive(Debug, thiserror::Error)]
pub enum MetricsStoreError {
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

/// Port trait for persisting and querying harness metric events.
///
/// Production adapter: `SqliteMetricsStore` in `ecc-infra`.
/// Test double: `InMemoryMetricsStore` in `ecc-test-support`.
pub trait MetricsStore: Send + Sync {
    /// Record a new metric event and return its assigned row ID.
    fn record(&self, event: &MetricEvent) -> Result<i64, MetricsStoreError>;
    /// Query events matching the given filter.
    fn query(&self, query: &MetricsQuery) -> Result<Vec<MetricEvent>, MetricsStoreError>;
    /// Return aggregated [`HarnessMetrics`] for events matching the filter.
    fn summarize(&self, query: &MetricsQuery) -> Result<HarnessMetrics, MetricsStoreError>;
    /// Delete events older than `older_than` and return the count of removed rows.
    fn prune(&self, older_than: Duration) -> Result<u64, MetricsStoreError>;
    /// Export events matching `query` in the given `format`.
    fn export(
        &self,
        query: &MetricsQuery,
        format: MetricsExportFormat,
    ) -> Result<String, MetricsStoreError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // PC-013: MetricsStoreError display messages
    #[test]
    fn metrics_store_error_display() {
        let io_err = MetricsStoreError::Io("disk full".to_owned());
        assert_eq!(io_err.to_string(), "I/O error: disk full");

        let query_err = MetricsStoreError::Query("bad filter".to_owned());
        assert_eq!(query_err.to_string(), "query error: bad filter");

        let db_err = MetricsStoreError::Database("connection lost".to_owned());
        assert_eq!(db_err.to_string(), "database error: connection lost");
    }

    // PC-014: MetricsQuery defaults
    #[test]
    fn metrics_query_default() {
        let q = MetricsQuery::default();
        assert!(q.session_id.is_none());
        assert!(q.event_type.is_none());
        assert!(q.since.is_none());
        assert!(q.outcome.is_none());
        assert!(q.date_range.is_none());
        assert!(q.limit.is_none());
    }
}
