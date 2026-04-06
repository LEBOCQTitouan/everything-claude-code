//! Session ID resolution for metric events.
//!
//! Accepts an optional env value (caller reads `CLAUDE_SESSION_ID`).
//! Falls back to a deterministic ID from timestamp and PID.

/// Resolve a session ID for metric events.
///
/// If `env_value` is `Some`, use it directly.
/// Otherwise, generate a deterministic fallback from the current timestamp and PID.
pub fn resolve_session_id(env_value: Option<&str>) -> String {
    if let Some(sid) = env_value {
        if !sid.is_empty() {
            return sid.to_owned();
        }
    }
    // Deterministic fallback: timestamp + PID
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let pid = std::process::id();
    format!("fallback-{ts}-{pid}")
}

#[cfg(test)]
mod tests {
    use super::*;

    // PC-012: resolve_session_id
    #[test]
    fn resolve_session_id_from_env() {
        assert_eq!(resolve_session_id(Some("my-session")), "my-session");
    }

    #[test]
    fn resolve_session_id_empty_env_falls_back() {
        let result = resolve_session_id(Some(""));
        assert!(result.starts_with("fallback-"));
    }

    #[test]
    fn resolve_session_id_none_falls_back() {
        let result = resolve_session_id(None);
        assert!(result.starts_with("fallback-"));
        assert!(result.contains('-'));
    }
}
