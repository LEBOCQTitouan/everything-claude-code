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
    fn append(&self, record: &TokenUsageRecord) -> Result<RecordId, CostStoreError> {
        let id = {
            let mut next = self.next_id.lock().expect("lock poisoned");
            let id = *next;
            *next += 1;
            id
        };
        let mut stored = record.clone();
        stored.record_id = Some(RecordId(id));
        self.records.lock().expect("lock poisoned").push(stored);
        Ok(RecordId(id))
    }

    fn query(&self, query: &CostQuery) -> Result<Vec<TokenUsageRecord>, CostStoreError> {
        let guard = self.records.lock().expect("lock poisoned");
        let filtered: Vec<TokenUsageRecord> = guard
            .iter()
            .filter(|r| matches_query(r, query))
            .cloned()
            .collect();

        let results = if let Some(limit) = query.limit {
            filtered.into_iter().take(limit).collect()
        } else {
            filtered
        };

        Ok(results)
    }

    fn summary(&self, query: &CostQuery) -> Result<CostSummary, CostStoreError> {
        let records = self.query(query)?;
        Ok(CostCalculator::summarize(&records))
    }

    fn prune(&self, older_than: Duration) -> Result<u64, CostStoreError> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let cutoff_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .saturating_sub(older_than)
            .as_secs();

        let mut guard = self.records.lock().expect("lock poisoned");
        let before = guard.len();
        guard.retain(|r| {
            parse_timestamp_secs(&r.timestamp)
                .is_none_or(|ts| ts >= cutoff_secs)
        });
        Ok((before - guard.len()) as u64)
    }

    fn export(
        &self,
        query: &CostQuery,
        format: CostExportFormat,
    ) -> Result<String, CostStoreError> {
        let records = self.query(query)?;
        match format {
            CostExportFormat::Json => {
                let items: Vec<String> = records
                    .iter()
                    .map(|r| {
                        format!(
                            r#"{{"record_id":{record_id},"session_id":{session_id},"timestamp":{timestamp},"model":{model},"input_tokens":{input_tokens},"output_tokens":{output_tokens},"thinking_tokens":{thinking_tokens},"estimated_cost":{estimated_cost},"agent_type":{agent_type}}}"#,
                            record_id = r
                                .record_id
                                .map_or("null".to_string(), |v| v.0.to_string()),
                            session_id = json_str(&r.session_id),
                            timestamp = json_str(&r.timestamp),
                            model = json_str(r.model.as_str()),
                            input_tokens = r.input_tokens.value(),
                            output_tokens = r.output_tokens.value(),
                            thinking_tokens = r.thinking_tokens.value(),
                            estimated_cost = r.estimated_cost.value(),
                            agent_type = json_str(&r.agent_type),
                        )
                    })
                    .collect();
                Ok(format!("[{}]", items.join(",")))
            }
            CostExportFormat::Csv => {
                let mut rows = vec!["record_id,session_id,timestamp,model,input_tokens,output_tokens,thinking_tokens,estimated_cost,agent_type".to_string()];
                for r in &records {
                    rows.push(format!(
                        "{},{},{},{},{},{},{},{},{}",
                        r.record_id.map_or("".to_string(), |v| v.0.to_string()),
                        csv_escape(&r.session_id),
                        csv_escape(&r.timestamp),
                        csv_escape(r.model.as_str()),
                        r.input_tokens.value(),
                        r.output_tokens.value(),
                        r.thinking_tokens.value(),
                        r.estimated_cost.value(),
                        csv_escape(&r.agent_type),
                    ));
                }
                Ok(rows.join("\n"))
            }
        }
    }
}

fn matches_query(record: &TokenUsageRecord, query: &CostQuery) -> bool {
    if let Some(ref model) = query.model
        && !record.model.as_str().contains(model.as_str())
    {
        return false;
    }
    if let Some(ref agent_type) = query.agent_type
        && record.agent_type != *agent_type
    {
        return false;
    }
    if let Some(ref session_id) = query.session_id
        && record.session_id != *session_id
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
        let record_secs = parse_timestamp_secs(&record.timestamp).unwrap_or(0);
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

fn json_str(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
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

        store
            .append(&make_record("claude-haiku-4-5", 500, 200, 0.001_5))
            .unwrap();
        store
            .append(&make_record("claude-sonnet-4-6", 1000, 400, 0.012))
            .unwrap();
        store
            .append(&make_record("claude-haiku-4-5", 300, 100, 0.000_8))
            .unwrap();

        let summary = store
            .summary(&CostQuery::default())
            .expect("summary should succeed");

        assert_eq!(summary.record_count, 3);
        assert_eq!(summary.breakdowns.len(), 2);

        let haiku_bd = summary
            .breakdowns
            .iter()
            .find(|b| b.model.as_str() == "claude-haiku-4-5")
            .unwrap();
        assert_eq!(haiku_bd.record_count, 2);
        assert_eq!(haiku_bd.input_tokens.value(), 800);
        assert_eq!(haiku_bd.output_tokens.value(), 300);

        let sonnet_bd = summary
            .breakdowns
            .iter()
            .find(|b| b.model.as_str() == "claude-sonnet-4-6")
            .unwrap();
        assert_eq!(sonnet_bd.record_count, 1);
        assert_eq!(sonnet_bd.input_tokens.value(), 1000);
    }
}
