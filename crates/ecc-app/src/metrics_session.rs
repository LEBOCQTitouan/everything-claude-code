//! Session ID resolution for metric events.
//!
//! Accepts an optional env value (caller reads `CLAUDE_SESSION_ID`).
//! Falls back to a deterministic ID from timestamp and PID.

/// Resolve a session ID for metric events.
///
/// If `env_value` is `Some`, use it directly.
/// Otherwise, generate a deterministic fallback from the current timestamp and PID.
pub fn resolve_session_id(env_value: Option<&str>, clock: &dyn ecc_ports::clock::Clock) -> String {
    if let Some(sid) = env_value
        && !sid.is_empty()
    {
        return sid.to_owned();
    }
    // Deterministic fallback: timestamp + PID
    let ts = clock.now_epoch_secs();
    let pid = std::process::id();
    format!("fallback-{ts}-{pid}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::TEST_CLOCK;

    // PC-012: resolve_session_id
    #[test]
    fn resolve_session_id_from_env() {
        assert_eq!(
            resolve_session_id(Some("my-session"), &*TEST_CLOCK),
            "my-session"
        );
    }

    #[test]
    fn resolve_session_id_empty_env_falls_back() {
        let result = resolve_session_id(Some(""), &*TEST_CLOCK);
        assert!(result.starts_with("fallback-"));
    }

    #[test]
    fn resolve_session_id_none_falls_back() {
        let result = resolve_session_id(None, &*TEST_CLOCK);
        assert!(result.starts_with("fallback-"));
        assert!(result.contains('-'));
    }
}
