//! Timestamp newtype for ISO 8601 datetime strings stored in workflow state.

use serde::{Deserialize, Serialize};

/// A newtype wrapper for ISO 8601 timestamp strings.
///
/// Wraps a `String` to prevent mixing semantically distinct string values.
/// Does not validate format — callers are responsible for supplying valid
/// ISO 8601 strings (e.g., `"2026-03-28T12:00:00Z"`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timestamp(pub String);

impl Timestamp {
    /// Create a new `Timestamp` from any string that is `Into<String>`.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Return a reference to the underlying string value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_from_string_literal() {
        let ts = Timestamp::new("2026-03-28T12:00:00Z");
        assert_eq!(ts.as_str(), "2026-03-28T12:00:00Z");
    }

    #[test]
    fn serializes_as_string() {
        let ts = Timestamp::new("2026-03-28T12:00:00Z");
        let json = serde_json::to_string(&ts).unwrap();
        assert_eq!(json, r#""2026-03-28T12:00:00Z""#);
    }

    #[test]
    fn deserializes_from_string() {
        let ts: Timestamp = serde_json::from_str(r#""2026-03-28T12:00:00Z""#).unwrap();
        assert_eq!(ts.as_str(), "2026-03-28T12:00:00Z");
    }

    #[test]
    fn round_trips() {
        let original = Timestamp::new("2026-03-28T12:00:00Z");
        let json = serde_json::to_string(&original).unwrap();
        let restored: Timestamp = serde_json::from_str(&json).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn display_shows_inner_string() {
        let ts = Timestamp::new("2026-01-01T00:00:00Z");
        assert_eq!(ts.to_string(), "2026-01-01T00:00:00Z");
    }
}
