//! Bypass token checking logic for hook dispatch.

use std::path::Path;

use crate::hook::{HookContext, HookPorts, HookResult};

/// Check for a session-scoped bypass token when a hook blocks (exit_code == 2).
///
/// Returns a passthrough `HookResult` if a valid bypass token is found,
/// or a blocking result (exit_code=2) with the bypass hint appended to stderr.
pub(super) fn apply_bypass_check(
    ctx: &HookContext,
    ports: &HookPorts<'_>,
    result_stdout: &str,
    mut stderr: String,
    start: std::time::Instant,
    stdin: &str,
) -> HookResult {
    // Append bypass-available hint to stderr
    stderr.push_str(&format!(
        "[Bypass available: {}] Use 'ecc bypass grant --hook {} --reason <reason>' to bypass\n",
        ctx.hook_id, ctx.hook_id
    ));

    let session_id = ports.env.var("CLAUDE_SESSION_ID");
    match session_id.as_deref() {
        Some(sid) if !sid.is_empty() && sid != "unknown" => {
            if let Some(passthrough) = try_token_bypass(ctx, ports, sid, stdin, start) {
                return passthrough;
            }
            HookResult {
                exit_code: 2,
                stdout: result_stdout.to_string(),
                stderr,
            }
        }
        _ => {
            tracing::debug!(
                hook_id = %ctx.hook_id,
                "no CLAUDE_SESSION_ID — bypass tokens unavailable"
            );
            HookResult {
                exit_code: 2,
                stdout: result_stdout.to_string(),
                stderr,
            }
        }
    }
}

/// Try to find and validate a bypass token for the given session. Returns `Some(passthrough)`
/// if a valid token exists, `None` otherwise.
fn try_token_bypass(
    ctx: &HookContext,
    ports: &HookPorts<'_>,
    sid: &str,
    stdin: &str,
    start: std::time::Instant,
) -> Option<HookResult> {
    let home = ports.env.var("HOME")?;
    let token_dir = format!("{}/.ecc/bypass-tokens/{}", home, sid);
    let encoded = ctx.hook_id.replace(':', "__");
    let token_path = format!("{}/{}.json", token_dir, encoded);

    let token_json = ports.fs.read_to_string(Path::new(&token_path)).ok()?;
    let token =
        serde_json::from_str::<ecc_domain::hook_runtime::bypass::BypassToken>(&token_json).ok()?;

    if token.session_id != sid || token.hook_id != ctx.hook_id {
        return None;
    }

    tracing::info!(hook_id = %ctx.hook_id, "bypass token found — allowing");

    if let Some(store) = ports.bypass_store
        && let Ok(decision) = ecc_domain::hook_runtime::bypass::BypassDecision::new(
            &ctx.hook_id,
            &token.reason,
            sid,
            ecc_domain::hook_runtime::bypass::Verdict::Applied,
            &token.granted_at,
        )
        && let Err(e) = store.record(&decision)
    {
        tracing::warn!(hook_id = %ctx.hook_id, error = %e, "failed to record bypass decision");
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    tracing::debug!(duration_ms, hook_id = %ctx.hook_id, "hook bypassed via token");
    Some(HookResult::passthrough(stdin))
}
