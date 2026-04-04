//! TokenUsageRecord entity.

use crate::cost::value_objects::{ModelId, Money, RecordId, TokenCount};

/// A single recorded token-usage event.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenUsageRecord {
    /// Optional primary key (None until persisted).
    pub record_id: Option<RecordId>,
    /// Session that produced the tokens.
    pub session_id: String,
    /// ISO-8601 timestamp.
    pub timestamp: String,
    /// Model that was called.
    pub model: ModelId,
    /// Number of input tokens consumed.
    pub input_tokens: TokenCount,
    /// Number of output tokens produced.
    pub output_tokens: TokenCount,
    /// Number of thinking tokens used.
    pub thinking_tokens: TokenCount,
    /// Estimated cost at time of recording.
    pub estimated_cost: Money,
    /// Agent type string (e.g. "orchestrator", "reviewer").
    pub agent_type: String,
    /// Parent session ID for nested sessions.
    pub parent_session_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // PC-005: TokenUsageRecord construction with all fields
    #[test]
    fn record_construction_all_fields() {
        let model = ModelId::new("claude-sonnet-4-6").unwrap();
        let record = TokenUsageRecord {
            record_id: Some(RecordId(1)),
            session_id: "sess-abc".to_owned(),
            timestamp: "2026-04-04T00:00:00Z".to_owned(),
            model: model.clone(),
            input_tokens: TokenCount::new(1000),
            output_tokens: TokenCount::new(500),
            thinking_tokens: TokenCount::new(200),
            estimated_cost: Money::usd(0.01),
            agent_type: "orchestrator".to_owned(),
            parent_session_id: Some("parent-sess".to_owned()),
        };

        assert_eq!(record.record_id, Some(RecordId(1)));
        assert_eq!(record.session_id, "sess-abc");
        assert_eq!(record.timestamp, "2026-04-04T00:00:00Z");
        assert_eq!(record.model, model);
        assert_eq!(record.input_tokens.value(), 1000);
        assert_eq!(record.output_tokens.value(), 500);
        assert_eq!(record.thinking_tokens.value(), 200);
        assert_eq!(record.estimated_cost.value(), 0.01);
        assert_eq!(record.agent_type, "orchestrator");
        assert_eq!(record.parent_session_id, Some("parent-sess".to_owned()));
    }
}
