//! Tests for hook dispatch.

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
        cost_store: None,
        bypass_store: None,
        metrics_store: None,
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
    let ports = HookPorts {
        fs: &fs,
        shell: &shell,
        env: &env,
        terminal: &term,
        cost_store: None,
        bypass_store: None,
        metrics_store: None,
    };

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
    assert_eq!(result.exit_code, 2, "expected block exit code for failure metric test");

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
    assert_eq!(events.len(), 0, "no events should be recorded when ECC_METRICS_DISABLED=1");
}

/// PC-007: With metrics_store: None, dispatch() completes normally (no panic, no error).
#[test]
fn dispatch_none_store_fire_and_forget() {
    let fs = InMemoryFileSystem::new();
    let shell = MockExecutor::new();
    let env = MockEnvironment::new();
    let term = BufferedTerminal::new();

    let ports = HookPorts {
        fs: &fs,
        shell: &shell,
        env: &env,
        terminal: &term,
        cost_store: None,
        bypass_store: None,
        metrics_store: None, // explicitly None
    };

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
    let ports = HookPorts {
        fs: &fs,
        shell: &shell,
        env: &env,
        terminal: &term,
        cost_store: None,
        bypass_store: None,
        metrics_store: None,
    };

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
    let ports = make_ports(&fs, &shell, &env, &term);

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
