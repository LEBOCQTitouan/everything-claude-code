//! Interactive merge orchestration — ports TS merge.ts interactive flow.
//!
//! Orchestrates file-by-file merge with user prompts for accept/keep/smart-merge.

use crate::config::merge as config_merge;
use crate::smart_merge;
use ecc_domain::config::merge::{
    self, FileToReview, MergeReport,
};
use ecc_domain::diff::{formatter, lcs};
use ecc_ports::env::Environment;
use ecc_ports::fs::FileSystem;
use ecc_ports::shell::ShellExecutor;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// User's choice when reviewing a file conflict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewChoice {
    /// Accept the incoming version.
    Accept,
    /// Keep the existing version.
    Keep,
    /// Use Claude CLI to smart-merge.
    SmartMerge,
}

/// Options controlling merge behavior.
#[derive(Debug, Clone)]
pub struct MergeOptions {
    pub dry_run: bool,
    pub force: bool,
    pub interactive: bool,
    /// When set, apply this choice to all remaining files without prompting.
    pub apply_all: Option<ReviewChoice>,
}

/// Create default merge options (interactive, no dry-run, no force).
pub fn default_merge_options() -> MergeOptions {
    MergeOptions {
        dry_run: false,
        force: false,
        interactive: true,
        apply_all: None,
    }
}

/// Context bundling all port references for merge operations.
pub struct MergeContext<'a> {
    pub fs: &'a dyn FileSystem,
    pub terminal: &'a dyn TerminalIO,
    pub env: &'a dyn Environment,
    pub shell: &'a dyn ShellExecutor,
}

// ---------------------------------------------------------------------------
// Prompt + review
// ---------------------------------------------------------------------------

/// Display a diff and prompt the user for a review choice.
///
/// Returns `(choice, apply_to_all)`. When `apply_to_all` is true, the choice
/// should be applied to all remaining files.
pub fn prompt_file_review(
    terminal: &dyn TerminalIO,
    fs: &dyn FileSystem,
    env: &dyn Environment,
    file: &FileToReview,
    progress: &str,
) -> Result<(ReviewChoice, bool), String> {
    let colored = env.var("NO_COLOR").is_none();

    if file.is_new {
        terminal.stdout_write(&format!(
            "\n{progress} NEW: {}\n",
            file.filename
        ));
        // Show preview of incoming file
        if let Ok(content) = fs.read_to_string(&file.src_path) {
            let preview: Vec<&str> = content.lines().take(10).collect();
            terminal.stdout_write(&format!("  Preview: {} lines\n", content.lines().count()));
            for line in &preview {
                terminal.stdout_write(&format!("    {line}\n"));
            }
            if content.lines().count() > 10 {
                terminal.stdout_write("    ...\n");
            }
        }
    } else {
        terminal.stdout_write(&format!(
            "\n{progress} CHANGED: {}\n",
            file.filename
        ));
        // Show diff
        let existing = fs.read_to_string(&file.dest_path).unwrap_or_default();
        let incoming = fs.read_to_string(&file.src_path).unwrap_or_default();
        let old_lines: Vec<&str> = existing.lines().collect();
        let new_lines: Vec<&str> = incoming.lines().collect();
        let diff = lcs::compute_line_diff(&old_lines, &new_lines);
        let diff_output = formatter::generate_diff(&diff, colored);
        terminal.stdout_write(&format!("{diff_output}\n"));
    }

    terminal.stdout_write("  [a]ccept | [k]eep | [s]mart-merge | [A]ccept all | [K]eep all: ");

    let input = terminal
        .prompt("")
        .map_err(|_| "Prompt cancelled".to_string())?;

    match input.trim() {
        "a" => Ok((ReviewChoice::Accept, false)),
        "k" => Ok((ReviewChoice::Keep, false)),
        "s" => Ok((ReviewChoice::SmartMerge, false)),
        "A" => Ok((ReviewChoice::Accept, true)),
        "K" => Ok((ReviewChoice::Keep, true)),
        _ => Ok((ReviewChoice::Accept, false)), // Default to accept
    }
}

/// Apply a review choice for a single file, updating the merge report.
///
/// Returns updated options (may have `apply_all` set if user chose "all").
pub fn apply_review_choice(
    fs: &dyn FileSystem,
    shell: &dyn ShellExecutor,
    choice: ReviewChoice,
    file: &FileToReview,
    dry_run: bool,
    report: &mut MergeReport,
) {
    match choice {
        ReviewChoice::Accept => {
            match config_merge::apply_accept(fs, &file.src_path, &file.dest_path, dry_run) {
                Ok(()) => {
                    if file.is_new {
                        report.added.push(file.filename.clone());
                    } else {
                        report.updated.push(file.filename.clone());
                    }
                }
                Err(e) => {
                    report.errors.push(format!("{}: {e}", file.filename));
                }
            }
        }
        ReviewChoice::Keep => {
            report.skipped.push(file.filename.clone());
        }
        ReviewChoice::SmartMerge => {
            let existing = fs
                .read_to_string(&file.dest_path)
                .unwrap_or_default();
            let incoming = fs
                .read_to_string(&file.src_path)
                .unwrap_or_default();

            let result = smart_merge::smart_merge(shell, &existing, &incoming, &file.filename);
            if result.success {
                if let Some(merged) = &result.merged {
                    if !dry_run
                        && let Err(e) = fs.write(&file.dest_path, merged)
                    {
                        report
                            .errors
                            .push(format!("{}: write error: {e}", file.filename));
                        return;
                    }
                    report.smart_merged.push(file.filename.clone());
                }
            } else {
                // Smart merge failed — fall back to accept
                let err = result.error.unwrap_or_else(|| "unknown error".to_string());
                report
                    .errors
                    .push(format!("{}: smart merge failed: {err}", file.filename));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Directory-level merge
// ---------------------------------------------------------------------------

/// Merge a directory of files (e.g., agents/, commands/).
///
/// Uses `pre_scan_directory` from domain to find changed/new files,
/// then prompts for each one (or applies force/apply_all).
pub fn merge_directory(
    ctx: &MergeContext,
    src_dir: &Path,
    dest_dir: &Path,
    artifact_type: &str,
    ext: &str,
    options: &mut MergeOptions,
) -> MergeReport {
    let mut report = merge::empty_report();

    let (files_to_review, unchanged) = config_merge::pre_scan_directory(ctx.fs, src_dir, dest_dir, ext);
    report.unchanged = unchanged;

    if files_to_review.is_empty() {
        return report;
    }

    let total = files_to_review.len();
    for (i, file) in files_to_review.iter().enumerate() {
        let progress = format!("[{}/{}]", i + 1, total);

        let choice = resolve_choice(ctx, file, &progress, options);
        apply_review_choice(ctx.fs, ctx.shell, choice, file, options.dry_run, &mut report);
    }

    if !report.added.is_empty() || !report.updated.is_empty() {
        ctx.terminal.stdout_write(&format!(
            "{}\n",
            merge::format_merge_report(artifact_type, &report)
        ));
    }

    report
}

/// Determine the review choice for a file based on options (force/non-interactive/apply-all/prompt).
fn resolve_choice(
    ctx: &MergeContext,
    file: &FileToReview,
    progress: &str,
    options: &mut MergeOptions,
) -> ReviewChoice {
    if options.force || !options.interactive {
        return ReviewChoice::Accept;
    }
    if let Some(all_choice) = options.apply_all {
        return all_choice;
    }
    match prompt_file_review(ctx.terminal, ctx.fs, ctx.env, file, progress) {
        Ok((choice, apply_all)) => {
            if apply_all {
                options.apply_all = Some(choice);
            }
            choice
        }
        Err(_) => ReviewChoice::Accept,
    }
}

/// Merge skills directories.
///
/// Skills are directories containing a SKILL.md. The SKILL.md content is used
/// for diffing, but the whole directory is copied on accept.
pub fn merge_skills(
    ctx: &MergeContext,
    src_dir: &Path,
    dest_dir: &Path,
    options: &mut MergeOptions,
) -> MergeReport {
    let mut report = merge::empty_report();

    let src_entries = match ctx.fs.read_dir(src_dir) {
        Ok(e) => e,
        Err(_) => return report,
    };

    let skill_dirs: Vec<String> = src_entries
        .iter()
        .filter(|p| ctx.fs.is_dir(p))
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();

    if skill_dirs.is_empty() {
        return report;
    }

    let total = skill_dirs.len();
    for (i, skill_name) in skill_dirs.iter().enumerate() {
        let src_skill = src_dir.join(skill_name);
        let dest_skill = dest_dir.join(skill_name);
        let src_skill_md = src_skill.join("SKILL.md");
        let dest_skill_md = dest_skill.join("SKILL.md");

        let is_new = !ctx.fs.exists(&dest_skill);
        let needs_update = if is_new {
            true
        } else {
            let src_content = ctx.fs.read_to_string(&src_skill_md).unwrap_or_default();
            let dest_content = ctx.fs.read_to_string(&dest_skill_md).unwrap_or_default();
            merge::contents_differ(&src_content, &dest_content)
        };

        if !needs_update {
            report.unchanged.push(skill_name.clone());
            continue;
        }

        let file = FileToReview {
            filename: skill_name.clone(),
            src_path: src_skill_md.clone(),
            dest_path: dest_skill_md.clone(),
            is_new,
        };

        let progress = format!("[{}/{}]", i + 1, total);
        let choice = resolve_choice(ctx, &file, &progress, options);

        match choice {
            ReviewChoice::Accept => {
                if !options.dry_run
                    && let Err(e) = copy_dir_recursive(ctx.fs, &src_skill, &dest_skill)
                {
                    report.errors.push(format!("{skill_name}: {e}"));
                    continue;
                }
                if is_new {
                    report.added.push(skill_name.clone());
                } else {
                    report.updated.push(skill_name.clone());
                }
            }
            ReviewChoice::Keep => {
                report.skipped.push(skill_name.clone());
            }
            ReviewChoice::SmartMerge => {
                let existing = ctx.fs.read_to_string(&dest_skill_md).unwrap_or_default();
                let incoming = ctx.fs.read_to_string(&src_skill_md).unwrap_or_default();
                let result = smart_merge::smart_merge(
                    ctx.shell,
                    &existing,
                    &incoming,
                    &format!("{skill_name}/SKILL.md"),
                );

                if result.success {
                    if let Some(merged) = &result.merged {
                        if !options.dry_run {
                            if let Err(e) = copy_dir_recursive(ctx.fs, &src_skill, &dest_skill) {
                                report.errors.push(format!("{skill_name}: {e}"));
                                continue;
                            }
                            if let Err(e) = ctx.fs.write(&dest_skill_md, merged) {
                                report.errors.push(format!(
                                    "{skill_name}/SKILL.md: write error: {e}"
                                ));
                                continue;
                            }
                        }
                        report.smart_merged.push(skill_name.clone());
                    }
                } else {
                    let err = result.error.unwrap_or_else(|| "unknown".into());
                    report
                        .errors
                        .push(format!("{skill_name}: smart merge failed: {err}"));
                }
            }
        }
    }

    if !report.added.is_empty() || !report.updated.is_empty() {
        ctx.terminal
            .stdout_write(&format!("{}\n", merge::format_merge_report("Skills", &report)));
    }

    report
}

/// Merge rules grouped by language/category subdirectory.
pub fn merge_rules(
    ctx: &MergeContext,
    src_dir: &Path,
    dest_dir: &Path,
    groups: &[String],
    options: &mut MergeOptions,
) -> MergeReport {
    let mut reports = Vec::new();

    for group in groups {
        let src_group = src_dir.join(group);
        let dest_group = dest_dir.join(group);

        if !ctx.fs.is_dir(&src_group) {
            continue;
        }

        let group_report = merge_directory(
            ctx,
            &src_group,
            &dest_group,
            &format!("Rules/{group}"),
            ".md",
            options,
        );
        reports.push(group_report);
    }

    merge::combine_reports(&reports)
}

/// Merge hooks from a source hooks.json into settings.json.
///
/// Returns `(added, existing, legacy_removed)` counts.
pub fn merge_hooks(
    fs: &dyn FileSystem,
    hooks_json_path: &Path,
    settings_json_path: &Path,
    dry_run: bool,
) -> Result<(usize, usize, usize), String> {
    let source_hooks = read_json(fs, hooks_json_path)?;
    let existing_settings = read_json_or_default(fs, settings_json_path);

    let existing_hooks = existing_settings
        .get("hooks")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));

    let (merged_hooks, added, existing, legacy_removed) =
        merge::merge_hooks_pure(&source_hooks, &existing_hooks);

    if added > 0 && !dry_run {
        let mut settings = existing_settings;
        settings
            .as_object_mut()
            .ok_or_else(|| "settings.json is not an object".to_string())?
            .insert("hooks".to_string(), merged_hooks);

        let json = serde_json::to_string_pretty(&settings)
            .map_err(|e| format!("JSON serialization error: {e}"))?;
        fs.write(settings_json_path, &format!("{json}\n"))
            .map_err(|e| format!("Write error: {e}"))?;
    }

    Ok((added, existing, legacy_removed))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn read_json(fs: &dyn FileSystem, path: &Path) -> Result<serde_json::Value, String> {
    let content = fs
        .read_to_string(path)
        .map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Invalid JSON in {}: {e}", path.display()))
}

fn read_json_or_default(fs: &dyn FileSystem, path: &Path) -> serde_json::Value {
    read_json(fs, path).unwrap_or_else(|_| serde_json::json!({}))
}

fn copy_dir_recursive(
    fs: &dyn FileSystem,
    src: &Path,
    dest: &Path,
) -> Result<(), String> {
    fs.create_dir_all(dest)
        .map_err(|e| format!("Cannot create directory {}: {e}", dest.display()))?;

    let entries = fs
        .read_dir_recursive(src)
        .map_err(|e| format!("Cannot read directory {}: {e}", src.display()))?;

    for entry in entries {
        if let Ok(relative) = entry.strip_prefix(src) {
            let dest_path = dest.join(relative);
            if let Some(parent) = dest_path.parent() {
                fs.create_dir_all(parent)
                    .map_err(|e| format!("Cannot create dir: {e}"))?;
            }
            fs.copy(&entry, &dest_path)
                .map_err(|e| format!("Cannot copy {}: {e}", entry.display()))?;
        }
    }

    Ok(())
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

    // --- prompt_file_review ---

    #[test]
    fn prompt_accept() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/agent.md", "new content")
            .with_file("/dest/agent.md", "old content");
        let terminal = BufferedTerminal::new().with_input("a");
        let env = no_color_env();
        let file = FileToReview {
            filename: "agent.md".into(),
            src_path: "/src/agent.md".into(),
            dest_path: "/dest/agent.md".into(),
            is_new: false,
        };

        let (choice, apply_all) =
            prompt_file_review(&terminal, &fs, &env, &file, "[1/2]").unwrap();
        assert_eq!(choice, ReviewChoice::Accept);
        assert!(!apply_all);
    }

    #[test]
    fn prompt_keep() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/agent.md", "new")
            .with_file("/dest/agent.md", "old");
        let terminal = BufferedTerminal::new().with_input("k");
        let env = no_color_env();
        let file = FileToReview {
            filename: "agent.md".into(),
            src_path: "/src/agent.md".into(),
            dest_path: "/dest/agent.md".into(),
            is_new: false,
        };

        let (choice, _) = prompt_file_review(&terminal, &fs, &env, &file, "[1/1]").unwrap();
        assert_eq!(choice, ReviewChoice::Keep);
    }

    #[test]
    fn prompt_smart_merge() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/agent.md", "new")
            .with_file("/dest/agent.md", "old");
        let terminal = BufferedTerminal::new().with_input("s");
        let env = no_color_env();
        let file = FileToReview {
            filename: "agent.md".into(),
            src_path: "/src/agent.md".into(),
            dest_path: "/dest/agent.md".into(),
            is_new: false,
        };

        let (choice, _) = prompt_file_review(&terminal, &fs, &env, &file, "[1/1]").unwrap();
        assert_eq!(choice, ReviewChoice::SmartMerge);
    }

    #[test]
    fn prompt_accept_all() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/agent.md", "new")
            .with_file("/dest/agent.md", "old");
        let terminal = BufferedTerminal::new().with_input("A");
        let env = no_color_env();
        let file = FileToReview {
            filename: "agent.md".into(),
            src_path: "/src/agent.md".into(),
            dest_path: "/dest/agent.md".into(),
            is_new: false,
        };

        let (choice, apply_all) =
            prompt_file_review(&terminal, &fs, &env, &file, "[1/2]").unwrap();
        assert_eq!(choice, ReviewChoice::Accept);
        assert!(apply_all);
    }

    #[test]
    fn prompt_keep_all() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/agent.md", "new")
            .with_file("/dest/agent.md", "old");
        let terminal = BufferedTerminal::new().with_input("K");
        let env = no_color_env();
        let file = FileToReview {
            filename: "agent.md".into(),
            src_path: "/src/agent.md".into(),
            dest_path: "/dest/agent.md".into(),
            is_new: false,
        };

        let (choice, apply_all) =
            prompt_file_review(&terminal, &fs, &env, &file, "[1/2]").unwrap();
        assert_eq!(choice, ReviewChoice::Keep);
        assert!(apply_all);
    }

    #[test]
    fn prompt_new_file_shows_preview() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/new.md", "# New agent\nLine 2\nLine 3");
        let terminal = BufferedTerminal::new().with_input("a");
        let env = no_color_env();
        let file = FileToReview {
            filename: "new.md".into(),
            src_path: "/src/new.md".into(),
            dest_path: "/dest/new.md".into(),
            is_new: true,
        };

        prompt_file_review(&terminal, &fs, &env, &file, "[1/1]").unwrap();
        let output = terminal.stdout_output().join("");
        assert!(output.contains("NEW: new.md"));
        assert!(output.contains("3 lines"));
    }

    // --- apply_review_choice ---

    #[test]
    fn apply_accept_copies_file() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/agent.md", "new content")
            .with_dir("/dest");
        let shell = MockExecutor::new();
        let file = FileToReview {
            filename: "agent.md".into(),
            src_path: "/src/agent.md".into(),
            dest_path: "/dest/agent.md".into(),
            is_new: true,
        };
        let mut report = merge::empty_report();

        apply_review_choice(&fs, &shell, ReviewChoice::Accept, &file, false, &mut report);

        assert_eq!(report.added, vec!["agent.md"]);
        assert_eq!(fs.read_to_string(Path::new("/dest/agent.md")).unwrap(), "new content");
    }

    #[test]
    fn apply_accept_dry_run_does_not_copy() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/agent.md", "new content")
            .with_dir("/dest");
        let shell = MockExecutor::new();
        let file = FileToReview {
            filename: "agent.md".into(),
            src_path: "/src/agent.md".into(),
            dest_path: "/dest/agent.md".into(),
            is_new: true,
        };
        let mut report = merge::empty_report();

        apply_review_choice(&fs, &shell, ReviewChoice::Accept, &file, true, &mut report);

        assert_eq!(report.added, vec!["agent.md"]);
        assert!(!fs.exists(Path::new("/dest/agent.md")));
    }

    #[test]
    fn apply_keep_skips() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let file = FileToReview {
            filename: "agent.md".into(),
            src_path: "/src/agent.md".into(),
            dest_path: "/dest/agent.md".into(),
            is_new: false,
        };
        let mut report = merge::empty_report();

        apply_review_choice(&fs, &shell, ReviewChoice::Keep, &file, false, &mut report);

        assert_eq!(report.skipped, vec!["agent.md"]);
    }

    #[test]
    fn apply_smart_merge_success() {
        let fs = InMemoryFileSystem::new()
            .with_file("/dest/agent.md", "old content")
            .with_file("/src/agent.md", "new content");
        let shell = MockExecutor::new()
            .with_command("claude")
            .on(
                "claude",
                CommandOutput {
                    stdout: "merged content".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            );
        let file = FileToReview {
            filename: "agent.md".into(),
            src_path: "/src/agent.md".into(),
            dest_path: "/dest/agent.md".into(),
            is_new: false,
        };
        let mut report = merge::empty_report();

        apply_review_choice(&fs, &shell, ReviewChoice::SmartMerge, &file, false, &mut report);

        assert_eq!(report.smart_merged, vec!["agent.md"]);
        assert_eq!(
            fs.read_to_string(Path::new("/dest/agent.md")).unwrap(),
            "merged content"
        );
    }

    #[test]
    fn apply_smart_merge_failure_records_error() {
        let fs = InMemoryFileSystem::new()
            .with_file("/dest/agent.md", "old")
            .with_file("/src/agent.md", "new");
        let shell = MockExecutor::new(); // No claude available
        let file = FileToReview {
            filename: "agent.md".into(),
            src_path: "/src/agent.md".into(),
            dest_path: "/dest/agent.md".into(),
            is_new: false,
        };
        let mut report = merge::empty_report();

        apply_review_choice(&fs, &shell, ReviewChoice::SmartMerge, &file, false, &mut report);

        assert!(!report.errors.is_empty());
        assert!(report.errors[0].contains("smart merge failed"));
    }

    // --- merge_directory ---

    #[test]
    fn merge_directory_force_mode() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/agents/planner.md", "new planner")
            .with_file("/src/agents/reviewer.md", "new reviewer")
            .with_dir("/dest/agents");
        let terminal = BufferedTerminal::new();
        let env = no_color_env();
        let shell = MockExecutor::new();
        let ctx = MergeContext { fs: &fs, terminal: &terminal, env: &env, shell: &shell };
        let mut options = MergeOptions {
            dry_run: false,
            force: true,
            interactive: true,
            apply_all: None,
        };

        let report = merge_directory(
            &ctx,
            Path::new("/src/agents"),
            Path::new("/dest/agents"),
            "Agents",
            ".md",
            &mut options,
        );

        assert_eq!(report.added.len(), 2);
        assert!(fs.exists(Path::new("/dest/agents/planner.md")));
    }

    #[test]
    fn merge_directory_skips_unchanged() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/agents/planner.md", "same content")
            .with_file("/dest/agents/planner.md", "same content");
        let terminal = BufferedTerminal::new();
        let env = no_color_env();
        let shell = MockExecutor::new();
        let ctx = MergeContext { fs: &fs, terminal: &terminal, env: &env, shell: &shell };
        let mut options = default_merge_options();
        options.force = true;

        let report = merge_directory(
            &ctx,
            Path::new("/src/agents"),
            Path::new("/dest/agents"),
            "Agents",
            ".md",
            &mut options,
        );

        assert!(report.added.is_empty());
        assert!(report.updated.is_empty());
        assert_eq!(report.unchanged, vec!["planner.md"]);
    }

    #[test]
    fn merge_directory_non_interactive_accepts_all() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/agents/a.md", "new a")
            .with_file("/src/agents/b.md", "new b")
            .with_dir("/dest/agents");
        let terminal = BufferedTerminal::new();
        let env = no_color_env();
        let shell = MockExecutor::new();
        let ctx = MergeContext { fs: &fs, terminal: &terminal, env: &env, shell: &shell };
        let mut options = MergeOptions {
            dry_run: false,
            force: false,
            interactive: false,
            apply_all: None,
        };

        let report = merge_directory(
            &ctx,
            Path::new("/src/agents"),
            Path::new("/dest/agents"),
            "Agents",
            ".md",
            &mut options,
        );

        assert_eq!(report.added.len(), 2);
    }

    #[test]
    fn merge_directory_dry_run() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/agents/new.md", "content")
            .with_dir("/dest/agents");
        let terminal = BufferedTerminal::new();
        let env = no_color_env();
        let shell = MockExecutor::new();
        let ctx = MergeContext { fs: &fs, terminal: &terminal, env: &env, shell: &shell };
        let mut options = MergeOptions {
            dry_run: true,
            force: true,
            interactive: true,
            apply_all: None,
        };

        let report = merge_directory(
            &ctx,
            Path::new("/src/agents"),
            Path::new("/dest/agents"),
            "Agents",
            ".md",
            &mut options,
        );

        assert_eq!(report.added, vec!["new.md"]);
        assert!(!fs.exists(Path::new("/dest/agents/new.md")));
    }

    // --- merge_skills ---

    #[test]
    fn merge_skills_force_copies_directory() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/src/skills/tdd")
            .with_file("/src/skills/tdd/SKILL.md", "# TDD Skill")
            .with_file("/src/skills/tdd/examples.md", "# Examples")
            .with_dir("/dest/skills");
        let terminal = BufferedTerminal::new();
        let env = no_color_env();
        let shell = MockExecutor::new();
        let ctx = MergeContext { fs: &fs, terminal: &terminal, env: &env, shell: &shell };
        let mut options = MergeOptions {
            dry_run: false,
            force: true,
            interactive: true,
            apply_all: None,
        };

        let report = merge_skills(
            &ctx,
            Path::new("/src/skills"),
            Path::new("/dest/skills"),
            &mut options,
        );

        assert_eq!(report.added, vec!["tdd"]);
        assert!(fs.exists(Path::new("/dest/skills/tdd/SKILL.md")));
        assert!(fs.exists(Path::new("/dest/skills/tdd/examples.md")));
    }

    #[test]
    fn merge_skills_unchanged_skipped() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/src/skills/tdd")
            .with_file("/src/skills/tdd/SKILL.md", "same content")
            .with_dir("/dest/skills/tdd")
            .with_file("/dest/skills/tdd/SKILL.md", "same content");
        let terminal = BufferedTerminal::new();
        let env = no_color_env();
        let shell = MockExecutor::new();
        let ctx = MergeContext { fs: &fs, terminal: &terminal, env: &env, shell: &shell };
        let mut options = default_merge_options();
        options.force = true;

        let report = merge_skills(
            &ctx,
            Path::new("/src/skills"),
            Path::new("/dest/skills"),
            &mut options,
        );

        assert!(report.added.is_empty());
        assert_eq!(report.unchanged, vec!["tdd"]);
    }

    // --- merge_rules ---

    #[test]
    fn merge_rules_force_mode() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/src/rules/common")
            .with_file("/src/rules/common/style.md", "# Style")
            .with_dir("/src/rules/typescript")
            .with_file("/src/rules/typescript/types.md", "# Types")
            .with_dir("/dest/rules");
        let terminal = BufferedTerminal::new();
        let env = no_color_env();
        let shell = MockExecutor::new();
        let ctx = MergeContext { fs: &fs, terminal: &terminal, env: &env, shell: &shell };
        let mut options = MergeOptions {
            dry_run: false,
            force: true,
            interactive: true,
            apply_all: None,
        };

        let groups = vec!["common".to_string(), "typescript".to_string()];
        let report = merge_rules(
            &ctx,
            Path::new("/src/rules"),
            Path::new("/dest/rules"),
            &groups,
            &mut options,
        );

        assert_eq!(report.added.len(), 2);
        assert!(fs.exists(Path::new("/dest/rules/common/style.md")));
        assert!(fs.exists(Path::new("/dest/rules/typescript/types.md")));
    }

    // --- merge_hooks ---

    #[test]
    fn merge_hooks_adds_new() {
        let source = serde_json::json!({
            "PreToolUse": [{
                "description": "ECC format",
                "hooks": [{"command": "ecc-hook format"}]
            }]
        });
        let fs = InMemoryFileSystem::new()
            .with_file("/hooks.json", &serde_json::to_string(&source).unwrap())
            .with_file("/settings.json", "{}");

        let (added, existing, legacy) = merge_hooks(
            &fs,
            Path::new("/hooks.json"),
            Path::new("/settings.json"),
            false,
        )
        .unwrap();

        assert_eq!(added, 1);
        assert_eq!(existing, 0);
        assert_eq!(legacy, 0);

        // Verify settings.json was updated
        let updated: serde_json::Value = serde_json::from_str(
            &fs.read_to_string(Path::new("/settings.json")).unwrap(),
        )
        .unwrap();
        assert!(updated["hooks"]["PreToolUse"].is_array());
    }

    #[test]
    fn merge_hooks_dedup() {
        let hook = serde_json::json!({
            "description": "ECC format",
            "hooks": [{"command": "ecc-hook format"}]
        });
        let source = serde_json::json!({ "PreToolUse": [hook.clone()] });
        let settings = serde_json::json!({
            "hooks": { "PreToolUse": [hook] }
        });
        let fs = InMemoryFileSystem::new()
            .with_file("/hooks.json", &serde_json::to_string(&source).unwrap())
            .with_file("/settings.json", &serde_json::to_string(&settings).unwrap());

        let (added, existing, _) = merge_hooks(
            &fs,
            Path::new("/hooks.json"),
            Path::new("/settings.json"),
            false,
        )
        .unwrap();

        assert_eq!(added, 0);
        assert_eq!(existing, 1);
    }

    #[test]
    fn merge_hooks_removes_legacy() {
        let legacy_hook = serde_json::json!({
            "description": "old hook",
            "hooks": [{"command": "node /path/to/everything-claude-code/dist/hooks/run.js"}]
        });
        let settings = serde_json::json!({
            "hooks": { "PreToolUse": [legacy_hook] }
        });
        let source = serde_json::json!({});
        let fs = InMemoryFileSystem::new()
            .with_file("/hooks.json", &serde_json::to_string(&source).unwrap())
            .with_file("/settings.json", &serde_json::to_string(&settings).unwrap());

        let (_, _, legacy_removed) = merge_hooks(
            &fs,
            Path::new("/hooks.json"),
            Path::new("/settings.json"),
            false,
        )
        .unwrap();

        assert_eq!(legacy_removed, 1);
    }

    #[test]
    fn merge_hooks_dry_run_does_not_write() {
        let source = serde_json::json!({
            "PreToolUse": [{"hooks": [{"command": "ecc-hook test"}]}]
        });
        let fs = InMemoryFileSystem::new()
            .with_file("/hooks.json", &serde_json::to_string(&source).unwrap())
            .with_file("/settings.json", "{}");

        let (added, _, _) = merge_hooks(
            &fs,
            Path::new("/hooks.json"),
            Path::new("/settings.json"),
            true,
        )
        .unwrap();

        assert_eq!(added, 1);
        // Settings should remain unchanged
        let settings = fs.read_to_string(Path::new("/settings.json")).unwrap();
        assert_eq!(settings, "{}");
    }

    #[test]
    fn merge_hooks_invalid_hooks_json() {
        let fs = InMemoryFileSystem::new()
            .with_file("/hooks.json", "not json")
            .with_file("/settings.json", "{}");

        let result = merge_hooks(
            &fs,
            Path::new("/hooks.json"),
            Path::new("/settings.json"),
            false,
        );

        assert!(result.is_err());
    }

    // --- merge_directory with scripted prompts ---

    #[test]
    fn merge_directory_interactive_accept() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/agents/new.md", "# New agent")
            .with_dir("/dest/agents");
        let terminal = BufferedTerminal::new().with_input("a");
        let env = no_color_env();
        let shell = MockExecutor::new();
        let ctx = MergeContext { fs: &fs, terminal: &terminal, env: &env, shell: &shell };
        let mut options = default_merge_options();

        let report = merge_directory(
            &ctx,
            Path::new("/src/agents"),
            Path::new("/dest/agents"),
            "Agents",
            ".md",
            &mut options,
        );

        assert_eq!(report.added, vec!["new.md"]);
    }

    #[test]
    fn merge_directory_interactive_keep() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/agents/existing.md", "updated content")
            .with_file("/dest/agents/existing.md", "old content");
        let terminal = BufferedTerminal::new().with_input("k");
        let env = no_color_env();
        let shell = MockExecutor::new();
        let ctx = MergeContext { fs: &fs, terminal: &terminal, env: &env, shell: &shell };
        let mut options = default_merge_options();

        let report = merge_directory(
            &ctx,
            Path::new("/src/agents"),
            Path::new("/dest/agents"),
            "Agents",
            ".md",
            &mut options,
        );

        assert_eq!(report.skipped, vec!["existing.md"]);
    }
}
