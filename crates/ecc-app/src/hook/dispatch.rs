//! Hook dispatch logic — routes hook ID to the appropriate handler.

use std::collections::HashMap;

use crate::hook::bypass_handling::apply_bypass_check;
use crate::hook::handlers;
use crate::hook::{Handler, HookContext, HookPorts, HookResult, MAX_STDIN};

// ---------------------------------------------------------------------------
// Concrete handler structs (thin wrappers around existing handler functions)
// ---------------------------------------------------------------------------

struct StopCartographyHandler;

impl Handler for StopCartographyHandler {
    fn hook_id(&self) -> &str {
        "stop:cartography"
    }

    fn handle(&self, stdin: &str, ports: &HookPorts<'_>) -> HookResult {
        handlers::stop_cartography(stdin, ports)
    }
}

struct StartCartographyHandler;

impl Handler for StartCartographyHandler {
    fn hook_id(&self) -> &str {
        "start:cartography"
    }

    fn handle(&self, stdin: &str, ports: &HookPorts<'_>) -> HookResult {
        handlers::start_cartography(stdin, ports)
    }
}

/// Build the handler registry — a `HashMap` keyed by hook ID.
///
/// Handlers registered here take precedence over the match-based dispatch below.
/// This is intentionally a function (not a static) so tests can call it directly.
pub fn build_handler_registry() -> HashMap<&'static str, Box<dyn Handler>> {
    let mut m: HashMap<&'static str, Box<dyn Handler>> = HashMap::new();

    // Cartography handlers registered as proof of concept (AC-009.2).
    m.insert("stop:cartography", Box::new(StopCartographyHandler));
    m.insert("start:cartography", Box::new(StartCartographyHandler));

    m
}

/// Truncate stdin payload to MAX_STDIN bytes.
pub fn truncate_stdin(raw: &str) -> &str {
    if raw.len() <= MAX_STDIN {
        raw
    } else {
        // Find a valid UTF-8 boundary
        let mut end = MAX_STDIN;
        while end > 0 && !raw.is_char_boundary(end) {
            end -= 1;
        }
        &raw[..end]
    }
}

/// Dispatch a hook by ID. Returns the hook result.
///
/// If the hook is disabled by profile/env, returns a passthrough result.
/// If the hook ID is unknown, returns a passthrough result with a stderr warning.
pub fn dispatch(ctx: &HookContext, ports: &HookPorts<'_>) -> HookResult {
    use ecc_domain::hook_runtime::profiles::{HookEnabledOptions, is_hook_enabled};

    let span = tracing::debug_span!("hook_dispatch", hook_id = %ctx.hook_id);
    let _guard = span.enter();

    let stdin = truncate_stdin(&ctx.stdin_payload);
    let start = std::time::Instant::now();

    // Deprecation warning for ECC_WORKFLOW_BYPASS=1 (AC-006.1, AC-006.2)
    if ports.env.var("ECC_WORKFLOW_BYPASS").as_deref() == Some("1") {
        ports.terminal.stderr_write(
            "[Deprecated] ECC_WORKFLOW_BYPASS=1 is deprecated. Use 'ecc bypass grant' for granular, auditable bypasses. See ADR-0056.\n"
        );
        // Still allow passthrough for backward compat (AC-006.2)
        return HookResult::passthrough(stdin);
    }

    // Check if hook is enabled
    let profile_env = ports.env.var("ECC_HOOK_PROFILE");
    let disabled_env = ports.env.var("ECC_DISABLED_HOOKS");
    let opts = HookEnabledOptions {
        profiles: ctx.profiles_csv.as_deref(),
    };

    if !ctx.hook_id.is_empty()
        && !is_hook_enabled(
            &ctx.hook_id,
            profile_env.as_deref(),
            disabled_env.as_deref(),
            &opts,
        )
    {
        tracing::debug!(hook_id = %ctx.hook_id, "hook skipped: disabled by profile/env");
        return HookResult::passthrough(stdin);
    }

    tracing::debug!(hook_id = %ctx.hook_id, "dispatching hook");

    // Dispatch to handler
    let result = match ctx.hook_id.as_str() {
        // Tier 1: Simple passthrough hooks
        "check:hook:enabled" => handlers::check_hook_enabled(stdin, ports),
        "session:end:marker" => handlers::session_end_marker(stdin, ports),
        "stop:check-console-log" => handlers::check_console_log(stdin, ports),
        "stop:uncommitted-reminder" => handlers::stop_uncommitted_reminder(stdin, ports),
        "pre:bash:git-push-reminder" => handlers::pre_bash_git_push_reminder(stdin),
        "pre:bash:tmux-reminder" => handlers::pre_bash_tmux_reminder(stdin, ports),
        "post:bash:pr-created" => handlers::post_bash_pr_created(stdin),
        "post:bash:build-complete" => handlers::post_bash_build_complete(stdin),
        "pre:write:doc-file-warning" | "pre:edit-write:doc-file-warning" => {
            handlers::doc_file_warning(stdin)
        }
        "post:edit-write:doc-coverage-reminder" => handlers::doc_coverage_reminder(stdin, ports),
        "post:edit:console-warn" => handlers::post_edit_console_warn(stdin, ports),
        "pre:edit-write:suggest-compact" => handlers::suggest_compact(stdin, ports),

        // Tier 2: External tool spawning hooks
        "stop:notify" => handlers::stop_notify(stdin, ports),
        "pre:bash:dev-server-block" => handlers::pre_bash_dev_server_block(stdin, ports),
        "post:edit:format" => handlers::post_edit_format(stdin, ports),
        "post:edit:typecheck" => handlers::post_edit_typecheck(stdin, ports),
        "post:quality-gate" => handlers::quality_gate(stdin, ports),

        "pre:edit-write:workflow-branch-guard" => {
            handlers::pre_edit_write_workflow_branch_guard(stdin, ports)
        }
        "pre:write-edit:worktree-guard" => handlers::pre_worktree_write_guard(stdin, ports),

        // Tier 1: Clean Craft hooks
        "pre:edit:boundary-crossing" => handlers::pre_edit_boundary_crossing(stdin, ports),
        "post:edit:boy-scout-delta" => handlers::post_edit_boy_scout_delta(stdin, ports),
        "post:edit:naming-review" => handlers::post_edit_naming_review(stdin, ports),
        "post:edit:newspaper-check" => handlers::post_edit_newspaper_check(stdin, ports),
        "pre:edit:stepdown-warning" => handlers::pre_edit_stepdown_warning(stdin, ports),

        // New event handlers
        "post:failure:error-context" => handlers::post_failure_error_context(stdin, ports),
        "pre:prompt:context-hydrate" => handlers::pre_prompt_context_hydrate(stdin, ports),
        "pre:prompt:context-inject" => handlers::pre_prompt_context_inject(stdin, ports),
        "post:compact:state-save" => handlers::post_compact(stdin, ports),
        "subagent:start:log" => handlers::subagent_start_log(stdin, ports),
        "subagent:start:effort" => handlers::subagent_start_effort(stdin, ports),
        "subagent:stop:log" => handlers::subagent_stop_log(stdin, ports),
        "instructions:loaded:validate" => handlers::instructions_loaded_validate(stdin, ports),
        "config:change:log" => handlers::config_change_log(stdin, ports),
        "post:enter-worktree:session-log" => {
            handlers::post_enter_worktree_session_log(stdin, ports)
        }
        "post:exit-worktree:cleanup-reminder" => {
            handlers::post_exit_worktree_cleanup_reminder(stdin, ports)
        }

        // Tier 3: Session/File I/O hooks
        "session:start" => handlers::session_start(stdin, ports),
        "stop:session-end" => handlers::session_end(stdin, ports),
        "session:end:worktree-merge" => handlers::session_end_merge(stdin, ports),
        "start:cartography" => handlers::start_cartography(stdin, ports),
        "stop:cartography" => handlers::stop_cartography(stdin, ports),
        "pre:compact" => handlers::pre_compact(stdin, ports),
        "stop:evaluate-session" => handlers::evaluate_session(stdin, ports),
        "stop:cost-tracker" => handlers::cost_tracker(stdin, ports),
        "stop:oath-reflection" => handlers::oath_reflection(stdin, ports),
        "stop:craft-velocity" => handlers::craft_velocity(stdin, ports),
        "stop:daily-summary" => handlers::daily_summary(stdin, ports),

        // Unknown hook — passthrough with warning
        _ => {
            let msg = format!("[Hook] Unknown hook ID: {}\n", ctx.hook_id);
            HookResult::warn(stdin, &msg)
        }
    };

    // Check for bypass token when hook blocks
    #[allow(clippy::collapsible_if)]
    let result = if result.exit_code == 2 {
        apply_bypass_check(ctx, ports, &result.stdout, result.stderr.clone(), start, stdin)
    } else {
        result
    };

    let duration_ms = start.elapsed().as_millis() as u64;
    tracing::debug!(duration_ms, hook_id = %ctx.hook_id, "hook dispatch completed");

    // Record hook execution metric (fire-and-forget)
    let metrics_disabled = ports.env.var("ECC_METRICS_DISABLED").as_deref() == Some("1");
    let session_id = crate::metrics_session::resolve_session_id(
        ports.env.var("CLAUDE_SESSION_ID").as_deref(),
    );
    let timestamp = {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        format!("{secs}")
    };
    let (outcome, error_message) = if result.exit_code == 0 {
        (ecc_domain::metrics::MetricOutcome::Success, None)
    } else {
        (
            ecc_domain::metrics::MetricOutcome::Failure,
            Some(result.stderr.clone()),
        )
    };
    if let Ok(event) = ecc_domain::metrics::MetricEvent::hook_execution(
        session_id,
        timestamp,
        ctx.hook_id.clone(),
        duration_ms,
        outcome,
        error_message,
    ) {
        let _ = crate::metrics_mgmt::record_if_enabled(
            ports.metrics_store,
            &event,
            metrics_disabled,
        );
    }

    result
}
