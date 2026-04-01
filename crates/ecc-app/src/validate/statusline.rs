use ecc_ports::env::Environment;
use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

pub(super) fn validate_statusline(
    root: &Path,
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
    env: &dyn Environment,
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

    // Check the INSTALLED script for unresolved placeholders, not the source template.
    // The source template intentionally has __ECC_VERSION__ — it's substituted during install.
    let home_script = env
        .home_dir()
        .map(|h| h.join(".claude").join("statusline-command.sh"))
        .unwrap_or_default();
    let installed_content = fs.read_to_string(&home_script).ok();
    let check_content = installed_content.as_deref().or(script_content.as_deref());

    let placeholder_ok = match check_content {
        Some(c) => {
            let has_placeholder = c.contains("__ECC_VERSION__");
            if !has_placeholder {
                terminal.stdout_write("✓ No unresolved placeholder\n");
            } else {
                terminal.stdout_write(
                    "✗ Unresolved placeholder: __ECC_VERSION__ found in installed script\n",
                );
            }
            !has_placeholder
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
                terminal.stdout_write(
                    "✗ Valid shebang: must start with #!/usr/bin/env bash or #!/bin/bash\n",
                );
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

    // Check settings.json — try root/settings.json first (tests, project-local),
    // then fall back to ~/.claude/settings.json (user global settings)
    let local_settings = root.join("settings.json");
    let home_settings = env
        .home_dir()
        .map(|h| h.join(".claude").join("settings.json"))
        .unwrap_or_default();
    let settings_path = if fs.exists(&local_settings) {
        local_settings
    } else {
        home_settings
    };
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
            terminal
                .stdout_write("✗ Script is executable: missing execute permission (chmod +x)\n");
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
    use super::super::{ValidateTarget, run_validate};
    use ecc_ports::fs::FileSystem;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment};
    use std::path::Path;

    fn term() -> BufferedTerminal {
        BufferedTerminal::new()
    }

    fn env() -> MockEnvironment {
        MockEnvironment::default()
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
            &env(),
            &ValidateTarget::Statusline,
            Path::new("/root")
        ));
        let stdout: Vec<_> = t.stdout_output();
        assert!(stdout.iter().any(|s| s.contains('✓')));
    }

    #[test]
    fn validate_statusline_fail_missing_script() {
        let fs = InMemoryFileSystem::new().with_file("/root/settings.json", valid_settings());
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &env(),
            &ValidateTarget::Statusline,
            Path::new("/root")
        ));
        let stdout: Vec<_> = t.stdout_output();
        assert!(
            stdout
                .iter()
                .any(|s| s.contains('✗') && s.contains("Script exists"))
        );
    }

    #[test]
    fn validate_statusline_fails_unresolved_placeholder_in_installed() {
        // Source template has placeholder (expected), but if the installed copy
        // also has it, that's a real failure
        let script = "#!/usr/bin/env bash\njq '.x'\nECC_VERSION=\"__ECC_VERSION__\"\n";
        let fs = InMemoryFileSystem::new()
            .with_file("/root/statusline/statusline-command.sh", script)
            .with_file("/root/settings.json", valid_settings());
        fs.set_permissions(Path::new("/root/statusline/statusline-command.sh"), 0o755)
            .unwrap();
        let t = term();
        // When no installed script exists and source has placeholder, it falls
        // back to source content and reports failure
        assert!(!run_validate(
            &fs,
            &t,
            &env(),
            &ValidateTarget::Statusline,
            Path::new("/root")
        ));
        let stdout: Vec<_> = t.stdout_output();
        assert!(
            stdout
                .iter()
                .any(|s| s.contains('✗') && s.contains("placeholder"))
        );
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
            &env(),
            &ValidateTarget::Statusline,
            Path::new("/root")
        ));
        let stdout: Vec<_> = t.stdout_output();
        assert!(
            stdout
                .iter()
                .any(|s| s.contains('✓') && s.contains("settings"))
        );
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
            &env(),
            &ValidateTarget::Statusline,
            Path::new("/root")
        ));
        let stdout: Vec<_> = t.stdout_output();
        assert!(
            stdout
                .iter()
                .any(|s| s.contains('✗') && s.contains("shebang"))
        );
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
            &env(),
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
            &env(),
            &ValidateTarget::Statusline,
            Path::new("/root")
        ));
        let stdout: Vec<_> = t.stdout_output();
        assert!(
            stdout
                .iter()
                .any(|s| s.contains('✗') && s.contains("executable"))
        );
    }

    #[test]
    fn validate_statusline_uses_env_for_home_lookup() {
        // This test verifies that HOME lookup uses the injected Environment trait,
        // not std::env::var("HOME"). We set HOME to /mock-home and put
        // a clean installed script there. We also put a script with an unresolved
        // placeholder at the real process HOME path (if accessible), so the test
        // fails if std::env::var("HOME") is used instead of the env trait.
        //
        // Concretely: the installed script at /mock-home has no placeholder (valid),
        // but if std::env::var("HOME") were used, the lookup path would be outside
        // the in-memory FS and fall back to source content — which also has no
        // placeholder, so we can't distinguish that way.
        //
        // Better approach: use a source script WITH a placeholder, and put the
        // clean installed script only at /mock-home. If HOME lookup falls back to
        // source (because the real HOME path isn't in the FS), placeholder check
        // fails. If HOME lookup uses the mock env and finds /mock-home script, pass.
        let source_with_placeholder = "#!/usr/bin/env bash
jq '.x'
VER=__ECC_VERSION__
";
        let installed_clean = valid_script(); // no placeholder
        let fs = InMemoryFileSystem::new()
            .with_file(
                "/root/statusline/statusline-command.sh",
                source_with_placeholder,
            )
            .with_file("/root/settings.json", valid_settings())
            .with_file("/mock-home/.claude/statusline-command.sh", installed_clean);
        fs.set_permissions(Path::new("/root/statusline/statusline-command.sh"), 0o755)
            .unwrap();
        let mock_env = MockEnvironment::default().with_home("/mock-home");
        let t = term();
        // If env.home_dir() is used: finds /mock-home/.claude/statusline-command.sh
        // (no placeholder) → placeholder check passes → overall passes.
        // If std::env::var("HOME") or env.var("HOME") is used: path not in FS →
        // falls back to source script which HAS placeholder → fails.
        assert!(run_validate(
            &fs,
            &t,
            &mock_env,
            &ValidateTarget::Statusline,
            Path::new("/root")
        ));
    }
}
