//! Cost calculator: PricingTable, CostCalculator, CostSummary, CostBreakdown.

use std::collections::HashMap;

use crate::cost::record::TokenUsageRecord;
use crate::cost::value_objects::{CostRate, ModelId, Money, TokenCount};

/// Maps model name fragments to per-token pricing rates.
pub struct PricingTable {
    rates: HashMap<String, CostRate>,
    fallback: CostRate,
}

impl PricingTable {
    /// Construct a `PricingTable` from explicit entries and a fallback rate.
    pub fn new(rates: HashMap<String, CostRate>, fallback: CostRate) -> Self {
        Self { rates, fallback }
    }

    /// Look up the rate for `model`.
    ///
    /// Performs a case-insensitive substring match against stored keys.
    /// Falls back to the fallback rate when no key matches.
    pub fn rate_for(&self, model: &ModelId) -> &CostRate {
        let name = model.as_str();
        for (key, rate) in &self.rates {
            if name.contains(key.as_str()) {
                return rate;
            }
        }
        &self.fallback
    }
}

impl Default for PricingTable {
    /// Returns hardcoded rates:
    /// - haiku:  $1 / $5 per MTok (input / output), thinking at output rate
    /// - sonnet: $3 / $15 per MTok
    /// - opus:   $15 / $75 per MTok
    fn default() -> Self {
        let mut rates = HashMap::new();

        rates.insert(
            "haiku".to_owned(),
            CostRate {
                input_per_mtok: Money::usd(1.0),
                output_per_mtok: Money::usd(5.0),
                thinking_per_mtok: Money::usd(5.0),
            },
        );
        rates.insert(
            "sonnet".to_owned(),
            CostRate {
                input_per_mtok: Money::usd(3.0),
                output_per_mtok: Money::usd(15.0),
                thinking_per_mtok: Money::usd(15.0),
            },
        );
        rates.insert(
            "opus".to_owned(),
            CostRate {
                input_per_mtok: Money::usd(15.0),
                output_per_mtok: Money::usd(75.0),
                thinking_per_mtok: Money::usd(75.0),
            },
        );

        let fallback = CostRate {
            input_per_mtok: Money::usd(3.0),
            output_per_mtok: Money::usd(15.0),
            thinking_per_mtok: Money::usd(15.0),
        };

        Self::new(rates, fallback)
    }
}

/// Per-model aggregated cost breakdown within a [`CostSummary`].
#[derive(Debug, Clone, PartialEq)]
pub struct CostBreakdown {
    /// The model this breakdown covers.
    pub model: ModelId,
    /// Total estimated cost for this model.
    pub cost: Money,
    /// Total input tokens for this model.
    pub input_tokens: TokenCount,
    /// Total output tokens for this model.
    pub output_tokens: TokenCount,
    /// Total thinking tokens for this model.
    pub thinking_tokens: TokenCount,
    /// Number of records included.
    pub record_count: usize,
}

/// Aggregate summary across all records.
#[derive(Debug, Clone, PartialEq)]
pub struct CostSummary {
    /// Grand total estimated cost.
    pub total_cost: Money,
    /// Grand total input tokens.
    pub total_input_tokens: TokenCount,
    /// Grand total output tokens.
    pub total_output_tokens: TokenCount,
    /// Grand total thinking tokens.
    pub total_thinking_tokens: TokenCount,
    /// Per-model breakdowns.
    pub breakdowns: Vec<CostBreakdown>,
    /// Total number of records summarised.
    pub record_count: usize,
}

/// Stateless cost calculation functions.
pub struct CostCalculator;

impl CostCalculator {
    /// Estimate the cost of a single inference call.
    ///
    /// Formula: `(input * input_rate + output * output_rate + thinking * thinking_rate) / 1_000_000`
    pub fn estimate(
        table: &PricingTable,
        model: &ModelId,
        input: TokenCount,
        output: TokenCount,
        thinking: TokenCount,
    ) -> Money {
        let rate = table.rate_for(model);
        let cost = (input.value() as f64 * rate.input_per_mtok.value()
            + output.value() as f64 * rate.output_per_mtok.value()
            + thinking.value() as f64 * rate.thinking_per_mtok.value())
            / 1_000_000.0;
        Money::usd(cost)
    }

    /// Summarise a slice of records, grouping by model.
    pub fn summarize(records: &[TokenUsageRecord]) -> CostSummary {
        // Use insertion-ordered map via Vec<(key, breakdown)>
        let mut order: Vec<String> = Vec::new();
        let mut map: HashMap<String, CostBreakdown> = HashMap::new();

        let mut total_cost = 0.0_f64;
        let mut total_input = 0_u64;
        let mut total_output = 0_u64;
        let mut total_thinking = 0_u64;

        for record in records {
            let key = record.model.as_str().to_owned();

            total_cost += record.estimated_cost.value();
            total_input += record.input_tokens.value();
            total_output += record.output_tokens.value();
            total_thinking += record.thinking_tokens.value();

            if let Some(bd) = map.get_mut(&key) {
                bd.cost = Money::usd(bd.cost.value() + record.estimated_cost.value());
                bd.input_tokens =
                    TokenCount::new(bd.input_tokens.value() + record.input_tokens.value());
                bd.output_tokens =
                    TokenCount::new(bd.output_tokens.value() + record.output_tokens.value());
                bd.thinking_tokens =
                    TokenCount::new(bd.thinking_tokens.value() + record.thinking_tokens.value());
                bd.record_count += 1;
            } else {
                order.push(key.clone());
                map.insert(
                    key,
                    CostBreakdown {
                        model: record.model.clone(),
                        cost: record.estimated_cost,
                        input_tokens: record.input_tokens,
                        output_tokens: record.output_tokens,
                        thinking_tokens: record.thinking_tokens,
                        record_count: 1,
                    },
                );
            }
        }

        let breakdowns = order.into_iter().filter_map(|k| map.remove(&k)).collect();

        CostSummary {
            total_cost: Money::usd(total_cost),
            total_input_tokens: TokenCount::new(total_input),
            total_output_tokens: TokenCount::new(total_output),
            total_thinking_tokens: TokenCount::new(total_thinking),
            breakdowns,
            record_count: records.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cost::record::TokenUsageRecord;
    use crate::cost::value_objects::RecordId;

    fn make_record(
        model: &str,
        input: u64,
        output: u64,
        thinking: u64,
        cost: f64,
    ) -> TokenUsageRecord {
        TokenUsageRecord {
            record_id: Some(RecordId(1)),
            session_id: "sess".to_owned(),
            timestamp: "2026-04-04T00:00:00Z".to_owned(),
            model: ModelId::new(model).unwrap(),
            input_tokens: TokenCount::new(input),
            output_tokens: TokenCount::new(output),
            thinking_tokens: TokenCount::new(thinking),
            estimated_cost: Money::usd(cost),
            agent_type: "test".to_owned(),
            parent_session_id: None,
        }
    }

    // PC-006: PricingTable haiku rates
    #[test]
    fn haiku_rates() {
        let table = PricingTable::default();
        let model = ModelId::new("claude-haiku-4-5").unwrap();
        let rate = table.rate_for(&model);
        assert_eq!(rate.input_per_mtok.value(), 1.0);
        assert_eq!(rate.output_per_mtok.value(), 5.0);
        assert_eq!(rate.thinking_per_mtok.value(), 5.0);
    }

    // PC-007: Sonnet cost estimate
    #[test]
    fn sonnet_cost_estimate() {
        let table = PricingTable::default();
        let model = ModelId::new("claude-sonnet-4-6").unwrap();
        // 1000 input * $3/MTok + 500 output * $15/MTok = $0.003 + $0.0075 = $0.0105
        let cost = CostCalculator::estimate(
            &table,
            &model,
            TokenCount::new(1000),
            TokenCount::new(500),
            TokenCount::new(0),
        );
        assert_eq!(cost.value(), 0.0105);
    }

    // PC-008: Opus with thinking tokens
    #[test]
    fn opus_with_thinking_tokens() {
        let table = PricingTable::default();
        let model = ModelId::new("claude-opus-4-6").unwrap();
        // 500 input * $15/MTok + 200 output * $75/MTok + 300 thinking * $75/MTok
        // = 0.0075 + 0.015 + 0.0225 = 0.045
        let cost = CostCalculator::estimate(
            &table,
            &model,
            TokenCount::new(500),
            TokenCount::new(200),
            TokenCount::new(300),
        );
        assert_eq!(cost.value(), 0.045);
    }

    // PC-009: Summarize groups by model
    #[test]
    fn summarize_groups_by_model() {
        let records = vec![
            make_record("claude-haiku-4-5", 1000, 500, 0, 0.003_5),
            make_record("claude-sonnet-4-6", 2000, 1000, 0, 0.021),
            make_record("claude-haiku-4-5", 500, 250, 0, 0.001_75),
        ];
        let summary = CostCalculator::summarize(&records);

        assert_eq!(summary.record_count, 3);
        assert_eq!(summary.breakdowns.len(), 2);

        // First breakdown should be haiku (insertion order)
        let haiku_bd = &summary.breakdowns[0];
        assert_eq!(haiku_bd.model.as_str(), "claude-haiku-4-5");
        assert_eq!(haiku_bd.record_count, 2);
        assert_eq!(haiku_bd.input_tokens.value(), 1500);
        assert_eq!(haiku_bd.output_tokens.value(), 750);

        let sonnet_bd = &summary.breakdowns[1];
        assert_eq!(sonnet_bd.model.as_str(), "claude-sonnet-4-6");
        assert_eq!(sonnet_bd.record_count, 1);
    }

    // PC-010: Unknown model fallback
    #[test]
    fn unknown_model_falls_back() {
        let table = PricingTable::default();
        let model = ModelId::new("unknown-model-xyz").unwrap();
        let rate = table.rate_for(&model);
        // Fallback is sonnet rates
        assert_eq!(rate.input_per_mtok.value(), 3.0);
        assert_eq!(rate.output_per_mtok.value(), 15.0);
    }
}
