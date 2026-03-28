//! Domain error type for all spec/design validation failures.

/// Errors that can occur during spec and design artifact validation.
#[derive(Debug, thiserror::Error)]
pub enum SpecError {
    #[error("no acceptance criteria found in spec file")]
    NoAcceptanceCriteria,

    #[error("no pass conditions table found in design file")]
    NoPassConditions,

    #[error("parse error: {0}")]
    ParseError(String),

    #[error("invalid AC ID: {0}")]
    InvalidAcId(String),

    #[error("invalid PC ID: {0}")]
    InvalidPcId(String),

    #[error("file not found: {0}")]
    FileNotFound(String),

    #[error("file is not valid UTF-8: {0}")]
    NotUtf8(String),
}
