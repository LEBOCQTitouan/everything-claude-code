//! PC-001: Full forward lifecycle integration test.

use ecc_domain::workflow::phase::Phase;
use ecc_domain::workflow::transition::resolve_transition;

mod lifecycle {
    use super::*;

    #[test]
    fn forward_lifecycle_completes() {
        // idle -> plan
        let phase =
            resolve_transition(Phase::Idle, Phase::Plan).expect("idle->plan should succeed");
        assert_eq!(phase, Phase::Plan);

        // plan -> solution
        let phase =
            resolve_transition(phase, Phase::Solution).expect("plan->solution should succeed");
        assert_eq!(phase, Phase::Solution);

        // solution -> implement
        let phase = resolve_transition(phase, Phase::Implement)
            .expect("solution->implement should succeed");
        assert_eq!(phase, Phase::Implement);

        // implement -> done
        let phase = resolve_transition(phase, Phase::Done).expect("implement->done should succeed");
        assert_eq!(phase, Phase::Done);

        // done -> idle
        let phase = resolve_transition(phase, Phase::Idle).expect("done->idle should succeed");
        assert_eq!(phase, Phase::Idle);
    }
}
