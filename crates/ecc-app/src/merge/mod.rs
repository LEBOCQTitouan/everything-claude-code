//! Interactive merge orchestration — ports TS merge.ts interactive flow.
//!
//! Orchestrates file-by-file merge with user prompts for accept/keep/smart-merge.

pub mod error;
mod helpers;
mod prompt;

use crate::config::merge as config_merge;
use crate::smart_merge;
use ecc_domain::config::merge::{self, FileToReview, MergeReport};
use ecc_ports::env::Environment;
use ecc_ports::fs::FileSystem;
use ecc_ports::shell::ShellExecutor;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

use helpers::{copy_dir_recursive, read_json, read_json_or_default};
pub use prompt::{apply_review_choice, prompt_file_review};

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

impl Default for MergeOptions {
    fn default() -> Self {
        Self {
            dry_run: false,
            force: false,
            interactive: true,
            apply_all: None,
        }
    }
}

/// Create default merge options (interactive, no dry-run, no force).
pub fn default_merge_options() -> MergeOptions {
    MergeOptions::default()
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

    let (files_to_review, unchanged, scan_errors) =
        config_merge::pre_scan_directory(ctx.fs, src_dir, dest_dir, ext);
    report.unchanged = unchanged;
    report.errors.extend(scan_errors);

    if files_to_review.is_empty() {
        return report;
    }

    let total = files_to_review.len();
    for (i, file) in files_to_review.iter().enumerate() {
        let progress = format!("[{}/{}]", i + 1, total);

        let choice = resolve_choice(ctx, file, &progress, options);
        apply_review_choice(
            ctx.fs,
            ctx.shell,
            choice,
            file,
            options.dry_run,
            &mut report,
        );
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
        Err(e) => {
            tracing::warn!("resolve_choice: prompt failed, defaulting to Accept: {e}");
            ReviewChoice::Accept
        }
    }
}

/// Apply an Accept choice for a skill directory.
fn apply_skill_accept(
    ctx: &MergeContext,
    skill_name: &str,
    src_skill: &Path,
    dest_skill: &Path,
    is_new: bool,
    dry_run: bool,
    report: &mut ecc_domain::config::merge::MergeReport,
) -> bool {
    if !dry_run
        && let Err(e) = copy_dir_recursive(ctx.fs, src_skill, dest_skill)
    {
        report.errors.push(format!("{skill_name}: {e}"));
        return false;
    }
    if is_new {
        report.added.push(skill_name.to_owned());
    } else {
        report.updated.push(skill_name.to_owned());
    }
    true
}

/// Paths for a single skill directory and its SKILL.md files.
struct SkillPaths<'a> {
    skill_name: &'a str,
    src_skill: &'a Path,
    dest_skill: &'a Path,
    src_skill_md: &'a Path,
    dest_skill_md: &'a Path,
}

/// Apply a SmartMerge choice for a skill directory.
fn apply_skill_smart_merge(
    ctx: &MergeContext,
    paths: &SkillPaths<'_>,
    dry_run: bool,
    report: &mut ecc_domain::config::merge::MergeReport,
) {
    let (skill_name, src_skill, dest_skill, src_skill_md, dest_skill_md) = (
        paths.skill_name,
        paths.src_skill,
        paths.dest_skill,
        paths.src_skill_md,
        paths.dest_skill_md,
    );
    let existing = ctx.fs.read_to_string(dest_skill_md).unwrap_or_default();
    let incoming = ctx.fs.read_to_string(src_skill_md).unwrap_or_default();
    let result = smart_merge::smart_merge(ctx.shell, &existing, &incoming, &format!("{skill_name}/SKILL.md"));
    if result.success {
        if let Some(merged) = &result.merged {
            if !dry_run {
                if let Err(e) = copy_dir_recursive(ctx.fs, src_skill, dest_skill) {
                    report.errors.push(format!("{skill_name}: {e}"));
                    return;
                }
                if let Err(e) = ctx.fs.write(dest_skill_md, merged) {
                    report.errors.push(format!("{skill_name}/SKILL.md: write error: {e}"));
                    return;
                }
            }
            report.smart_merged.push(skill_name.to_owned());
        }
    } else {
        let err = result.error.unwrap_or_else(|| "unknown".into());
        report.errors.push(format!("{skill_name}: smart merge failed: {err}"));
    }
}

/// Process a single skill: determine if update is needed, prompt, apply choice.
fn process_one_skill(
    ctx: &MergeContext,
    src_dir: &Path,
    dest_dir: &Path,
    skill_name: &str,
    progress: &str,
    options: &mut MergeOptions,
    report: &mut ecc_domain::config::merge::MergeReport,
) {
    let src_skill = src_dir.join(skill_name);
    let dest_skill = dest_dir.join(skill_name);
    let src_skill_md = src_skill.join("SKILL.md");
    let dest_skill_md = dest_skill.join("SKILL.md");

    let is_new = !ctx.fs.exists(&dest_skill);
    let needs_update = is_new || {
        let src_content = ctx.fs.read_to_string(&src_skill_md).unwrap_or_default();
        let dest_content = ctx.fs.read_to_string(&dest_skill_md).unwrap_or_default();
        merge::contents_differ(&src_content, &dest_content)
    };

    if !needs_update {
        report.unchanged.push(skill_name.to_owned());
        return;
    }

    let file = FileToReview {
        filename: skill_name.to_owned(),
        src_path: src_skill_md.clone(),
        dest_path: dest_skill_md.clone(),
        is_new,
    };
    let choice = resolve_choice(ctx, &file, progress, options);
    match choice {
        ReviewChoice::Accept => {
            apply_skill_accept(ctx, skill_name, &src_skill, &dest_skill, is_new, options.dry_run, report);
        }
        ReviewChoice::Keep => {
            report.skipped.push(skill_name.to_owned());
        }
        ReviewChoice::SmartMerge => {
            apply_skill_smart_merge(
                ctx,
                &SkillPaths {
                    skill_name,
                    src_skill: &src_skill,
                    dest_skill: &dest_skill,
                    src_skill_md: &src_skill_md,
                    dest_skill_md: &dest_skill_md,
                },
                options.dry_run,
                report,
            );
        }
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
        Err(e) => {
            tracing::warn!("merge_skills: cannot read {}: {e}", src_dir.display());
            return report;
        }
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
        let progress = format!("[{}/{}]", i + 1, total);
        process_one_skill(ctx, src_dir, dest_dir, skill_name, &progress, options, &mut report);
    }

    if !report.added.is_empty() || !report.updated.is_empty() {
        ctx.terminal.stdout_write(&format!("{}\n", merge::format_merge_report("Skills", &report)));
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
///
/// Deserializes JSON at the boundary, uses typed domain functions for merge,
/// and serializes back when writing to disk.
pub fn merge_hooks(
    fs: &dyn FileSystem,
    hooks_json_path: &Path,
    settings_json_path: &Path,
    dry_run: bool,
) -> Result<(usize, usize, usize), error::MergeError> {
    use ecc_domain::config::hook_types::HooksMap;

    let source_file = read_json(fs, hooks_json_path)?;
    let source_hooks_value = source_file
        .get("hooks")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    let existing_settings = read_json_or_default(fs, settings_json_path);
    let existing_hooks_value = existing_settings
        .get("hooks")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));

    // Deserialize at boundary into typed model
    let source_hooks: HooksMap = serde_json::from_value(source_hooks_value).unwrap_or_default();
    let existing_hooks: HooksMap = serde_json::from_value(existing_hooks_value).unwrap_or_default();

    // Call typed domain function
    let merge_result = merge::merge_hooks_typed(&source_hooks, &existing_hooks);
    let added = merge_result.added;
    let existing = merge_result.existing;
    let legacy_removed = merge_result.legacy_removed;

    if (added > 0 || legacy_removed > 0) && !dry_run {
        // Serialize back at boundary
        let merged_value =
            serde_json::to_value(&merge_result.merged).map_err(|e| error::MergeError::Serialization {
                reason: e.to_string(),
            })?;

        let mut settings = existing_settings;
        settings
            .as_object_mut()
            .ok_or(error::MergeError::SettingsNotObject)?
            .insert("hooks".to_string(), merged_value);

        let json = serde_json::to_string_pretty(&settings).map_err(|e| {
            error::MergeError::Serialization {
                reason: e.to_string(),
            }
        })?;
        fs.write(settings_json_path, &format!("{json}\n"))
            .map_err(|e| error::MergeError::WriteSettings {
                reason: e.to_string(),
            })?;
    }

    Ok((added, existing, legacy_removed))
}
