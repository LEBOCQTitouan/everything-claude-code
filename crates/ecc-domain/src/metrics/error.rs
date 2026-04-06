//! Metrics domain error types.

/// Errors from metrics domain validation.
#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    /// An invalid outcome was used for the given event type.
    #[error("invalid outcome {outcome} for event type {event_type}")]
    InvalidOutcome {
        /// The event type that was used.
        event_type: String,
        /// The outcome that was invalid for it.
        outcome: String,
    },
}
