use super::InstallSummary;
use ecc_domain::ansi;
use ecc_domain::config::deny_rules;
use ecc_domain::config::manifest::Artifacts;
use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

/// Collect rule group directories from ecc_root/rules/, filtered by language.
pub(super) fn collect_rule_groups(
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
pub(super) fn collect_installed_artifacts(fs: &dyn FileSystem, claude_dir: &Path) -> Artifacts {
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
pub(super) fn ensure_deny_rules_in_settings(
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

pub(super) fn print_summary(
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
    use crate::config::manifest::read_manifest;
    use crate::install::{
        default_install_options, init_project, install_global, InstallContext, InstallOptions,
    };
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
        let ctx = InstallContext { fs: &fs, shell: &shell, env: &env, terminal: &terminal };

        let options = InstallOptions {
            dry_run: false, force: true, no_gitignore: false, interactive: false,
            clean: false, clean_all: false, languages: vec![],
        };

        let summary = install_global(&ctx, Path::new("/ecc"), Path::new("/claude"), "4.0.0", "2026-03-14T00:00:00Z", &options);

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
        let ctx = InstallContext { fs: &fs, shell: &shell, env: &env, terminal: &terminal };

        let options = InstallOptions {
            dry_run: true, force: true, no_gitignore: false, interactive: false,
            clean: false, clean_all: false, languages: vec![],
        };

        let summary = install_global(&ctx, Path::new("/ecc"), Path::new("/claude"), "4.0.0", "2026-03-14T00:00:00Z", &options);

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
        let ctx = InstallContext { fs: &fs, shell: &shell, env: &env, terminal: &terminal };

        let options = InstallOptions {
            dry_run: false, force: true, no_gitignore: false, interactive: false,
            clean: false, clean_all: false, languages: vec![],
        };

        let summary = install_global(&ctx, Path::new("/ecc"), Path::new("/claude"), "4.0.0", "2026-03-14T00:00:00Z", &options);

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
        let ctx = InstallContext { fs: &fs, shell: &shell, env: &env, terminal: &terminal };

        let options = InstallOptions {
            dry_run: false, force: false, no_gitignore: false, interactive: false,
            clean: false, clean_all: false, languages: vec![],
        };

        let summary = install_global(&ctx, Path::new("/ecc"), Path::new("/claude"), "4.0.0", "2026-03-14T00:00:00Z", &options);

        assert!(summary.success);
        assert!(summary.added > 0);
    }

    #[test]
    fn install_with_clean_all() {
        let fs = ecc_source_fs()
            .with_dir("/claude/agents").with_file("/claude/agents/old.md", "# Old agent")
            .with_dir("/claude/commands").with_file("/claude/commands/old.md", "# Old command");
        let env = no_color_env();
        let terminal = BufferedTerminal::new();
        let shell = MockExecutor::new();
        let ctx = InstallContext { fs: &fs, shell: &shell, env: &env, terminal: &terminal };

        let options = InstallOptions {
            dry_run: false, force: true, no_gitignore: false, interactive: false,
            clean: false, clean_all: true, languages: vec![],
        };

        let summary = install_global(&ctx, Path::new("/ecc"), Path::new("/claude"), "4.0.0", "2026-03-14T00:00:00Z", &options);

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
        let ctx = InstallContext { fs: &fs, shell: &shell, env: &env, terminal: &terminal };

        let options = InstallOptions {
            dry_run: false, force: true, no_gitignore: false, interactive: false,
            clean: true, clean_all: false, languages: vec![],
        };

        install_global(&ctx, Path::new("/ecc"), Path::new("/claude"), "4.0.0", "2026-03-14T00:00:00Z", &options);

        let output = terminal.stdout_output().join("");
        assert!(output.contains("No manifest found"));
    }

    #[test]
    fn install_deny_rules_added() {
        let fs = ecc_source_fs();
        let env = no_color_env();
        let terminal = BufferedTerminal::new();
        let shell = MockExecutor::new();
        let ctx = InstallContext { fs: &fs, shell: &shell, env: &env, terminal: &terminal };

        let options = InstallOptions {
            dry_run: false, force: true, no_gitignore: false, interactive: false,
            clean: false, clean_all: false, languages: vec![],
        };

        install_global(&ctx, Path::new("/ecc"), Path::new("/claude"), "4.0.0", "2026-03-14T00:00:00Z", &options);

        let settings_content = fs.read_to_string(Path::new("/claude/settings.json")).unwrap();
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
        let ctx = InstallContext { fs: &fs, shell: &shell, env: &env, terminal: &terminal };

        let options = InstallOptions {
            dry_run: false, force: true, no_gitignore: false, interactive: false,
            clean: false, clean_all: false, languages: vec![],
        };

        install_global(&ctx, Path::new("/ecc"), Path::new("/claude"), "4.0.0", "2026-03-14T00:00:00Z", &options);

        let output = terminal.stdout_output().join("");
        assert!(output.contains("Install Summary"));
        assert!(output.contains("Install complete!"));
    }

    #[test]
    fn install_with_languages() {
        let fs = ecc_source_fs()
            .with_dir("/ecc/rules/typescript").with_file("/ecc/rules/typescript/types.md", "# Types")
            .with_dir("/ecc/rules/python").with_file("/ecc/rules/python/style.md", "# Python style");
        let env = no_color_env();
        let terminal = BufferedTerminal::new();
        let shell = MockExecutor::new();
        let ctx = InstallContext { fs: &fs, shell: &shell, env: &env, terminal: &terminal };

        let options = InstallOptions {
            dry_run: false, force: true, no_gitignore: false, interactive: false,
            clean: false, clean_all: false, languages: vec!["typescript".to_string()],
        };

        install_global(&ctx, Path::new("/ecc"), Path::new("/claude"), "4.0.0", "2026-03-14T00:00:00Z", &options);

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
        let shell = MockExecutor::new().on("git", CommandOutput { stdout: String::new(), stderr: String::new(), exit_code: 0 });
        let ctx = InstallContext { fs: &fs, shell: &shell, env: &env, terminal: &terminal };

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
        let ctx = InstallContext { fs: &fs, shell: &shell, env: &env, terminal: &terminal };

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
        let shell = MockExecutor::new().on("git", CommandOutput { stdout: String::new(), stderr: String::new(), exit_code: 0 });
        let ctx = InstallContext { fs: &fs, shell: &shell, env: &env, terminal: &terminal };

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
        let shell = MockExecutor::new().on("git", CommandOutput { stdout: String::new(), stderr: "not a git repo".into(), exit_code: 128 });
        let ctx = InstallContext { fs: &fs, shell: &shell, env: &env, terminal: &terminal };

        let result = init_project(&ctx, Path::new("/project"), false, false);
        assert!(result);
        let output = terminal.stdout_output().join("");
        assert!(output.contains("Not a git repository"));
    }

    // --- helpers ---

    #[test]
    fn collect_rule_groups_with_languages() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/ecc/rules/common").with_dir("/ecc/rules/typescript").with_dir("/ecc/rules/python");

        let groups = collect_rule_groups(&fs, Path::new("/ecc"), &["typescript".to_string()]);
        assert!(groups.contains(&"common".to_string()));
        assert!(groups.contains(&"typescript".to_string()));
        assert!(!groups.contains(&"python".to_string()));
    }

    #[test]
    fn collect_rule_groups_empty_languages() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/ecc/rules/common").with_dir("/ecc/rules/typescript");

        let groups = collect_rule_groups(&fs, Path::new("/ecc"), &[]);
        assert!(groups.contains(&"common".to_string()));
        assert!(groups.contains(&"typescript".to_string()));
    }
}
