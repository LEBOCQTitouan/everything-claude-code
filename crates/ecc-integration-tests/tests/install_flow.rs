mod common;

use common::EccTestEnv;

#[test]
fn fresh_install_creates_agent_files() {
    let env = EccTestEnv::new();
    env.install(&[]).success();

    assert!(
        env.file_exists("agents"),
        "agents/ directory should exist after install"
    );
    // At least one agent file should be present
    let agents_dir = env.claude_dir().join("agents");
    let count = std::fs::read_dir(&agents_dir)
        .expect("failed to read agents dir")
        .count();
    assert!(count > 0, "agents/ should contain files after install");
}

#[test]
fn fresh_install_creates_manifest() {
    let env = EccTestEnv::new();
    env.install(&[]).success();

    assert!(
        env.file_exists(".ecc-manifest.json"),
        "manifest should exist after install"
    );

    let manifest: serde_json::Value =
        serde_json::from_str(&env.read_file(".ecc-manifest.json")).expect("manifest is valid JSON");
    assert!(
        manifest.get("version").is_some(),
        "manifest should have a version field"
    );
    assert!(
        manifest.get("artifacts").is_some(),
        "manifest should have an artifacts field"
    );
}

#[test]
fn fresh_install_writes_deny_rules_to_settings() {
    let env = EccTestEnv::new();
    env.install(&[]).success();

    let settings = env.settings_json();
    assert!(
        settings.get("permissions").is_some(),
        "settings.json should have permissions after install"
    );
    let deny = settings["permissions"]["deny"]
        .as_array()
        .expect("deny should be an array");
    assert!(!deny.is_empty(), "deny rules should be populated");
}

#[test]
fn install_dry_run_writes_nothing() {
    let env = EccTestEnv::new();
    env.install(&["--dry-run"]).success();

    assert!(
        !env.file_exists("agents"),
        "agents/ should NOT exist after dry-run install"
    );
    assert!(
        !env.file_exists(".ecc-manifest.json"),
        "manifest should NOT exist after dry-run install"
    );
}

#[test]
fn install_idempotent() {
    let env = EccTestEnv::new();

    // First install
    env.install(&[]).success();
    let manifest_first = env.read_file(".ecc-manifest.json");

    // Second install
    env.install(&[]).success();
    let manifest_second = env.read_file(".ecc-manifest.json");

    // Both should produce valid manifests with same version and artifacts
    let v1: serde_json::Value = serde_json::from_str(&manifest_first).unwrap();
    let v2: serde_json::Value = serde_json::from_str(&manifest_second).unwrap();
    assert_eq!(v1["version"], v2["version"]);
    assert_eq!(v1["artifacts"], v2["artifacts"]);
}

#[test]
fn install_preserves_user_settings() {
    let env = EccTestEnv::new();

    // Pre-seed settings.json with user content
    let user_settings = r#"{
  "permissions": {
    "deny": ["Bash(rm -rf:*)"]
  },
  "customField": "user-value"
}"#;
    env.write_file("settings.json", user_settings);

    env.install(&[]).success();

    let settings = env.settings_json();
    // User's custom field should survive install
    assert_eq!(
        settings["customField"].as_str(),
        Some("user-value"),
        "user fields should survive install"
    );
}

/// Extract all `ecc-hook` command strings from settings.json hooks sections.
fn extract_hook_commands(settings: &serde_json::Value) -> Vec<String> {
    let mut commands = Vec::new();
    let hooks = match settings.get("hooks").and_then(|h| h.as_object()) {
        Some(h) => h,
        None => return commands,
    };
    for (_event_type, entries) in hooks {
        let entries = match entries.as_array() {
            Some(a) => a,
            None => continue,
        };
        for entry in entries {
            let inner = match entry.get("hooks").and_then(|h| h.as_array()) {
                Some(a) => a,
                None => continue,
            };
            for hook in inner {
                if let Some(cmd) = hook.get("command").and_then(|c| c.as_str())
                    && (cmd.contains("ecc-hook") || cmd.contains("ecc hook")) {
                        commands.push(cmd.to_string());
                    }
            }
        }
    }
    commands
}

#[test]
fn install_hooks_round_trip_parseable_by_hook_command() {
    let env = EccTestEnv::new();
    env.install(&[]).success();

    let settings = env.settings_json();
    let commands = extract_hook_commands(&settings);

    assert!(
        !commands.is_empty(),
        "install should produce at least one hook command in settings.json"
    );

    for cmd_str in &commands {
        // Parse shell command string into argv
        let argv = shell_words::split(cmd_str)
            .unwrap_or_else(|e| panic!("failed to parse hook command {cmd_str:?}: {e}"));

        // argv[0] is "ecc-hook", skip it — we call `ecc hook` with the rest
        let args: Vec<&str> = argv.iter().skip(1).map(|s| s.as_str()).collect();
        assert!(
            !args.is_empty(),
            "hook command should have arguments: {cmd_str}"
        );

        // Run `ecc hook <args>` with empty JSON stdin
        let mut cmd = env.cmd();
        cmd.arg("hook");
        for arg in &args {
            cmd.arg(arg);
        }
        cmd.write_stdin("{}");
        cmd.assert().success();
    }
}

#[test]
fn install_replaces_legacy_3arg_hooks() {
    let env = EccTestEnv::new();

    // Pre-seed settings.json with a legacy 3-arg hook (cairn-style)
    // Format: ecc-hook "<hook_id>" "dist/hooks/<script>.sh" "<profiles>"
    let legacy_settings = r#"{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "ecc-hook \"pre:bash:dev-server-block\" \"dist/hooks/pre-bash-dev-server-block.sh\" \"standard,strict\""
          }
        ],
        "description": "Legacy hook entry"
      }
    ]
  }
}"#;
    env.write_file("settings.json", legacy_settings);

    env.install(&[]).success();

    let settings = env.settings_json();

    // Legacy dist/hooks/ path should be gone
    let raw = env.read_file("settings.json");
    assert!(
        !raw.contains("dist/hooks/"),
        "legacy dist/hooks/ path should be removed after install"
    );

    // All remaining hooks should be parseable
    let commands = extract_hook_commands(&settings);
    assert!(
        !commands.is_empty(),
        "replacement hooks should be present after install"
    );

    for cmd_str in &commands {
        let argv = shell_words::split(cmd_str)
            .unwrap_or_else(|e| panic!("failed to parse hook command {cmd_str:?}: {e}"));
        let args: Vec<&str> = argv.iter().skip(1).map(|s| s.as_str()).collect();

        let mut cmd = env.cmd();
        cmd.arg("hook");
        for arg in &args {
            cmd.arg(arg);
        }
        cmd.write_stdin("{}");
        cmd.assert().success();
    }
}
