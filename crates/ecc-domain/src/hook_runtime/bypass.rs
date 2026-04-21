//! Bypass domain value objects.
//!
//! Models auditable workflow bypass decisions and session-scoped bypass tokens.
//! Pure domain types with validation — zero I/O imports.

use serde::{Deserialize, Serialize};

/// Verdict for a bypass decision.
///
/// State-transition diagram — Accepted may be Applied once; Refused is terminal:
///
/// ```text
///   [request grant]
///         |
///         +--> [Accepted] --consumed by hook--> [Applied]
///         +--> [Refused]  (terminal)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    /// User approved the bypass request.
    Accepted,
    /// User refused the bypass request.
    Refused,
    /// An accepted bypass was consumed by hook dispatch.
    Applied,
}

impl std::fmt::Display for Verdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Verdict::Accepted => write!(f, "accepted"),
            Verdict::Refused => write!(f, "refused"),
            Verdict::Applied => write!(f, "applied"),
        }
    }
}

/// Maximum allowed length for bypass reason text.
const MAX_REASON_LENGTH: usize = 500;

/// Errors from bypass domain validation.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum BypassError {
    /// Hook ID must not be empty.
    #[error("hook_id must not be empty")]
    EmptyHookId,
    /// Bypass reason must not be empty.
    #[error("reason must not be empty")]
    EmptyReason,
    /// Bypass reason exceeds the maximum length.
    #[error("reason exceeds {MAX_REASON_LENGTH} characters")]
    ReasonTooLong,
    /// Session ID must not be empty or the literal string "unknown".
    #[error("session_id must not be empty or 'unknown'")]
    InvalidSessionId,
}

/// Immutable audit record of a bypass request and its verdict.
///
/// Composition diagram — validated aggregate (see `new()` for invariants):
///
/// ```text
/// +-------------- BypassDecision --------------+
/// | id:         Option<i64>   (db PK, None new)|
/// | hook_id:    String        (non-empty)      |
/// | reason:     String        (1..=500 chars)  |
/// | session_id: String        (non-empty, not  |
/// |                            "unknown")      |
/// | verdict:    Verdict                        |
/// | timestamp:  String        (ISO 8601)       |
/// +--------------------------------------------+
/// ```
///
/// # Pattern
///
/// Value Object \[DDD\] — equality by all fields, validated at construction.
#[derive(Debug, Clone, PartialEq)]
pub struct BypassDecision {
    /// Database row ID (None when newly constructed, Some when loaded from storage).
    pub id: Option<i64>,
    /// Hook ID being bypassed.
    pub hook_id: String,
    /// Human-readable reason for the bypass.
    pub reason: String,
    /// Session ID during which the bypass was requested.
    pub session_id: String,
    /// Outcome of the bypass request (accepted, refused, or applied).
    pub verdict: Verdict,
    /// ISO 8601 timestamp of the decision.
    pub timestamp: String,
}

impl BypassDecision {
    /// Construct a validated bypass decision.
    pub fn new(
        hook_id: &str,
        reason: &str,
        session_id: &str,
        verdict: Verdict,
        timestamp: &str,
    ) -> Result<Self, BypassError> {
        if hook_id.is_empty() {
            return Err(BypassError::EmptyHookId);
        }
        if reason.is_empty() {
            return Err(BypassError::EmptyReason);
        }
        if reason.len() > MAX_REASON_LENGTH {
            return Err(BypassError::ReasonTooLong);
        }
        validate_session_id(session_id)?;
        Ok(Self {
            id: None,
            hook_id: hook_id.to_string(),
            reason: reason.to_string(),
            session_id: session_id.to_string(),
            verdict,
            timestamp: timestamp.to_string(),
        })
    }

    /// Construct from stored data (already validated, includes ID).
    pub fn from_stored(
        id: i64,
        hook_id: String,
        reason: String,
        session_id: String,
        verdict: Verdict,
        timestamp: String,
    ) -> Self {
        Self {
            id: Some(id),
            hook_id,
            reason,
            session_id,
            verdict,
            timestamp,
        }
    }
}

/// Session-scoped file authorizing a specific hook to pass.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BypassToken {
    /// Hook ID this token authorizes to bypass.
    pub hook_id: String,
    /// Session ID during which this token is valid.
    pub session_id: String,
    /// ISO 8601 timestamp when the token was granted.
    pub granted_at: String,
    /// Human-readable reason for granting this bypass.
    pub reason: String,
}

impl BypassToken {
    /// Construct a validated bypass token.
    pub fn new(
        hook_id: &str,
        session_id: &str,
        granted_at: &str,
        reason: &str,
    ) -> Result<Self, BypassError> {
        if hook_id.is_empty() {
            return Err(BypassError::EmptyHookId);
        }
        if reason.is_empty() {
            return Err(BypassError::EmptyReason);
        }
        validate_session_id(session_id)?;
        Ok(Self {
            hook_id: hook_id.to_string(),
            session_id: session_id.to_string(),
            granted_at: granted_at.to_string(),
            reason: reason.to_string(),
        })
    }
}

/// Aggregate bypass statistics per hook.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct HookBypassCount {
    /// Hook ID being tracked.
    pub hook_id: String,
    /// Number of accepted bypasses for this hook.
    pub accepted: u64,
    /// Number of refused bypasses for this hook.
    pub refused: u64,
}

/// Summary of bypass patterns across all hooks.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BypassSummary {
    /// Per-hook bypass statistics.
    pub per_hook: Vec<HookBypassCount>,
    /// Total accepted bypasses across all hooks.
    pub total_accepted: u64,
    /// Total refused bypasses across all hooks.
    pub total_refused: u64,
}

/// Policy trait for bypass decision logic.
pub trait BypassPolicy: Send + Sync {
    /// Determine whether a hook should be bypassed for the given session.
    fn should_bypass(&self, hook_id: &str, session_id: &str) -> bool;
}

/// Validate that session_id is non-empty and not "unknown".
fn validate_session_id(session_id: &str) -> Result<(), BypassError> {
    if session_id.is_empty() || session_id == "unknown" {
        return Err(BypassError::InvalidSessionId);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bypass_decision_valid_construction() {
        let d = BypassDecision::new(
            "pre:write-edit:worktree-guard",
            "Need to edit main for hotfix",
            "session-abc-123",
            Verdict::Accepted,
            "2026-04-06T10:00:00Z",
        );
        assert!(d.is_ok());
        let d = d.unwrap();
        assert_eq!(d.hook_id, "pre:write-edit:worktree-guard");
        assert_eq!(d.verdict, Verdict::Accepted);
        assert!(d.id.is_none());
    }

    #[test]
    fn bypass_decision_rejects_empty_hook_id() {
        let d = BypassDecision::new("", "reason", "session-1", Verdict::Accepted, "ts");
        assert_eq!(d, Err(BypassError::EmptyHookId));
    }

    #[test]
    fn bypass_decision_rejects_empty_reason() {
        let d = BypassDecision::new("hook", "", "session-1", Verdict::Accepted, "ts");
        assert_eq!(d, Err(BypassError::EmptyReason));
    }

    #[test]
    fn bypass_decision_caps_reason() {
        let long_reason = "x".repeat(501);
        let d = BypassDecision::new("hook", &long_reason, "session-1", Verdict::Accepted, "ts");
        assert_eq!(d, Err(BypassError::ReasonTooLong));

        // Exactly 500 should pass
        let exact = "x".repeat(500);
        let d = BypassDecision::new("hook", &exact, "session-1", Verdict::Accepted, "ts");
        assert!(d.is_ok());
    }

    #[test]
    fn bypass_decision_rejects_invalid_session_id() {
        let d = BypassDecision::new("hook", "reason", "", Verdict::Accepted, "ts");
        assert_eq!(d, Err(BypassError::InvalidSessionId));

        let d = BypassDecision::new("hook", "reason", "unknown", Verdict::Accepted, "ts");
        assert_eq!(d, Err(BypassError::InvalidSessionId));
    }

    #[test]
    fn bypass_token_valid_construction() {
        let t = BypassToken::new(
            "pre:write-edit:worktree-guard",
            "session-abc",
            "2026-04-06T10:00:00Z",
            "hotfix needed",
        );
        assert!(t.is_ok());
    }

    #[test]
    fn bypass_token_rejects_invalid_session_id() {
        let t = BypassToken::new("hook", "", "ts", "reason");
        assert_eq!(t, Err(BypassError::InvalidSessionId));

        let t = BypassToken::new("hook", "unknown", "ts", "reason");
        assert_eq!(t, Err(BypassError::InvalidSessionId));
    }

    #[test]
    fn bypass_token_json_serialization() {
        let t = BypassToken::new(
            "pre:edit:guard",
            "session-1",
            "2026-04-06T10:00:00Z",
            "test",
        )
        .unwrap();
        let json = serde_json::to_string(&t).unwrap();
        assert!(json.contains("\"hook_id\":\"pre:edit:guard\""));
        assert!(json.contains("\"session_id\":\"session-1\""));

        let deserialized: BypassToken = serde_json::from_str(&json).unwrap();
        assert_eq!(t, deserialized);
    }

    #[test]
    fn verdict_display() {
        assert_eq!(Verdict::Accepted.to_string(), "accepted");
        assert_eq!(Verdict::Refused.to_string(), "refused");
        assert_eq!(Verdict::Applied.to_string(), "applied");
    }

    #[test]
    fn bypass_summary_default() {
        let s = BypassSummary::default();
        assert!(s.per_hook.is_empty());
        assert_eq!(s.total_accepted, 0);
        assert_eq!(s.total_refused, 0);
    }

    struct AlwaysAllowPolicy;

    impl BypassPolicy for AlwaysAllowPolicy {
        fn should_bypass(&self, _hook_id: &str, _session_id: &str) -> bool {
            true
        }
    }

    #[test]
    fn bypass_policy_trait_compiles() {
        let policy = AlwaysAllowPolicy;
        assert!(policy.should_bypass("pre:edit:guard", "session-123"));
        assert!(policy.should_bypass("stop:notify", "session-456"));
    }
}
