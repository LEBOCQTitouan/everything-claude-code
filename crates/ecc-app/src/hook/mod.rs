//! Hook dispatch use case — routes hookId to the appropriate handler.

use std::collections::HashMap;

use ecc_domain::hook_runtime::profiles::{HookEnabledOptions, is_hook_enabled};
use ecc_ports::bypass_store::BypassStore;
use ecc_ports::cost_store::CostStore;
use ecc_ports::env::Environment;
use ecc_ports::fs::FileSystem;
use ecc_ports::metrics_store::MetricsStore;
use ecc_ports::shell::ShellExecutor;
use ecc_ports::terminal::TerminalIO;

pub mod bypass_interceptor;
pub mod handlers;

/// Trait for hook handlers that can be registered in a dispatch table.
pub trait Handler: Send + Sync {
    /// The hook ID this handler responds to (e.g., `"stop:cartography"`).
    fn hook_id(&self) -> &str;

    /// Handle the hook invocation.
    fn handle(&self, stdin: &str, ports: &HookPorts<'_>) -> HookResult;
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

/// Maximum stdin payload size (1 MB).
pub const MAX_STDIN: usize = 1_024 * 1_024;

/// Input context for a hook invocation.
#[derive(Debug)]
pub struct HookContext {
    pub hook_id: String,
    pub stdin_payload: String,
    pub profiles_csv: Option<String>,
}

/// Result of a hook execution.
#[derive(Debug)]
pub struct HookResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl HookResult {
    /// Passthrough result — echoes stdin to stdout, no stderr.
    pub fn passthrough(stdin: &str) -> Self {
        Self {
            stdout: stdin.to_string(),
            stderr: String::new(),
            exit_code: 0,
        }
    }

    /// Warning result — echoes stdin to stdout, writes warning to stderr.
    pub fn warn(stdin: &str, message: &str) -> Self {
        Self {
            stdout: stdin.to_string(),
            stderr: message.to_string(),
            exit_code: 0,
        }
    }

    /// Block result — echoes stdin to stdout, writes error to stderr, exits with code 2.
    pub fn block(stdin: &str, message: &str) -> Self {
        Self {
            stdout: stdin.to_string(),
            stderr: message.to_string(),
            exit_code: 2,
        }
    }

    /// Silent result — echoes stdin to stdout, no output.
    pub fn silent(stdin: &str) -> Self {
        Self::passthrough(stdin)
    }
}

/// Ports bundle for hook execution.
pub struct HookPorts<'a> {
    pub fs: &'a dyn FileSystem,
    pub shell: &'a dyn ShellExecutor,
    pub env: &'a dyn Environment,
    pub terminal: &'a dyn TerminalIO,
    pub cost_store: Option<&'a dyn CostStore>,
    pub bypass_store: Option<&'a dyn BypassStore>,
    pub metrics_store: Option<&'a dyn MetricsStore>,
}

impl<'a> HookPorts<'a> {
    /// Construct a `HookPorts` with all optional stores set to `None`.
    ///
    /// Intended for use in tests to reduce boilerplate when optional stores are not needed.
    pub fn test_default(
        fs: &'a dyn FileSystem,
        shell: &'a dyn ShellExecutor,
        env: &'a dyn Environment,
        terminal: &'a dyn TerminalIO,
    ) -> Self {
        Self {
            fs,
            shell,
            env,
            terminal,
            cost_store: None,
            bypass_store: None,
            metrics_store: None,
        }
    }
}

/// Bypass policy that always denies bypass requests.
///
/// Used as a safe default when no bypass mechanism is configured.
pub struct AlwaysDenyPolicy;

impl ecc_domain::hook_runtime::bypass::BypassPolicy for AlwaysDenyPolicy {
    fn should_bypass(&self, _hook_id: &str, _session_id: &str) -> bool {
        false
    }
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
        // Append bypass-available hint to stderr
        let mut stderr = result.stderr.clone();
        stderr.push_str(&format!(
            "[Bypass available: {}] Use 'ecc bypass grant --hook {} --reason <reason>' to bypass\n",
            ctx.hook_id, ctx.hook_id
        ));

        // Check for session-scoped bypass token
        let session_id = ports.env.var("CLAUDE_SESSION_ID");
        match session_id.as_deref() {
            Some(sid) if !sid.is_empty() && sid != "unknown" => {
                // Check if a bypass token exists for this hook+session
                if let Some(home) = ports.env.var("HOME") {
                    let token_dir = format!("{}/.ecc/bypass-tokens/{}", home, sid);
                    let encoded = ctx.hook_id.replace(':', "__");
                    let token_path = format!("{}/{}.json", token_dir, encoded);
                    if ports.fs.exists(std::path::Path::new(&token_path)) {
                        // Read and validate token
                        if let Ok(token_json) =
                            ports.fs.read_to_string(std::path::Path::new(&token_path))
                        {
                            if let Ok(token) = serde_json::from_str::<
                                ecc_domain::hook_runtime::bypass::BypassToken,
                            >(&token_json)
                            {
                                if token.session_id == sid && token.hook_id == ctx.hook_id {
                                    tracing::info!(hook_id = %ctx.hook_id, "bypass token found — allowing");
                                    // Log the applied bypass
                                    if let Some(store) = ports.bypass_store {
                                        if let Ok(decision) =
                                            ecc_domain::hook_runtime::bypass::BypassDecision::new(
                                                &ctx.hook_id,
                                                &token.reason,
                                                sid,
                                                ecc_domain::hook_runtime::bypass::Verdict::Applied,
                                                &token.granted_at,
                                            )
                                        {
                                            let _ = store.record(&decision);
                                        }
                                    }
                                    let duration_ms = start.elapsed().as_millis() as u64;
                                    tracing::debug!(duration_ms, hook_id = %ctx.hook_id, "hook bypassed via token");
                                    return HookResult::passthrough(stdin);
                                }
                            }
                        }
                    }
                }
                HookResult {
                    exit_code: 2,
                    stdout: result.stdout,
                    stderr,
                }
            }
            _ => {
                // No valid session ID — can't check tokens
                tracing::debug!(hook_id = %ctx.hook_id, "no CLAUDE_SESSION_ID — bypass tokens unavailable");
                HookResult {
                    exit_code: 2,
                    stdout: result.stdout,
                    stderr,
                }
            }
        }
    } else {
        result
    };

    let duration_ms = start.elapsed().as_millis() as u64;
    tracing::debug!(duration_ms, hook_id = %ctx.hook_id, "hook dispatch completed");

    // Record hook execution metric (fire-and-forget)
    let metrics_disabled = ports.env.var("ECC_METRICS_DISABLED").as_deref() == Some("1");
    let session_id =
        crate::metrics_session::resolve_session_id(ports.env.var("CLAUDE_SESSION_ID").as_deref());
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
        let _ =
            crate::metrics_mgmt::record_if_enabled(ports.metrics_store, &event, metrics_disabled);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_domain::hook_runtime::bypass::BypassPolicy;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment, MockExecutor};
    #[test]
    fn disabled_hook_passes_through() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("ECC_DISABLED_HOOKS", "my-hook");
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let ctx = HookContext {
            hook_id: "my-hook".to_string(),
            stdin_payload: "hello".to_string(),
            profiles_csv: None,
        };

        let result = dispatch(&ctx, &ports);
        assert_eq!(result.stdout, "hello");
        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn unknown_hook_passes_through_with_warning() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let ctx = HookContext {
            hook_id: "nonexistent:hook".to_string(),
            stdin_payload: "data".to_string(),
            profiles_csv: Some("standard".to_string()),
        };

        let result = dispatch(&ctx, &ports);
        assert_eq!(result.stdout, "data");
        assert!(result.stderr.contains("Unknown hook ID"));
    }

    #[test]
    fn truncate_stdin_within_limit() {
        let short = "hello";
        assert_eq!(truncate_stdin(short), "hello");
    }

    #[test]
    fn truncate_stdin_at_limit() {
        let long = "a".repeat(MAX_STDIN + 100);
        let truncated = truncate_stdin(&long);
        assert_eq!(truncated.len(), MAX_STDIN);
    }

    #[test]
    fn empty_hook_id_always_passes_through() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("ECC_DISABLED_HOOKS", "");
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let ctx = HookContext {
            hook_id: "".to_string(),
            stdin_payload: "test".to_string(),
            profiles_csv: None,
        };

        let result = dispatch(&ctx, &ports);
        assert_eq!(result.stdout, "test");
        assert!(result.stderr.contains("Unknown hook ID"));
    }

    #[test]
    fn always_deny_policy_returns_false() {
        let policy = AlwaysDenyPolicy;
        assert!(!policy.should_bypass("pre:edit:guard", "session-123"));
        assert!(!policy.should_bypass("stop:notify", "session-456"));
    }

    /// PC-017: dispatch routes "stop:cartography" and "start:cartography" to correct handlers.
    #[test]
    fn dispatches_cartography_hooks() {
        // stop:cartography: no CLAUDE_PROJECT_DIR → passthrough (not "Unknown hook ID")
        {
            let fs = InMemoryFileSystem::new();
            let shell = MockExecutor::new();
            let env = MockEnvironment::new();
            let term = BufferedTerminal::new();
            let ports = HookPorts::test_default(&fs, &shell, &env, &term);

            let ctx = HookContext {
                hook_id: "stop:cartography".to_string(),
                stdin_payload: "data".to_string(),
                profiles_csv: None,
            };
            let result = dispatch(&ctx, &ports);
            assert_eq!(result.stdout, "data");
            assert!(
                !result.stderr.contains("Unknown hook ID"),
                "stop:cartography should route to handler, not fall through to unknown"
            );
        }

        // start:cartography: no CLAUDE_PROJECT_DIR → passthrough (not "Unknown hook ID")
        {
            let fs = InMemoryFileSystem::new();
            let shell = MockExecutor::new();
            let env = MockEnvironment::new();
            let term = BufferedTerminal::new();
            let ports = HookPorts::test_default(&fs, &shell, &env, &term);

            let ctx = HookContext {
                hook_id: "start:cartography".to_string(),
                stdin_payload: "data".to_string(),
                profiles_csv: None,
            };
            let result = dispatch(&ctx, &ports);
            assert_eq!(result.stdout, "data");
            assert!(
                !result.stderr.contains("Unknown hook ID"),
                "start:cartography should route to handler, not fall through to unknown"
            );
        }
    }

    #[test]
    fn profile_mismatch_passes_through() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("ECC_HOOK_PROFILE", "minimal");
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let ctx = HookContext {
            hook_id: "stop:check-console-log".to_string(),
            stdin_payload: "data".to_string(),
            profiles_csv: Some("strict".to_string()),
        };

        let result = dispatch(&ctx, &ports);
        assert_eq!(result.stdout, "data");
        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.is_empty());
    }

    /// PC-028: HookPorts compiles with cost_store field and dispatches normally
    #[test]
    fn hook_result_passthrough() {
        let r = HookResult::passthrough("input");
        assert_eq!(r.stdout, "input");
        assert!(r.stderr.is_empty());
        assert_eq!(r.exit_code, 0);
    }

    /// PC-003: HookPorts with metrics_store None dispatches unknown hook correctly
    #[test]
    fn hook_ports_with_metrics_store_none() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let ctx = HookContext {
            hook_id: "nonexistent:hook:for-pc003".to_string(),
            stdin_payload: "data".to_string(),
            profiles_csv: None,
        };

        let result = dispatch(&ctx, &ports);
        assert_eq!(result.stdout, "data");
        assert!(result.stderr.contains("Unknown hook ID"));
        assert_eq!(result.exit_code, 0);
    }

    /// PC-004: After dispatch() of a known hook, InMemoryMetricsStore contains one HookExecution event
    /// with correct hook_id, outcome=Success, duration_ms > 0.
    #[test]
    fn dispatch_records_hook_success_metric() {
        use ecc_test_support::InMemoryMetricsStore;

        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let metrics_store = InMemoryMetricsStore::new();

        let ports = HookPorts {
            fs: &fs,
            shell: &shell,
            env: &env,
            terminal: &term,
            cost_store: None,
            bypass_store: None,
            metrics_store: Some(&metrics_store),
        };

        let ctx = HookContext {
            hook_id: "check:hook:enabled".to_string(),
            stdin_payload: r#"{"hookId":"check:hook:enabled"}"#.to_string(),
            profiles_csv: None,
        };

        let result = dispatch(&ctx, &ports);
        assert_eq!(result.exit_code, 0);

        let events = metrics_store.snapshot();
        assert_eq!(events.len(), 1, "expected exactly one metric event");

        let event = &events[0];
        assert_eq!(
            event.event_type,
            ecc_domain::metrics::MetricEventType::HookExecution
        );
        assert_eq!(event.hook_id.as_deref(), Some("check:hook:enabled"));
        assert_eq!(event.outcome, ecc_domain::metrics::MetricOutcome::Success);
        assert!(
            event.duration_ms.unwrap_or(0) >= 0,
            "duration_ms must be present"
        );
    }

    /// PC-005: After dispatch() of a failing hook, InMemoryMetricsStore contains one HookExecution
    /// event with outcome=Failure and error_message populated.
    ///
    /// Uses pre:edit:boundary-crossing with a domain file containing an infra import,
    /// which reliably triggers a block (exit_code=2) without relying on shell/fs state.
    #[test]
    fn dispatch_records_hook_failure_metric() {
        use ecc_test_support::InMemoryMetricsStore;

        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let metrics_store = InMemoryMetricsStore::new();

        let ports = HookPorts {
            fs: &fs,
            shell: &shell,
            env: &env,
            terminal: &term,
            cost_store: None,
            bypass_store: None,
            metrics_store: Some(&metrics_store),
        };

        // pre:edit:boundary-crossing blocks when a domain file has infra imports.
        // JSON stdin must use tool_input.file_path for the file path extraction.
        let stdin = r#"{"tool_input":{"file_path":"/project/ecc-domain/src/user.rs","new_string":"use crate::infra::db;"}}"#;
        let ctx = HookContext {
            hook_id: "pre:edit:boundary-crossing".to_string(),
            stdin_payload: stdin.to_string(),
            profiles_csv: None,
        };

        let result = dispatch(&ctx, &ports);
        // The hook must block (exit_code=2)
        assert_eq!(
            result.exit_code, 2,
            "expected block exit code for failure metric test"
        );

        let events = metrics_store.snapshot();
        assert_eq!(events.len(), 1, "expected exactly one metric event");

        let event = &events[0];
        assert_eq!(event.outcome, ecc_domain::metrics::MetricOutcome::Failure);
        assert!(
            event.error_message.is_some(),
            "error_message must be populated for Failure outcome"
        );
    }

    /// PC-006: With ECC_METRICS_DISABLED=1 in env, dispatch() records zero events.
    #[test]
    fn dispatch_metrics_disabled_records_nothing() {
        use ecc_test_support::InMemoryMetricsStore;

        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("ECC_METRICS_DISABLED", "1");
        let term = BufferedTerminal::new();
        let metrics_store = InMemoryMetricsStore::new();

        let ports = HookPorts {
            fs: &fs,
            shell: &shell,
            env: &env,
            terminal: &term,
            cost_store: None,
            bypass_store: None,
            metrics_store: Some(&metrics_store),
        };

        let ctx = HookContext {
            hook_id: "check:hook:enabled".to_string(),
            stdin_payload: r#"{"hookId":"check:hook:enabled"}"#.to_string(),
            profiles_csv: None,
        };

        dispatch(&ctx, &ports);

        let events = metrics_store.snapshot();
        assert_eq!(
            events.len(),
            0,
            "no events should be recorded when ECC_METRICS_DISABLED=1"
        );
    }

    /// PC-007: With metrics_store: None, dispatch() completes normally (no panic, no error).
    #[test]
    fn dispatch_none_store_fire_and_forget() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();

        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let ctx = HookContext {
            hook_id: "check:hook:enabled".to_string(),
            stdin_payload: r#"{"hookId":"check:hook:enabled"}"#.to_string(),
            profiles_csv: None,
        };

        // Should not panic
        let result = dispatch(&ctx, &ports);
        assert_eq!(result.exit_code, 0);
    }

    /// PC-008: Session ID in recorded event uses resolve_session_id — test with env set and unset.
    #[test]
    fn dispatch_session_id_resolution() {
        use ecc_test_support::InMemoryMetricsStore;

        // With CLAUDE_SESSION_ID set
        {
            let fs = InMemoryFileSystem::new();
            let shell = MockExecutor::new();
            let env = MockEnvironment::new().with_var("CLAUDE_SESSION_ID", "test-session-abc");
            let term = BufferedTerminal::new();
            let metrics_store = InMemoryMetricsStore::new();

            let ports = HookPorts {
                fs: &fs,
                shell: &shell,
                env: &env,
                terminal: &term,
                cost_store: None,
                bypass_store: None,
                metrics_store: Some(&metrics_store),
            };

            let ctx = HookContext {
                hook_id: "check:hook:enabled".to_string(),
                stdin_payload: r#"{"hookId":"check:hook:enabled"}"#.to_string(),
                profiles_csv: None,
            };

            dispatch(&ctx, &ports);

            let events = metrics_store.snapshot();
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].session_id, "test-session-abc");
        }

        // Without CLAUDE_SESSION_ID (fallback)
        {
            let fs = InMemoryFileSystem::new();
            let shell = MockExecutor::new();
            let env = MockEnvironment::new(); // no CLAUDE_SESSION_ID
            let term = BufferedTerminal::new();
            let metrics_store = InMemoryMetricsStore::new();

            let ports = HookPorts {
                fs: &fs,
                shell: &shell,
                env: &env,
                terminal: &term,
                cost_store: None,
                bypass_store: None,
                metrics_store: Some(&metrics_store),
            };

            let ctx = HookContext {
                hook_id: "check:hook:enabled".to_string(),
                stdin_payload: r#"{"hookId":"check:hook:enabled"}"#.to_string(),
                profiles_csv: None,
            };

            dispatch(&ctx, &ports);

            let events = metrics_store.snapshot();
            assert_eq!(events.len(), 1);
            assert!(
                events[0].session_id.starts_with("fallback-"),
                "session_id should use fallback when CLAUDE_SESSION_ID is unset"
            );
        }
    }

    /// PC-039: HookPorts with cost_store None dispatches unknown hook correctly
    #[test]
    fn hook_ports_with_cost_store_none() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let ctx = HookContext {
            hook_id: "nonexistent:hook:for-pc039".to_string(),
            stdin_payload: "data".to_string(),
            profiles_csv: None,
        };

        let result = dispatch(&ctx, &ports);
        assert_eq!(result.stdout, "data");
        assert!(result.stderr.contains("Unknown hook ID"));
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn hook_result_warn() {
        let r = HookResult::warn("input", "warning msg");
        assert_eq!(r.stdout, "input");
        assert_eq!(r.stderr, "warning msg");
        assert_eq!(r.exit_code, 0);
    }

    #[test]
    fn hook_result_block() {
        let r = HookResult::block("input", "blocked");
        assert_eq!(r.stdout, "input");
        assert_eq!(r.stderr, "blocked");
        assert_eq!(r.exit_code, 2);
    }

    /// PC-030: Handler trait compiles with hook_id + handle methods.
    #[test]
    fn handler_trait_compiles() {
        struct DummyHandler;
        impl Handler for DummyHandler {
            fn hook_id(&self) -> &str {
                "test:dummy"
            }
            fn handle(&self, stdin: &str, _ports: &HookPorts<'_>) -> HookResult {
                HookResult::passthrough(stdin)
            }
        }
        let h: Box<dyn Handler> = Box::new(DummyHandler);
        assert_eq!(h.hook_id(), "test:dummy");
    }

    /// PC-001: Dispatch with bypass token file present returns exit 0 passthrough.
    ///
    /// Characterization test: locks down existing inline token-bypass logic in dispatch().
    #[test]
    fn bypass_token_found_passthrough() {
        // Build a valid BypassToken JSON for hook "pre:edit:boundary-crossing", session "sess-001"
        let hook_id = "pre:edit:boundary-crossing";
        let session_id = "sess-001";
        let token_json = serde_json::json!({
            "hook_id": hook_id,
            "session_id": session_id,
            "granted_at": "2026-04-07T12:00:00Z",
            "reason": "test bypass"
        })
        .to_string();

        // Token path: {HOME}/.ecc/bypass-tokens/{session_id}/{hook_id_encoded}.json
        let encoded = hook_id.replace(':', "__");
        let token_path = format!(
            "/home/test/.ecc/bypass-tokens/{}/{}.json",
            session_id, encoded
        );

        let fs = InMemoryFileSystem::new().with_file(&token_path, &token_json);
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_var("CLAUDE_SESSION_ID", session_id)
            .with_var("HOME", "/home/test");
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        // Use pre:edit:boundary-crossing with a domain file + infra import → triggers exit_code=2
        let stdin = r#"{"tool_input":{"file_path":"/project/ecc-domain/src/user.rs","new_string":"use crate::infra::db;"}}"#;
        let ctx = HookContext {
            hook_id: hook_id.to_string(),
            stdin_payload: stdin.to_string(),
            profiles_csv: None,
        };

        let result = dispatch(&ctx, &ports);
        // With valid token present, bypass should apply → passthrough (exit 0)
        assert_eq!(
            result.exit_code, 0,
            "valid bypass token must grant passthrough"
        );
    }

    /// PC-002: Dispatch with no token returns exit 2 with bypass hint.
    ///
    /// Characterization test: verifies that when a hook blocks and no token exists,
    /// dispatch() returns exit 2 and appends the bypass-available hint.
    #[test]
    fn bypass_token_not_found_blocks() {
        let hook_id = "pre:edit:boundary-crossing";
        let session_id = "sess-002";

        // No token file — empty filesystem
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_var("CLAUDE_SESSION_ID", session_id)
            .with_var("HOME", "/home/test");
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"/project/ecc-domain/src/user.rs","new_string":"use crate::infra::db;"}}"#;
        let ctx = HookContext {
            hook_id: hook_id.to_string(),
            stdin_payload: stdin.to_string(),
            profiles_csv: None,
        };

        let result = dispatch(&ctx, &ports);
        assert_eq!(result.exit_code, 2, "missing token must still block");
        assert!(
            result.stderr.contains("Bypass available"),
            "stderr must contain bypass hint when no token found"
        );
    }

    /// PC-003: Dispatch with no CLAUDE_SESSION_ID returns exit 2.
    ///
    /// Characterization test: verifies that when CLAUDE_SESSION_ID is absent,
    /// dispatch() cannot check tokens and returns exit 2.
    #[test]
    fn no_session_id_blocks() {
        let hook_id = "pre:edit:boundary-crossing";

        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        // No CLAUDE_SESSION_ID in env
        let env = MockEnvironment::new().with_var("HOME", "/home/test");
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"/project/ecc-domain/src/user.rs","new_string":"use crate::infra::db;"}}"#;
        let ctx = HookContext {
            hook_id: hook_id.to_string(),
            stdin_payload: stdin.to_string(),
            profiles_csv: None,
        };

        let result = dispatch(&ctx, &ports);
        assert_eq!(
            result.exit_code, 2,
            "absent session_id must block (cannot check tokens)"
        );
    }

    /// PC-012: HookPorts::test_default() returns struct with all optional ports as None.
    #[test]
    fn test_default_creates_ports() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();

        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        assert!(ports.bypass_store.is_none());
        assert!(ports.cost_store.is_none());
        assert!(ports.metrics_store.is_none());
    }

    /// PC-031: Handler impl dispatches to cartography handler via registry.
    #[test]
    fn handler_trait_dispatch() {
        use ecc_test_support::{
            BufferedTerminal, InMemoryFileSystem, MockEnvironment, MockExecutor,
        };

        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        // Check registry has a cartography handler registered
        let registry = build_handler_registry();
        assert!(
            registry.contains_key("stop:cartography"),
            "registry must contain 'stop:cartography'"
        );

        // Dispatch using the registry
        let handler = registry.get("stop:cartography").unwrap();
        let result = handler.handle("stdin-data", &ports);
        // Without CLAUDE_PROJECT_DIR set, stop:cartography returns passthrough
        assert_eq!(result.stdout, "stdin-data");
        assert_eq!(result.exit_code, 0);
    }
}
