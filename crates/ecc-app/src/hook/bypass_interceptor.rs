//! Bypass interceptor — handles token checking and audit logging when a hook blocks.

use ecc_ports::bypass_store::BypassStore;

use super::HookResult;

/// Intercept a blocked hook result and check for a valid bypass token.
///
/// If `result.exit_code != 2`, returns `result` unchanged.
/// Otherwise:
/// - Appends a bypass-available hint to stderr
/// - If no `session_id`, returns exit 2
/// - If `bypass_store.check_token()` returns a token, records an Applied decision and returns passthrough
/// - Otherwise returns exit 2 with augmented stderr
pub fn intercept(
    hook_id: &str,
    stdin: &str,
    result: HookResult,
    session_id: Option<&str>,
    bypass_store: Option<&dyn BypassStore>,
    start: std::time::Instant,
) -> HookResult {
    if result.exit_code != 2 {
        return result;
    }

    // Append bypass-available hint to stderr
    let mut stderr = result.stderr.clone();
    stderr.push_str(&format!(
        "[Bypass available: {hook_id}] Use 'ecc bypass grant --hook {hook_id} --reason <reason>' to bypass\n",
    ));

    // No valid session ID — cannot check tokens
    let sid = match session_id {
        Some(s) if !s.is_empty() && s != "unknown" => s,
        _ => {
            tracing::debug!(hook_id, "no session_id — bypass tokens unavailable");
            return HookResult {
                exit_code: 2,
                stdout: result.stdout,
                stderr,
            };
        }
    };

    // Check bypass token via store
    if let Some(store) = bypass_store
        && let Some(token) = store.check_token(hook_id, sid)
    {
        // Record Applied decision
        if let Ok(decision) = ecc_domain::hook_runtime::bypass::BypassDecision::new(
            hook_id,
            &token.reason,
            sid,
            ecc_domain::hook_runtime::bypass::Verdict::Applied,
            &token.granted_at,
        ) {
            let _ = store.record(&decision);
        }

        let duration_ms = start.elapsed().as_millis() as u64;
        tracing::info!(hook_id, "bypass token found — allowing");
        tracing::debug!(duration_ms, hook_id, "hook bypassed via token");
        return HookResult::passthrough(stdin);
    }

    HookResult {
        exit_code: 2,
        stdout: result.stdout,
        stderr,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_domain::hook_runtime::bypass::{BypassToken, Verdict};
    use ecc_test_support::InMemoryBypassStore;

    fn make_block_result(stdin: &str) -> HookResult {
        HookResult {
            stdout: stdin.to_string(),
            stderr: "blocked by hook".to_string(),
            exit_code: 2,
        }
    }

    /// PC-019: token found returns passthrough (exit 0)
    #[test]
    fn token_found_passthrough() {
        let hook_id = "pre:edit:boundary-crossing";
        let session_id = "sess-019";

        let token = BypassToken::new(hook_id, session_id, "2026-04-07T12:00:00Z", "test bypass")
            .expect("valid token");

        let store = InMemoryBypassStore::new().with_token(token);
        let result = make_block_result("stdin-data");
        let start = std::time::Instant::now();

        let out = intercept(
            hook_id,
            "stdin-data",
            result,
            Some(session_id),
            Some(&store),
            start,
        );

        assert_eq!(
            out.exit_code, 0,
            "valid token must grant passthrough (exit 0)"
        );
        assert_eq!(out.stdout, "stdin-data");
    }

    /// PC-020: token not found returns exit 2
    #[test]
    fn token_not_found_blocks() {
        let hook_id = "pre:edit:boundary-crossing";
        let session_id = "sess-020";

        let store = InMemoryBypassStore::new(); // empty — no tokens
        let result = make_block_result("stdin-data");
        let start = std::time::Instant::now();

        let out = intercept(
            hook_id,
            "stdin-data",
            result,
            Some(session_id),
            Some(&store),
            start,
        );

        assert_eq!(out.exit_code, 2, "missing token must block (exit 2)");
        assert!(
            out.stderr.contains("Bypass available"),
            "stderr must contain bypass hint; got: {}",
            out.stderr
        );
    }

    /// PC-021: no session_id returns exit 2
    #[test]
    fn no_session_id_blocks() {
        let hook_id = "pre:edit:boundary-crossing";

        let store = InMemoryBypassStore::new();
        let result = make_block_result("stdin-data");
        let start = std::time::Instant::now();

        let out = intercept(hook_id, "stdin-data", result, None, Some(&store), start);

        assert_eq!(out.exit_code, 2, "no session_id must block (exit 2)");
    }

    /// PC-022: records Applied decision in InMemoryBypassStore
    #[test]
    fn records_applied_decision() {
        let hook_id = "pre:edit:boundary-crossing";
        let session_id = "sess-022";

        let token = BypassToken::new(hook_id, session_id, "2026-04-07T12:00:00Z", "test bypass")
            .expect("valid token");

        let store = InMemoryBypassStore::new().with_token(token);
        let result = make_block_result("stdin-data");
        let start = std::time::Instant::now();

        let _ = intercept(
            hook_id,
            "stdin-data",
            result,
            Some(session_id),
            Some(&store),
            start,
        );

        let decisions = store.snapshot();
        assert_eq!(
            decisions.len(),
            1,
            "exactly one Applied decision must be recorded"
        );
        assert_eq!(decisions[0].verdict, Verdict::Applied);
        assert_eq!(decisions[0].hook_id, hook_id);
        assert_eq!(decisions[0].session_id, session_id);
    }
}
