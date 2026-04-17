//! In-memory test double for [`MetricsStore`].

use std::sync::Mutex;
use std::time::Duration;

use ecc_domain::metrics::{HarnessMetrics, MetricAggregator, MetricEvent};
use ecc_ports::metrics_store::{
    MetricsExportFormat, MetricsQuery, MetricsStore, MetricsStoreError,
};

/// In-memory test double for [`MetricsStore`].
///
/// All writes are held in a `Mutex<Vec<_>>`. Thread-safe and fully deterministic.
pub struct InMemoryMetricsStore {
    events: Mutex<Vec<MetricEvent>>,
    next_id: Mutex<i64>,
}

impl InMemoryMetricsStore {
    /// Create an empty store.
    pub fn new() -> Self {
        Self {
            events: Mutex::new(Vec::new()),
            next_id: Mutex::new(1),
        }
    }

    /// Return a snapshot of all stored events.
    pub fn snapshot(&self) -> Vec<MetricEvent> {
        self.events.lock().expect("lock poisoned").clone()
    }
}

impl Default for InMemoryMetricsStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsStore for InMemoryMetricsStore {
    fn record(&self, event: &MetricEvent) -> Result<i64, MetricsStoreError> {
        let id = {
            let mut next = self.next_id.lock().expect("lock poisoned");
            let id = *next;
            *next += 1;
            id
        };
        let mut stored = event.clone();
        stored.id = Some(id);
        self.events.lock().expect("lock poisoned").push(stored);
        Ok(id)
    }

    fn query(&self, query: &MetricsQuery) -> Result<Vec<MetricEvent>, MetricsStoreError> {
        let guard = self.events.lock().expect("lock poisoned");
        let filtered: Vec<MetricEvent> = guard
            .iter()
            .filter(|e| matches_query(e, query))
            .cloned()
            .collect();

        let results = if let Some(limit) = query.limit {
            filtered.into_iter().take(limit).collect()
        } else {
            filtered
        };

        Ok(results)
    }

    fn summarize(&self, query: &MetricsQuery) -> Result<HarnessMetrics, MetricsStoreError> {
        let events = self.query(query)?;
        Ok(MetricAggregator::summarize(&events))
    }

    fn prune(&self, older_than: Duration) -> Result<u64, MetricsStoreError> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let cutoff_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .saturating_sub(older_than)
            .as_secs();

        let mut guard = self.events.lock().expect("lock poisoned");
        let before = guard.len();
        guard.retain(|e| parse_timestamp_secs(&e.timestamp).is_none_or(|ts| ts >= cutoff_secs));
        Ok((before - guard.len()) as u64)
    }

    fn export(
        &self,
        query: &MetricsQuery,
        format: MetricsExportFormat,
    ) -> Result<String, MetricsStoreError> {
        let events = self.query(query)?;
        match format {
            MetricsExportFormat::Json => {
                let items: Vec<String> = events.iter().map(event_to_json).collect();
                Ok(format!("[{}]", items.join(",")))
            }
            MetricsExportFormat::Csv => {
                let mut rows = vec![
                    "id,event_type,session_id,timestamp,outcome,hook_id,duration_ms,error_message"
                        .to_string(),
                ];
                for e in &events {
                    rows.push(format!(
                        "{},{},{},{},{},{},{},{}",
                        e.id.map_or(String::new(), |v| v.to_string()),
                        e.event_type,
                        csv_escape(&e.session_id),
                        csv_escape(&e.timestamp),
                        e.outcome,
                        e.hook_id.as_deref().unwrap_or(""),
                        e.duration_ms.map_or(String::new(), |v| v.to_string()),
                        csv_escape(e.error_message.as_deref().unwrap_or("")),
                    ));
                }
                Ok(rows.join("\n"))
            }
        }
    }
}

fn matches_query(event: &MetricEvent, query: &MetricsQuery) -> bool {
    if let Some(ref sid) = query.session_id
        && event.session_id != *sid
    {
        return false;
    }
    if let Some(ref et) = query.event_type
        && event.event_type != *et
    {
        return false;
    }
    if let Some(ref outcome) = query.outcome
        && event.outcome != *outcome
    {
        return false;
    }
    if let Some((ref start, ref end)) = query.date_range
        && (event.timestamp.as_str() < start.as_str() || event.timestamp.as_str() > end.as_str())
    {
        return false;
    }
    if let Some(since) = query.since {
        use std::time::{SystemTime, UNIX_EPOCH};
        let cutoff = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .saturating_sub(since)
            .as_secs();
        let record_secs = parse_timestamp_secs(&event.timestamp).unwrap_or(0);
        if record_secs < cutoff {
            return false;
        }
    }
    true
}

/// Best-effort ISO-8601 timestamp to Unix seconds parser.
fn parse_timestamp_secs(ts: &str) -> Option<u64> {
    let ts = ts.trim_end_matches('Z').trim_end_matches("+00:00");
    let parts: Vec<&str> = ts.splitn(2, 'T').collect();
    if parts.len() != 2 {
        return None;
    }
    let date_parts: Vec<u32> = parts[0].split('-').filter_map(|p| p.parse().ok()).collect();
    let time_parts: Vec<u32> = parts[1]
        .splitn(4, ':')
        .filter_map(|p| p.parse().ok())
        .collect();
    if date_parts.len() != 3 || time_parts.len() < 2 {
        return None;
    }
    let year = date_parts[0] as u64;
    let month = date_parts[1] as u64;
    let day = date_parts[2] as u64;
    let hour = time_parts[0] as u64;
    let min = time_parts[1] as u64;
    let sec = time_parts.get(2).copied().unwrap_or(0) as u64;
    let days = (year - 1970) * 365 + (year - 1970) / 4 + month * 30 + day;
    Some(days * 86400 + hour * 3600 + min * 60 + sec)
}

fn event_to_json(e: &MetricEvent) -> String {
    format!(
        r#"{{"id":{},"event_type":"{}","session_id":"{}","timestamp":"{}","outcome":"{}"}}"#,
        e.id.map_or("null".to_string(), |v| v.to_string()),
        e.event_type,
        e.session_id,
        e.timestamp,
        e.outcome,
    )
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_domain::metrics::{CommitGateKind, MetricEvent, MetricEventType, MetricOutcome};

    fn hook(session: &str, ts: &str, outcome: MetricOutcome) -> MetricEvent {
        MetricEvent::hook_execution(session.into(), ts.into(), "h".into(), 100, outcome, None)
            .unwrap()
    }

    fn transition(session: &str, outcome: MetricOutcome) -> MetricEvent {
        MetricEvent::phase_transition(
            session.into(),
            "2026-04-06T10:00:00Z".into(),
            "plan".into(),
            "solution".into(),
            outcome,
            None,
        )
        .unwrap()
    }

    #[allow(dead_code)]
    fn agent(outcome: MetricOutcome, retry: Option<u32>) -> MetricEvent {
        MetricEvent::agent_spawn(
            "s1".into(),
            "2026-04-06T10:00:00Z".into(),
            "tdd".into(),
            outcome,
            retry,
        )
        .unwrap()
    }

    #[allow(dead_code)]
    fn commit(outcome: MetricOutcome) -> MetricEvent {
        MetricEvent::commit_gate(
            "s1".into(),
            "2026-04-06T10:00:00Z".into(),
            outcome,
            if outcome == MetricOutcome::Failure {
                vec![CommitGateKind::Test]
            } else {
                vec![]
            },
        )
        .unwrap()
    }

    // PC-015: record + query round-trip
    #[test]
    fn metrics_store_round_trip() {
        let store = InMemoryMetricsStore::new();
        let event = hook("s1", "2026-04-06T10:00:00Z", MetricOutcome::Success);

        let id = store.record(&event).unwrap();
        assert!(id > 0);

        let results = store.query(&MetricsQuery::default()).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, Some(id));
        assert_eq!(results[0].outcome, MetricOutcome::Success);
    }

    // PC-016: summarize computes HarnessMetrics
    #[test]
    fn metrics_store_summarize() {
        let store = InMemoryMetricsStore::new();
        store
            .record(&hook("s1", "2026-04-06T10:00:00Z", MetricOutcome::Success))
            .unwrap();
        store
            .record(&hook("s1", "2026-04-06T10:01:00Z", MetricOutcome::Failure))
            .unwrap();

        let metrics = store.summarize(&MetricsQuery::default()).unwrap();
        assert_eq!(metrics.total_events, 2);
        assert!((metrics.hook_success_rate.unwrap() - 0.5).abs() < f64::EPSILON);
    }

    // PC-017: empty summarize returns None rates
    #[test]
    fn metrics_store_empty_summarize() {
        let store = InMemoryMetricsStore::new();
        let metrics = store.summarize(&MetricsQuery::default()).unwrap();
        assert!(metrics.hook_success_rate.is_none());
        assert!(metrics.phase_gate_violation_rate.is_none());
        assert!(metrics.agent_failure_recovery_rate.is_none());
        assert!(metrics.commit_atomicity_score.is_none());
        assert_eq!(metrics.total_events, 0);
    }

    // PC-018: query filters
    #[test]
    fn metrics_store_query_filters() {
        let store = InMemoryMetricsStore::new();
        store
            .record(&hook("s1", "2026-04-06T10:00:00Z", MetricOutcome::Success))
            .unwrap();
        store
            .record(&hook("s2", "2026-04-06T10:01:00Z", MetricOutcome::Failure))
            .unwrap();
        store
            .record(&transition("s3", MetricOutcome::Success))
            .unwrap();

        // Filter by session
        let q = MetricsQuery {
            session_id: Some("s1".into()),
            ..Default::default()
        };
        assert_eq!(store.query(&q).unwrap().len(), 1);

        // Filter by event type
        let q = MetricsQuery {
            event_type: Some(MetricEventType::PhaseTransition),
            ..Default::default()
        };
        assert_eq!(store.query(&q).unwrap().len(), 1);

        // Filter by outcome
        let q = MetricsQuery {
            outcome: Some(MetricOutcome::Failure),
            ..Default::default()
        };
        assert_eq!(store.query(&q).unwrap().len(), 1);
    }

    // PC-019: prune removes old events
    #[test]
    fn metrics_store_prune() {
        let store = InMemoryMetricsStore::new();
        // Old event
        store
            .record(&hook("s1", "2020-01-01T00:00:00Z", MetricOutcome::Success))
            .unwrap();
        // Recent event (use current-ish timestamp)
        store
            .record(&hook("s1", "2026-04-06T10:00:00Z", MetricOutcome::Success))
            .unwrap();

        let removed = store.prune(Duration::from_secs(30 * 86400)).unwrap();
        assert_eq!(removed, 1);
        assert_eq!(store.query(&MetricsQuery::default()).unwrap().len(), 1);
    }

    // PC-020: export JSON and CSV
    #[test]
    fn metrics_store_export() {
        let store = InMemoryMetricsStore::new();
        store
            .record(&hook("s1", "2026-04-06T10:00:00Z", MetricOutcome::Success))
            .unwrap();

        let json = store
            .export(&MetricsQuery::default(), MetricsExportFormat::Json)
            .unwrap();
        assert!(json.starts_with('['));
        assert!(json.ends_with(']'));
        assert!(json.contains("hook_execution"));
        assert!(json.contains("s1"));

        let csv = store
            .export(&MetricsQuery::default(), MetricsExportFormat::Csv)
            .unwrap();
        assert!(csv.contains("id,event_type,session_id"));
        assert!(csv.contains("hook_execution"));
    }

    // PC-035: time-range query filter
    #[test]
    fn metrics_store_time_range_filter() {
        let store = InMemoryMetricsStore::new();
        store
            .record(&hook("s1", "2026-01-01T00:00:00Z", MetricOutcome::Success))
            .unwrap();
        store
            .record(&hook("s1", "2026-03-01T00:00:00Z", MetricOutcome::Success))
            .unwrap();
        store
            .record(&hook("s1", "2026-05-01T00:00:00Z", MetricOutcome::Success))
            .unwrap();

        let q = MetricsQuery {
            date_range: Some(("2026-02-01T00:00:00Z".into(), "2026-04-01T00:00:00Z".into())),
            ..Default::default()
        };
        let results = store.query(&q).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].timestamp, "2026-03-01T00:00:00Z");
    }
}
