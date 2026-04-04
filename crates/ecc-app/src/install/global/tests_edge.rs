use super::install_global;
use super::tests::{ecc_source_fs, no_color_env};
use crate::install::{InstallContext, InstallOptions, default_install_options};
use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment, MockExecutor};
use std::path::Path;

#[test]
fn install_with_empty_source_dir_succeeds_with_zero_artifacts() {
    let fs = InMemoryFileSystem::new()
        .with_dir("/ecc/agents")
        .with_dir("/ecc/commands")
        .with_dir("/ecc/skills")
        .with_dir("/ecc/rules")
        .with_dir("/claude");
    let env = no_color_env();
    let terminal = BufferedTerminal::new();
    let shell = MockExecutor::new();
    let ctx = InstallContext {
        fs: &fs,
        shell: &shell,
        env: &env,
        terminal: &terminal,
    };
    let options = InstallOptions {
        dry_run: false,
        force: true,
        no_gitignore: false,
        interactive: false,
        clean: false,
        clean_all: false,
        languages: vec![],
        all_rules: true,
    };
    let summary = install_global(
        &ctx,
        Path::new("/ecc"),
        Path::new("/claude"),
        "1.0.0",
        "2026-03-15T00:00:00Z",
        &options,
    );
    assert!(summary.success);
    assert_eq!(summary.added, 0);
    assert_eq!(summary.updated, 0);
}

#[test]
fn install_with_missing_agents_dir_still_succeeds() {
    let fs = InMemoryFileSystem::new()
        .with_dir("/ecc")
        .with_dir("/claude");
    let env = no_color_env();
    let terminal = BufferedTerminal::new();
    let shell = MockExecutor::new();
    let ctx = InstallContext {
        fs: &fs,
        shell: &shell,
        env: &env,
        terminal: &terminal,
    };
    let options = InstallOptions {
        dry_run: false,
        force: true,
        no_gitignore: false,
        interactive: false,
        clean: false,
        clean_all: false,
        languages: vec![],
        all_rules: true,
    };
    let summary = install_global(
        &ctx,
        Path::new("/ecc"),
        Path::new("/claude"),
        "1.0.0",
        "2026-03-15T00:00:00Z",
        &options,
    );
    assert!(summary.success);
    assert_eq!(summary.added, 0);
}

#[test]
fn install_output_contains_install_header() {
    let fs = InMemoryFileSystem::new()
        .with_dir("/ecc")
        .with_dir("/claude");
    let env = no_color_env();
    let terminal = BufferedTerminal::new();
    let shell = MockExecutor::new();
    let ctx = InstallContext {
        fs: &fs,
        shell: &shell,
        env: &env,
        terminal: &terminal,
    };
    let options = default_install_options();
    install_global(
        &ctx,
        Path::new("/ecc"),
        Path::new("/claude"),
        "1.0.0",
        "2026-03-15T00:00:00Z",
        &options,
    );
    let output = terminal.stdout_output().join("");
    assert!(output.contains("ECC Install"));
}

#[test]
fn install_first_time_installs_statusline() {
    let fs = ecc_source_fs();
    let env = MockEnvironment::new()
        .with_var("NO_COLOR", "1")
        .with_home("/claude");
    let terminal = BufferedTerminal::new();
    let shell = MockExecutor::new();
    let ctx = InstallContext {
        fs: &fs,
        shell: &shell,
        env: &env,
        terminal: &terminal,
    };
    let options = InstallOptions {
        dry_run: false,
        force: true,
        no_gitignore: false,
        interactive: false,
        clean: false,
        clean_all: false,
        languages: vec![],
        all_rules: true,
    };
    let summary = install_global(
        &ctx,
        Path::new("/ecc"),
        Path::new("/claude/.claude"),
        "4.2.0",
        "2026-03-14T00:00:00Z",
        &options,
    );
    assert!(summary.success);
    use ecc_ports::fs::FileSystem;
    let script = fs
        .read_to_string(Path::new("/claude/.claude/statusline-command.sh"))
        .unwrap();
    assert!(script.contains("4.2.0"));
    assert!(!script.contains("__ECC_VERSION__"));
    let settings_str = fs
        .read_to_string(Path::new("/claude/.claude/settings.json"))
        .unwrap();
    let settings: serde_json::Value = serde_json::from_str(&settings_str).unwrap();
    assert!(settings["statusLine"]["command"].as_str().is_some());
}

#[test]
fn install_preserves_custom_statusline() {
    let fs = ecc_source_fs().with_file(
        "/claude/settings.json",
        &serde_json::json!({"statusLine": {"command": "my-custom.sh"}}).to_string(),
    );
    let env = MockEnvironment::new()
        .with_var("NO_COLOR", "1")
        .with_home("/claude");
    let terminal = BufferedTerminal::new();
    let shell = MockExecutor::new();
    let ctx = InstallContext {
        fs: &fs,
        shell: &shell,
        env: &env,
        terminal: &terminal,
    };
    let options = InstallOptions {
        dry_run: false,
        force: true,
        no_gitignore: false,
        interactive: false,
        clean: false,
        clean_all: false,
        languages: vec![],
        all_rules: true,
    };
    install_global(
        &ctx,
        Path::new("/ecc"),
        Path::new("/claude"),
        "4.2.0",
        "2026-03-14T00:00:00Z",
        &options,
    );
    let output = terminal.stdout_output().join("");
    assert!(output.contains("already custom"));
}

#[test]
fn install_missing_statusline_source_does_not_fail() {
    let fs = InMemoryFileSystem::new()
        .with_dir("/ecc/agents")
        .with_dir("/ecc/commands")
        .with_dir("/ecc/skills")
        .with_dir("/ecc/rules")
        .with_dir("/claude");
    let env = MockEnvironment::new()
        .with_var("NO_COLOR", "1")
        .with_home("/home/user");
    let terminal = BufferedTerminal::new();
    let shell = MockExecutor::new();
    let ctx = InstallContext {
        fs: &fs,
        shell: &shell,
        env: &env,
        terminal: &terminal,
    };
    let options = InstallOptions {
        dry_run: false,
        force: true,
        no_gitignore: false,
        interactive: false,
        clean: false,
        clean_all: false,
        languages: vec![],
        all_rules: true,
    };
    let summary = install_global(
        &ctx,
        Path::new("/ecc"),
        Path::new("/claude"),
        "4.2.0",
        "2026-03-14T00:00:00Z",
        &options,
    );
    assert!(summary.success);
}
