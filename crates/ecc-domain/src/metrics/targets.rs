//! Reference SLO targets for harness reliability metrics.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_slo_values() {
        let t = ReferenceTargets::default();
        assert!((t.hook_success - 0.99).abs() < f64::EPSILON);
        assert!((t.phase_gate_violation - 0.05).abs() < f64::EPSILON);
        assert!((t.agent_recovery - 0.80).abs() < f64::EPSILON);
        assert!((t.commit_atomicity - 0.95).abs() < f64::EPSILON);
    }
}
