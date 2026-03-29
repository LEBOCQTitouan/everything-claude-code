//! Domain error type for all task parsing and validation failures.

/// Errors that can occur during task parsing, validation, and FSM transitions.
#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("parse error at line {line}: {message}")]
    ParseError { line: usize, message: String },

    #[error("invalid status transition: {from} -> {to}")]
    InvalidTransition { from: String, to: String },

    #[error("same-state transition: already in {status}")]
    SameState { status: String },

    #[error("entry not found: {id}")]
    EntryNotFound { id: String },

    #[error("no PC table found in design file")]
    NoPcTable,

    #[error("duplicate PC ID: {id}")]
    DuplicatePcId { id: String },

    #[error("invalid status: {0}")]
    InvalidStatus(String),
}
