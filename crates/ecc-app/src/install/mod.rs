//! Install orchestrator — full ECC installation flow.
//!
//! Ports `install-orchestrator.ts`.

mod helpers;
mod resolve;

pub use resolve::resolve_ecc_root;

use crate::config::clean as app_clean;
use crate::config::gitignore as app_gitignore;
use crate::config::manifest::{read_manifest, write_manifest};
use crate::detect;
use crate::merge::{self, MergeContext, MergeOptions, ReviewChoice};
use ecc_domain::ansi;
use ecc_domain::config::clean;
use ecc_domain::config::gitignore;
use ecc_domain::config::manifest;
use ecc_domain::config::merge as domain_merge;
use ecc_ports::env::Environment;
use ecc_ports::fs::FileSystem;
use ecc_ports::shell::ShellExecutor;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

use helpers::{
    collect_installed_artifacts, collect_rule_groups, ensure_deny_rules_in_settings,
    ensure_statusline_in_settings, print_summary,
};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Options for the install command.
#[derive(Debug, Clone)]
pub struct InstallOptions {
    pub dry_run: bool,
    pub force: bool,
    pub no_gitignore: bool,
    pub interactive: bool,
    pub clean: bool,
    pub clean_all: bool,
    pub languages: Vec<String>,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            dry_run: false,
            force: false,
            no_gitignore: false,
            interactive: true,
            clean: false,
            clean_all: false,
            languages: vec![],
        }
    }
}

/// Default install options — interactive, no flags.
pub fn default_install_options() -> InstallOptions {
    InstallOptions::default()
}

/// Context bundling all ports for install operations.
pub struct InstallContext<'a> {
    pub fs: &'a dyn FileSystem,
    pub shell: &'a dyn ShellExecutor,
    pub env: &'a dyn Environment,
    pub terminal: &'a dyn TerminalIO,
}

/// Summary of an install operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallSummary {
    pub added: usize,
    pub updated: usize,
    pub unchanged: usize,
    pub skipped: usize,
    pub smart_merged: usize,
    pub errors: Vec<String>,
    pub success: bool,
}

// ---------------------------------------------------------------------------
// Install orchestration
// ---------------------------------------------------------------------------

/// Run a full global install of ECC configuration to `claude_dir`.
///
/// 9-step flow: clean → detect → manifest → merge → hooks → deny → statusline → manifest → summary
#[allow(clippy::too_many_arguments)]
pub fn install_global(
    ctx: &InstallContext,
    ecc_root: &Path,
    claude_dir: &Path,
    version: &str,
    now: &str,
    options: &InstallOptions,
) -> InstallSummary {
    let colored = ctx.env.var("NO_COLOR").is_none();
    let prefix = if options.dry_run { "[DRY RUN] " } else { "" };

    ctx.terminal.stdout_write(&format!(
        "\n{}{}\n\n",
        prefix,
        ansi::bold("ECC Install", colored)
    ));

    step_clean(ctx, claude_dir, options);
    step_detect(ctx, claude_dir);
    let existing_manifest = read_manifest(ctx.fs, claude_dir);
    let mut combined = step_merge_artifacts(ctx, ecc_root, claude_dir, options);
    step_hooks_and_settings(ctx, ecc_root, claude_dir, version, options);
    step_write_manifest(
        ctx,
        claude_dir,
        version,
        now,
        options,
        &existing_manifest,
        &mut combined,
    );

    let summary = InstallSummary {
        added: combined.added.len(),
        updated: combined.updated.len(),
        unchanged: combined.unchanged.len(),
        skipped: combined.skipped.len(),
        smart_merged: combined.smart_merged.len(),
        errors: combined.errors.clone(),
        success: combined.errors.is_empty(),
    };

    print_summary(ctx.terminal, &summary, colored, options.dry_run);
    summary
}

fn step_clean(ctx: &InstallContext, claude_dir: &Path, options: &InstallOptions) {
    if options.clean_all {
        ctx.terminal.stdout_write("Cleaning all ECC files...\n");
        let report = app_clean::clean_all(
            ctx.fs,
            claude_dir,
            &|entry| domain_merge::is_legacy_ecc_hook(entry),
            options.dry_run,
        );
        ctx.terminal.stdout_write(&format!(
            "{}\n",
            clean::format_clean_report(&report, options.dry_run)
        ));
    } else if options.clean {
        if let Some(existing_manifest) = read_manifest(ctx.fs, claude_dir) {
            ctx.terminal
                .stdout_write("Cleaning ECC-managed files from manifest...\n");
            let report = app_clean::clean_from_manifest(
                ctx.fs,
                claude_dir,
                &existing_manifest,
                &|entry| domain_merge::is_legacy_ecc_hook(entry),
                options.dry_run,
            );
            ctx.terminal.stdout_write(&format!(
                "{}\n",
                clean::format_clean_report(&report, options.dry_run)
            ));
        } else {
            ctx.terminal.stdout_write(
                "No manifest found — nothing to clean. Use --clean-all for nuclear cleanup.\n",
            );
        }
    }
}

fn step_detect(ctx: &InstallContext, claude_dir: &Path) {
    let detection = detect::detect_and_report(ctx.fs, ctx.terminal, claude_dir, None);
    let is_update = !detect::is_empty_setup(&detection);

    ctx.terminal.stdout_write(&format!(
        "\n{}\n",
        if is_update {
            "Updating existing configuration..."
        } else {
            "First-time installation..."
        }
    ));
}

fn step_merge_artifacts(
    ctx: &InstallContext,
    ecc_root: &Path,
    claude_dir: &Path,
    options: &InstallOptions,
) -> domain_merge::MergeReport {
    let mut merge_options = MergeOptions {
        dry_run: options.dry_run,
        force: options.force,
        interactive: options.interactive,
        apply_all: if options.force {
            Some(ReviewChoice::Accept)
        } else {
            None
        },
    };

    let merge_ctx = MergeContext {
        fs: ctx.fs,
        terminal: ctx.terminal,
        env: ctx.env,
        shell: ctx.shell,
    };

    let mut all_reports = Vec::new();

    all_reports.push(merge::merge_directory(
        &merge_ctx,
        &ecc_root.join("agents"),
        &claude_dir.join("agents"),
        "Agents",
        ".md",
        &mut merge_options,
    ));

    all_reports.push(merge::merge_directory(
        &merge_ctx,
        &ecc_root.join("commands"),
        &claude_dir.join("commands"),
        "Commands",
        ".md",
        &mut merge_options,
    ));

    all_reports.push(merge::merge_skills(
        &merge_ctx,
        &ecc_root.join("skills"),
        &claude_dir.join("skills"),
        &mut merge_options,
    ));

    let rule_groups = collect_rule_groups(ctx.fs, ecc_root, &options.languages);
    all_reports.push(merge::merge_rules(
        &merge_ctx,
        &ecc_root.join("rules"),
        &claude_dir.join("rules"),
        &rule_groups,
        &mut merge_options,
    ));

    domain_merge::combine_reports(&all_reports)
}

fn step_hooks_and_settings(
    ctx: &InstallContext,
    ecc_root: &Path,
    claude_dir: &Path,
    version: &str,
    options: &InstallOptions,
) {
    let hooks_json = ecc_root.join("hooks").join("hooks.json");
    let settings_json = claude_dir.join("settings.json");
    let (hooks_added, hooks_existing, hooks_legacy) = if ctx.fs.exists(&hooks_json) {
        match merge::merge_hooks(ctx.fs, &hooks_json, &settings_json, options.dry_run) {
            Ok(counts) => counts,
            Err(e) => {
                ctx.terminal
                    .stderr_write(&format!("Hook merge error: {e}\n"));
                (0, 0, 0)
            }
        }
    } else {
        (0, 0, 0)
    };

    if hooks_added > 0 || hooks_legacy > 0 {
        ctx.terminal.stdout_write(&format!(
            "  Hooks: {hooks_added} added, {hooks_existing} existing, {hooks_legacy} legacy removed\n"
        ));
    }

    let deny_result = ensure_deny_rules_in_settings(ctx.fs, &settings_json, options.dry_run);
    if let Some((added, existing)) = deny_result
        && added > 0
    {
        ctx.terminal.stdout_write(&format!(
            "  Deny rules: {added} added, {existing} already present\n"
        ));
    }

    let statusline_result = ensure_statusline_in_settings(
        ctx.fs,
        ctx.env,
        &settings_json,
        ecc_root,
        version,
        options.dry_run,
    );
    match &statusline_result {
        Some(ecc_domain::config::statusline::StatusLineResult::Installed) => {
            ctx.terminal.stdout_write("  Statusline: installed\n");
        }
        Some(ecc_domain::config::statusline::StatusLineResult::Updated) => {
            ctx.terminal.stdout_write("  Statusline: updated\n");
        }
        Some(ecc_domain::config::statusline::StatusLineResult::AlreadyCustom) => {
            ctx.terminal
                .stdout_write("  Statusline: already custom (skipped)\n");
        }
        None => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn step_write_manifest(
    ctx: &InstallContext,
    claude_dir: &Path,
    version: &str,
    now: &str,
    options: &InstallOptions,
    existing_manifest: &Option<ecc_domain::config::manifest::EccManifest>,
    combined: &mut domain_merge::MergeReport,
) {
    let installed_artifacts = collect_installed_artifacts(ctx.fs, claude_dir);
    if !options.dry_run {
        let new_manifest = match existing_manifest {
            Some(existing) => manifest::update_manifest(
                existing,
                version,
                now,
                &options.languages,
                installed_artifacts,
            ),
            None => {
                manifest::create_manifest(version, now, &options.languages, installed_artifacts)
            }
        };
        if let Err(e) = write_manifest(ctx.fs, claude_dir, &new_manifest) {
            log::warn!("Failed to write manifest: {}", e);
            combined
                .errors
                .push(format!("Failed to write manifest: {e}"));
        }
    }
}

/// Initialize ECC in a project directory (gitignore + untrack).
pub fn init_project(
    ctx: &InstallContext,
    project_dir: &Path,
    no_gitignore: bool,
    dry_run: bool,
) -> bool {
    let colored = ctx.env.var("NO_COLOR").is_none();
    let prefix = if dry_run { "[DRY RUN] " } else { "" };

    ctx.terminal.stdout_write(&format!(
        "\n{}{}\n\n",
        prefix,
        ansi::bold("ECC Init", colored)
    ));

    if no_gitignore {
        ctx.terminal
            .stdout_write("Skipping .gitignore (--no-gitignore)\n");
        return true;
    }

    if dry_run {
        init_project_dry_run(ctx, project_dir);
    } else {
        init_project_apply(ctx, project_dir);
    }

    true
}

fn init_project_dry_run(ctx: &InstallContext, project_dir: &Path) {
    let existing_content = ctx
        .fs
        .read_to_string(&project_dir.join(".gitignore"))
        .unwrap_or_default();
    let existing_patterns = gitignore::parse_gitignore_patterns(&existing_content);

    let missing: Vec<&str> = gitignore::ECC_GITIGNORE_ENTRIES
        .iter()
        .filter(|e| !existing_patterns.contains(e.pattern))
        .map(|e| e.pattern)
        .collect();

    if missing.is_empty() {
        ctx.terminal
            .stdout_write("All gitignore entries already present.\n");
    } else {
        ctx.terminal
            .stdout_write(&format!("Would add {} gitignore entries:\n", missing.len()));
        for pattern in &missing {
            ctx.terminal.stdout_write(&format!("  {pattern}\n"));
        }
    }

    let tracked = app_gitignore::find_tracked_ecc_files(ctx.shell, ctx.fs, project_dir);
    if !tracked.is_empty() {
        ctx.terminal
            .stdout_write(&format!("Would untrack {} file(s):\n", tracked.len()));
        for file in &tracked {
            ctx.terminal.stdout_write(&format!("  {file}\n"));
        }
    }
}

fn init_project_apply(ctx: &InstallContext, project_dir: &Path) {
    let result = app_gitignore::ensure_gitignore_entries(ctx.fs, ctx.shell, project_dir, None);

    if result.skipped {
        ctx.terminal
            .stdout_write("Not a git repository — skipping .gitignore.\n");
        return;
    }

    if result.added.is_empty() {
        ctx.terminal
            .stdout_write("All gitignore entries already present.\n");
    } else {
        ctx.terminal.stdout_write(&format!(
            "Added {} gitignore entries:\n",
            result.added.len()
        ));
        for pattern in &result.added {
            ctx.terminal.stdout_write(&format!("  {pattern}\n"));
        }
    }

    let tracked = app_gitignore::find_tracked_ecc_files(ctx.shell, ctx.fs, project_dir);
    for file in &tracked {
        if let Err(err) =
            ctx.shell
                .run_command_in_dir("git", &["rm", "--cached", file], project_dir)
        {
            log::warn!("git rm --cached failed: {err}");
        }
        ctx.terminal.stdout_write(&format!("Untracked: {file}\n"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::manifest::read_manifest;
    use ecc_domain::config::deny_rules;
    use ecc_ports::fs::FileSystem;
    use ecc_ports::shell::CommandOutput;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment, MockExecutor};

    fn no_color_env() -> MockEnvironment {
        MockEnvironment::new().with_var("NO_COLOR", "1")
    }

    fn ecc_source_fs() -> InMemoryFileSystem {
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

    // --- install_global ---

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

    // --- init_project ---

    #[test]
    fn init_project_adds_gitignore() {
        let fs = InMemoryFileSystem::new().with_dir("/project");
        let env = no_color_env();
        let terminal = BufferedTerminal::new();
        let shell = MockExecutor::new().on(
            "git",
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let ctx = InstallContext {
            fs: &fs,
            shell: &shell,
            env: &env,
            terminal: &terminal,
        };
        assert!(init_project(&ctx, Path::new("/project"), false, false));
        let output = terminal.stdout_output().join("");
        assert!(output.contains("Added"));
    }

    #[test]
    fn init_project_no_gitignore_flag() {
        let fs = InMemoryFileSystem::new().with_dir("/project");
        let env = no_color_env();
        let terminal = BufferedTerminal::new();
        let shell = MockExecutor::new();
        let ctx = InstallContext {
            fs: &fs,
            shell: &shell,
            env: &env,
            terminal: &terminal,
        };
        assert!(init_project(&ctx, Path::new("/project"), true, false));
        let output = terminal.stdout_output().join("");
        assert!(output.contains("Skipping .gitignore"));
    }

    #[test]
    fn init_project_dry_run() {
        let fs = InMemoryFileSystem::new().with_dir("/project");
        let env = no_color_env();
        let terminal = BufferedTerminal::new();
        let shell = MockExecutor::new().on(
            "git",
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let ctx = InstallContext {
            fs: &fs,
            shell: &shell,
            env: &env,
            terminal: &terminal,
        };
        assert!(init_project(&ctx, Path::new("/project"), false, true));
        let output = terminal.stdout_output().join("");
        assert!(output.contains("DRY RUN"));
        assert!(output.contains("Would add"));
    }

    #[test]
    fn init_project_not_git_repo() {
        let fs = InMemoryFileSystem::new().with_dir("/project");
        let env = no_color_env();
        let terminal = BufferedTerminal::new();
        let shell = MockExecutor::new().on(
            "git",
            CommandOutput {
                stdout: String::new(),
                stderr: "not a git repo".into(),
                exit_code: 128,
            },
        );
        let ctx = InstallContext {
            fs: &fs,
            shell: &shell,
            env: &env,
            terminal: &terminal,
        };
        assert!(init_project(&ctx, Path::new("/project"), false, false));
        let output = terminal.stdout_output().join("");
        assert!(output.contains("Not a git repository"));
    }

    // --- error paths ---

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

    // --- install + statusline integration ---

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
}
