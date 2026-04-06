//! Reference SLO targets for harness reliability metrics.

/// Default SLO targets for harness reliability.
#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceTargets {
    /// Minimum fraction of hook executions that must succeed (0.99 = 99%).
    pub hook_success: f64,
    /// Maximum fraction of phase transitions that may be rejected (0.05 = 5%).
    pub phase_gate_violation: f64,
    /// Minimum fraction of failed agents that should recover via retry (0.80 = 80%).
    pub agent_recovery: f64,
    /// Minimum fraction of commits that pass all gates (0.95 = 95%).
    pub commit_atomicity: f64,
}

impl Default for ReferenceTargets {
    fn default() -> Self {
        Self {
            hook_success: 0.99,
            phase_gate_violation: 0.05,
            agent_recovery: 0.80,
            commit_atomicity: 0.95,
        }
    }
}

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
