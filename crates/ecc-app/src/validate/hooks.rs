use ecc_domain::config::validate::{VALID_HOOK_EVENTS, check_hook_entry};
use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

pub(super) fn validate_hooks(root: &Path, fs: &dyn FileSystem, terminal: &dyn TerminalIO) -> bool {
    let hooks_file = root.join("hooks").join("hooks.json");
    if !fs.exists(&hooks_file) {
        terminal.stdout_write("No hooks.json found, skipping validation\n");
        return true;
    }

    let content = match fs.read_to_string(&hooks_file) {
        Ok(c) => c,
        Err(e) => {
            terminal.stderr_write(&format!("ERROR: Cannot read hooks.json: {e}\n"));
            return false;
        }
    };
    let data: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            terminal.stderr_write(&format!("ERROR: Invalid JSON in hooks.json: {e}\n"));
            return false;
        }
    };

    let hooks = data.get("hooks").unwrap_or(&data);
    let mut has_errors = false;
    let mut total_matchers = 0;

    if let Some(obj) = hooks.as_object() {
        for (event_type, matchers) in obj {
            if !VALID_HOOK_EVENTS.contains(&event_type.as_str()) {
                terminal.stderr_write(&format!("ERROR: Invalid event type: {}\n", event_type));
                has_errors = true;
                continue;
            }

            let matchers = match matchers.as_array() {
                Some(a) => a,
                None => {
                    terminal.stderr_write(&format!("ERROR: {} must be an array\n", event_type));
                    has_errors = true;
                    continue;
                }
            };

            for (i, matcher) in matchers.iter().enumerate() {
                if !validate_hook_matcher(matcher, event_type, i, terminal) {
                    has_errors = true;
                }
                total_matchers += 1;
            }
        }
    }

    if has_errors {
        return false;
    }

    terminal.stdout_write(&format!("Validated {} hook matchers\n", total_matchers));
    true
}

fn validate_hook_matcher(
    matcher: &serde_json::Value,
    event_type: &str,
    idx: usize,
    terminal: &dyn TerminalIO,
) -> bool {
    let obj = match matcher.as_object() {
        Some(o) => o,
        None => {
            terminal.stderr_write(&format!(
                "ERROR: {}[{}] is not an object\n",
                event_type, idx
            ));
            return false;
        }
    };

    let mut valid = true;

    if obj.get("matcher").and_then(|v| v.as_str()).is_none() {
        terminal.stderr_write(&format!(
            "ERROR: {}[{}] missing 'matcher' field\n",
            event_type, idx
        ));
        valid = false;
    }

    match obj.get("hooks").and_then(|v| v.as_array()) {
        Some(hooks) => {
            for (j, hook) in hooks.iter().enumerate() {
                let label = format!("{}[{}].hooks[{}]", event_type, idx, j);
                let errors = check_hook_entry(hook, &label);
                for err in &errors {
                    terminal.stderr_write(&format!("ERROR: {err}\n"));
                }
                if !errors.is_empty() {
                    valid = false;
                }
            }
        }
        None => {
            terminal.stderr_write(&format!(
                "ERROR: {}[{}] missing 'hooks' array\n",
                event_type, idx
            ));
            valid = false;
        }
    }

    valid
}

#[cfg(test)]
mod tests {
    use super::super::{ValidateTarget, run_validate};
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment};
    use std::path::Path;

    fn term() -> BufferedTerminal {
        BufferedTerminal::new()
    }

    #[test]
    fn hooks_no_file_succeeds() {
        let fs = InMemoryFileSystem::new();
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Hooks,
            Path::new("/root")
        ));
    }

    #[test]
    fn hooks_valid() {
        let json = r#"{"PreToolUse": [{"matcher": "Write", "hooks": [{"type": "command", "command": "echo ok"}]}]}"#;
        let fs = InMemoryFileSystem::new().with_file("/root/hooks/hooks.json", json);
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Hooks,
            Path::new("/root")
        ));
    }

    #[test]
    fn hooks_invalid_json() {
        let fs = InMemoryFileSystem::new().with_file("/root/hooks/hooks.json", "not json");
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Hooks,
            Path::new("/root")
        ));
        assert!(t.stderr_output().iter().any(|s| s.contains("Invalid JSON")));
    }

    #[test]
    fn hooks_invalid_event() {
        let json = r#"{"InvalidEvent": [{"matcher": "x", "hooks": [{"type": "command", "command": "echo"}]}]}"#;
        let fs = InMemoryFileSystem::new().with_file("/root/hooks/hooks.json", json);
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Hooks,
            Path::new("/root")
        ));
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("Invalid event type"))
        );
    }
}
