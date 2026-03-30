use std::time::Duration;

/// A single structured log entry.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Optional database row id (None before persistence).
    pub id: Option<i64>,
    /// Session identifier (from CLAUDE_SESSION_ID or fallback).
    pub session_id: String,
    /// ISO-8601 timestamp string.
    pub timestamp: String,
    /// Log level string (e.g., "INFO", "WARN", "ERROR").
    pub level: String,
    /// Tracing target (module path).
    pub target: String,
    /// Human-readable log message.
    pub message: String,
    /// Opaque JSON object with additional structured fields.
    pub fields_json: String,
}

/// Query parameters for searching log entries.
#[derive(Debug, Clone)]
pub struct LogQuery {
    /// Full-text search string (applied to message + fields_json).
    pub text: Option<String>,
    /// Filter by session id.
    pub session_id: Option<String>,
    /// Only return entries newer than this duration ago.
    pub since: Option<Duration>,
    /// Filter by minimum log level string.
    pub level: Option<String>,
    /// Maximum number of results (default 100).
    pub limit: usize,
}

impl Default for LogQuery {
    fn default() -> Self {
        Self {
            text: None,
            session_id: None,
            since: None,
            level: None,
            limit: 100,
        }
    }
}

/// Output format for log export.
#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    /// Export as JSON array.
    Json,
    /// Export as CSV.
    Csv,
}

/// Errors that can occur during log store operations.
#[derive(Debug, thiserror::Error)]
pub enum LogStoreError {
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

/// Port trait for reading structured log entries.
///
/// Production adapter: `SqliteLogStore` in `ecc-infra`.
/// Test double: `InMemoryLogStore` in `ecc-test-support`.
pub trait LogStore: Send + Sync {
    /// Search log entries matching the query.
    fn search(&self, query: &LogQuery) -> Result<Vec<LogEntry>, LogStoreError>;
    /// Return the last `count` entries, optionally filtered by session.
    fn tail(&self, count: usize, session_id: Option<&str>) -> Result<Vec<LogEntry>, LogStoreError>;
    /// Delete entries older than `older_than` and return the count of removed rows.
    fn prune(&self, older_than: Duration) -> Result<u64, LogStoreError>;
    /// Export entries matching `query` in the given `format`.
    fn export(&self, query: &LogQuery, format: ExportFormat) -> Result<String, LogStoreError>;
}
