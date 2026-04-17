//! Workflow domain errors.

use std::fmt;

use super::phase::Phase;

/// Errors that can occur during workflow operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowError {
    /// A phase transition that violates the state machine rules was attempted.
    IllegalTransition { from: Phase, to: Phase },
    /// The workflow state is invalid (e.g. corrupted JSON).
    InvalidState(String),
    /// An unknown phase name was provided.
    UnknownPhase(String),
    /// A backward transition was attempted without a non-empty justification.
    MissingJustification,
}

impl fmt::Display for WorkflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IllegalTransition { from, to } => {
                write!(f, "illegal transition from {from} to {to}")
            }
            Self::InvalidState(msg) => write!(f, "invalid workflow state: {msg}"),
            Self::UnknownPhase(name) => write!(f, "unknown phase: {name}"),
            Self::MissingJustification => {
                write!(
                    f,
                    "justification must be non-empty for backward transitions"
                )
            }
        }
    }
}

impl std::error::Error for WorkflowError {}
