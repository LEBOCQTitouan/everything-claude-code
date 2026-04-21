//! Phase verification — pure function, no I/O.

use super::phase::Phase;
use std::fmt;

/// Error returned when the current workflow phase does not match the expected phase.
#[derive(Debug, PartialEq, Eq)]
pub struct PhaseError {
    /// The actual current phase.
    pub current: Phase,
    /// The expected phase.
    pub expected: Phase,
    /// Suggestion for what to do next.
    pub hint: String,
}

impl fmt::Display for PhaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Expected phase: {}, current: {}. {}",
            self.expected, self.current, self.hint
        )
    }
}

impl std::error::Error for PhaseError {}

/// Verify that the current workflow phase matches the expected phase.
///
/// - If `current` is `None` (no active workflow) and `expected` is `Plan`, returns `Ok`
///   (spec commands are allowed to initialize).
/// - If `current` matches `expected`, returns `Ok`.
/// - Otherwise, returns `Err(PhaseError)` with a hint about which command to run.
pub fn verify_phase(current: Option<Phase>, expected: Phase) -> Result<(), PhaseError> {
    match current {
        None if expected == Phase::Plan => Ok(()),
        None => Err(PhaseError {
            current: Phase::Idle,
            expected,
            hint: format!("Run {} first.", phase_hint(expected)),
        }),
        Some(c) if c == expected => Ok(()),
        Some(c) => Err(PhaseError {
            current: c,
            expected,
            hint: format!("Run {} first.", phase_hint(expected)),
        }),
    }
}

/// Returns the command the user should run to reach the expected phase.
/// The hint points to the *prerequisite* command that produces the expected phase.
fn phase_hint(expected: Phase) -> &'static str {
    match expected {
        Phase::Plan => "/spec",
        Phase::Solution => "/spec",
        Phase::Implement => "/design",
        _ => "the appropriate command",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::phase::Phase;

    #[test]
    fn rejects_idle_when_solution_expected() {
        let result = verify_phase(Some(Phase::Idle), Phase::Solution);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.current, Phase::Idle);
        assert_eq!(err.expected, Phase::Solution);
        assert!(
            err.hint.contains("/spec"),
            "hint should suggest /spec, got: {}",
            err.hint
        );
    }

    #[test]
    fn rejects_plan_when_implement_expected() {
        let result = verify_phase(Some(Phase::Plan), Phase::Implement);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.current, Phase::Plan);
        assert_eq!(err.expected, Phase::Implement);
        assert!(
            err.hint.contains("/design"),
            "hint should suggest /design (prerequisite for implement), got: {}",
            err.hint
        );
    }

    #[test]
    fn accepts_correct_phase() {
        assert!(verify_phase(Some(Phase::Solution), Phase::Solution).is_ok());
        assert!(verify_phase(Some(Phase::Plan), Phase::Plan).is_ok());
        assert!(verify_phase(Some(Phase::Implement), Phase::Implement).is_ok());
    }

    #[test]
    fn allows_none_for_plan_init() {
        assert!(
            verify_phase(None, Phase::Plan).is_ok(),
            "None state should allow Plan phase (spec init)"
        );
    }

    #[test]
    fn rejects_none_for_non_plan() {
        assert!(
            verify_phase(None, Phase::Solution).is_err(),
            "None state should reject Solution phase"
        );
        assert!(
            verify_phase(None, Phase::Implement).is_err(),
            "None state should reject Implement phase"
        );
    }
}
