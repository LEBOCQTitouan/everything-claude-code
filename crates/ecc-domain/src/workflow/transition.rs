//! Workflow transition rules — pure function, no I/O.

use super::error::WorkflowError;
use super::phase::Phase;

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
    if current == target {
        return Ok(target);
    }
    let legal = matches!(
        (current, target),
        (Phase::Plan, Phase::Solution)
            | (Phase::Solution, Phase::Implement)
            | (Phase::Implement, Phase::Done)
    );
    if legal {
        Ok(target)
    } else {
        Err(WorkflowError::IllegalTransition {
            from: current,
            to: target,
        })
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    mod legal_transitions {
        use super::*;

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
}
