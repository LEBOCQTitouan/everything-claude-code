//! PC-003: TransitionPolicy::default() forward pairs test.

use ecc_domain::workflow::phase::Phase;
use ecc_domain::workflow::transition::{Direction, TransitionPolicy, TransitionResolver};

mod policy {
    use super::*;

    #[test]
    fn default_policy_has_forward_pairs() {
        let policy = TransitionPolicy::default_forward();
        // Verify the 6 forward pairs are present by resolving each
        let forward_pairs = [
            (Phase::Done, Phase::Idle),
            (Phase::Idle, Phase::Plan),
            (Phase::Plan, Phase::Solution),
            (Phase::Solution, Phase::Implement),
            (Phase::Implement, Phase::Done),
        ];
        for (from, to) in forward_pairs {
            let result = policy.resolve(from, to, None).unwrap_or_else(|e| {
                panic!("expected {from}->{to} to be a legal forward pair, got: {e}")
            });
            assert_eq!(
                result.direction,
                Direction::Forward,
                "{from}->{to} should be Forward"
            );
        }
        // Verify backward pairs are NOT in default policy
        let backward_pairs = [
            (Phase::Implement, Phase::Solution),
            (Phase::Solution, Phase::Plan),
            (Phase::Implement, Phase::Plan),
        ];
        for (from, to) in backward_pairs {
            let result = policy.resolve(from, to, Some("reason"));
            assert!(
                result.is_err(),
                "{from}->{to} should be illegal in default policy"
            );
        }
    }
}
