//! Harness metrics aggregation.

use super::event::{MetricEvent, MetricEventType, MetricOutcome};

/// Aggregated harness reliability scores for a set of events.
#[derive(Debug, Clone, PartialEq)]
pub struct HarnessMetrics {
    /// Percentage of hooks completing without error. None if no hook events.
    pub hook_success_rate: Option<f64>,
    /// Percentage of phase transitions rejected. None if no transition events.
    pub phase_gate_violation_rate: Option<f64>,
    /// Percentage of failed agents that recovered via retry. None if no relevant agent events.
    pub agent_failure_recovery_rate: Option<f64>,
    /// Percentage of commits passing all gates. None if no commit events.
    pub commit_atomicity_score: Option<f64>,
    /// Total number of events in the aggregation.
    pub total_events: u64,
    /// Session ID (if all events share the same session).
    pub session_id: Option<String>,
    /// Time range (earliest timestamp, latest timestamp).
    pub time_range: Option<(String, String)>,
}

/// Computes aggregated metrics from a slice of events.
pub struct MetricAggregator;

impl MetricAggregator {
    /// Compute [`HarnessMetrics`] from a slice of events.
    pub fn summarize(events: &[MetricEvent]) -> HarnessMetrics {
        let total_events = events.len() as u64;

        HarnessMetrics {
            hook_success_rate: Self::hook_success_rate(events),
            phase_gate_violation_rate: Self::phase_gate_violation_rate(events),
            agent_failure_recovery_rate: Self::agent_failure_recovery_rate(events),
            commit_atomicity_score: Self::commit_atomicity_score(events),
            total_events,
            session_id: Self::common_session_id(events),
            time_range: Self::time_range(events),
        }
    }

    fn hook_success_rate(events: &[MetricEvent]) -> Option<f64> {
        let hooks: Vec<_> = events
            .iter()
            .filter(|e| e.event_type == MetricEventType::HookExecution)
            .collect();
        let total = hooks.len();
        if total == 0 {
            return None;
        }
        let ok = hooks.iter().filter(|e| e.outcome == MetricOutcome::Success).count();
        Some(ok as f64 / total as f64)
    }

    fn phase_gate_violation_rate(events: &[MetricEvent]) -> Option<f64> {
        let transitions: Vec<_> = events
            .iter()
            .filter(|e| e.event_type == MetricEventType::PhaseTransition)
            .collect();
        let total = transitions.len();
        if total == 0 {
            return None;
        }
        let rejected = transitions.iter().filter(|e| e.outcome == MetricOutcome::Rejected).count();
        Some(rejected as f64 / total as f64)
    }

    fn agent_failure_recovery_rate(events: &[MetricEvent]) -> Option<f64> {
        let agents: Vec<_> = events
            .iter()
            .filter(|e| e.event_type == MetricEventType::AgentSpawn)
            .collect();

        // Denominator: failed OR (succeeded with retry > 0)
        let denominator = agents
            .iter()
            .filter(|e| {
                e.outcome == MetricOutcome::Failure
                    || (e.outcome == MetricOutcome::Success
                        && e.retry_count.is_some_and(|r| r > 0))
            })
            .count();
        if denominator == 0 {
            return None;
        }

        // Numerator: succeeded with retry > 0
        let recovered = agents
            .iter()
            .filter(|e| {
                e.outcome == MetricOutcome::Success && e.retry_count.is_some_and(|r| r > 0)
            })
            .count();
        Some(recovered as f64 / denominator as f64)
    }

    fn commit_atomicity_score(events: &[MetricEvent]) -> Option<f64> {
        let commits: Vec<_> = events
            .iter()
            .filter(|e| e.event_type == MetricEventType::CommitGate)
            .collect();
        let total = commits.len();
        if total == 0 {
            return None;
        }
        let passed = commits.iter().filter(|e| e.outcome == MetricOutcome::Passed).count();
        Some(passed as f64 / total as f64)
    }

    fn common_session_id(events: &[MetricEvent]) -> Option<String> {
        let first = events.first()?;
        let sid = &first.session_id;
        if events.iter().all(|e| e.session_id == *sid) {
            Some(sid.clone())
        } else {
            None
        }
    }

    fn time_range(events: &[MetricEvent]) -> Option<(String, String)> {
        if events.is_empty() {
            return None;
        }
        let min = events.iter().map(|e| e.timestamp.as_str()).min().unwrap().to_owned();
        let max = events.iter().map(|e| e.timestamp.as_str()).max().unwrap().to_owned();
        Some((min, max))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::event::{CommitGateKind, MetricEvent, MetricOutcome};

    fn hook(outcome: MetricOutcome) -> MetricEvent {
        MetricEvent::hook_execution("sess-1".into(), "2026-04-06T10:00:00Z".into(), "h".into(), 100, outcome, None).unwrap()
    }

    fn transition(outcome: MetricOutcome) -> MetricEvent {
        MetricEvent::phase_transition("sess-1".into(), "2026-04-06T10:00:00Z".into(), "plan".into(), "solution".into(), outcome, None).unwrap()
    }

    fn agent(outcome: MetricOutcome, retry: Option<u32>) -> MetricEvent {
        MetricEvent::agent_spawn("sess-1".into(), "2026-04-06T10:00:00Z".into(), "tdd".into(), outcome, retry).unwrap()
    }

    fn commit(outcome: MetricOutcome) -> MetricEvent {
        MetricEvent::commit_gate(
            "sess-1".into(), "2026-04-06T10:00:00Z".into(), outcome,
            if outcome == MetricOutcome::Failure { vec![CommitGateKind::Test] } else { vec![] },
        ).unwrap()
    }

    // PC-009: aggregator computes rates
    #[test]
    fn aggregator_computes_rates() {
        let events = vec![
            hook(MetricOutcome::Success),
            hook(MetricOutcome::Success),
            hook(MetricOutcome::Failure),
            transition(MetricOutcome::Success),
            transition(MetricOutcome::Rejected),
            agent(MetricOutcome::Failure, None),
            agent(MetricOutcome::Success, Some(1)),
            commit(MetricOutcome::Passed),
            commit(MetricOutcome::Passed),
            commit(MetricOutcome::Failure),
        ];

        let m = MetricAggregator::summarize(&events);
        assert!((m.hook_success_rate.unwrap() - 2.0 / 3.0).abs() < f64::EPSILON);
        assert!((m.phase_gate_violation_rate.unwrap() - 0.5).abs() < f64::EPSILON);
        assert!((m.agent_failure_recovery_rate.unwrap() - 0.5).abs() < f64::EPSILON);
        assert!((m.commit_atomicity_score.unwrap() - 2.0 / 3.0).abs() < f64::EPSILON);
    }

    // PC-010: zero-denominator returns None
    #[test]
    fn aggregator_zero_denominator() {
        let m = MetricAggregator::summarize(&[]);
        assert!(m.hook_success_rate.is_none());
        assert!(m.phase_gate_violation_rate.is_none());
        assert!(m.agent_failure_recovery_rate.is_none());
        assert!(m.commit_atomicity_score.is_none());
        assert_eq!(m.total_events, 0);
    }

    // PC-011: total events
    #[test]
    fn harness_metrics_total_events() {
        let events = vec![hook(MetricOutcome::Success), transition(MetricOutcome::Success), commit(MetricOutcome::Passed)];
        assert_eq!(MetricAggregator::summarize(&events).total_events, 3);
    }
}
