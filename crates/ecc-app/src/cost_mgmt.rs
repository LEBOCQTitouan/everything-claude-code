use std::path::Path;
use std::time::Duration;

use ecc_domain::cost::calculator::CostSummary;
use ecc_domain::cost::record::TokenUsageRecord;
use ecc_ports::cost_store::{CostExportFormat, CostQuery, CostStore, CostStoreError};
use ecc_ports::fs::FileSystem;

/// Summary of a prune operation.
pub use ecc_domain::cost::calculator::CostBreakdown;

/// Dimension by which to break down cost records.
pub enum BreakdownBy {
    /// Group by agent type.
    Agent,
    /// Group by model.
    Model,
}

/// Comparison of cost summaries across two time windows.
#[derive(Debug)]
pub struct CostComparison {
    /// Summary for the "before" window.
    pub before: CostSummary,
    /// Summary for the "after" window.
    pub after: CostSummary,
}

/// Result of a migration operation.
#[derive(Debug, PartialEq)]
pub struct MigrateResult {
    /// Number of records successfully imported.
    pub imported: u64,
    /// Number of malformed lines skipped.
    pub skipped: u64,
    /// `true` if the source file was not found.
    pub not_found: bool,
}

/// Delegate summary to the store.
pub fn summary(store: &dyn CostStore, query: &CostQuery) -> Result<CostSummary, CostStoreError> {
    store.summary(query)
}

/// Compute per-agent or per-model breakdowns.
pub fn breakdown(
    store: &dyn CostStore,
    query: &CostQuery,
    by: BreakdownBy,
) -> Result<Vec<CostBreakdown>, CostStoreError> {
    let records = store.query(query)?;
    let breakdowns = aggregate_breakdowns(&records, &by);
    Ok(breakdowns)
}

/// Compare cost summaries across two query windows.
pub fn compare(
    store: &dyn CostStore,
    before: &CostQuery,
    after: &CostQuery,
) -> Result<CostComparison, CostStoreError> {
    Ok(CostComparison {
        before: store.summary(before)?,
        after: store.summary(after)?,
    })
}

/// Delegate export to the store.
pub fn export(
    store: &dyn CostStore,
    query: &CostQuery,
    format: CostExportFormat,
) -> Result<String, CostStoreError> {
    store.export(query, format)
}

/// Delegate prune to the store.
pub fn prune(store: &dyn CostStore, older_than: Duration) -> Result<u64, CostStoreError> {
    store.prune(older_than)
}

/// Migrate records from a JSONL file into the store.
///
/// Each line must be a valid JSON object with the token usage fields.
/// Malformed lines are skipped and counted in `MigrateResult::skipped`.
/// If the file is not found, returns `MigrateResult { not_found: true, .. }`.
pub fn migrate(
    store: &dyn CostStore,
    fs: &dyn FileSystem,
    jsonl_path: &Path,
) -> Result<MigrateResult, CostStoreError> {
    let content = match fs.read_to_string(jsonl_path) {
        Ok(c) => c,
        Err(_) => {
            return Ok(MigrateResult {
                imported: 0,
                skipped: 0,
                not_found: true,
            });
        }
    };

    let mut imported = 0u64;
    let mut skipped = 0u64;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match parse_jsonl_record(line) {
            Some(record) => {
                store
                    .append(&record)
                    .map_err(|e| CostStoreError::Database(e.to_string()))?;
                imported += 1;
            }
            None => {
                skipped += 1;
            }
        }
    }

    Ok(MigrateResult {
        imported,
        skipped,
        not_found: false,
    })
}

fn aggregate_breakdowns(records: &[TokenUsageRecord], by: &BreakdownBy) -> Vec<CostBreakdown> {
    use ecc_domain::cost::calculator::CostCalculator;
    use std::collections::HashMap;

    let mut order: Vec<String> = Vec::new();
    let mut groups: HashMap<String, Vec<TokenUsageRecord>> = HashMap::new();

    for record in records {
        let key = match by {
            BreakdownBy::Agent => record.agent_type.clone(),
            BreakdownBy::Model => record.model.as_str().to_owned(),
        };
        if !groups.contains_key(&key) {
            order.push(key.clone());
        }
        groups.entry(key).or_default().push(record.clone());
    }

    order
        .into_iter()
        .map(|key| {
            let group = groups.remove(&key).unwrap_or_default();
            let summary = CostCalculator::summarize(&group);
            // Return a CostBreakdown using the first record's model (for Model) or a
            // synthetic ModelId (for Agent). We reuse the first model as a proxy key.
            let model = group.first().map(|r| r.model.clone()).unwrap_or_else(|| {
                ecc_domain::cost::value_objects::ModelId::new(&key).unwrap_or_else(|_| {
                    ecc_domain::cost::value_objects::ModelId::new("unknown").unwrap()
                })
            });
            CostBreakdown {
                model,
                cost: summary.total_cost,
                input_tokens: summary.total_input_tokens,
                output_tokens: summary.total_output_tokens,
                thinking_tokens: summary.total_thinking_tokens,
                record_count: summary.record_count,
            }
        })
        .collect()
}

/// Best-effort JSONL line parser — returns `None` for any parse failure.
fn parse_jsonl_record(line: &str) -> Option<TokenUsageRecord> {
    use ecc_domain::cost::value_objects::{ModelId, Money, TokenCount};

    let v: serde_json::Value = serde_json::from_str(line).ok()?;
    let obj = v.as_object()?;

    let session_id = obj.get("session_id")?.as_str()?.to_owned();
    let timestamp = obj.get("timestamp")?.as_str()?.to_owned();
    let model_str = obj.get("model")?.as_str()?;
    let model = ModelId::new(model_str).ok()?;
    let input_tokens = TokenCount::new(obj.get("input_tokens")?.as_u64()?);
    let output_tokens = TokenCount::new(obj.get("output_tokens")?.as_u64()?);
    let thinking_tokens = TokenCount::new(obj.get("thinking_tokens")?.as_u64().unwrap_or(0));
    let estimated_cost = Money::usd(obj.get("estimated_cost")?.as_f64()?);
    let agent_type = obj
        .get("agent_type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_owned();
    let parent_session_id = obj
        .get("parent_session_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_owned());

    Some(TokenUsageRecord {
        record_id: None,
        session_id,
        timestamp,
        model,
        input_tokens,
        output_tokens,
        thinking_tokens,
        estimated_cost,
        agent_type,
        parent_session_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_domain::cost::value_objects::{ModelId, Money, TokenCount};
    use ecc_test_support::{InMemoryCostStore, InMemoryFileSystem};

    fn make_record(
        model: &str,
        agent: &str,
        input: u64,
        output: u64,
        cost: f64,
    ) -> TokenUsageRecord {
        TokenUsageRecord {
            record_id: None,
            session_id: "sess-001".into(),
            timestamp: "2026-04-04T10:00:00Z".into(),
            model: ModelId::new(model).unwrap(),
            input_tokens: TokenCount::new(input),
            output_tokens: TokenCount::new(output),
            thinking_tokens: TokenCount::new(0),
            estimated_cost: Money::usd(cost),
            agent_type: agent.into(),
            parent_session_id: None,
        }
    }

    // PC-019: summary delegates to store
    #[test]
    fn summary_delegates_to_store() {
        let store = InMemoryCostStore::new();
        store.seed(vec![
            make_record("claude-sonnet-4-6", "main", 1000, 500, 0.0105),
            make_record("claude-haiku-4-5", "reviewer", 500, 200, 0.0015),
        ]);
        let result = summary(&store, &CostQuery::default()).unwrap();
        assert_eq!(result.record_count, 2);
        assert!(result.total_cost.value() > 0.0);
    }

    // PC-020: prune delegates to store
    #[test]
    fn prune_delegates_to_store() {
        let store = InMemoryCostStore::new();
        // Seed a record with a timestamp far in the past (epoch)
        let mut old_record = make_record("claude-sonnet-4-6", "main", 100, 50, 0.001);
        old_record.timestamp = "1970-01-01T00:00:00Z".into();
        store.seed(vec![old_record]);

        // Prune anything older than 1 second — the 1970 record should be removed
        let removed = prune(&store, Duration::from_secs(1)).unwrap();
        assert_eq!(removed, 1);
        assert_eq!(store.snapshot().len(), 0);
    }

    // PC-021: migrate imports valid JSONL
    #[test]
    fn migrate_imports_valid_jsonl() {
        let store = InMemoryCostStore::new();
        let fs = InMemoryFileSystem::new().with_file(
            "/data/records.jsonl",
            r#"{"session_id":"s1","timestamp":"2026-04-04T10:00:00Z","model":"claude-sonnet-4-6","input_tokens":100,"output_tokens":50,"thinking_tokens":0,"estimated_cost":0.001,"agent_type":"main"}
{"session_id":"s2","timestamp":"2026-04-04T10:01:00Z","model":"claude-haiku-4-5","input_tokens":200,"output_tokens":80,"thinking_tokens":0,"estimated_cost":0.0005,"agent_type":"reviewer"}
{"session_id":"s3","timestamp":"2026-04-04T10:02:00Z","model":"claude-sonnet-4-6","input_tokens":300,"output_tokens":120,"thinking_tokens":0,"estimated_cost":0.002,"agent_type":"main"}
"#,
        );
        let result = migrate(&store, &fs, Path::new("/data/records.jsonl")).unwrap();
        assert_eq!(
            result,
            MigrateResult {
                imported: 3,
                skipped: 0,
                not_found: false
            }
        );
        assert_eq!(store.snapshot().len(), 3);
    }

    // PC-022: migrate skips malformed lines
    #[test]
    fn migrate_skips_malformed_lines() {
        let store = InMemoryCostStore::new();
        let fs = InMemoryFileSystem::new().with_file(
            "/data/records.jsonl",
            r#"{"session_id":"s1","timestamp":"2026-04-04T10:00:00Z","model":"claude-sonnet-4-6","input_tokens":100,"output_tokens":50,"thinking_tokens":0,"estimated_cost":0.001,"agent_type":"main"}
not valid json at all
{"session_id":"s3","timestamp":"2026-04-04T10:02:00Z","model":"claude-sonnet-4-6","input_tokens":300,"output_tokens":120,"thinking_tokens":0,"estimated_cost":0.002,"agent_type":"main"}
"#,
        );
        let result = migrate(&store, &fs, Path::new("/data/records.jsonl")).unwrap();
        assert_eq!(
            result,
            MigrateResult {
                imported: 2,
                skipped: 1,
                not_found: false
            }
        );
        assert_eq!(store.snapshot().len(), 2);
    }

    // PC-023: migrate missing file
    #[test]
    fn migrate_missing_file() {
        let store = InMemoryCostStore::new();
        let fs = InMemoryFileSystem::new();
        let result = migrate(&store, &fs, Path::new("/nonexistent/file.jsonl")).unwrap();
        assert_eq!(
            result,
            MigrateResult {
                imported: 0,
                skipped: 0,
                not_found: true
            }
        );
    }

    // PC-036: breakdown delegates to store
    #[test]
    fn breakdown_delegates_to_store() {
        let store = InMemoryCostStore::new();
        store.seed(vec![
            make_record("claude-sonnet-4-6", "main", 1000, 500, 0.0105),
            make_record("claude-haiku-4-5", "reviewer", 500, 200, 0.0015),
            make_record("claude-sonnet-4-6", "main", 200, 100, 0.002),
        ]);
        let result = breakdown(&store, &CostQuery::default(), BreakdownBy::Model).unwrap();
        assert_eq!(result.len(), 2);
    }

    // PC-037: compare delegates to store
    #[test]
    fn compare_delegates_to_store() {
        let store = InMemoryCostStore::new();
        store.seed(vec![
            make_record("claude-sonnet-4-6", "main", 1000, 500, 0.0105),
            make_record("claude-haiku-4-5", "reviewer", 500, 200, 0.0015),
        ]);
        let before_query = CostQuery {
            model: Some("sonnet".into()),
            ..Default::default()
        };
        let after_query = CostQuery {
            model: Some("haiku".into()),
            ..Default::default()
        };
        let result = compare(&store, &before_query, &after_query).unwrap();
        assert_eq!(result.before.record_count, 1);
        assert_eq!(result.after.record_count, 1);
    }

    // PC-038: export delegates to store
    #[test]
    fn export_delegates_to_store() {
        let store = InMemoryCostStore::new();
        store.seed(vec![make_record(
            "claude-sonnet-4-6",
            "main",
            1000,
            500,
            0.0105,
        )]);
        let output = export(&store, &CostQuery::default(), CostExportFormat::Json).unwrap();
        assert!(!output.is_empty());
        assert!(output.starts_with('['));
    }
}
