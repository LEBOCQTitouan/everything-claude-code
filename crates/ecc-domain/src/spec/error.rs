//! Domain error type for all spec/design validation failures.

/// Errors that can occur during spec and design artifact validation.
#[derive(Debug, thiserror::Error)]
pub enum SpecError {
    /// No acceptance criteria found in spec file.
    #[error("no acceptance criteria found in spec file")]
    NoAcceptanceCriteria,

    /// No pass conditions table found in design file.
    #[error("no pass conditions table found in design file")]
    NoPassConditions,

    /// Generic parse error with message.
    #[error("parse error: {0}")]
    ParseError(String),

    /// Invalid AC ID format.
    #[error("invalid AC ID: {0}")]
    InvalidAcId(String),

    /// Invalid PC ID format.
    #[error("invalid PC ID: {0}")]
    InvalidPcId(String),

    /// File does not exist.
    #[error("file not found: {0}")]
    FileNotFound(String),

    /// File is not valid UTF-8.
    #[error("file is not valid UTF-8: {0}")]
    NotUtf8(String),
}
