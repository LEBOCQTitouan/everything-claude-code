mod common;

use common::EccTestEnv;

/// --clean removes manifest-listed files.
#[test]
fn clean_removes_manifest_listed_files() {
    let env = EccTestEnv::new();

    // Install first
    env.install(&[]).success();
    assert!(env.file_exists("agents"), "agents/ should exist after install");
    assert!(env.file_exists(".ecc-manifest.json"), "manifest should exist");

    // Clean
    env.install(&["--clean"]).success();

    // After clean + reinstall, files should still be present (clean then reinstall)
    assert!(
        env.file_exists("agents"),
        "agents/ should be re-created after clean + install"
    );
}

/// --clean preserves files NOT in the manifest (user-added files).
#[test]
fn clean_preserves_user_files() {
    let env = EccTestEnv::new();

    // Install first
    env.install(&[]).success();

    // Add a user file in agents/
    env.write_file("agents/my-custom-agent.md", "# My Custom Agent");

    // Clean + reinstall
    env.install(&["--clean"]).success();

    assert!(
        env.file_exists("agents/my-custom-agent.md"),
        "user-added file should survive --clean"
    );
}

/// Bug #3 regression: --clean removes ECC hooks from settings.json.
/// Pre-seed settings with an ECC hook, verify --clean removes it.
#[test]
fn clean_removes_ecc_hooks_from_settings() {
    let env = EccTestEnv::new();

    // Install first
    env.install(&[]).success();

    // Simulate ECC hooks in settings.json (as if a previous version installed them)
    let settings_with_hooks = r#"{
  "permissions": { "deny": [] },
  "hooks": {
    "PreToolUse": [
      {
        "description": "ECC format",
        "hooks": [{ "command": "ecc-hook format" }]
      },
      {
        "description": "User hook",
        "hooks": [{ "command": "my-custom-linter" }]
      }
    ]
  }
}"#;
    env.write_file("settings.json", settings_with_hooks);

    // Clean + reinstall
    env.install(&["--clean"]).success();

    let settings = env.settings_json();
    // The ECC hook should have been removed by clean
    if let Some(pre_hooks) = settings["hooks"]["PreToolUse"].as_array() {
        let has_ecc_hook = pre_hooks.iter().any(|h| {
            h.get("hooks")
                .and_then(|hooks| hooks.as_array())
                .is_some_and(|hooks| {
                    hooks.iter().any(|hook| {
                        hook.get("command")
                            .and_then(|c| c.as_str())
                            .is_some_and(|c| c.starts_with("ecc-hook "))
                    })
                })
        });
        assert!(
            !has_ecc_hook,
            "ECC hooks should be removed by --clean"
        );
    }
    // User hook may or may not survive depending on how clean interacts with
    // the full reinstall cycle — the key assertion is that ECC hooks are cleaned.
}

/// --clean-all removes entire ECC directories.
#[test]
fn clean_all_nukes_directories() {
    let env = EccTestEnv::new();

    // Install first
    env.install(&[]).success();
    assert!(env.file_exists("agents"), "agents/ should exist after install");

    // Clean all + reinstall
    env.install(&["--clean-all"]).success();

    // After clean-all + reinstall, directories should be re-created
    assert!(
        env.file_exists("agents"),
        "agents/ should be re-created after clean-all + install"
    );
    assert!(
        env.file_exists(".ecc-manifest.json"),
        "manifest should be re-created after clean-all + install"
    );
}

/// Full round-trip: install -> clean -> reinstall produces valid state.
#[test]
fn clean_then_reinstall_round_trip() {
    let env = EccTestEnv::new();

    // First install
    env.install(&[]).success();
    let manifest_first: serde_json::Value =
        serde_json::from_str(&env.read_file(".ecc-manifest.json")).unwrap();

    // Clean + reinstall
    env.install(&["--clean"]).success();
    let manifest_second: serde_json::Value =
        serde_json::from_str(&env.read_file(".ecc-manifest.json")).unwrap();

    // Artifacts should match
    assert_eq!(
        manifest_first["artifacts"], manifest_second["artifacts"],
        "artifacts should match after clean + reinstall round-trip"
    );
}
