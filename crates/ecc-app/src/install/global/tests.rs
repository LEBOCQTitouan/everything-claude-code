use super::install_global;
use crate::config::manifest::read_manifest;
use crate::install::{InstallContext, InstallOptions};
use ecc_ports::fs::{FileSystem, FsError};
use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment, MockExecutor};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

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
        all_rules: true,
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
        all_rules: true,
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
        all_rules: true,
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
        all_rules: true,
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
        all_rules: true,
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
        all_rules: true,
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
        all_rules: true,
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
        all_rules: true,
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
        all_rules: true,
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

// ---------------------------------------------------------------------------
// FailingFs — inject write failures for error-path tests (PC-012..PC-014)
// ---------------------------------------------------------------------------

pub(super) struct FailingFs {
    inner: InMemoryFileSystem,
    fail_write_paths: HashSet<PathBuf>,
}

impl FailingFs {
    pub(super) fn new(inner: InMemoryFileSystem) -> Self {
        Self {
            inner,
            fail_write_paths: HashSet::new(),
        }
    }

    pub(super) fn with_fail_write(mut self, path: impl Into<PathBuf>) -> Self {
        self.fail_write_paths.insert(path.into());
        self
    }
}

impl FileSystem for FailingFs {
    fn read_to_string(&self, path: &Path) -> Result<String, FsError> {
        self.inner.read_to_string(path)
    }

    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, FsError> {
        self.inner.read_bytes(path)
    }

    fn write(&self, path: &Path, content: &str) -> Result<(), FsError> {
        if self.fail_write_paths.contains(path) {
            return Err(FsError::PermissionDenied(path.to_path_buf()));
        }
        self.inner.write(path, content)
    }

    fn write_bytes(&self, path: &Path, content: &[u8]) -> Result<(), FsError> {
        if self.fail_write_paths.contains(path) {
            return Err(FsError::PermissionDenied(path.to_path_buf()));
        }
        self.inner.write_bytes(path, content)
    }

    fn exists(&self, path: &Path) -> bool {
        self.inner.exists(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.inner.is_dir(path)
    }

    fn is_file(&self, path: &Path) -> bool {
        self.inner.is_file(path)
    }

    fn create_dir_all(&self, path: &Path) -> Result<(), FsError> {
        self.inner.create_dir_all(path)
    }

    fn remove_file(&self, path: &Path) -> Result<(), FsError> {
        self.inner.remove_file(path)
    }

    fn remove_dir_all(&self, path: &Path) -> Result<(), FsError> {
        self.inner.remove_dir_all(path)
    }

    fn copy(&self, from: &Path, to: &Path) -> Result<(), FsError> {
        self.inner.copy(from, to)
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FsError> {
        self.inner.read_dir(path)
    }

    fn read_dir_recursive(&self, path: &Path) -> Result<Vec<PathBuf>, FsError> {
        self.inner.read_dir_recursive(path)
    }

    fn create_symlink(&self, target: &Path, link: &Path) -> Result<(), FsError> {
        self.inner.create_symlink(target, link)
    }

    fn read_symlink(&self, link: &Path) -> Result<PathBuf, FsError> {
        self.inner.read_symlink(link)
    }

    fn is_symlink(&self, path: &Path) -> bool {
        self.inner.is_symlink(path)
    }

    fn set_permissions(&self, path: &Path, mode: u32) -> Result<(), FsError> {
        self.inner.set_permissions(path, mode)
    }

    fn is_executable(&self, path: &Path) -> bool {
        self.inner.is_executable(path)
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<(), FsError> {
        self.inner.rename(from, to)
    }
}

// ---------------------------------------------------------------------------
// Error path tests (PC-012, PC-013, PC-014)
// ---------------------------------------------------------------------------

/// PC-012: step_hooks_and_settings error propagates to InstallSummary.
/// When hooks.json has invalid content (malformed JSON), the hook merge fails.
/// The error should be collected and InstallSummary.success should be false.
#[test]
fn step_hooks_and_settings_error_propagates_to_summary() {
    // hooks.json exists but contains invalid JSON — merge_hooks will fail
    let fs = ecc_source_fs().with_file("/ecc/hooks/hooks.json", "{ invalid json !!!");
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
        "4.0.0",
        "2026-03-14T00:00:00Z",
        &options,
    );
    assert!(!summary.success, "summary should indicate failure");
    assert!(
        !summary.errors.is_empty(),
        "errors should be populated when hook merge fails"
    );
}

/// PC-013: Install accumulates errors when multiple steps fail (not fail-fast).
/// Both manifest write failure and hook merge failure should both appear in errors.
#[test]
fn install_accumulates_errors() {
    // hooks.json has invalid content (hook merge fails) +
    // manifest write path will fail via FailingFs
    let inner = ecc_source_fs().with_file("/ecc/hooks/hooks.json", "{ bad json }");
    let fs = FailingFs::new(inner).with_fail_write("/claude/.ecc-manifest.json");
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
        "4.0.0",
        "2026-03-14T00:00:00Z",
        &options,
    );
    assert!(!summary.success, "summary should indicate failure");
    // Both errors should be accumulated (not fail-fast)
    assert!(
        summary.errors.len() >= 2,
        "both hook and manifest errors should be accumulated, got: {:?}",
        summary.errors
    );
}
