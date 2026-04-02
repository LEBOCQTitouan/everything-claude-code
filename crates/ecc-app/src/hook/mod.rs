//! Hook dispatch use case — routes hookId to the appropriate handler.

use ecc_domain::hook_runtime::profiles::{HookEnabledOptions, is_hook_enabled};
use ecc_ports::env::Environment;
use ecc_ports::fs::FileSystem;
use ecc_ports::shell::ShellExecutor;
use ecc_ports::terminal::TerminalIO;

pub mod handlers;

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

        // Tier 1: Clean Craft hooks
        "pre:edit:boundary-crossing" => handlers::pre_edit_boundary_crossing(stdin, ports),
        "post:edit:boy-scout-delta" => handlers::post_edit_boy_scout_delta(stdin, ports),
        "post:edit:naming-review" => handlers::post_edit_naming_review(stdin, ports),
        "post:edit:newspaper-check" => handlers::post_edit_newspaper_check(stdin, ports),
        "pre:edit:stepdown-warning" => handlers::pre_edit_stepdown_warning(stdin, ports),

        // New event handlers
        "post:failure:error-context" => handlers::post_failure_error_context(stdin, ports),
        "pre:prompt:context-inject" => handlers::pre_prompt_context_inject(stdin, ports),
        "post:compact:state-save" => handlers::post_compact(stdin, ports),
        "subagent:start:log" => handlers::subagent_start_log(stdin, ports),
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
        "start:cartography" => handlers::start_cartography(stdin, ports),
        "stop:cartography" => handlers::stop_cartography(stdin, ports),
        "pre:compact" => handlers::pre_compact(stdin, ports),
        "stop:evaluate-session" => handlers::evaluate_session(stdin, ports),
        "stop:cost-tracker" => handlers::cost_tracker(stdin, ports),
        "stop:oath-reflection" => handlers::oath_reflection(stdin, ports),
        "stop:craft-velocity" => handlers::craft_velocity(stdin, ports),
        "stop:daily-summary" => handlers::daily_summary(stdin, ports),
        "stop:cartography" => handlers::stop_cartography(stdin, ports),
        "start:cartography" => handlers::start_cartography(stdin, ports),

        // Unknown hook — passthrough with warning
        _ => {
            let msg = format!("[Hook] Unknown hook ID: {}\n", ctx.hook_id);
            HookResult::warn(stdin, &msg)
        }
    };

    let duration_ms = start.elapsed().as_millis() as u64;
    tracing::debug!(duration_ms, hook_id = %ctx.hook_id, "hook dispatch completed");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment, MockExecutor};

    fn make_ports<'a>(
        fs: &'a InMemoryFileSystem,
        shell: &'a MockExecutor,
        env: &'a MockEnvironment,
        term: &'a BufferedTerminal,
    ) -> HookPorts<'a> {
        HookPorts {
            fs,
            shell,
            env,
            terminal: term,
        }
    }

    #[test]
    fn disabled_hook_passes_through() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("ECC_DISABLED_HOOKS", "my-hook");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

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
        let ports = make_ports(&fs, &shell, &env, &term);

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
        let ports = make_ports(&fs, &shell, &env, &term);

        let ctx = HookContext {
            hook_id: "".to_string(),
            stdin_payload: "test".to_string(),
            profiles_csv: None,
        };

        let result = dispatch(&ctx, &ports);
        assert_eq!(result.stdout, "test");
        assert!(result.stderr.contains("Unknown hook ID"));
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
            let ports = make_ports(&fs, &shell, &env, &term);

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
            let ports = make_ports(&fs, &shell, &env, &term);

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
        let ports = make_ports(&fs, &shell, &env, &term);

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

    #[test]
    fn hook_result_passthrough() {
        let r = HookResult::passthrough("input");
        assert_eq!(r.stdout, "input");
        assert!(r.stderr.is_empty());
        assert_eq!(r.exit_code, 0);
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
}
