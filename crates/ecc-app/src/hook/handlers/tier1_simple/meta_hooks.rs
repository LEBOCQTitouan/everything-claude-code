use crate::hook::{HookPorts, HookResult};
use ecc_domain::hook_runtime::profiles::{HookEnabledOptions, is_hook_enabled};

/// check-hook-enabled: returns "yes" or "no" based on profile.
pub fn check_hook_enabled(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "check_hook_enabled", "executing handler");
    // This hook checks if a *different* hook is enabled.
    // The hook_id to check comes from stdin (JSON with hook_id field) or is just the raw stdin.
    let check_id = match serde_json::from_str::<serde_json::Value>(stdin) {
        Ok(v) => v
            .get("hook_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        Err(_) => stdin.trim().to_string(),
    };

    let profile_env = ports.env.var("ECC_HOOK_PROFILE");
    let disabled_env = ports.env.var("ECC_DISABLED_HOOKS");
    let opts = HookEnabledOptions::default();

    let enabled = if check_id.is_empty() {
        true
    } else {
        is_hook_enabled(
            &check_id,
            profile_env.as_deref(),
            disabled_env.as_deref(),
            &opts,
        )
    };

    HookResult {
        stdout: if enabled { "yes" } else { "no" }.to_string(),
        stderr: String::new(),
        exit_code: 0,
    }
}

/// session-end-marker: passthrough stdin (lifecycle marker, non-blocking).
pub fn session_end_marker(stdin: &str, _ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "session_end_marker", "executing handler");
    HookResult::passthrough(stdin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hook::HookPorts;
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
        }
    }

    // --- check_hook_enabled ---

    #[test]
    fn check_hook_enabled_enabled_hook_returns_yes() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = check_hook_enabled("some-hook", &ports);
        assert_eq!(result.stdout, "yes");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn check_hook_enabled_disabled_hook_returns_no() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("ECC_DISABLED_HOOKS", "some-hook");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = check_hook_enabled("some-hook", &ports);
        assert_eq!(result.stdout, "no");
    }

    #[test]
    fn check_hook_enabled_empty_stdin_always_returns_yes() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("ECC_DISABLED_HOOKS", "some-hook");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        // empty check_id → always enabled regardless of disabled list
        let result = check_hook_enabled("", &ports);
        assert_eq!(result.stdout, "yes");
    }

    #[test]
    fn check_hook_enabled_parses_json_hook_id() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("ECC_DISABLED_HOOKS", "json-hook");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"hook_id":"json-hook"}"#;
        let result = check_hook_enabled(stdin, &ports);
        assert_eq!(result.stdout, "no");
    }

    // --- session_end_marker ---

    #[test]
    fn session_end_marker_passthrough() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = session_end_marker("session data", &ports);
        assert_eq!(result.stdout, "session data");
        assert!(result.stderr.is_empty());
        assert_eq!(result.exit_code, 0);
    }
}
