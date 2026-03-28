use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

pub(super) fn validate_statusline(
    root: &Path,
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
) -> bool {
    let script_path = root.join("statusline").join("statusline-command.sh");
    let script_exists = fs.exists(&script_path);

    if script_exists {
        terminal.stdout_write("✓ Script exists\n");
    } else {
        terminal.stdout_write("✗ Script exists: statusline/statusline-command.sh not found\n");
    }

    let script_content = if script_exists {
        match fs.read_to_string(&script_path) {
            Ok(c) => Some(c),
            Err(e) => {
                terminal.stderr_write(&format!("ERROR: Cannot read statusline script: {e}\n"));
                None
            }
        }
    } else {
        None
    };

    let placeholder_ok = match &script_content {
        Some(c) => {
            let ok = !c.contains("__ECC_VERSION__");
            if ok {
                terminal.stdout_write("✓ No unresolved placeholder\n");
            } else {
                terminal.stdout_write("✗ No unresolved placeholder: __ECC_VERSION__ found in script\n");
            }
            ok
        }
        None => {
            terminal.stdout_write("✗ No unresolved placeholder: script unavailable\n");
            false
        }
    };

    let shebang_ok = match &script_content {
        Some(c) => {
            let ok = c.starts_with("#!/usr/bin/env bash") || c.starts_with("#!/bin/bash");
            if ok {
                terminal.stdout_write("✓ Valid shebang\n");
            } else {
                terminal.stdout_write("✗ Valid shebang: must start with #!/usr/bin/env bash or #!/bin/bash\n");
            }
            ok
        }
        None => {
            terminal.stdout_write("✗ Valid shebang: script unavailable\n");
            false
        }
    };

    let jq_ok = match &script_content {
        Some(c) => {
            let ok = c.contains("jq");
            if ok {
                terminal.stdout_write("✓ Uses jq\n");
            } else {
                terminal.stdout_write("✗ Uses jq: jq not found in script\n");
            }
            ok
        }
        None => {
            terminal.stdout_write("✗ Uses jq: script unavailable\n");
            false
        }
    };

    let settings_path = root.join("settings.json");
    let settings_ok = match fs.read_to_string(&settings_path) {
        Ok(content) => {
            let ok = content.contains("statusline-command.sh");
            if ok {
                terminal.stdout_write("✓ settings.json references statusline-command.sh\n");
            } else {
                terminal.stdout_write(
                    "✗ settings.json references statusline-command.sh: statusLine not configured\n",
                );
            }
            ok
        }
        Err(_) => {
            terminal.stdout_write(
                "✗ settings.json references statusline-command.sh: settings.json not found\n",
            );
            false
        }
    };

    let executable_ok = if script_exists {
        let ok = fs.is_executable(&script_path);
        if ok {
            terminal.stdout_write("✓ Script is executable\n");
        } else {
            terminal.stdout_write("✗ Script is executable: missing execute permission (chmod +x)\n");
        }
        ok
    } else {
        terminal.stdout_write("✗ Script is executable: script not found\n");
        false
    };

    script_exists && placeholder_ok && shebang_ok && jq_ok && settings_ok && executable_ok
}

#[cfg(test)]
mod tests {
    use super::super::{run_validate, ValidateTarget};
    use ecc_ports::fs::FileSystem;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem};
    use std::path::Path;

    fn term() -> BufferedTerminal {
        BufferedTerminal::new()
    }

    fn valid_script() -> &'static str {
        "#!/usr/bin/env bash\n# ECC statusline\njq '.model' <<< \"$CLAUDE_DATA\"\necho done\n"
    }

    fn valid_settings() -> &'static str {
        r#"{"statusLine": {"command": "/home/user/.claude/statusline-command.sh"}}"#
    }

    #[test]
    fn validate_statusline_pass_valid() {
        let fs = InMemoryFileSystem::new()
            .with_file("/root/statusline/statusline-command.sh", valid_script())
            .with_file("/root/settings.json", valid_settings());
        fs.set_permissions(Path::new("/root/statusline/statusline-command.sh"), 0o755)
            .unwrap();
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &ValidateTarget::Statusline,
            Path::new("/root")
        ));
        let stdout: Vec<_> = t.stdout_output();
        assert!(stdout.iter().any(|s| s.contains('✓')));
    }

    #[test]
    fn validate_statusline_fail_missing_script() {
        let fs = InMemoryFileSystem::new()
            .with_file("/root/settings.json", valid_settings());
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &ValidateTarget::Statusline,
            Path::new("/root")
        ));
        let stdout: Vec<_> = t.stdout_output();
        assert!(stdout.iter().any(|s| s.contains('✗') && s.contains("Script exists")));
    }

    #[test]
    fn validate_statusline_fail_unresolved_placeholder() {
        let script = "#!/usr/bin/env bash\njq '.x'\nECC_VERSION=\"__ECC_VERSION__\"\n";
        let fs = InMemoryFileSystem::new()
            .with_file("/root/statusline/statusline-command.sh", script)
            .with_file("/root/settings.json", valid_settings());
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &ValidateTarget::Statusline,
            Path::new("/root")
        ));
        let stdout: Vec<_> = t.stdout_output();
        assert!(stdout.iter().any(|s| s.contains('✗') && s.contains("placeholder")));
    }

    #[test]
    fn validate_statusline_pass_settings_command() {
        let fs = InMemoryFileSystem::new()
            .with_file("/root/statusline/statusline-command.sh", valid_script())
            .with_file("/root/settings.json", valid_settings());
        fs.set_permissions(Path::new("/root/statusline/statusline-command.sh"), 0o755)
            .unwrap();
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &ValidateTarget::Statusline,
            Path::new("/root")
        ));
        let stdout: Vec<_> = t.stdout_output();
        assert!(stdout.iter().any(|s| s.contains('✓') && s.contains("settings")));
    }

    #[test]
    fn validate_statusline_fail_bad_shebang() {
        let script = "#!/usr/bin/python\njq '.x'\n";
        let fs = InMemoryFileSystem::new()
            .with_file("/root/statusline/statusline-command.sh", script)
            .with_file("/root/settings.json", valid_settings());
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &ValidateTarget::Statusline,
            Path::new("/root")
        ));
        let stdout: Vec<_> = t.stdout_output();
        assert!(stdout.iter().any(|s| s.contains('✗') && s.contains("shebang")));
    }

    #[test]
    fn validate_statusline_fail_no_jq() {
        let script = "#!/usr/bin/env bash\necho hello\n";
        let fs = InMemoryFileSystem::new()
            .with_file("/root/statusline/statusline-command.sh", script)
            .with_file("/root/settings.json", valid_settings());
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &ValidateTarget::Statusline,
            Path::new("/root")
        ));
        let stdout: Vec<_> = t.stdout_output();
        assert!(stdout.iter().any(|s| s.contains('✗') && s.contains("jq")));
    }

    #[test]
    fn validate_statusline_fail_not_executable() {
        let fs = InMemoryFileSystem::new()
            .with_file("/root/statusline/statusline-command.sh", valid_script())
            .with_file("/root/settings.json", valid_settings());
        // Script exists but no executable permission set
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &ValidateTarget::Statusline,
            Path::new("/root")
        ));
        let stdout: Vec<_> = t.stdout_output();
        assert!(stdout.iter().any(|s| s.contains('✗') && s.contains("executable")));
    }
}
