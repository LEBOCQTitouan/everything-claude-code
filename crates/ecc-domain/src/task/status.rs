//! TaskStatus enum and FSM transition validation.

use std::fmt;

use serde::Serialize;

use crate::task::error::TaskError;

/// The lifecycle status of a task (PC or Post-TDD entry).
///
/// State-transition diagram (TDD cycle + failure/retry + post-TDD skip):
///
/// ```text
///   [Pending] --> [Red] --> [Green] --> [Done]
///       |         |  ^       |
///       |         v  |       v
///       |       [Failed] <---+
///       |         ^
///       |         | (retry)
///       +---------+
///       |
///       +--> [Done]  (post-TDD only: Pending may skip directly to Done)
/// ```
///
/// `Done` is terminal. `Failed -> Red` is the retry arc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    /// Task not yet started.
    Pending,
    /// Test written, failing (RED phase).
    Red,
    /// Test passing (GREEN phase).
    Green,
    /// Task completed and verified.
    Done,
    /// Task execution failed (terminal state).
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

impl std::str::FromStr for TaskStatus {
    type Err = TaskError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "pending" => Ok(Self::Pending),
            "red" => Ok(Self::Red),
            "green" => Ok(Self::Green),
            "done" => Ok(Self::Done),
            "failed" => Ok(Self::Failed),
            other => Err(TaskError::InvalidStatus(other.to_owned())),
        }
    }
}

impl TaskStatus {
    /// Validate a status transition according to the FSM rules.
    ///
    /// `is_post_tdd` must be `true` for Post-TDD entries (e.g. "E2E tests"),
    /// which are allowed to skip directly from `Pending` to `Done`.
    ///
    /// # Errors
    ///
    /// - [`TaskError::SameState`] when `from == to`
    /// - [`TaskError::InvalidTransition`] when the transition is not allowed
    pub fn can_transition(from: Self, to: Self, is_post_tdd: bool) -> Result<(), TaskError> {
        // Same-state transitions are always rejected (AC-004.8)
        if from == to {
            return Err(TaskError::SameState {
                status: from.to_string(),
            });
        }

        // FSM transition table
        let allowed = match (from, to) {
            // TDD cycle (AC-002.1, AC-002.2, AC-002.3)
            (Self::Pending, Self::Red) => true,
            (Self::Red, Self::Green) => true,
            (Self::Green, Self::Done) => true,
            // Failure paths (AC-002.6)
            (Self::Red, Self::Failed) => true,
            (Self::Green, Self::Failed) => true,
            // Retry after failure (AC-002.7)
            (Self::Failed, Self::Red) => true,
            // Post-TDD entries skip TDD cycle (AC-002.8)
            (Self::Pending, Self::Done) => is_post_tdd,
            // Done is terminal (AC-002.5); all other combinations reject
            _ => false,
        };

        if allowed {
            Ok(())
        } else {
            Err(TaskError::InvalidTransition {
                from: from.to_string(),
                to: to.to_string(),
            })
        }
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

    // PC-028: TaskStatus serde serialization produces lowercase strings
    mod serde_format {
        use super::*;

        #[test]
        fn serializes_pending_as_lowercase() {
            let json = serde_json::to_string(&TaskStatus::Pending).unwrap();
            assert_eq!(json, "\"pending\"");
        }

        #[test]
        fn serializes_red_as_lowercase() {
            let json = serde_json::to_string(&TaskStatus::Red).unwrap();
            assert_eq!(json, "\"red\"");
        }

        #[test]
        fn serializes_green_as_lowercase() {
            let json = serde_json::to_string(&TaskStatus::Green).unwrap();
            assert_eq!(json, "\"green\"");
        }

        #[test]
        fn serializes_done_as_lowercase() {
            let json = serde_json::to_string(&TaskStatus::Done).unwrap();
            assert_eq!(json, "\"done\"");
        }

        #[test]
        fn serializes_failed_as_lowercase() {
            let json = serde_json::to_string(&TaskStatus::Failed).unwrap();
            assert_eq!(json, "\"failed\"");
        }
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
            let err = TaskStatus::can_transition(TaskStatus::Pending, TaskStatus::Pending, false)
                .unwrap_err();
            assert!(
                matches!(err, TaskError::SameState { .. }),
                "same-state pending transition should produce SameState error, got: {err}"
            );
        }
    }
}
