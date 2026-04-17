//! Resolver trait tests: PC-004, PC-005, PC-006, PC-007, PC-008, PC-021, PC-022.

use ecc_domain::workflow::error::WorkflowError;
use ecc_domain::workflow::phase::Phase;
use ecc_domain::workflow::transition::{Direction, TransitionPolicy, TransitionResolver};

mod resolver {
    use super::*;

    /// PC-004: Forward via TransitionResolver trait returns Ok with direction=Forward
    #[test]
    fn forward_via_trait() {
        let policy = TransitionPolicy::default_forward();
        let result = policy
            .resolve(Phase::Plan, Phase::Solution, None)
            .expect("Plan->Solution should be a legal forward transition");
        assert_eq!(result.from, Phase::Plan);
        assert_eq!(result.to, Phase::Solution);
        assert_eq!(result.direction, Direction::Forward);
    }

    /// PC-005: Backward with justification via with_backward() returns Ok with direction=Backward
    #[test]
    fn backward_with_justification() {
        let policy = TransitionPolicy::with_backward();
        let result = policy
            .resolve(Phase::Implement, Phase::Solution, Some("design flaw found"))
            .expect("Implement->Solution with justification should succeed");
        assert_eq!(result.from, Phase::Implement);
        assert_eq!(result.to, Phase::Solution);
        assert_eq!(result.direction, Direction::Backward);
    }

    /// PC-006: Backward with None justification returns Err(MissingJustification)
    #[test]
    fn backward_missing_justification() {
        let policy = TransitionPolicy::with_backward();
        let result = policy.resolve(Phase::Implement, Phase::Solution, None);
        assert!(
            matches!(result, Err(WorkflowError::MissingJustification)),
            "expected MissingJustification, got: {result:?}"
        );
    }

    /// PC-007: Backward with empty string justification returns Err(MissingJustification)
    #[test]
    fn backward_empty_justification() {
        let policy = TransitionPolicy::with_backward();
        let result = policy.resolve(Phase::Implement, Phase::Solution, Some(""));
        assert!(
            matches!(result, Err(WorkflowError::MissingJustification)),
            "expected MissingJustification for empty string, got: {result:?}"
        );
    }

    /// PC-008: Backward with whitespace-only justification returns Err(MissingJustification)
    #[test]
    fn backward_whitespace_justification() {
        let policy = TransitionPolicy::with_backward();
        let result = policy.resolve(Phase::Implement, Phase::Solution, Some("  "));
        assert!(
            matches!(result, Err(WorkflowError::MissingJustification)),
            "expected MissingJustification for whitespace-only, got: {result:?}"
        );
    }

    /// PC-021: Done->Plan, Done->Solution, Done->Implement remain illegal even with with_backward()
    #[test]
    fn done_backward_still_illegal() {
        let policy = TransitionPolicy::with_backward();
        for to in [Phase::Plan, Phase::Solution, Phase::Implement] {
            let result = policy.resolve(Phase::Done, to, Some("reason"));
            assert!(
                matches!(result, Err(WorkflowError::IllegalTransition { .. })),
                "Done->{to} should be illegal even in with_backward() policy, got: {result:?}"
            );
        }
    }

    /// PC-022: Forward transitions accept None justification (backward compat)
    #[test]
    fn forward_accepts_none_justification() {
        let policy = TransitionPolicy::default_forward();
        let forward_pairs = [
            (Phase::Idle, Phase::Plan),
            (Phase::Plan, Phase::Solution),
            (Phase::Solution, Phase::Implement),
            (Phase::Implement, Phase::Done),
            (Phase::Done, Phase::Idle),
        ];
        for (from, to) in forward_pairs {
            let result = policy.resolve(from, to, None);
            assert!(
                result.is_ok(),
                "{from}->{to} should accept None justification, got: {result:?}"
            );
        }
    }
}
