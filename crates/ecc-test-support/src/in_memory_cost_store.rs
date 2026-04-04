//! In-memory test double for [`CostStore`].

use std::sync::Mutex;
use std::time::Duration;

use ecc_domain::cost::{
    calculator::{CostCalculator, CostSummary},
    record::TokenUsageRecord,
    value_objects::RecordId,
};
use ecc_ports::cost_store::{CostExportFormat, CostQuery, CostStore, CostStoreError};

/// In-memory test double for [`CostStore`].
///
/// All writes are held in a `Mutex<Vec<_>>`. Thread-safe and fully deterministic.
pub struct InMemoryCostStore {
    records: Mutex<Vec<TokenUsageRecord>>,
    next_id: Mutex<i64>,
}

impl InMemoryCostStore {
    /// Create an empty store.
    pub fn new() -> Self {
        Self {
            records: Mutex::new(Vec::new()),
            next_id: Mutex::new(1),
        }
    }

    /// Seed the store with pre-built records (for test setup).
    pub fn seed(&self, records: Vec<TokenUsageRecord>) {
        let mut guard = self.records.lock().expect("lock poisoned");
        guard.extend(records);
    }

    /// Return a snapshot of all stored records.
    pub fn snapshot(&self) -> Vec<TokenUsageRecord> {
        self.records.lock().expect("lock poisoned").clone()
    }
}

impl Default for InMemoryCostStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CostStore for InMemoryCostStore {
    fn append(&self, _record: &TokenUsageRecord) -> Result<RecordId, CostStoreError> {
        Err(CostStoreError::Io("not implemented".to_string()))
    }

    fn query(&self, _query: &CostQuery) -> Result<Vec<TokenUsageRecord>, CostStoreError> {
        Err(CostStoreError::Io("not implemented".to_string()))
    }

    fn summary(&self, _query: &CostQuery) -> Result<CostSummary, CostStoreError> {
        Err(CostStoreError::Io("not implemented".to_string()))
    }

    fn prune(&self, _older_than: Duration) -> Result<u64, CostStoreError> {
        Err(CostStoreError::Io("not implemented".to_string()))
    }

    fn export(&self, _query: &CostQuery, _format: CostExportFormat) -> Result<String, CostStoreError> {
        Err(CostStoreError::Io("not implemented".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_domain::cost::value_objects::{ModelId, Money, TokenCount};

    fn make_record(model: &str, input: u64, output: u64, cost: f64) -> TokenUsageRecord {
        TokenUsageRecord {
            record_id: None,
            session_id: "sess-001".to_string(),
            timestamp: "2026-04-04T10:00:00Z".to_string(),
            model: ModelId::new(model).unwrap(),
            input_tokens: TokenCount::new(input),
            output_tokens: TokenCount::new(output),
            thinking_tokens: TokenCount::new(0),
            estimated_cost: Money::usd(cost),
            agent_type: "main".to_string(),
            parent_session_id: None,
        }
    }

    // PC-012: InMemoryCostStore round-trip
    #[test]
    fn append_and_query_round_trip() {
        let store = InMemoryCostStore::new();
        let record = make_record("claude-sonnet-4-6", 1000, 500, 0.0105);

        let id = store.append(&record).expect("append should succeed");
        assert_eq!(id, RecordId(1));

        let results = store.query(&CostQuery::default()).expect("query should succeed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].record_id, Some(RecordId(1)));
        assert_eq!(results[0].model.as_str(), "claude-sonnet-4-6");
        assert_eq!(results[0].input_tokens.value(), 1000);
        assert_eq!(results[0].output_tokens.value(), 500);
        assert_eq!(results[0].estimated_cost.value(), 0.0105);
    }

    // PC-013: InMemoryCostStore summary aggregates
    #[test]
    fn summary_aggregates() {
        let store = InMemoryCostStore::new();

        store.append(&make_record("claude-haiku-4-5", 500, 200, 0.001_5)).unwrap();
        store.append(&make_record("claude-sonnet-4-6", 1000, 400, 0.012)).unwrap();
        store.append(&make_record("claude-haiku-4-5", 300, 100, 0.000_8)).unwrap();

        let summary = store.summary(&CostQuery::default()).expect("summary should succeed");

        assert_eq!(summary.record_count, 3);
        assert_eq!(summary.breakdowns.len(), 2);

        let haiku_bd = summary.breakdowns.iter().find(|b| b.model.as_str() == "claude-haiku-4-5").unwrap();
        assert_eq!(haiku_bd.record_count, 2);
        assert_eq!(haiku_bd.input_tokens.value(), 800);
        assert_eq!(haiku_bd.output_tokens.value(), 300);

        let sonnet_bd = summary.breakdowns.iter().find(|b| b.model.as_str() == "claude-sonnet-4-6").unwrap();
        assert_eq!(sonnet_bd.record_count, 1);
        assert_eq!(sonnet_bd.input_tokens.value(), 1000);
    }
}
