mod common;

use common::EccTestEnv;
use predicates::prelude::*;

/// Known hook dispatches and exits zero.
#[test]
fn known_hook_passthrough_exits_zero() {
    let env = EccTestEnv::new();
    env.cmd()
        .args(["hook", "check:hook:enabled"])
        .write_stdin("{}")
        .assert()
        .success();
}

/// Unknown hook ID produces a warning on stderr but still exits zero (passthrough).
#[test]
fn unknown_hook_warns_on_stderr() {
    let env = EccTestEnv::new();
    env.cmd()
        .args(["hook", "totally:unknown:hook"])
        .write_stdin("{}")
        .assert()
        .success()
        .stderr(predicate::str::contains("unknown").or(predicate::str::contains("Unknown")));
}

/// Stdin payload flows through to stdout on an unknown hook (passthrough behavior).
#[test]
fn hook_stdin_flows_through() {
    let env = EccTestEnv::new();
    let payload = r#"{"tool":"Bash","input":{"command":"echo hello"}}"#;
    // Unknown hooks pass stdin through to stdout
    env.cmd()
        .args(["hook", "totally:unknown:hook"])
        .write_stdin(payload)
        .assert()
        .success()
        .stdout(predicate::str::contains("Bash"));
}

/// Hook disabled via ECC_DISABLED_HOOKS env var results in passthrough.
#[test]
fn hook_disabled_via_env_var() {
    let env = EccTestEnv::new();
    let payload = r#"{"tool":"Bash","input":{"command":"echo hello"}}"#;
    env.cmd()
        .env("ECC_DISABLED_HOOKS", "check:hook:enabled")
        .args(["hook", "check:hook:enabled"])
        .write_stdin(payload)
        .assert()
        .success()
        // When disabled, stdin is passed through (not the "yes" output)
        .stdout(predicate::str::contains("Bash"));
}

/// stop:notify dispatches and exits zero (fire-and-forget).
#[test]
fn stop_notify_dispatches_exits_zero() {
    let env = EccTestEnv::new();
    env.cmd()
        .args(["hook", "stop:notify", "minimal,standard,strict"])
        .write_stdin("{}")
        .assert()
        .success();
}

/// stop:notify disabled via ECC_NOTIFY_ENABLED=0 still exits zero.
#[test]
fn stop_notify_disabled_exits_zero() {
    let env = EccTestEnv::new();
    env.cmd()
        .env("ECC_NOTIFY_ENABLED", "0")
        .args(["hook", "stop:notify", "minimal,standard,strict"])
        .write_stdin("{}")
        .assert()
        .success()
        .stdout(predicate::str::contains("{}"));
}

/// post:failure:error-context dispatches and exits zero.
#[test]
fn post_failure_error_context_exits_zero() {
    let env = EccTestEnv::new();
    env.cmd()
        .args(["hook", "post:failure:error-context", "standard,strict"])
        .write_stdin(r#"{"tool_name":"Bash","error":"Build failed"}"#)
        .assert()
        .success();
}

/// pre:prompt:context-inject dispatches and exits zero.
#[test]
fn pre_prompt_context_inject_exits_zero() {
    let env = EccTestEnv::new();
    env.cmd()
        .args(["hook", "pre:prompt:context-inject", "standard,strict"])
        .write_stdin("{}")
        .assert()
        .success();
}

/// post:compact:state-save dispatches and exits zero.
#[test]
fn post_compact_state_save_exits_zero() {
    let env = EccTestEnv::new();
    env.cmd()
        .args(["hook", "post:compact:state-save", "standard,strict"])
        .write_stdin(r#"{"compact_summary":"test summary"}"#)
        .assert()
        .success();
}

/// subagent:start:log dispatches and exits zero.
#[test]
fn subagent_start_log_exits_zero() {
    let env = EccTestEnv::new();
    env.cmd()
        .args(["hook", "subagent:start:log", "standard,strict"])
        .write_stdin(r#"{"agent_type":"code-reviewer"}"#)
        .assert()
        .success();
}

/// subagent:stop:log dispatches and exits zero.
#[test]
fn subagent_stop_log_exits_zero() {
    let env = EccTestEnv::new();
    env.cmd()
        .args(["hook", "subagent:stop:log", "standard,strict"])
        .write_stdin(r#"{"agent_type":"architect"}"#)
        .assert()
        .success();
}

/// post:enter-worktree:session-log dispatches and exits zero.
#[test]
fn enter_worktree_session_log_exits_zero() {
    let env = EccTestEnv::new();
    env.cmd()
        .args(["hook", "post:enter-worktree:session-log", "standard,strict"])
        .write_stdin(r#"{"tool_name":"EnterWorktree","tool_input":{"worktree_path":"/tmp/wt"}}"#)
        .assert()
        .success();
}

/// instructions:loaded:validate dispatches and exits zero.
#[test]
fn instructions_loaded_validate_exits_zero() {
    let env = EccTestEnv::new();
    env.cmd()
        .args(["hook", "instructions:loaded:validate", "standard,strict"])
        .write_stdin(r#"{"instructions_path":"/tmp/nonexistent.md"}"#)
        .assert()
        .success();
}

/// config:change:log dispatches and exits zero.
#[test]
fn config_change_log_exits_zero() {
    let env = EccTestEnv::new();
    env.cmd()
        .args(["hook", "config:change:log", "minimal,standard,strict"])
        .write_stdin(r#"{"config_key":"theme","config_value":"dark"}"#)
        .assert()
        .success();
}

/// post:exit-worktree:cleanup-reminder dispatches and exits zero.
#[test]
fn exit_worktree_cleanup_reminder_exits_zero() {
    let env = EccTestEnv::new();
    env.cmd()
        .args([
            "hook",
            "post:exit-worktree:cleanup-reminder",
            "standard,strict",
        ])
        .write_stdin(r#"{"tool_name":"ExitWorktree","tool_input":{"worktree_path":"/tmp/wt"}}"#)
        .assert()
        .success();
}

/// 3-arg legacy format actually dispatches (not just parses).
/// check:hook:enabled outputs "yes" — verifying it actually ran the handler.
#[test]
fn hook_three_arg_legacy_dispatches_correctly() {
    let env = EccTestEnv::new();
    let payload = r#"{"tool":"Bash","input":{"command":"echo hello"}}"#;
    env.cmd()
        .args([
            "hook",
            "check:hook:enabled",
            "path/to/old-script.js",
            "standard",
        ])
        .write_stdin(payload)
        .assert()
        .success()
        // check:hook:enabled outputs "yes" when it successfully dispatches
        .stdout(predicate::str::contains("yes"));
}
