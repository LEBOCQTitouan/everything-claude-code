//! Cost domain value objects: ModelId, TokenCount, Money, CostRate, RecordId.

use crate::cost::error::CostError;

/// Newtype wrapping a model identifier string.
///
/// Normalised to lowercase; rejects empty strings.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelId(String);

impl ModelId {
    /// Create a new `ModelId`, normalising to lowercase.
    ///
    /// # Errors
    /// Returns [`CostError::InvalidModelId`] if `s` is empty.
    pub fn new(s: &str) -> Result<Self, CostError> {
        if s.is_empty() {
            return Err(CostError::InvalidModelId(
                "model ID must not be empty".to_owned(),
            ));
        }
        Ok(Self(s.to_lowercase()))
    }

    /// Return the inner string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Newtype wrapping a token count (u64).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TokenCount(u64);

impl TokenCount {
    /// Create a new `TokenCount`.
    pub fn new(n: u64) -> Self {
        Self(n)
    }

    /// Return the inner value.
    pub fn value(self) -> u64 {
        self.0
    }
}

/// Newtype wrapping a USD amount, rounded to 6 decimal places.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Money(f64);

impl Money {
    /// Create a `Money` value, rounding `f` to 6 decimal places.
    pub fn usd(f: f64) -> Self {
        let factor = 1_000_000.0_f64;
        Self((f * factor).round() / factor)
    }

    /// Return the inner value.
    pub fn value(self) -> f64 {
        self.0
    }
}

/// Per-model pricing rates expressed as USD per million tokens.
#[derive(Debug, Clone, PartialEq)]
pub struct CostRate {
    /// Cost per million input tokens (USD).
    pub input_per_mtok: Money,
    /// Cost per million output tokens (USD).
    pub output_per_mtok: Money,
    /// Cost per million thinking tokens (USD).
    pub thinking_per_mtok: Money,
}

/// Newtype wrapping a record primary key (i64).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RecordId(pub i64);

#[cfg(test)]
mod tests {
    use super::*;

    // PC-001: ModelId rejects empty string
    #[test]
    fn rejects_empty_model_id() {
        let result = ModelId::new("");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CostError::InvalidModelId(_)));
    }

    // PC-002: ModelId accepts valid string
    #[test]
    fn accepts_valid_model_id() {
        let result = ModelId::new("claude-sonnet-4-6");
        assert!(result.is_ok());
        let model = result.unwrap();
        assert_eq!(model.as_str(), "claude-sonnet-4-6");
    }

    // PC-003: Money rounds to 6 decimals
    #[test]
    fn money_rounds_to_six_decimals() {
        let m = Money::usd(1.123456789);
        // 1.123456789 rounded to 6 decimals => 1.123457
        assert_eq!(m.value(), 1.123457);
    }

    // PC-004: TokenCount zero is valid
    #[test]
    fn token_count_zero_valid() {
        let tc = TokenCount::new(0);
        assert_eq!(tc.value(), 0);
    }
}
