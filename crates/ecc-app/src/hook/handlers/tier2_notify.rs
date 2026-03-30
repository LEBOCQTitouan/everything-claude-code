//! Tier 2 Hook — System notification when Claude stops and needs user input.

use crate::hook::{HookPorts, HookResult};
use ecc_ports::env::Platform;

/// Default notification title.
const DEFAULT_TITLE: &str = "Claude Code";

/// Default notification message.
const DEFAULT_MESSAGE: &str = "Claude needs your attention";

/// Maximum length for sanitized notification strings.
const MAX_NOTIFY_LEN: usize = 256;

/// Truncate `s` to at most `max_bytes` bytes, walking back to a UTF-8 char boundary if needed.
fn truncate_to_char_boundary(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// Sanitize a string for safe interpolation into an AppleScript string delimited by double quotes.
/// Escapes backslashes first, then double quotes. Caps length at 256 chars.
fn sanitize_osascript(s: &str) -> String {
    let truncated = truncate_to_char_boundary(s, MAX_NOTIFY_LEN);
    truncated.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Sanitize a string for safe interpolation into a PowerShell string delimited by single quotes.
/// Escapes single quotes by doubling them. Caps length at 256 chars.
fn sanitize_powershell(s: &str) -> String {
    let truncated = truncate_to_char_boundary(s, MAX_NOTIFY_LEN);
    truncated.replace('\'', "''")
}

/// Send a cross-platform system notification. Fire-and-forget: failures are logged.
fn send_notification(title: &str, message: &str, ports: &HookPorts<'_>) {
    match ports.env.platform() {
        Platform::MacOS => {
            let safe_message = sanitize_osascript(message);
            let safe_title = sanitize_osascript(title);
            let script = format!(
                "display notification \"{}\" with title \"{}\" sound name \"Glass\"",
                safe_message, safe_title
            );
            if let Err(err) = ports.shell.run_command("osascript", &["-e", &script]) {
                tracing::warn!("osascript notification failed: {err}");
            }
        }
        Platform::Linux => {
            if ports.shell.command_exists("notify-send")
                && let Err(err) = ports.shell.run_command("notify-send", &[title, message])
            {
                tracing::warn!("notify-send failed: {err}");
            }
        }
        Platform::Windows => {
            let safe_title = sanitize_powershell(title);
            let safe_message = sanitize_powershell(message);
            let ps_cmd = format!(
                "New-BurntToastNotification -Text '{}','{}'",
                safe_title, safe_message
            );
            let result = ports
                .shell
                .run_command("powershell", &["-Command", &ps_cmd]);
            if result.is_err()
                && let Err(err) = ports.shell.run_command("msg", &["*", message])
            {
                tracing::warn!("msg fallback notification failed: {err}");
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

// ── Sanitization tests (PC-013, PC-014, PC-015, PC-016, PC-055) ──────────────

#[cfg(test)]
mod tier2_notify {
    pub mod tests {
        use super::super::*;

        // PC-013: osascript builder escapes single and double quotes
        #[test]
        fn sanitize_osascript_escapes_quotes() {
            // Single quotes pass through unchanged (AppleScript uses double-quote delimited strings)
            assert_eq!(sanitize_osascript("it's fine"), "it's fine");
            // Double quotes are escaped with backslash
            assert_eq!(sanitize_osascript(r#"say "hello""#), r#"say \"hello\""#);
            // Both together
            assert_eq!(sanitize_osascript(r#"it's "great""#), r#"it's \"great\""#);
        }

        // PC-014: PowerShell builder escapes single quotes
        #[test]
        fn sanitize_powershell_escapes_quotes() {
            // Single quotes are doubled
            assert_eq!(sanitize_powershell("it's fine"), "it''s fine");
            // Double quotes pass through unchanged
            assert_eq!(sanitize_powershell(r#"say "hello""#), r#"say "hello""#);
            // Multiple single quotes
            assert_eq!(sanitize_powershell("it's a 'test'"), "it''s a ''test''");
        }

        // PC-015: Adversarial injection input blocked
        #[test]
        fn adversarial_injection_blocked() {
            // AppleScript injection via double quote (osascript uses double-quoted strings).
            // Every `"` in the input must be escaped to `\"` in the output so it cannot
            // break out of the surrounding AppleScript `display notification "..."` string.
            let injected = sanitize_osascript(r#"" with title "x" do shell script "echo pwned""#);
            // No `"` should appear in the output unless preceded by `\`
            let has_bare_quote = injected
                .char_indices()
                .filter(|&(_, c)| c == '"')
                .any(|(i, _)| i == 0 || injected.as_bytes()[i - 1] != b'\\');
            assert!(
                !has_bare_quote,
                "osascript output must have no bare double-quotes: {injected}"
            );

            // Backslash + double quote combo is also blocked (backslash escaped first)
            let injected2 = sanitize_osascript(r#"\"; system(\"rm -rf /\"); \""#);
            assert!(
                injected2.contains("\\\\"),
                "backslashes must be doubled: {injected2}"
            );

            // PowerShell injection via single quote
            let injected3 = sanitize_powershell("'; Start-Process cmd");
            assert!(
                injected3.starts_with("''; Start-Process"),
                "powershell injection should be escaped: {injected3}"
            );
        }

        // PC-055: osascript escapes backslashes + caps input length at 256
        #[test]
        fn sanitize_osascript_backslash_and_length() {
            // Backslash is escaped
            assert_eq!(sanitize_osascript("path\\to\\file"), "path\\\\to\\\\file");
            // Backslash + double quote: backslash must be escaped FIRST
            assert_eq!(sanitize_osascript("\\\""), "\\\\\\\"");
            // Length cap: 257 chars → truncated to 256
            let long_input = "a".repeat(257);
            let result = sanitize_osascript(&long_input);
            assert_eq!(result.len(), 256, "should be capped at 256 chars");
        }

        // PC-016: >= 5 adversarial inputs per platform (>= 10 total)
        // osascript adversarial inputs (5)
        #[test]
        fn adversarial_osascript_shell_injection() {
            let result = sanitize_osascript("'; rm -rf /; echo '");
            // Must not contain unescaped double quotes that break out of the AppleScript string
            assert!(!result.contains(r#"" "#), "no unescaped quotes: {result}");
        }

        #[test]
        fn adversarial_osascript_applescript_injection() {
            let result = sanitize_osascript(r#"" with title "x" \n do shell script "echo pwned""#);
            assert!(
                result.contains(r#"\""#),
                "double quotes must be escaped: {result}"
            );
        }

        #[test]
        fn adversarial_osascript_backslash_quote_combo() {
            let result = sanitize_osascript(r#"\"; system(\"rm -rf /\"); \""#);
            // Backslashes must be escaped before quotes
            assert!(
                result.contains("\\\\"),
                "backslashes must be doubled: {result}"
            );
            assert!(result.contains(r#"\""#), "quotes must be escaped: {result}");
        }

        #[test]
        fn adversarial_osascript_empty_string() {
            assert_eq!(sanitize_osascript(""), "");
        }

        #[test]
        fn adversarial_osascript_long_input_truncation() {
            let long_input = "x".repeat(300);
            let result = sanitize_osascript(&long_input);
            assert!(
                result.len() <= 256,
                "must be capped at 256: len={}",
                result.len()
            );
        }

        // PowerShell adversarial inputs (5)
        #[test]
        fn adversarial_powershell_single_quote_injection() {
            let result = sanitize_powershell("'; Start-Process cmd");
            assert_eq!(&result[..2], "''", "single quote must be doubled: {result}");
        }

        #[test]
        fn adversarial_powershell_empty_string() {
            assert_eq!(sanitize_powershell(""), "");
        }

        #[test]
        fn adversarial_powershell_long_input_truncation() {
            let long_input = "y".repeat(300);
            let result = sanitize_powershell(&long_input);
            assert!(
                result.len() <= 256,
                "must be capped at 256: len={}",
                result.len()
            );
        }

        #[test]
        fn adversarial_powershell_unicode() {
            let result = sanitize_powershell("hello 🔥 world");
            // Unicode passes through; no single quotes to escape
            assert!(
                result.contains("🔥"),
                "unicode should pass through: {result}"
            );
        }

        #[test]
        fn adversarial_powershell_mixed_injection() {
            let result = sanitize_powershell("Hello \"World\" it's a \\'test\\\\");
            // Single quotes are doubled
            assert!(
                result.contains("''"),
                "single quotes must be doubled: {result}"
            );
        }

        // Additional osascript adversarial inputs to exceed 5 per platform
        #[test]
        fn adversarial_osascript_unicode_passthrough() {
            let result = sanitize_osascript("hello 🔥 world");
            assert!(
                result.contains("🔥"),
                "unicode should pass through: {result}"
            );
        }

        #[test]
        fn adversarial_osascript_newlines() {
            let result = sanitize_osascript("line1\nline2");
            // Newlines pass through (osascript handles them)
            assert!(
                result.contains('\n'),
                "newlines should pass through: {result}"
            );
        }

    }
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

    // PC-025: sanitize_osascript uses is_char_boundary() walk-back for multi-byte truncation
    #[test]
    fn sanitize_osascript_multibyte_truncation() {
        // Build a string: 254 ASCII bytes + one emoji (4 bytes) = 258 bytes total.
        // Slicing at byte 256 falls in the middle of the emoji — current code panics.
        let mut input = "a".repeat(254);
        input.push('🔥'); // 4-byte UTF-8 character
        assert_eq!(input.len(), 258, "test string must be 258 bytes");
        // Must not panic; result must be valid UTF-8 with length <= 256
        let result = sanitize_osascript(&input);
        assert!(result.len() <= 256, "result must be capped at 256 bytes: len={}", result.len());
        // Emoji starts at byte 254, ends at 258 — truncation at 256 is mid-emoji, walk-back drops to 254
        assert_eq!(result, "a".repeat(254), "ascii prefix must be preserved");
    }

    // PC-026: sanitize_powershell uses is_char_boundary() walk-back for multi-byte truncation
    #[test]
    fn sanitize_powershell_multibyte_truncation() {
        // Build a string: 254 ASCII bytes + one emoji (4 bytes) = 258 bytes total.
        // Slicing at byte 256 falls in the middle of the emoji — current code panics.
        let mut input = "b".repeat(254);
        input.push('🌊'); // 4-byte UTF-8 character
        assert_eq!(input.len(), 258, "test string must be 258 bytes");
        // Must not panic; result must be valid UTF-8 with length <= 256
        let result = sanitize_powershell(&input);
        assert!(result.len() <= 256, "result must be capped at 256 bytes: len={}", result.len());
        // Emoji starts at byte 254, so truncation at 256 is mid-emoji; walk-back drops to 254
        assert_eq!(result, "b".repeat(254), "ascii prefix must be preserved");
    }
}
