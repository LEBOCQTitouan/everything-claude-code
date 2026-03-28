use super::install_global;
use crate::config::manifest::read_manifest;
use crate::install::{InstallContext, InstallOptions};
use ecc_ports::fs::FileSystem;
use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment, MockExecutor};
use std::path::Path;

pub(super) fn no_color_env() -> MockEnvironment {
    MockEnvironment::new().with_var("NO_COLOR", "1")
}

pub(super) fn ecc_source_fs() -> InMemoryFileSystem {
    InMemoryFileSystem::new()
        .with_dir("/ecc/agents")
        .with_file(
            "/ecc/agents/planner.md",
            "---\nname: planner\n---\n# Planner",
        )
        .with_file(
            "/ecc/agents/reviewer.md",
            "---\nname: reviewer\n---\n# Reviewer",
        )
        .with_dir("/ecc/commands")
        .with_file("/ecc/commands/plan.md", "# Plan command")
        .with_dir("/ecc/skills")
        .with_dir("/ecc/skills/tdd")
        .with_file("/ecc/skills/tdd/SKILL.md", "# TDD Skill")
        .with_dir("/ecc/rules")
        .with_dir("/ecc/rules/common")
        .with_file("/ecc/rules/common/style.md", "# Style rules")
        .with_dir("/ecc/statusline")
        .with_file(
            "/ecc/statusline/statusline-command.sh",
            "#!/bin/bash\nECC_VERSION=\"__ECC_VERSION__\"\necho $ECC_VERSION",
        )
        .with_dir("/claude")
}

#[test]
fn install_first_time() {
    let fs = ecc_source_fs();
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
    };
    let summary = install_global(
        &ctx,
        Path::new("/ecc"),
        Path::new("/claude"),
        "4.0.0",
        "2026-03-14T00:00:00Z",
        &options,
    );
    assert!(summary.success);
    assert!(summary.added > 0);
    assert!(fs.exists(Path::new("/claude/agents/planner.md")));
    assert!(fs.exists(Path::new("/claude/commands/plan.md")));
    assert!(fs.exists(Path::new("/claude/rules/common/style.md")));
    let manifest = read_manifest(&fs, Path::new("/claude"));
    assert!(manifest.is_some());
    assert_eq!(manifest.unwrap().version, "4.0.0");
}

#[test]
fn install_dry_run() {
    let fs = ecc_source_fs();
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
        dry_run: true,
        force: true,
        no_gitignore: false,
        interactive: false,
        clean: false,
        clean_all: false,
        languages: vec![],
    };
    let summary = install_global(
        &ctx,
        Path::new("/ecc"),
        Path::new("/claude"),
        "4.0.0",
        "2026-03-14T00:00:00Z",
        &options,
    );
    assert!(summary.added > 0);
    assert!(!fs.exists(Path::new("/claude/agents/planner.md")));
    assert!(read_manifest(&fs, Path::new("/claude")).is_none());
}

#[test]
fn install_update_existing() {
    let fs = ecc_source_fs()
        .with_dir("/claude/agents")
        .with_file("/claude/agents/planner.md", "# Old planner");
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
    };
    let summary = install_global(
        &ctx,
        Path::new("/ecc"),
        Path::new("/claude"),
        "4.0.0",
        "2026-03-14T00:00:00Z",
        &options,
    );
    assert!(summary.success);
    assert!(summary.updated > 0);
    let output = terminal.stdout_output().join("");
    assert!(output.contains("Updating existing configuration"));
}

#[test]
fn install_non_interactive() {
    let fs = ecc_source_fs();
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
        force: false,
        no_gitignore: false,
        interactive: false,
        clean: false,
        clean_all: false,
        languages: vec![],
    };
    let summary = install_global(
        &ctx,
        Path::new("/ecc"),
        Path::new("/claude"),
        "4.0.0",
        "2026-03-14T00:00:00Z",
        &options,
    );
    assert!(summary.success);
    assert!(summary.added > 0);
}

#[test]
fn install_with_clean_all() {
    let fs = ecc_source_fs()
        .with_dir("/claude/agents")
        .with_file("/claude/agents/old.md", "# Old agent")
        .with_dir("/claude/commands")
        .with_file("/claude/commands/old.md", "# Old command");
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
        clean_all: true,
        languages: vec![],
    };
    let summary = install_global(
        &ctx,
        Path::new("/ecc"),
        Path::new("/claude"),
        "4.0.0",
        "2026-03-14T00:00:00Z",
        &options,
    );
    assert!(summary.success);
    let output = terminal.stdout_output().join("");
    assert!(output.contains("Cleaning all ECC files"));
}

#[test]
fn install_with_clean_needs_manifest() {
    let fs = ecc_source_fs();
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
        clean: true,
        clean_all: false,
        languages: vec![],
    };
    install_global(
        &ctx,
        Path::new("/ecc"),
        Path::new("/claude"),
        "4.0.0",
        "2026-03-14T00:00:00Z",
        &options,
    );
    let output = terminal.stdout_output().join("");
    assert!(output.contains("No manifest found"));
}

#[test]
fn install_deny_rules_added() {
    use ecc_domain::config::deny_rules;

    let fs = ecc_source_fs();
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
    };
    install_global(
        &ctx,
        Path::new("/ecc"),
        Path::new("/claude"),
        "4.0.0",
        "2026-03-14T00:00:00Z",
        &options,
    );
    let settings_content = fs
        .read_to_string(Path::new("/claude/settings.json"))
        .unwrap();
    let settings: serde_json::Value = serde_json::from_str(&settings_content).unwrap();
    let deny = settings["permissions"]["deny"].as_array().unwrap();
    assert!(deny.len() >= deny_rules::ECC_DENY_RULES.len());
}

#[test]
fn install_shows_summary() {
    let fs = ecc_source_fs();
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
    };
    install_global(
        &ctx,
        Path::new("/ecc"),
        Path::new("/claude"),
        "4.0.0",
        "2026-03-14T00:00:00Z",
        &options,
    );
    let output = terminal.stdout_output().join("");
    assert!(output.contains("Install Summary"));
    assert!(output.contains("Install complete!"));
}

#[test]
fn install_with_languages() {
    let fs = ecc_source_fs()
        .with_dir("/ecc/rules/typescript")
        .with_file("/ecc/rules/typescript/types.md", "# Types")
        .with_dir("/ecc/rules/python")
        .with_file("/ecc/rules/python/style.md", "# Python style");
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
        languages: vec!["typescript".to_string()],
    };
    install_global(
        &ctx,
        Path::new("/ecc"),
        Path::new("/claude"),
        "4.0.0",
        "2026-03-14T00:00:00Z",
        &options,
    );
    assert!(fs.exists(Path::new("/claude/rules/typescript/types.md")));
    assert!(fs.exists(Path::new("/claude/rules/common/style.md")));
    assert!(!fs.exists(Path::new("/claude/rules/python/style.md")));
}
