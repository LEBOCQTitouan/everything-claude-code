//! Workflow transition rules — pure function, no I/O.

use serde::{Deserialize, Serialize};

use super::error::WorkflowError;
use super::phase::Phase;

// ── Direction ────────────────────────────────────────────────────────────────

/// Direction of a workflow phase transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Forward,
    Backward,
}

// ── TransitionPair (private) ─────────────────────────────────────────────────

struct TransitionPair {
    from: Phase,
    to: Phase,
    direction: Direction,
}

// ── TransitionResult ─────────────────────────────────────────────────────────

/// The result of a successful phase transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionResult {
    pub from: Phase,
    pub to: Phase,
    pub direction: Direction,
}

// ── TransitionResolver trait ─────────────────────────────────────────────────

/// Trait for types that can resolve workflow phase transitions.
pub trait TransitionResolver {
    /// Resolve a phase transition.
    ///
    /// - Forward transitions accept `None` justification.
    /// - Backward transitions require a non-empty, non-whitespace justification.
    /// - `Phase::Unknown` source or target always returns `Err(IllegalTransition)`.
    /// - Re-entry (from == to) returns `Ok` with `direction = Forward`.
    fn resolve(
        &self,
        from: Phase,
        to: Phase,
        justification: Option<&str>,
    ) -> Result<TransitionResult, WorkflowError>;
}

// ── TransitionPolicy ─────────────────────────────────────────────────────────

/// Policy-driven transition table.
///
/// Use [`TransitionPolicy::default_forward`] for forward-only (backward-compatible) behaviour.
/// Use [`TransitionPolicy::with_backward`] to permit backward transitions with justification.
pub struct TransitionPolicy {
    pairs: Vec<TransitionPair>,
}

impl TransitionPolicy {
    /// Returns the standard forward-only policy (6 forward pairs).
    ///
    /// Pairs: Done→Idle, Idle→Plan, Plan→Solution, Solution→Implement, Implement→Done.
    /// Re-entry (from == to) is handled separately and always permitted.
    pub fn default_forward() -> Self {
        Self {
            pairs: vec![
                TransitionPair {
                    from: Phase::Done,
                    to: Phase::Idle,
                    direction: Direction::Forward,
                },
                TransitionPair {
                    from: Phase::Idle,
                    to: Phase::Plan,
                    direction: Direction::Forward,
                },
                TransitionPair {
                    from: Phase::Plan,
                    to: Phase::Solution,
                    direction: Direction::Forward,
                },
                TransitionPair {
                    from: Phase::Solution,
                    to: Phase::Implement,
                    direction: Direction::Forward,
                },
                TransitionPair {
                    from: Phase::Implement,
                    to: Phase::Done,
                    direction: Direction::Forward,
                },
            ],
        }
    }

    /// Returns the forward-only policy plus 3 backward pairs.
    ///
    /// Backward pairs: Implement→Solution, Solution→Plan, Implement→Plan.
    /// All backward transitions require a non-empty justification.
    pub fn with_backward() -> Self {
        let mut policy = Self::default_forward();
        policy.pairs.push(TransitionPair {
            from: Phase::Implement,
            to: Phase::Solution,
            direction: Direction::Backward,
        });
        policy.pairs.push(TransitionPair {
            from: Phase::Solution,
            to: Phase::Plan,
            direction: Direction::Backward,
        });
        policy.pairs.push(TransitionPair {
            from: Phase::Implement,
            to: Phase::Plan,
            direction: Direction::Backward,
        });
        policy
    }
}

impl TransitionResolver for TransitionPolicy {
    fn resolve(
        &self,
        from: Phase,
        to: Phase,
        justification: Option<&str>,
    ) -> Result<TransitionResult, WorkflowError> {
        // Phase::Unknown is never valid as source or target.
        if matches!(from, Phase::Unknown) || matches!(to, Phase::Unknown) {
            return Err(WorkflowError::IllegalTransition { from, to });
        }

        // Re-entry: from == to is always permitted (direction = Forward).
        if from == to {
            return Ok(TransitionResult {
                from,
                to,
                direction: Direction::Forward,
            });
        }

        // Look up the pair in the policy table.
        let pair = self.pairs.iter().find(|p| p.from == from && p.to == to);
        match pair {
            Some(p) if p.direction == Direction::Backward => {
                // Backward transitions require non-empty, non-whitespace justification.
                let valid = justification.map(|j| !j.trim().is_empty()).unwrap_or(false);
                if valid {
                    Ok(TransitionResult {
                        from,
                        to,
                        direction: Direction::Backward,
                    })
                } else {
                    Err(WorkflowError::MissingJustification)
                }
            }
            Some(_) => Ok(TransitionResult {
                from,
                to,
                direction: Direction::Forward,
            }),
            None => Err(WorkflowError::IllegalTransition { from, to }),
        }
    }
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Forward => write!(f, "forward"),
            Self::Backward => write!(f, "backward"),
        }
    }
}

// ── Legacy free functions (backward-compatible API) ───────────────────────────

/// Resolve a phase transition.
///
/// Returns `Ok(target)` for legal transitions, `Err(WorkflowError::IllegalTransition)` for illegal ones.
///
/// Legal transitions:
/// - Plan -> Solution
/// - Solution -> Implement
/// - Implement -> Done
///
/// Re-entry transitions (current == target) are accepted idempotently.
///
/// Everything else is illegal (including Done -> anything and backward transitions).
pub fn resolve_transition(current: Phase, target: Phase) -> Result<Phase, WorkflowError> {
    let policy = TransitionPolicy::default_forward();
    policy.resolve(current, target, None).map(|r| r.to)
}

/// Resolve a phase transition by target name string.
///
/// The `target_name` is parsed via `Phase::from_str`, which handles aliases:
/// - "spec" -> Plan
/// - "design" -> Solution
///
/// Returns `Ok(target_phase)` for legal transitions, or an error for unknown names or illegal transitions.
pub fn resolve_transition_by_name(
    current: Phase,
    target_name: &str,
) -> Result<Phase, WorkflowError> {
    let target = target_name
        .parse::<Phase>()
        .map_err(|e| WorkflowError::UnknownPhase(e.0))?;
    resolve_transition(current, target)
}

/// Resolve a phase transition with an optional justification (supports backward transitions).
///
/// Uses [`TransitionPolicy::with_backward`] — backward transitions (implement→solution,
/// solution→plan, implement→plan) are accepted when `justification` is non-empty.
pub fn resolve_transition_with_justification(
    current: Phase,
    target: Phase,
    justification: Option<&str>,
) -> Result<TransitionResult, WorkflowError> {
    let policy = TransitionPolicy::with_backward();
    policy.resolve(current, target, justification)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod legal_transitions {
        use super::*;

        #[test]
        fn done_to_idle_returns_ok() {
            let result = resolve_transition(Phase::Done, Phase::Idle);
            assert_eq!(result, Ok(Phase::Idle));
        }

        #[test]
        fn idle_to_plan_returns_ok() {
            let result = resolve_transition(Phase::Idle, Phase::Plan);
            assert_eq!(result, Ok(Phase::Plan));
        }

        #[test]
        fn plan_to_solution_returns_ok() {
            let result = resolve_transition(Phase::Plan, Phase::Solution);
            assert_eq!(result, Ok(Phase::Solution));
        }

        #[test]
        fn solution_to_implement_returns_ok() {
            let result = resolve_transition(Phase::Solution, Phase::Implement);
            assert_eq!(result, Ok(Phase::Implement));
        }

        #[test]
        fn implement_to_done_returns_ok() {
            let result = resolve_transition(Phase::Implement, Phase::Done);
            assert_eq!(result, Ok(Phase::Done));
        }
    }

    mod illegal_transitions {
        use super::*;

        #[test]
        fn idle_to_solution_returns_err() {
            let result = resolve_transition(Phase::Idle, Phase::Solution);
            assert!(result.is_err(), "idle->solution should be illegal");
        }

        #[test]
        fn idle_to_implement_returns_err() {
            let result = resolve_transition(Phase::Idle, Phase::Implement);
            assert!(result.is_err(), "idle->implement should be illegal");
        }

        #[test]
        fn idle_to_done_returns_err() {
            let result = resolve_transition(Phase::Idle, Phase::Done);
            assert!(result.is_err(), "idle->done should be illegal");
        }

        #[test]
        fn plan_to_implement_returns_err() {
            let result = resolve_transition(Phase::Plan, Phase::Implement);
            assert!(result.is_err(), "plan->implement should be illegal");
            let err = result.unwrap_err();
            let msg = err.to_string();
            assert!(
                msg.contains("plan") && msg.contains("implement"),
                "error message should mention both phases, got: {msg}"
            );
        }

        #[test]
        fn plan_to_done_returns_err() {
            let result = resolve_transition(Phase::Plan, Phase::Done);
            assert!(result.is_err(), "plan->done should be illegal");
            let err = result.unwrap_err();
            let msg = err.to_string();
            assert!(
                msg.contains("plan") && msg.contains("done"),
                "error message should mention both phases, got: {msg}"
            );
        }

        #[test]
        fn solution_to_done_returns_err() {
            let result = resolve_transition(Phase::Solution, Phase::Done);
            assert!(result.is_err(), "solution->done should be illegal");
            let err = result.unwrap_err();
            let msg = err.to_string();
            assert!(
                msg.contains("solution") && msg.contains("done"),
                "error message should mention both phases, got: {msg}"
            );
        }

        #[test]
        fn solution_to_plan_returns_err() {
            let result = resolve_transition(Phase::Solution, Phase::Plan);
            assert!(result.is_err(), "solution->plan should be illegal");
            let err = result.unwrap_err();
            let msg = err.to_string();
            assert!(
                msg.contains("solution") && msg.contains("plan"),
                "error message should mention both phases, got: {msg}"
            );
        }

        #[test]
        fn implement_to_plan_returns_err() {
            let result = resolve_transition(Phase::Implement, Phase::Plan);
            assert!(result.is_err(), "implement->plan should be illegal");
            let err = result.unwrap_err();
            let msg = err.to_string();
            assert!(
                msg.contains("implement") && msg.contains("plan"),
                "error message should mention both phases, got: {msg}"
            );
        }

        #[test]
        fn implement_to_solution_returns_err() {
            let result = resolve_transition(Phase::Implement, Phase::Solution);
            assert!(result.is_err(), "implement->solution should be illegal");
            let err = result.unwrap_err();
            let msg = err.to_string();
            assert!(
                msg.contains("implement") && msg.contains("solution"),
                "error message should mention both phases, got: {msg}"
            );
        }

        #[test]
        fn done_to_plan_returns_err() {
            let result = resolve_transition(Phase::Done, Phase::Plan);
            assert!(result.is_err(), "done->plan should be illegal");
            let err = result.unwrap_err();
            let msg = err.to_string();
            assert!(
                msg.contains("done") && msg.contains("plan"),
                "error message should mention both phases, got: {msg}"
            );
        }

        #[test]
        fn done_to_solution_returns_err() {
            let result = resolve_transition(Phase::Done, Phase::Solution);
            assert!(result.is_err(), "done->solution should be illegal");
            let err = result.unwrap_err();
            let msg = err.to_string();
            assert!(
                msg.contains("done") && msg.contains("solution"),
                "error message should mention both phases, got: {msg}"
            );
        }

        #[test]
        fn done_to_implement_returns_err() {
            let result = resolve_transition(Phase::Done, Phase::Implement);
            assert!(result.is_err(), "done->implement should be illegal");
            let err = result.unwrap_err();
            let msg = err.to_string();
            assert!(
                msg.contains("done") && msg.contains("implement"),
                "error message should mention both phases, got: {msg}"
            );
        }
    }

    mod reentry_transitions {
        use super::*;

        #[test]
        fn idle_to_idle_returns_ok() {
            let result = resolve_transition(Phase::Idle, Phase::Idle);
            assert_eq!(result, Ok(Phase::Idle));
        }

        #[test]
        fn plan_to_plan_returns_ok() {
            let result = resolve_transition(Phase::Plan, Phase::Plan);
            assert_eq!(result, Ok(Phase::Plan));
        }

        #[test]
        fn solution_to_solution_returns_ok() {
            let result = resolve_transition(Phase::Solution, Phase::Solution);
            assert_eq!(result, Ok(Phase::Solution));
        }

        #[test]
        fn implement_to_implement_returns_ok() {
            let result = resolve_transition(Phase::Implement, Phase::Implement);
            assert_eq!(result, Ok(Phase::Implement));
        }

        #[test]
        fn done_to_done_returns_ok() {
            let result = resolve_transition(Phase::Done, Phase::Done);
            assert_eq!(result, Ok(Phase::Done));
        }
    }

    mod alias_transitions {
        use super::*;
        use std::str::FromStr;

        #[test]
        fn spec_to_design_resolves_plan_to_solution() {
            // "spec" alias resolves to Plan, "design" alias resolves to Solution
            let current = Phase::from_str("spec").expect("spec should parse to Plan");
            let result = resolve_transition_by_name(current, "design");
            assert_eq!(result, Ok(Phase::Solution));
        }

        #[test]
        fn design_to_implement_resolves_solution_to_implement() {
            // "design" alias resolves to Solution, "implement" resolves to Implement
            let current = Phase::from_str("design").expect("design should parse to Solution");
            let result = resolve_transition_by_name(current, "implement");
            assert_eq!(result, Ok(Phase::Implement));
        }

        #[test]
        fn spec_alias_parses_to_plan() {
            let phase = Phase::from_str("spec").expect("spec should resolve");
            assert_eq!(phase, Phase::Plan);
        }

        #[test]
        fn design_alias_parses_to_solution() {
            let phase = Phase::from_str("design").expect("design should resolve");
            assert_eq!(phase, Phase::Solution);
        }

        #[test]
        fn unknown_alias_returns_err() {
            let current = Phase::Plan;
            let result = resolve_transition_by_name(current, "unknown-alias");
            assert!(result.is_err(), "unknown alias should return an error");
        }
    }

    mod unknown_phase_transition_rejected {
        use super::*;

        /// PC-054: Phase::Unknown rejected by transition logic.
        #[test]
        fn unknown_phase_as_source_is_rejected() {
            let result = resolve_transition(Phase::Unknown, Phase::Plan);
            assert!(
                result.is_err(),
                "transition FROM Unknown should be rejected, got: {result:?}"
            );
        }

        #[test]
        fn unknown_phase_as_target_is_rejected() {
            let result = resolve_transition(Phase::Idle, Phase::Unknown);
            assert!(
                result.is_err(),
                "transition TO Unknown should be rejected, got: {result:?}"
            );
        }

        #[test]
        fn unknown_to_unknown_is_rejected() {
            // Even re-entry for Unknown should be rejected (it is not a valid state)
            let result = resolve_transition(Phase::Unknown, Phase::Unknown);
            assert!(
                result.is_err(),
                "Unknown->Unknown should be rejected, got: {result:?}"
            );
        }
    }
}
