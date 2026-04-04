use super::super::helpers::{
    collect_installed_artifacts, collect_rule_groups, ensure_deny_rules_in_settings,
    ensure_statusline_in_settings, rule_filter, stack_detect,
};
use super::super::{InstallContext, InstallOptions};
use crate::config::clean as app_clean;
use crate::config::manifest::{read_manifest, write_manifest};
use crate::detect;
use crate::merge::{self, MergeContext, MergeOptions, ReviewChoice};
use ecc_domain::config::clean;
use ecc_domain::config::manifest;
use ecc_domain::config::merge as domain_merge;
use std::path::Path;

pub(super) fn step_clean(ctx: &InstallContext, claude_dir: &Path, options: &InstallOptions) {
    tracing::info!("install: cleaning previous installation");
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

pub(super) fn step_detect(ctx: &InstallContext, claude_dir: &Path) {
    tracing::info!("install: detecting existing configuration");
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

pub(super) fn step_merge_artifacts(
    ctx: &InstallContext,
    ecc_root: &Path,
    claude_dir: &Path,
    options: &InstallOptions,
) -> domain_merge::MergeReport {
    tracing::info!("install: merging artifacts");
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
    let rules_src = ecc_root.join("rules");
    let skip_paths = resolve_rule_skip_paths(ctx, &rules_src, &rule_groups, options);

    all_reports.push(merge::merge_rules_filtered(
        &merge_ctx,
        &rules_src,
        &claude_dir.join("rules"),
        &rule_groups,
        &skip_paths,
        &mut merge_options,
    ));

    domain_merge::combine_reports(&all_reports)
}

/// Determine which rule files to skip based on detected project stack.
///
/// Returns an empty vec when `--all-rules` is active or when no stack is
/// detected (fail-open). Otherwise returns paths of rules whose `applies-to`
/// conditions do not match the detected stack.
fn resolve_rule_skip_paths(
    ctx: &InstallContext,
    rules_src: &Path,
    rule_groups: &[String],
    options: &InstallOptions,
) -> Vec<std::path::PathBuf> {
    if options.all_rules {
        return vec![];
    }

    let stack = stack_detect::detect_project_stack(
        ctx.fs,
        &std::env::current_dir().unwrap_or_default(),
    );

    if stack.languages.is_empty() && stack.frameworks.is_empty() {
        ctx.terminal
            .stderr_write("Warning: No stack detected, installing all rules\n");
        return vec![];
    }

    let lang_list = stack.languages.join(", ");
    ctx.terminal
        .stderr_write(&format!("Detected: [{lang_list}]\n"));

    let filter_result =
        rule_filter::filter_rules_by_stack(ctx.fs, rules_src, rule_groups, &stack);

    let skipped = filter_result.skipped.len();
    if skipped > 0 {
        ctx.terminal.stderr_write(&format!(
            "Skipped {skipped} rules (not matching detected stack)\n"
        ));
    }

    filter_result.skipped
}

pub(super) fn step_hooks_and_settings(
    ctx: &InstallContext,
    ecc_root: &Path,
    claude_dir: &Path,
    version: &str,
    options: &InstallOptions,
) -> Result<(usize, usize, usize), String> {
    tracing::info!("install: merging hooks and settings");
    let hooks_json = ecc_root.join("hooks").join("hooks.json");
    let settings_json = claude_dir.join("settings.json");
    let (hooks_added, hooks_existing, hooks_legacy) = if ctx.fs.exists(&hooks_json) {
        match merge::merge_hooks(ctx.fs, &hooks_json, &settings_json, options.dry_run) {
            Ok(counts) => counts,
            Err(e) => {
                return Err(format!("Hook merge error: {e}"));
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

    Ok((hooks_added, hooks_existing, hooks_legacy))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn step_write_manifest(
    ctx: &InstallContext,
    claude_dir: &Path,
    version: &str,
    now: &str,
    options: &InstallOptions,
    existing_manifest: &Option<ecc_domain::config::manifest::EccManifest>,
    combined: &mut domain_merge::MergeReport,
) {
    tracing::info!("install: writing install manifest");
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
            tracing::error!("Failed to write manifest: {}", e);
            combined
                .errors
                .push(format!("Failed to write manifest: {e}"));
        }
    }
}
