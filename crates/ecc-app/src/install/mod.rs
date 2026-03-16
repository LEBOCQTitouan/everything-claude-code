//! Install orchestrator — full ECC installation flow.
//!
//! Ports `install-orchestrator.ts`.

mod helpers;

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
/// 9-step flow:
/// 1. Clean if requested (--clean / --clean-all)
/// 2. Detect existing setup
/// 3. Read existing manifest
/// 4. Merge artifacts (agents, commands, skills, rules)
/// 5. Merge hooks
/// 6. Ensure deny rules
/// 6b. Ensure statusline
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

    let mut combined = domain_merge::combine_reports(&all_reports);

    // Step 5: Merge hooks
    let hooks_json = ecc_root.join("hooks").join("hooks.json");
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

    // Step 6b: Ensure statusline
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
            ctx.terminal.stdout_write("  Statusline: already custom (skipped)\n");
        }
        None => {
            // Source script missing or other error — not fatal
        }
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
        if let Err(e) = write_manifest(ctx.fs, claude_dir, &new_manifest) {
            log::warn!("Failed to write manifest: {}", e);
            combined.errors.push(format!("Failed to write manifest: {e}"));
        }
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

/// Resolve the ECC root directory containing agents/, commands/, etc.
///
/// Resolution order:
/// 1. `ECC_ROOT` env var (explicit override)
/// 2. Repo root detection (dev scenario: walk up from cwd looking for Cargo.toml + agents/)
/// 3. Relative to binary: `parent_of_binary/..` (handles `~/.ecc/bin/ecc` → `~/.ecc/`)
/// 4. `$HOME/.ecc/`
/// 5. Legacy npm global install paths (backward compat)
/// 6. Error with instructions
pub fn resolve_ecc_root(
    fs: &dyn FileSystem,
    env: &dyn Environment,
) -> Result<std::path::PathBuf, String> {
    // 1. ECC_ROOT env var (explicit override)
    if let Some(ecc_root) = env.var("ECC_ROOT") {
        let root = std::path::PathBuf::from(&ecc_root);
        if fs.exists(&root.join("agents")) {
            return Ok(root);
        }
    }

    // 2. Repo root detection (dev scenario: walk up from cwd looking for Cargo.toml + agents/)
    if let Some(cwd) = env.current_dir() {
        let mut dir = cwd.as_path();
        loop {
            if fs.exists(&dir.join("Cargo.toml")) && fs.exists(&dir.join("agents")) {
                return Ok(dir.to_path_buf());
            }
            match dir.parent() {
                Some(parent) => dir = parent,
                None => break,
            }
        }
    }

    // 3. Relative to binary: parent/.. (e.g. ~/.ecc/bin/ecc → ~/.ecc/)
    if let Ok(exe) = std::env::current_exe()
        && let Some(bin_dir) = exe.parent()
    {
        let relative = bin_dir.join("..");
        if fs.exists(&relative.join("agents")) {
            return Ok(relative);
        }
    }

    // 4. $HOME/.ecc/
    if let Some(home) = env.home_dir() {
        let home_ecc = home.join(".ecc");
        if fs.exists(&home_ecc.join("agents")) {
            return Ok(home_ecc);
        }
    }

    // 5. Legacy npm paths (backward compat)
    let npm_paths = [
        "/usr/local/lib/node_modules/@lebocqtitouan/ecc",
        "/usr/lib/node_modules/@lebocqtitouan/ecc",
    ];

    for path in &npm_paths {
        let p = std::path::PathBuf::from(path);
        if fs.exists(&p.join("agents")) {
            return Ok(p);
        }
    }

    Err(
        "Cannot find ECC assets directory. Install with: \
         curl -fsSL https://raw.githubusercontent.com/LEBOCQTitouan/everything-claude-code/main/scripts/get-ecc.sh | bash\n\
         Or set ECC_ROOT environment variable / use --ecc-root flag."
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::{InMemoryFileSystem, MockEnvironment};

    #[test]
    fn resolve_ecc_root_finds_home_ecc() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/home/user/.ecc/agents");
        let env = MockEnvironment::new().with_home("/home/user");

        let result = resolve_ecc_root(&fs, &env);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            std::path::PathBuf::from("/home/user/.ecc")
        );
    }

    #[test]
    fn resolve_ecc_root_finds_legacy_npm_path() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/usr/local/lib/node_modules/@lebocqtitouan/ecc/agents");
        let env = MockEnvironment::new().with_home("/home/user");

        let result = resolve_ecc_root(&fs, &env);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            std::path::PathBuf::from("/usr/local/lib/node_modules/@lebocqtitouan/ecc")
        );
    }

    #[test]
    fn resolve_ecc_root_prefers_home_over_npm() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/home/user/.ecc/agents")
            .with_dir("/usr/local/lib/node_modules/@lebocqtitouan/ecc/agents");
        let env = MockEnvironment::new().with_home("/home/user");

        let result = resolve_ecc_root(&fs, &env);
        assert!(result.is_ok());
        // $HOME/.ecc/ should be preferred over npm paths
        assert_eq!(
            result.unwrap(),
            std::path::PathBuf::from("/home/user/.ecc")
        );
    }

    #[test]
    fn resolve_ecc_root_errors_when_no_paths_found() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new().with_home("/home/user");

        let result = resolve_ecc_root(&fs, &env);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot find ECC assets directory"));
    }

    #[test]
    fn resolve_ecc_root_uses_ecc_root_env_var() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/opt/ecc/agents");
        let env = MockEnvironment::new()
            .with_home("/home/user")
            .with_var("ECC_ROOT", "/opt/ecc");

        let result = resolve_ecc_root(&fs, &env);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), std::path::PathBuf::from("/opt/ecc"));
    }

    #[test]
    fn resolve_ecc_root_ecc_root_env_var_takes_priority() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/opt/ecc/agents")
            .with_dir("/home/user/.ecc/agents");
        let env = MockEnvironment::new()
            .with_home("/home/user")
            .with_var("ECC_ROOT", "/opt/ecc");

        let result = resolve_ecc_root(&fs, &env);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), std::path::PathBuf::from("/opt/ecc"));
    }

    #[test]
    fn resolve_ecc_root_skips_invalid_ecc_root_env_var() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/home/user/.ecc/agents");
        let env = MockEnvironment::new()
            .with_home("/home/user")
            .with_var("ECC_ROOT", "/nonexistent/path");

        let result = resolve_ecc_root(&fs, &env);
        assert!(result.is_ok());
        // Falls through to $HOME/.ecc/
        assert_eq!(
            result.unwrap(),
            std::path::PathBuf::from("/home/user/.ecc")
        );
    }

    #[test]
    fn resolve_ecc_root_finds_repo_root() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/code/ecc/agents")
            .with_file("/code/ecc/Cargo.toml", "[workspace]");
        let env = MockEnvironment::new()
            .with_home("/home/user")
            .with_current_dir("/code/ecc/crates/ecc-cli");

        let result = resolve_ecc_root(&fs, &env);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), std::path::PathBuf::from("/code/ecc"));
    }
}
