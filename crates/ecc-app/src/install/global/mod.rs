//! Global install orchestration — full ECC installation flow.

mod steps;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_edge;

use super::helpers::print_summary;
use super::{InstallContext, InstallOptions, InstallSummary};
use crate::config::manifest::read_manifest;
use ecc_domain::ansi;
use std::path::Path;

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

    steps::step_clean(ctx, claude_dir, options);
    steps::step_detect(ctx, claude_dir);
    let existing_manifest = read_manifest(ctx.fs, claude_dir);
    let mut combined = steps::step_merge_artifacts(ctx, ecc_root, claude_dir, options);
    if let Err(e) = steps::step_hooks_and_settings(ctx, ecc_root, claude_dir, version, options) {
        combined.errors.push(e);
    }
    steps::step_write_manifest(
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
