//! Domain errors for the analyze module.

/// Errors that can occur during analysis operations.
#[derive(Debug, thiserror::Error)]
pub enum AnalyzeError {
    /// The `top_n` parameter is 0 or invalid (must be > 0).
    #[error("invalid top value: {0} (must be > 0)")]
    InvalidTopN(usize),
}
