//! Liveness value object and pure predicate for session heartbeat.
//!
//! Zero I/O — all data is passed in by the caller.

/// Minimum valid PID (kernel threads use 0 and 1).
pub const MIN_VALID_PID: u32 = 2;

/// Maximum clock-skew tolerance for future timestamps (60 seconds).
pub const FUTURE_SKEW_TOLERANCE_SECS: u64 = 60;

/// A parsed `.ecc-session` heartbeat record.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LivenessRecord {
    /// Schema version — must be 1.
    pub schema_version: u32,
    /// PID of the Claude Code process that owns this worktree.
    pub claude_code_pid: u32,
    /// Unix timestamp (seconds) when the heartbeat was last written.
    pub last_seen_unix_ts: u64,
}

/// Errors that can occur when parsing a `.ecc-session` file.
#[derive(Debug, thiserror::Error)]
pub enum LivenessParseError {
    /// The schema_version field is not 1.
    #[error("unsupported schema_version: expected 1, got {0}")]
    UnsupportedSchemaVersion(u32),
    /// The PID is reserved (0 or 1).
    #[error("reserved PID {0} (must be >= 2)")]
    ReservedPid(u32),
    /// The JSON was malformed.
    #[error("malformed JSON: {0}")]
    MalformedJson(#[from] serde_json::Error),
}

impl LivenessRecord {
    /// Parse a JSON string into a `LivenessRecord`, validating schema and PID.
    ///
    /// # Errors
    ///
    /// - [`LivenessParseError::MalformedJson`] if the JSON is invalid.
    /// - [`LivenessParseError::UnsupportedSchemaVersion`] if schema_version != 1.
    /// - [`LivenessParseError::ReservedPid`] if claude_code_pid < 2.
    pub fn parse(json: &str) -> Result<Self, LivenessParseError> {
        // Stub: always return an error so RED tests fail.
        let _ = json;
        Err(LivenessParseError::UnsupportedSchemaVersion(0))
    }
}

/// Pure liveness predicate. Returns `true` when the session is considered live.
///
/// A session is live when ALL of the following hold:
/// - `pid_alive` is `true`
/// - `last_seen_unix_ts` is not more than `FUTURE_SKEW_TOLERANCE_SECS` ahead of `now`
/// - `now - last_seen_unix_ts < threshold_secs` (heartbeat is fresh)
pub fn is_live(_r: &LivenessRecord, _now: u64, _pid_alive: bool, _threshold_secs: u64) -> bool {
    // Stub: always return false so RED tests fail.
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_json(schema_version: u32, pid: u32, ts: u64) -> String {
        format!(
            r#"{{"schema_version":{schema_version},"claude_code_pid":{pid},"last_seen_unix_ts":{ts}}}"#
        )
    }

    #[test]
    fn parse_round_trip() {
        let json = valid_json(1, 12345, 1_000_000);
        let record = LivenessRecord::parse(&json).expect("valid JSON should parse");
        assert_eq!(record.schema_version, 1);
        assert_eq!(record.claude_code_pid, 12345);
        assert_eq!(record.last_seen_unix_ts, 1_000_000);
    }

    #[test]
    fn parse_rejects_unknown_schema() {
        let json = valid_json(2, 12345, 1_000_000);
        let err = LivenessRecord::parse(&json).expect_err("schema_version=2 must be rejected");
        assert!(
            matches!(err, LivenessParseError::UnsupportedSchemaVersion(2)),
            "expected UnsupportedSchemaVersion(2), got: {err}"
        );
    }

    #[test]
    fn parse_rejects_pid_0() {
        let json = valid_json(1, 0, 1_000_000);
        let err = LivenessRecord::parse(&json).expect_err("PID 0 must be rejected");
        assert!(
            matches!(err, LivenessParseError::ReservedPid(0)),
            "expected ReservedPid(0), got: {err}"
        );
    }

    #[test]
    fn parse_rejects_pid_1() {
        let json = valid_json(1, 1, 1_000_000);
        let err = LivenessRecord::parse(&json).expect_err("PID 1 must be rejected");
        assert!(
            matches!(err, LivenessParseError::ReservedPid(1)),
            "expected ReservedPid(1), got: {err}"
        );
    }

    #[test]
    fn is_live_false_for_stale() {
        let record = LivenessRecord {
            schema_version: 1,
            claude_code_pid: 1234,
            last_seen_unix_ts: 1_000_000,
        };
        let now = 1_000_000 + 3601; // 3601 seconds later — exceeds 3600s threshold
        let threshold = 3600;
        assert!(
            !is_live(&record, now, true, threshold),
            "heartbeat older than threshold must not be live"
        );
    }

    #[test]
    fn is_live_false_for_future_ts() {
        let record = LivenessRecord {
            schema_version: 1,
            claude_code_pid: 1234,
            last_seen_unix_ts: 1_000_000 + 61, // 61 seconds in the future
        };
        let now = 1_000_000;
        let threshold = 3600;
        assert!(
            !is_live(&record, now, true, threshold),
            "timestamp more than 60s in the future must not be live"
        );
    }
}
