/// Concern tracking and logging.
pub mod concern;
/// Workflow error types.
pub mod error;
/// Workflow state file path resolution.
pub mod path;
/// Workflow phase state machine.
pub mod phase;
/// Phase verification utilities.
pub mod phase_verify;
/// Workflow state staleness detection.
pub mod staleness;
/// Workflow state persistence.
pub mod state;
/// Timestamp utilities.
pub mod timestamp;
/// Phase transition rules and validation.
pub mod transition;

pub use state::TransitionRecord;
pub use transition::{Direction, TransitionPolicy, TransitionResolver, TransitionResult};
