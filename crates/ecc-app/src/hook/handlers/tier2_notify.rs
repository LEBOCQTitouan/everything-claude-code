//! Tier 2 Hook — System notification when Claude stops and needs user input.

use crate::hook::{HookPorts, HookResult};
use ecc_ports::env::Platform;

/// Default notification title.
const DEFAULT_TITLE: &str = "Claude Code";

/// Default notification message.
const DEFAULT_MESSAGE: &str = "Claude needs your attention";

/// Send a cross-platform system notification. Fire-and-forget: failures are silently ignored.
fn send_notification(title: &str, message: &str, ports: &HookPorts<'_>) {
    match ports.env.platform() {
        Platform::MacOS => {
            let script = format!(
                "display notification \"{}\" with title \"{}\" sound name \"Glass\"",
                message, title
            );
            let _ = ports.shell.run_command("osascript", &["-e", &script]);
        }
        Platform::Linux => {
            if ports.shell.command_exists("notify-send") {
                let _ = ports.shell.run_command("notify-send", &[title, message]);
            }
        }
        Platform::Windows => {
            let ps_cmd = format!("New-BurntToastNotification -Text '{}','{}'", title, message);
            let result = ports
                .shell
                .run_command("powershell", &["-Command", &ps_cmd]);
            if result.is_err() {
                let _ = ports.shell.run_command("msg", &["*", message]);
            }
        }
        Platform::Unknown => {}
    }
}

/// stop:notify — Send a system notification when Claude stops.
///
/// Reads optional env vars:
/// - `ECC_NOTIFY_ENABLED`: set to `"0"` or `"false"` to disable (default: enabled)
/// - `ECC_NOTIFY_TITLE`: notification title (default: "Claude Code")
/// - `ECC_NOTIFY_MESSAGE`: notification body (default: "Claude needs your attention")
///
/// Fire-and-forget: notification failures are silently ignored.
pub fn stop_notify(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    // Opt-out check
    if let Some(val) = ports.env.var("ECC_NOTIFY_ENABLED") {
        let lower = val.to_lowercase();
        if lower == "0" || lower == "false" {
            return HookResult::passthrough(stdin);
        }
    }

    let title = ports
        .env
        .var("ECC_NOTIFY_TITLE")
        .unwrap_or_else(|| DEFAULT_TITLE.to_string());
    let message = ports
        .env
        .var("ECC_NOTIFY_MESSAGE")
        .unwrap_or_else(|| DEFAULT_MESSAGE.to_string());

    send_notification(&title, &message, ports);

    HookResult::passthrough(stdin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hook::HookPorts;
    use ecc_ports::shell::CommandOutput;
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

    fn ok_output() -> CommandOutput {
        CommandOutput {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
        }
    }

    #[test]
    fn macos_runs_osascript() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new().on("osascript", ok_output());
        let env = MockEnvironment::new().with_platform(Platform::MacOS);
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = stop_notify("{}", &ports);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "{}");
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn linux_runs_notify_send() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new()
            .with_command("notify-send")
            .on("notify-send", ok_output());
        let env = MockEnvironment::new().with_platform(Platform::Linux);
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = stop_notify("{}", &ports);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "{}");
    }

    #[test]
    fn linux_without_notify_send_passthrough() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new(); // notify-send not registered
        let env = MockEnvironment::new().with_platform(Platform::Linux);
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = stop_notify("{}", &ports);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "{}");
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn windows_runs_powershell() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new().on("powershell", ok_output());
        let env = MockEnvironment::new().with_platform(Platform::Windows);
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = stop_notify("{}", &ports);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "{}");
    }

    #[test]
    fn windows_falls_back_to_msg_on_powershell_failure() {
        let fs = InMemoryFileSystem::new();
        // powershell not registered (will return NotFound error), msg is available
        let shell = MockExecutor::new().on("msg", ok_output());
        let env = MockEnvironment::new().with_platform(Platform::Windows);
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = stop_notify("{}", &ports);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "{}");
    }

    #[test]
    fn unknown_platform_passthrough() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_platform(Platform::Unknown);
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = stop_notify("{}", &ports);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "{}");
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn disabled_via_env_zero() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new(); // No commands registered — would fail if called
        let env = MockEnvironment::new()
            .with_platform(Platform::MacOS)
            .with_var("ECC_NOTIFY_ENABLED", "0");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = stop_notify("{}", &ports);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "{}");
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn disabled_via_env_false() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_platform(Platform::MacOS)
            .with_var("ECC_NOTIFY_ENABLED", "false");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = stop_notify("{}", &ports);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "{}");
    }

    #[test]
    fn custom_title_and_message() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new().on("osascript", ok_output());
        let env = MockEnvironment::new()
            .with_platform(Platform::MacOS)
            .with_var("ECC_NOTIFY_TITLE", "My Tool")
            .with_var("ECC_NOTIFY_MESSAGE", "Done!");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = stop_notify("{}", &ports);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "{}");
    }

    #[test]
    fn notification_failure_still_passthrough() {
        let fs = InMemoryFileSystem::new();
        // osascript not registered — run_command will return Err
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_platform(Platform::MacOS);
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = stop_notify("{}", &ports);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "{}");
        assert!(result.stderr.is_empty());
    }
}
