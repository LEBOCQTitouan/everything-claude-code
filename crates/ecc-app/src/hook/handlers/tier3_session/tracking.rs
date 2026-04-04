//! Tracking hooks — session evaluation and cost tracking.

use tracing::warn;

use crate::hook::{HookPorts, HookResult};
use ecc_domain::cost::{
    calculator::{CostCalculator, PricingTable},
    record::TokenUsageRecord,
    value_objects::{ModelId, TokenCount},
};
use ecc_domain::time::{datetime_from_epoch, format_datetime};
use std::path::Path;

use super::epoch_secs;
use super::helpers::to_u64;

/// evaluate-session: count messages and log evaluation hint.
pub fn evaluate_session(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "evaluate_session", "executing handler");
    let home = match ports.env.home_dir() {
        Some(h) => h,
        None => return HookResult::passthrough(stdin),
    };

    // Parse transcript_path from stdin JSON
    let transcript_path = serde_json::from_str::<serde_json::Value>(stdin)
        .ok()
        .and_then(|v| v.get("transcript_path")?.as_str().map(|s| s.to_string()))
        .or_else(|| ports.env.var("CLAUDE_TRANSCRIPT_PATH"));

    let transcript_path = match transcript_path {
        Some(tp) => tp,
        None => return HookResult::passthrough(stdin),
    };

    let path = Path::new(&transcript_path);
    if !ports.fs.exists(path) {
        return HookResult::passthrough(stdin);
    }

    let content = match ports.fs.read_to_string(path) {
        Ok(c) => c,
        Err(_) => return HookResult::passthrough(stdin),
    };

    // Count user messages
    let message_count = content
        .lines()
        .filter(|line| line.contains("\"type\"") && line.contains("\"user\""))
        .count();

    let min_session_length: usize = 10;

    if message_count < min_session_length {
        let msg = format!(
            "[ContinuousLearning] Session too short ({} messages), skipping\n",
            message_count
        );
        return HookResult::warn(stdin, &msg);
    }

    let learned_dir = home.join(".claude").join("learned-skills");
    if let Err(e) = ports.fs.create_dir_all(&learned_dir) {
        warn!("Cannot create learned-skills dir: {}", e);
    }

    let msg = format!(
        "[ContinuousLearning] Session has {} messages - evaluate for extractable patterns\n\
         [ContinuousLearning] Save learned skills to: {}\n",
        message_count,
        learned_dir.display()
    );
    HookResult::warn(stdin, &msg)
}

/// cost-tracker: estimate cost and persist via CostStore (or JSONL fallback).
pub fn cost_tracker(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "cost_tracker", "executing handler");
    let home = match ports.env.home_dir() {
        Some(h) => h,
        None => return HookResult::passthrough(stdin),
    };

    let input: serde_json::Value = match serde_json::from_str(stdin) {
        Ok(v) => v,
        Err(_) => return HookResult::passthrough(stdin),
    };

    let usage = input
        .get("usage")
        .or_else(|| input.get("token_usage"))
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    let input_tokens = to_u64(&usage, "input_tokens")
        .or_else(|| to_u64(&usage, "prompt_tokens"))
        .unwrap_or(0);
    let output_tokens = to_u64(&usage, "output_tokens")
        .or_else(|| to_u64(&usage, "completion_tokens"))
        .unwrap_or(0);
    let thinking_tokens = to_u64(&usage, "thinking_tokens").unwrap_or(0);

    let model_str = input
        .get("model")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            ports
                .env
                .var("CLAUDE_MODEL")
                .unwrap_or_else(|| "unknown".to_string())
        });

    let agent_type = input
        .get("agent_type")
        .and_then(|v| v.as_str())
        .unwrap_or("main")
        .to_string();

    let session_id = ports
        .env
        .var("CLAUDE_SESSION_ID")
        .unwrap_or_else(|| "default".to_string());

    let timestamp = format_datetime(&datetime_from_epoch(epoch_secs()));

    // Try to persist via CostStore; fall back to JSONL otherwise.
    if let Some(store) = ports.cost_store {
        let table = PricingTable::default();
        let model_id = ModelId::new(&model_str).unwrap_or_else(|_| {
            ModelId::new("unknown").expect("'unknown' is a valid ModelId")
        });
        let cost = CostCalculator::estimate(
            &table,
            &model_id,
            TokenCount::new(input_tokens),
            TokenCount::new(output_tokens),
            TokenCount::new(thinking_tokens),
        );
        let record = TokenUsageRecord {
            record_id: None,
            session_id,
            timestamp,
            model: model_id,
            input_tokens: TokenCount::new(input_tokens),
            output_tokens: TokenCount::new(output_tokens),
            thinking_tokens: TokenCount::new(thinking_tokens),
            estimated_cost: cost,
            agent_type,
            parent_session_id: None,
        };
        if let Err(e) = store.append(&record) {
            warn!("CostStore::append failed: {}", e);
        }
    } else {
        // JSONL fallback
        let metrics_dir = home.join(".claude").join("metrics");
        if let Err(e) = ports.fs.create_dir_all(&metrics_dir) {
            warn!("Cannot create metrics dir: {}", e);
        }

        let table = PricingTable::default();
        let model_id = ModelId::new(&model_str).unwrap_or_else(|_| {
            ModelId::new("unknown").expect("'unknown' is a valid ModelId")
        });
        let cost = CostCalculator::estimate(
            &table,
            &model_id,
            TokenCount::new(input_tokens),
            TokenCount::new(output_tokens),
            TokenCount::new(thinking_tokens),
        );

        let row = serde_json::json!({
            "timestamp": timestamp,
            "session_id": session_id,
            "model": model_str,
            "input_tokens": input_tokens,
            "output_tokens": output_tokens,
            "thinking_tokens": thinking_tokens,
            "estimated_cost_usd": cost.value(),
        });

        let costs_file = metrics_dir.join("costs.jsonl");
        let existing = ports.fs.read_to_string(&costs_file).unwrap_or_default();
        let new_content = format!("{}{}\n", existing, row);
        if let Err(e) = ports.fs.write(&costs_file, &new_content) {
            super::log_write_failure(&costs_file, &e, None);
        }
    }

    HookResult::passthrough(stdin)
}
