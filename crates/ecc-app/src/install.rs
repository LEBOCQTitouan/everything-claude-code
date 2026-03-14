//! Install orchestrator — full ECC installation flow.
//!
//! Ports `install-orchestrator.ts`.

use crate::config::clean as app_clean;
use crate::config::gitignore as app_gitignore;
use crate::config::manifest::{read_manifest, write_manifest};
use crate::detect;
use crate::merge::{self, MergeContext, MergeOptions, ReviewChoice};
use ecc_domain::ansi;
use ecc_domain::config::clean;
use ecc_domain::config::deny_rules;
use ecc_domain::config::gitignore;
use ecc_domain::config::manifest::{self, Artifacts};
use ecc_domain::config::merge as domain_merge;
use ecc_ports::env::Environment;
use ecc_ports::fs::FileSystem;
use ecc_ports::shell::ShellExecutor;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

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

/// Default install options — interactive, no flags.
pub fn default_install_options() -> InstallOptions {
    InstallOptions {
        dry_run: false,
        force: false,
        no_gitignore: false,
        interactive: true,
        clean: false,
        clean_all: false,
        languages: vec![],
    }
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
/// 8-step flow:
/// 1. Clean if requested (--clean / --clean-all)
/// 2. Detect existing setup
/// 3. Read existing manifest
/// 4. Merge artifacts (agents, commands, skills, rules)
/// 5. Merge hooks
/// 6. Ensure deny rules
/// 7. Write/update manifest
/// 8. Print summary
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

    // Step 1: Clean if requested
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

    // Step 2: Detect existing setup
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

    // Step 3: Read existing manifest
    let existing_manifest = read_manifest(ctx.fs, claude_dir);

    // Step 4: Merge artifacts
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

    // Agents
    let agents_report = merge::merge_directory(
        &merge_ctx,
        &ecc_root.join("agents"),
        &claude_dir.join("agents"),
        "Agents",
        ".md",
        &mut merge_options,
    );
    all_reports.push(agents_report);

    // Commands
    let commands_report = merge::merge_directory(
        &merge_ctx,
        &ecc_root.join("commands"),
        &claude_dir.join("commands"),
        "Commands",
        ".md",
        &mut merge_options,
    );
    all_reports.push(commands_report);

    // Skills
    let skills_report = merge::merge_skills(
        &merge_ctx,
        &ecc_root.join("skills"),
        &claude_dir.join("skills"),
        &mut merge_options,
    );
    all_reports.push(skills_report);

    // Rules — determine groups from source + languages
    let rule_groups = collect_rule_groups(ctx.fs, ecc_root, &options.languages);
    let rules_report = merge::merge_rules(
        &merge_ctx,
        &ecc_root.join("rules"),
        &claude_dir.join("rules"),
        &rule_groups,
        &mut merge_options,
    );
    all_reports.push(rules_report);

    let combined = domain_merge::combine_reports(&all_reports);

    // Step 5: Merge hooks
    let hooks_json = ecc_root.join("hooks.json");
    let settings_json = claude_dir.join("settings.json");
    let (hooks_added, hooks_existing, hooks_legacy) =
        if ctx.fs.exists(&hooks_json) {
            match merge::merge_hooks(ctx.fs, &hooks_json, &settings_json, options.dry_run) {
                Ok(counts) => counts,
                Err(e) => {
                    ctx.terminal.stderr_write(&format!("Hook merge error: {e}\n"));
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

    // Step 6: Ensure deny rules
    let deny_result = ensure_deny_rules_in_settings(ctx.fs, &settings_json, options.dry_run);
    if let Some((added, existing)) = deny_result
        && added > 0
    {
        ctx.terminal.stdout_write(&format!(
            "  Deny rules: {added} added, {existing} already present\n"
        ));
    }

    // Step 7: Write/update manifest
    let installed_artifacts = collect_installed_artifacts(ctx.fs, claude_dir);
    if !options.dry_run {
        let new_manifest = match existing_manifest {
            Some(ref existing) => {
                manifest::update_manifest(existing, version, now, &options.languages, installed_artifacts)
            }
            None => manifest::create_manifest(version, now, &options.languages, installed_artifacts),
        };
        let _ = write_manifest(ctx.fs, claude_dir, &new_manifest);
    }

    // Step 8: Print summary
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
        // Just show what would happen
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
            ctx.terminal.stdout_write(&format!(
                "Would add {} gitignore entries:\n",
                missing.len()
            ));
            for pattern in &missing {
                ctx.terminal.stdout_write(&format!("  {pattern}\n"));
            }
        }

        // Check for tracked files
        let tracked = app_gitignore::find_tracked_ecc_files(ctx.shell, ctx.fs, project_dir);
        if !tracked.is_empty() {
            ctx.terminal.stdout_write(&format!(
                "Would untrack {} file(s):\n",
                tracked.len()
            ));
            for file in &tracked {
                ctx.terminal.stdout_write(&format!("  {file}\n"));
            }
        }

        return true;
    }

    let result = app_gitignore::ensure_gitignore_entries(ctx.fs, ctx.shell, project_dir, None);

    if result.skipped {
        ctx.terminal
            .stdout_write("Not a git repository — skipping .gitignore.\n");
        return true;
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

    // Untrack files that are now gitignored
    let tracked = app_gitignore::find_tracked_ecc_files(ctx.shell, ctx.fs, project_dir);
    for file in &tracked {
        let _ = ctx.shell.run_command_in_dir(
            "git",
            &["rm", "--cached", file],
            project_dir,
        );
        ctx.terminal.stdout_write(&format!("Untracked: {file}\n"));
    }

    true
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Collect rule group directories from ecc_root/rules/, filtered by language.
fn collect_rule_groups(
    fs: &dyn FileSystem,
    ecc_root: &Path,
    languages: &[String],
) -> Vec<String> {
    let rules_dir = ecc_root.join("rules");
    let entries = match fs.read_dir(&rules_dir) {
        Ok(e) => e,
        Err(_) => return vec!["common".to_string()],
    };

    let mut groups: Vec<String> = entries
        .iter()
        .filter(|p| fs.is_dir(p))
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .filter(|name| {
            name == "common"
                || languages.is_empty()
                || languages.iter().any(|l| l == name)
        })
        .collect();

    groups.sort();
    groups
}

/// Scan claude_dir for currently installed artifacts.
fn collect_installed_artifacts(fs: &dyn FileSystem, claude_dir: &Path) -> Artifacts {
    let agents = list_files_with_ext(fs, &claude_dir.join("agents"), ".md");
    let commands = list_files_with_ext(fs, &claude_dir.join("commands"), ".md");
    let skills = list_dirs(fs, &claude_dir.join("skills"));
    let rules = collect_rules_map(fs, &claude_dir.join("rules"));

    Artifacts {
        agents,
        commands,
        skills,
        rules,
        hook_descriptions: vec![],
    }
}

fn list_files_with_ext(fs: &dyn FileSystem, dir: &Path, ext: &str) -> Vec<String> {
    let entries = match fs.read_dir(dir) {
        Ok(e) => e,
        Err(_) => return vec![],
    };
    let mut files: Vec<String> = entries
        .iter()
        .filter_map(|p| {
            let name = p.file_name()?.to_string_lossy().into_owned();
            if name.ends_with(ext) {
                Some(name)
            } else {
                None
            }
        })
        .collect();
    files.sort();
    files
}

fn list_dirs(fs: &dyn FileSystem, dir: &Path) -> Vec<String> {
    let entries = match fs.read_dir(dir) {
        Ok(e) => e,
        Err(_) => return vec![],
    };
    let mut dirs: Vec<String> = entries
        .iter()
        .filter(|p| fs.is_dir(p))
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    dirs.sort();
    dirs
}

fn collect_rules_map(
    fs: &dyn FileSystem,
    rules_dir: &Path,
) -> std::collections::BTreeMap<String, Vec<String>> {
    let mut map = std::collections::BTreeMap::new();
    let groups = list_dirs(fs, rules_dir);
    for group in groups {
        let files = list_files_with_ext(fs, &rules_dir.join(&group), ".md");
        if !files.is_empty() {
            map.insert(group, files);
        }
    }
    map
}

/// Ensure deny rules are present in settings.json.
/// Returns `(added, existing)` if settings were updated, `None` on error.
fn ensure_deny_rules_in_settings(
    fs: &dyn FileSystem,
    settings_path: &Path,
    dry_run: bool,
) -> Option<(usize, usize)> {
    let content = fs.read_to_string(settings_path).unwrap_or_else(|_| "{}".to_string());
    let mut settings: serde_json::Value = serde_json::from_str(&content).ok()?;

    let existing_deny: Vec<String> = settings
        .get("permissions")
        .and_then(|p| p.get("deny"))
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let (merged, result) = deny_rules::ensure_deny_rules(&existing_deny);

    if result.added > 0 && !dry_run {
        let permissions = settings
            .as_object_mut()?
            .entry("permissions")
            .or_insert_with(|| serde_json::json!({}));
        permissions
            .as_object_mut()?
            .insert(
                "deny".to_string(),
                serde_json::Value::Array(
                    merged.into_iter().map(serde_json::Value::String).collect(),
                ),
            );

        let json = serde_json::to_string_pretty(&settings).ok()?;
        fs.write(settings_path, &format!("{json}\n")).ok()?;
    }

    Some((result.added, result.existing))
}

fn print_summary(
    terminal: &dyn TerminalIO,
    summary: &InstallSummary,
    colored: bool,
    dry_run: bool,
) {
    let prefix = if dry_run { "[DRY RUN] " } else { "" };

    terminal.stdout_write(&format!(
        "\n{prefix}{}\n",
        ansi::bold("Install Summary", colored)
    ));

    if summary.added > 0 {
        terminal.stdout_write(&format!(
            "  {} {}\n",
            ansi::green(&format!("{}", summary.added), colored),
            "added"
        ));
    }
    if summary.updated > 0 {
        terminal.stdout_write(&format!(
            "  {} {}\n",
            ansi::yellow(&format!("{}", summary.updated), colored),
            "updated"
        ));
    }
    if summary.unchanged > 0 {
        terminal.stdout_write(&format!(
            "  {} unchanged\n",
            summary.unchanged
        ));
    }
    if summary.skipped > 0 {
        terminal.stdout_write(&format!(
            "  {} skipped\n",
            summary.skipped
        ));
    }
    if summary.smart_merged > 0 {
        terminal.stdout_write(&format!(
            "  {} smart-merged\n",
            summary.smart_merged
        ));
    }
    if !summary.errors.is_empty() {
        terminal.stdout_write(&format!(
            "  {} {}\n",
            ansi::red(&format!("{}", summary.errors.len()), colored),
            "errors"
        ));
        for err in &summary.errors {
            terminal.stdout_write(&format!("    - {err}\n"));
        }
    }

    if summary.success {
        terminal.stdout_write(&format!(
            "\n{}\n",
            ansi::green("Install complete!", colored)
        ));
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_ports::shell::CommandOutput;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment, MockExecutor};
    use std::path::Path;

    fn no_color_env() -> MockEnvironment {
        MockEnvironment::new().with_var("NO_COLOR", "1")
    }

    fn ecc_source_fs() -> InMemoryFileSystem {
        InMemoryFileSystem::new()
            .with_dir("/ecc/agents")
            .with_file("/ecc/agents/planner.md", "---\nname: planner\n---\n# Planner")
            .with_file("/ecc/agents/reviewer.md", "---\nname: reviewer\n---\n# Reviewer")
            .with_dir("/ecc/commands")
            .with_file("/ecc/commands/plan.md", "# Plan command")
            .with_dir("/ecc/skills")
            .with_dir("/ecc/skills/tdd")
            .with_file("/ecc/skills/tdd/SKILL.md", "# TDD Skill")
            .with_dir("/ecc/rules")
            .with_dir("/ecc/rules/common")
            .with_file("/ecc/rules/common/style.md", "# Style rules")
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

        // Manifest should be written
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
        // Files should NOT be written in dry run
        assert!(!fs.exists(Path::new("/claude/agents/planner.md")));
        // Manifest should NOT exist
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
        let terminal = BufferedTerminal::new(); // No input needed
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

        // Settings should have deny rules
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

        // Should have typescript rules
        assert!(fs.exists(Path::new("/claude/rules/typescript/types.md")));
        // Should have common rules
        assert!(fs.exists(Path::new("/claude/rules/common/style.md")));
        // Should NOT have python rules
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

        let result = init_project(&ctx, Path::new("/project"), false, false);
        assert!(result);

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

        let result = init_project(&ctx, Path::new("/project"), true, false);
        assert!(result);

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

        let result = init_project(&ctx, Path::new("/project"), false, true);
        assert!(result);

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

        let result = init_project(&ctx, Path::new("/project"), false, false);
        assert!(result);

        let output = terminal.stdout_output().join("");
        assert!(output.contains("Not a git repository"));
    }

    // --- helpers ---

    #[test]
    fn collect_rule_groups_with_languages() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/ecc/rules/common")
            .with_dir("/ecc/rules/typescript")
            .with_dir("/ecc/rules/python");

        let groups = collect_rule_groups(&fs, Path::new("/ecc"), &["typescript".to_string()]);
        assert!(groups.contains(&"common".to_string()));
        assert!(groups.contains(&"typescript".to_string()));
        assert!(!groups.contains(&"python".to_string()));
    }

    #[test]
    fn collect_rule_groups_empty_languages() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/ecc/rules/common")
            .with_dir("/ecc/rules/typescript");

        let groups = collect_rule_groups(&fs, Path::new("/ecc"), &[]);
        // All groups included when no language filter
        assert!(groups.contains(&"common".to_string()));
        assert!(groups.contains(&"typescript".to_string()));
    }
}
