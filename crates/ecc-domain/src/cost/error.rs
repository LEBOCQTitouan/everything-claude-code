//! Cost domain errors.

use thiserror::Error;

/// Errors that can occur in the cost domain.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum CostError {
    /// The model ID provided is empty or otherwise invalid.
    #[error("invalid model ID: {0}")]
    InvalidModelId(String),
}
