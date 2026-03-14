//! Interactive merge orchestration — ports TS merge.ts interactive flow.
//!
//! Orchestrates file-by-file merge with user prompts for accept/keep/smart-merge.

mod helpers;
mod prompt;

use crate::config::merge as config_merge;
use crate::smart_merge;
use ecc_domain::config::merge::{
    self, FileToReview, MergeReport,
};
use ecc_ports::env::Environment;
use ecc_ports::fs::FileSystem;
use ecc_ports::shell::ShellExecutor;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

pub use prompt::{apply_review_choice, prompt_file_review};
use helpers::{copy_dir_recursive, read_json, read_json_or_default};

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
