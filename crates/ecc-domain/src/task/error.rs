//! Domain error type for all task parsing and validation failures.

/// Errors that can occur during task parsing, validation, and FSM transitions.
#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    /// An error occurred while parsing a task file at a specific line.
    #[error("parse error at line {line}: {message}")]
    ParseError {
        /// The line number where the error occurred.
        line: usize,
        /// Details about the parsing error.
        message: String,
    },

    /// A task status transition is not valid.
    #[error("invalid status transition: {from} -> {to}")]
    InvalidTransition {
        /// The source status.
        from: String,
        /// The target status.
        to: String,
    },

    /// Attempted to transition a task to its current status.
    #[error("same-state transition: already in {status}")]
    SameState {
        /// The current status.
        status: String,
    },

    /// A task entry with the given ID was not found.
    #[error("entry not found: {id}")]
    EntryNotFound {
        /// The entry ID that was not found.
        id: String,
    },

    /// No Pass Condition table was found in the design file.
    #[error("no PC table found in design file")]
    NoPcTable,

    /// A Pass Condition ID is duplicated.
    #[error("duplicate PC ID: {id}")]
    DuplicatePcId {
        /// The duplicated PC ID.
        id: String,
    },

    /// A task status value is invalid.
    #[error("invalid status: {0}")]
    InvalidStatus(String),
}
