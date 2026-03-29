//! TaskStatus enum and FSM transition validation.

use std::fmt;

use serde::Serialize;

use crate::task::error::TaskError;

/// The lifecycle status of a task (PC or Post-TDD entry).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Red,
    Green,
    Done,
    Failed,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Red => write!(f, "red"),
            Self::Green => write!(f, "green"),
            Self::Done => write!(f, "done"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

impl TaskStatus {
    /// Parse a lowercase status string into a `TaskStatus`.
    ///
    /// # Errors
    ///
    /// Returns [`TaskError::InvalidStatus`] if the string is not a known status.
    pub fn from_str(s: &str) -> Result<Self, TaskError> {
        match s.trim() {
            "pending" => Ok(Self::Pending),
            "red" => Ok(Self::Red),
            "green" => Ok(Self::Green),
            "done" => Ok(Self::Done),
            "failed" => Ok(Self::Failed),
            other => Err(TaskError::InvalidStatus(other.to_owned())),
        }
    }

    /// Validate a status transition according to the FSM rules.
    ///
    /// `is_post_tdd` must be `true` for Post-TDD entries (e.g. "E2E tests"),
    /// which are allowed to skip directly from `Pending` to `Done`.
    ///
    /// # Errors
    ///
    /// - [`TaskError::SameState`] when `from == to`
    /// - [`TaskError::InvalidTransition`] when the transition is not allowed
    pub fn can_transition(from: Self, to: Self, _is_post_tdd: bool) -> Result<(), TaskError> {
        // Same-state transitions are always rejected (AC-004.8)
        if from == to {
            return Err(TaskError::SameState {
                status: from.to_string(),
            });
        }

        // Stub: always reject — will be replaced in GREEN phase
        Err(TaskError::InvalidTransition {
            from: from.to_string(),
            to: to.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // PC-001: valid transitions

    #[test]
    fn allows_pending_to_red() {
        assert!(
            TaskStatus::can_transition(TaskStatus::Pending, TaskStatus::Red, false).is_ok(),
            "pending -> red should be allowed for TDD entries"
        );
    }

    #[test]
    fn allows_red_to_green() {
        assert!(
            TaskStatus::can_transition(TaskStatus::Red, TaskStatus::Green, false).is_ok(),
            "red -> green should be allowed"
        );
    }

    #[test]
    fn allows_green_to_done() {
        assert!(
            TaskStatus::can_transition(TaskStatus::Green, TaskStatus::Done, false).is_ok(),
            "green -> done should be allowed"
        );
    }

    #[test]
    fn allows_red_to_failed() {
        assert!(
            TaskStatus::can_transition(TaskStatus::Red, TaskStatus::Failed, false).is_ok(),
            "red -> failed should be allowed"
        );
    }

    #[test]
    fn allows_green_to_failed() {
        assert!(
            TaskStatus::can_transition(TaskStatus::Green, TaskStatus::Failed, false).is_ok(),
            "green -> failed should be allowed"
        );
    }

    #[test]
    fn allows_failed_to_red() {
        assert!(
            TaskStatus::can_transition(TaskStatus::Failed, TaskStatus::Red, false).is_ok(),
            "failed -> red should be allowed (retry)"
        );
    }

    mod rejects {
        use super::*;

        // PC-002: invalid transitions

        #[test]
        fn rejects_pending_to_green() {
            assert!(
                TaskStatus::can_transition(TaskStatus::Pending, TaskStatus::Green, false).is_err(),
                "pending -> green should be rejected (must go through red)"
            );
        }

        #[test]
        fn rejects_pending_to_done_for_tdd() {
            assert!(
                TaskStatus::can_transition(TaskStatus::Pending, TaskStatus::Done, false).is_err(),
                "pending -> done should be rejected for TDD entries"
            );
        }

        #[test]
        fn allows_pending_to_done_for_post_tdd() {
            assert!(
                TaskStatus::can_transition(TaskStatus::Pending, TaskStatus::Done, true).is_ok(),
                "pending -> done should be allowed for Post-TDD entries (AC-002.8)"
            );
        }

        #[test]
        fn rejects_done_to_pending() {
            assert!(
                TaskStatus::can_transition(TaskStatus::Done, TaskStatus::Pending, false).is_err(),
                "done -> pending should be rejected (terminal state)"
            );
        }

        #[test]
        fn rejects_done_to_red() {
            assert!(
                TaskStatus::can_transition(TaskStatus::Done, TaskStatus::Red, false).is_err(),
                "done -> red should be rejected (terminal state)"
            );
        }

        #[test]
        fn rejects_done_to_green() {
            assert!(
                TaskStatus::can_transition(TaskStatus::Done, TaskStatus::Green, false).is_err(),
                "done -> green should be rejected (terminal state)"
            );
        }

        #[test]
        fn rejects_done_to_failed() {
            assert!(
                TaskStatus::can_transition(TaskStatus::Done, TaskStatus::Failed, false).is_err(),
                "done -> failed should be rejected (terminal state)"
            );
        }

        #[test]
        fn rejects_same_state_red() {
            let err =
                TaskStatus::can_transition(TaskStatus::Red, TaskStatus::Red, false).unwrap_err();
            assert!(
                matches!(err, TaskError::SameState { .. }),
                "same-state red transition should produce SameState error, got: {err}"
            );
        }

        #[test]
        fn rejects_same_state_pending() {
            let err =
                TaskStatus::can_transition(TaskStatus::Pending, TaskStatus::Pending, false)
                    .unwrap_err();
            assert!(
                matches!(err, TaskError::SameState { .. }),
                "same-state pending transition should produce SameState error, got: {err}"
            );
        }
    }
}
