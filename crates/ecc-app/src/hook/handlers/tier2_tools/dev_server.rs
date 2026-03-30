//! Dev server blocking hook.

use crate::hook::{HookPorts, HookResult};
use ecc_ports::env::Platform;

use super::helpers::{extract_command, split_shell_segments};

/// pre-bash-dev-server-block: block dev servers outside tmux.
pub fn pre_bash_dev_server_block(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "pre_bash_dev_server_block", "executing handler");
    if ports.env.platform() == Platform::Windows {
        return HookResult::passthrough(stdin);
    }

    let cmd = extract_command(stdin);
    if cmd.is_empty() {
        return HookResult::passthrough(stdin);
    }

    let segments = split_shell_segments(&cmd);
    let dev_pattern = [
        "npm run dev",
        "pnpm dev",
        "pnpm run dev",
        "yarn dev",
        "bun run dev",
    ];

    let has_blocked_dev = segments.iter().any(|seg| {
        let is_dev = dev_pattern.iter().any(|p| seg.contains(p));
        let is_tmux_launched = seg.trim_start().starts_with("tmux ");
        is_dev && !is_tmux_launched
    });

    if has_blocked_dev {
        let msg = "[Hook] BLOCKED: Dev server must run in tmux for log access\n\
                   [Hook] Use: tmux new-session -d -s dev \"npm run dev\"\n\
                   [Hook] Then: tmux attach -t dev\n";
        return HookResult::block(stdin, msg);
    }

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
        }
    }

    #[test]
    fn dev_server_blocks_npm_run_dev() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"command":"npm run dev"}}"#;
        let result = pre_bash_dev_server_block(stdin, &ports);
        assert_eq!(result.exit_code, 2);
        assert!(result.stderr.contains("BLOCKED"));
    }

    #[test]
    fn dev_server_allows_tmux_wrapped() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"command":"tmux new-session -d -s dev \"npm run dev\""}}"#;
        let result = pre_bash_dev_server_block(stdin, &ports);
        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn dev_server_allows_non_dev_commands() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"command":"npm test"}}"#;
        let result = pre_bash_dev_server_block(stdin, &ports);
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn dev_server_passthrough_on_windows() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_platform(ecc_ports::env::Platform::Windows);
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"command":"npm run dev"}}"#;
        let result = pre_bash_dev_server_block(stdin, &ports);
        assert_eq!(result.exit_code, 0);
    }
}
