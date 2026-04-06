//! Trend comparison between two HarnessMetrics snapshots.

use super::aggregate::HarnessMetrics;

/// Comparison of current vs previous harness metrics, with computed deltas.
#[derive(Debug, Clone, PartialEq)]
pub struct TrendComparison {
    /// Current snapshot.
    pub current: HarnessMetrics,
    /// Previous snapshot.
    pub previous: HarnessMetrics,
    /// Delta for hook_success_rate (current − previous). None if either is None.
    pub hook_success_rate_delta: Option<f64>,
    /// Delta for phase_gate_violation_rate (current − previous). None if either is None.
    pub phase_gate_violation_rate_delta: Option<f64>,
    /// Delta for agent_failure_recovery_rate (current − previous). None if either is None.
    pub agent_failure_recovery_rate_delta: Option<f64>,
    /// Delta for commit_atomicity_score (current − previous). None if either is None.
    pub commit_atomicity_score_delta: Option<f64>,
}

impl TrendComparison {
    /// Compute a [`TrendComparison`] from current and previous snapshots.
    pub fn compute(current: HarnessMetrics, previous: HarnessMetrics) -> Self {
        Self {
            current,
            previous,
            hook_success_rate_delta: None,
            phase_gate_violation_rate_delta: None,
            agent_failure_recovery_rate_delta: None,
            commit_atomicity_score_delta: None,
        }
    }

    /// Format a delta value as "+X.X%", "-X.X%", or "N/A".
    pub fn format_delta(_delta: Option<f64>) -> String {
        "N/A".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics(
        hook: Option<f64>,
        phase: Option<f64>,
        agent: Option<f64>,
        commit: Option<f64>,
    ) -> HarnessMetrics {
        HarnessMetrics {
            hook_success_rate: hook,
            phase_gate_violation_rate: phase,
            agent_failure_recovery_rate: agent,
            commit_atomicity_score: commit,
            total_events: 0,
            session_id: None,
            time_range: None,
        }
    }

    // AC-007.1, AC-007.2: compute returns correct deltas with "+" sign for positive
    #[test]
    fn compute_returns_correct_deltas() {
        let current = metrics(Some(0.9), Some(0.1), Some(0.8), Some(0.95));
        let previous = metrics(Some(0.7), Some(0.2), Some(0.6), Some(0.9));
        let trend = TrendComparison::compute(current, previous);

        let eps = 1e-9;
        assert!((trend.hook_success_rate_delta.unwrap() - 0.2).abs() < eps);
        assert!((trend.phase_gate_violation_rate_delta.unwrap() - (-0.1)).abs() < eps);
        assert!((trend.agent_failure_recovery_rate_delta.unwrap() - 0.2).abs() < eps);
        assert!((trend.commit_atomicity_score_delta.unwrap() - 0.05).abs() < eps);
    }

    // AC-007.2: positive delta formatted with "+" sign
    #[test]
    fn format_delta_positive_has_plus_sign() {
        let result = TrendComparison::format_delta(Some(0.15));
        assert!(result.starts_with('+'), "expected '+' prefix, got: {result}");
        assert_eq!(result, "+15.0%");
    }

    // AC-007.2: negative delta formatted without "+" sign
    #[test]
    fn format_delta_negative_no_plus_sign() {
        let result = TrendComparison::format_delta(Some(-0.05));
        assert!(!result.starts_with('+'), "did not expect '+' prefix, got: {result}");
        assert_eq!(result, "-5.0%");
    }

    // AC-007.3: both N/A yields N/A delta
    #[test]
    fn both_na_yields_na_delta() {
        let current = metrics(None, None, None, None);
        let previous = metrics(None, None, None, None);
        let trend = TrendComparison::compute(current, previous);

        assert!(trend.hook_success_rate_delta.is_none());
        assert!(trend.phase_gate_violation_rate_delta.is_none());
        assert!(trend.agent_failure_recovery_rate_delta.is_none());
        assert!(trend.commit_atomicity_score_delta.is_none());
    }

    // AC-007.3: format_delta(None) returns "N/A"
    #[test]
    fn format_delta_none_returns_na() {
        assert_eq!(TrendComparison::format_delta(None), "N/A");
    }

    // AC-007.5: one side None yields None delta
    #[test]
    fn one_side_none_yields_none_delta() {
        let current = metrics(Some(0.8), None, Some(0.5), None);
        let previous = metrics(None, Some(0.3), None, Some(0.7));
        let trend = TrendComparison::compute(current, previous);

        assert!(trend.hook_success_rate_delta.is_none());
        assert!(trend.phase_gate_violation_rate_delta.is_none());
        assert!(trend.agent_failure_recovery_rate_delta.is_none());
        assert!(trend.commit_atomicity_score_delta.is_none());
    }
}
