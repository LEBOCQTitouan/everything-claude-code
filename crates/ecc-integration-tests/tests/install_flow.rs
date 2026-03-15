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
