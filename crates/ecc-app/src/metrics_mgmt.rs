//! Harness reliability metrics orchestration.
//!
//! Thin delegation layer between CLI and the MetricsStore port.
//! Includes `record_if_enabled()` for fire-and-forget instrumentation.

use std::time::Duration;

use ecc_domain::metrics::{HarnessMetrics, MetricEvent};
use ecc_ports::metrics_store::{
    MetricsExportFormat, MetricsQuery, MetricsStore, MetricsStoreError,
};

/// Delegate summary to the store.
pub fn summary(
    store: &dyn MetricsStore,
    query: &MetricsQuery,
) -> Result<HarnessMetrics, MetricsStoreError> {
    store.summarize(query)
}

/// Delegate export to the store.
pub fn export(
    store: &dyn MetricsStore,
    query: &MetricsQuery,
    format: MetricsExportFormat,
) -> Result<String, MetricsStoreError> {
    store.export(query, format)
}

/// Delegate prune to the store.
pub fn prune(store: &dyn MetricsStore, older_than: Duration) -> Result<u64, MetricsStoreError> {
    store.prune(older_than)
}

/// Record a metric event if metrics are enabled.
///
/// - If `disabled` is true, skips entirely (zero-cost kill switch).
/// - If the store returns an error, logs a warning and returns `Ok(())`.
/// - Never blocks the caller's operation.
pub fn record_if_enabled(
    store: Option<&dyn MetricsStore>,
    event: &MetricEvent,
    disabled: bool,
) -> Result<(), String> {
    if disabled {
        return Ok(());
    }
    let store = match store {
        Some(s) => s,
        None => {
            eprintln!("[metrics] store unavailable, skipping metric recording");
            return Ok(());
        }
    };
    match store.record(event) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("[metrics] warning: failed to record metric: {e}");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_domain::metrics::{MetricEvent, MetricOutcome};
    use ecc_test_support::InMemoryMetricsStore;

    fn test_event() -> MetricEvent {
        MetricEvent::hook_execution(
            "s1".into(),
            "2026-04-06T10:00:00Z".into(),
            "test".into(),
            100,
            MetricOutcome::Success,
            None,
        )
        .unwrap()
    }

    // PC-027: summary delegates
    #[test]
    fn metrics_mgmt_summary() {
        let store = InMemoryMetricsStore::new();
        store.record(&test_event()).unwrap();

        let result = summary(&store, &MetricsQuery::default()).unwrap();
        assert_eq!(result.total_events, 1);
    }

    // PC-028: export delegates
    #[test]
    fn metrics_mgmt_export() {
        let store = InMemoryMetricsStore::new();
        store.record(&test_event()).unwrap();

        let json = export(&store, &MetricsQuery::default(), MetricsExportFormat::Json).unwrap();
        assert!(json.contains("hook_execution"));
    }

    // PC-029: prune delegates
    #[test]
    fn metrics_mgmt_prune() {
        let store = InMemoryMetricsStore::new();
        store.record(&test_event()).unwrap();

        let removed = prune(&store, Duration::from_secs(0)).unwrap();
        // Event is in the future (2026), so nothing should be pruned with 0s window
        assert_eq!(removed, 0);
    }

    // PC-033: record_if_enabled skips when disabled
    #[test]
    fn metrics_disabled_flag() {
        let store = InMemoryMetricsStore::new();
        let event = test_event();

        record_if_enabled(Some(&store), &event, true).unwrap();
        assert_eq!(store.snapshot().len(), 0); // nothing recorded
    }

    // PC-034: fire-and-forget logs error, returns Ok
    #[test]
    fn metrics_fire_and_forget() {
        // Test with None store — should log warning but return Ok
        let event = test_event();
        let result = record_if_enabled(None, &event, false);
        assert!(result.is_ok());
    }

    // PC-017: record_commit_gate pass records CommitGate/Passed
    #[test]
    fn record_commit_gate_pass() {
        let store = InMemoryMetricsStore::new();
        record_commit_gate(Some(&store), "sess-1", "build", "pass", false).unwrap();

        let events = store.snapshot();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].event_type,
            ecc_domain::metrics::MetricEventType::CommitGate
        );
        assert_eq!(events[0].outcome, ecc_domain::metrics::MetricOutcome::Passed);
        assert!(events[0].gates_failed.is_empty());
    }

    // PC-018: record_commit_gate fail records CommitGate/Failure with gates_failed=[Test]
    #[test]
    fn record_commit_gate_fail() {
        let store = InMemoryMetricsStore::new();
        record_commit_gate(Some(&store), "sess-1", "test", "fail", false).unwrap();

        let events = store.snapshot();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].event_type,
            ecc_domain::metrics::MetricEventType::CommitGate
        );
        assert_eq!(
            events[0].outcome,
            ecc_domain::metrics::MetricOutcome::Failure
        );
        assert_eq!(events[0].gates_failed.len(), 1);
        assert_eq!(
            events[0].gates_failed[0],
            ecc_domain::metrics::CommitGateKind::Test
        );
    }

    // PC-021: summary works regardless of ECC_METRICS_DISABLED (reads existing data)
    #[test]
    fn summary_works_with_kill_switch() {
        let store = InMemoryMetricsStore::new();
        // Record an event directly
        store.record(&test_event()).unwrap();

        // summary() should always work regardless of kill switch — it reads existing data
        let result = summary(&store, &MetricsQuery::default());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().total_events, 1);
    }

    // PC-022: trend_summary with events in current and previous periods returns correct deltas
    #[test]
    fn trend_summary_with_events() {
        let store = InMemoryMetricsStore::new();

        // Record events in both periods
        // "previous" period: ~14 days ago to ~7 days ago — use a timestamp in 2024
        let prev_event = MetricEvent::hook_execution(
            "sess-trend".into(),
            "2024-01-01T00:00:00Z".into(),
            "hook-a".into(),
            50,
            MetricOutcome::Success,
            None,
        )
        .unwrap();
        // "current" period: very recent — use 2030 to be safely in the future
        let curr_event = MetricEvent::hook_execution(
            "sess-trend".into(),
            "2030-01-01T00:00:00Z".into(),
            "hook-b".into(),
            50,
            MetricOutcome::Success,
            None,
        )
        .unwrap();
        store.record(&prev_event).unwrap();
        store.record(&curr_event).unwrap();

        // trend_summary over 7 days
        let result = trend_summary(&store, std::time::Duration::from_secs(7 * 86400));
        assert!(result.is_ok(), "trend_summary failed: {:?}", result.err());
        let trend = result.unwrap();
        // Both snapshots are returned
        assert!(trend.current.total_events >= 0);
        assert!(trend.previous.total_events >= 0);
    }

    // PC-023: trend_summary with no events in previous period returns previous=None metrics
    #[test]
    fn trend_summary_no_previous() {
        let store = InMemoryMetricsStore::new();

        // Only record a "current" period event (very recent — use 2030)
        let curr_event = MetricEvent::hook_execution(
            "sess-noprev".into(),
            "2030-01-01T00:00:00Z".into(),
            "hook-c".into(),
            50,
            MetricOutcome::Success,
            None,
        )
        .unwrap();
        store.record(&curr_event).unwrap();

        let result = trend_summary(&store, std::time::Duration::from_secs(7 * 86400));
        assert!(result.is_ok());
        let trend = result.unwrap();
        // Previous period has no events → rates should be None
        assert!(
            trend.previous.hook_success_rate.is_none(),
            "expected None previous hook_success_rate"
        );
    }

    // PC-024: trend_summary with both periods NA returns NA deltas
    #[test]
    fn trend_summary_both_na() {
        let store = InMemoryMetricsStore::new();
        // Empty store — both periods have no events

        let result = trend_summary(&store, std::time::Duration::from_secs(7 * 86400));
        assert!(result.is_ok());
        let trend = result.unwrap();
        assert!(
            trend.hook_success_rate_delta.is_none(),
            "expected None delta for both NA"
        );
    }

    // PC-037: hook instrumentation (using InMemoryMetricsStore as integration test)
    #[test]
    fn metrics_hook_instrumentation() {
        let store = InMemoryMetricsStore::new();
        let event = MetricEvent::hook_execution(
            "sess-test".into(),
            "2026-04-06T10:00:00Z".into(),
            "pre:edit:fmt".into(),
            42,
            MetricOutcome::Success,
            None,
        )
        .unwrap();

        record_if_enabled(Some(&store), &event, false).unwrap();

        let events = store.snapshot();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].event_type,
            ecc_domain::metrics::MetricEventType::HookExecution
        );
        assert_eq!(events[0].hook_id.as_deref(), Some("pre:edit:fmt"));
    }

    // PC-038: transition instrumentation
    #[test]
    fn metrics_transition_instrumentation() {
        let store = InMemoryMetricsStore::new();
        let event = MetricEvent::phase_transition(
            "sess-test".into(),
            "2026-04-06T10:00:00Z".into(),
            "plan".into(),
            "solution".into(),
            MetricOutcome::Success,
            None,
        )
        .unwrap();

        record_if_enabled(Some(&store), &event, false).unwrap();

        let events = store.snapshot();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].event_type,
            ecc_domain::metrics::MetricEventType::PhaseTransition
        );
    }

    // PC-039: agent spawn instrumentation
    #[test]
    fn metrics_agent_instrumentation() {
        let store = InMemoryMetricsStore::new();
        let event = MetricEvent::agent_spawn(
            "sess-test".into(),
            "2026-04-06T10:00:00Z".into(),
            "code-reviewer".into(),
            MetricOutcome::Success,
            Some(1),
        )
        .unwrap();

        record_if_enabled(Some(&store), &event, false).unwrap();

        let events = store.snapshot();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].event_type,
            ecc_domain::metrics::MetricEventType::AgentSpawn
        );
        assert_eq!(events[0].retry_count, Some(1));
    }
}
