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

#[allow(clippy::vec_init_then_push)]
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

    all_reports.push(merge::merge_directory(
        &merge_ctx,
        &ecc_root.join("teams"),
        &claude_dir.join("teams"),
        "Teams",
        ".md",
        &mut merge_options,
    ));

    all_reports.push(merge::merge_skills(
        &merge_ctx,
        &ecc_root.join("skills"),
        &claude_dir.join("skills"),
        &mut merge_options,
    ));

    all_reports.push(merge::merge_patterns(
        &merge_ctx,
        &ecc_root.join("patterns"),
        &claude_dir.join("patterns"),
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

    // Expand tracking: todowrite frontmatter in installed agents
    expand_agents_tracking(ctx.fs, ecc_root, &claude_dir.join("agents"));

    domain_merge::combine_reports(&all_reports)
}

/// Expand `tracking: todowrite` frontmatter in all agent files at `dest_dir`.
///
/// Reads the canonical template from `ecc_root/agents/.templates/todowrite-block.md`
/// and inserts it before the first `TodoWrite items:` line (or at end) in each
/// agent file that has `tracking: todowrite` in its YAML frontmatter.
fn expand_agents_tracking(fs: &dyn ecc_ports::fs::FileSystem, ecc_root: &Path, dest_dir: &Path) {
    let template_path = ecc_root
        .join("agents")
        .join(".templates")
        .join("todowrite-block.md");
    let Ok(template) = fs.read_to_string(&template_path) else {
        tracing::debug!("todowrite template not found, skipping expansion");
        return;
    };
    let template = template.trim();
    if template.is_empty() {
        return;
    }

    let Ok(entries) = fs.read_dir(dest_dir) else {
        return;
    };
    for entry in entries {
        if entry.extension().and_then(|e| e.to_str()) == Some("md") {
            expand_tracking_field(fs, &entry, template);
        }
    }
}

/// Expand a single agent file's `tracking: todowrite` into the canonical block.
fn expand_tracking_field(fs: &dyn ecc_ports::fs::FileSystem, agent_path: &Path, template: &str) {
    let Ok(content) = fs.read_to_string(agent_path) else {
        return;
    };

    // Check frontmatter for tracking: todowrite
    if !has_tracking_todowrite(&content) {
        return;
    }

    // Idempotency: skip if template already present
    if content.contains(template) {
        return;
    }

    // Find insertion point: before first "TodoWrite items:" line
    let lines: Vec<&str> = content.lines().collect();
    let insert_idx = lines
        .iter()
        .position(|l| l.trim().starts_with("TodoWrite items:"));

    let new_content = if let Some(idx) = insert_idx {
        let mut result = lines[..idx].join("\n");
        result.push('\n');
        result.push_str(template);
        result.push_str("\n\n");
        result.push_str(&lines[idx..].join("\n"));
        result.push('\n');
        result
    } else {
        let mut result = content.clone();
        if !result.ends_with('\n') {
            result.push('\n');
        }
        result.push('\n');
        result.push_str(template);
        result.push('\n');
        result
    };

    if let Err(e) = fs.write(agent_path, &new_content) {
        tracing::warn!(path = %agent_path.display(), error = %e, "failed to write expanded agent tracking field");
    }
}

/// Check if content has `tracking: todowrite` in YAML frontmatter.
fn has_tracking_todowrite(content: &str) -> bool {
    let Some(fm_end) = content.strip_prefix("---\n").and_then(|rest| {
        rest.find("\n---").map(|i| i + 4 + 4) // +4 for "---\n" prefix, +4 for "\n---"
    }) else {
        return false;
    };
    let frontmatter = &content[..fm_end];
    frontmatter
        .lines()
        .any(|l| l.trim() == "tracking: todowrite")
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

    let stack =
        stack_detect::detect_project_stack(ctx.fs, &std::env::current_dir().unwrap_or_default());

    if stack.languages.is_empty() && stack.frameworks.is_empty() {
        ctx.terminal
            .stderr_write("Warning: No stack detected, installing all rules\n");
        return vec![];
    }

    let lang_list = stack.languages.join(", ");
    ctx.terminal
        .stderr_write(&format!("Detected: [{lang_list}]\n"));

    let filter_result = rule_filter::filter_rules_by_stack(ctx.fs, rules_src, rule_groups, &stack);

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

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_ports::fs::FileSystem;
    use ecc_test_support::InMemoryFileSystem;

    const TEMPLATE: &str = "> **Tracking**: Create a TodoWrite checklist for this workflow. If TodoWrite is unavailable, proceed without tracking — the workflow executes identically.\n\nMark each item complete as the step finishes.";

    // PC-001: expand inserts block before TodoWrite items line
    #[test]
    fn expand_tracking_inserts_before_items() {
        let fs = InMemoryFileSystem::new();
        let agent = "---\nname: test\ntracking: todowrite\n---\n\nSome intro.\n\nTodoWrite items:\n- \"Step 1\"\n- \"Step 2\"\n";
        fs.write(std::path::Path::new("/agent.md"), agent).unwrap();

        expand_tracking_field(&fs, std::path::Path::new("/agent.md"), TEMPLATE);

        let result = fs
            .read_to_string(std::path::Path::new("/agent.md"))
            .unwrap();
        assert!(result.contains(TEMPLATE), "template should be inserted");
        let template_pos = result.find(TEMPLATE).unwrap();
        let items_pos = result.find("TodoWrite items:").unwrap();
        assert!(template_pos < items_pos, "template must come before items");
    }

    // PC-002: expand appends at end when no items line
    #[test]
    fn expand_tracking_appends_at_end() {
        let fs = InMemoryFileSystem::new();
        let agent = "---\nname: test\ntracking: todowrite\n---\n\nSome content without items.\n";
        fs.write(std::path::Path::new("/agent.md"), agent).unwrap();

        expand_tracking_field(&fs, std::path::Path::new("/agent.md"), TEMPLATE);

        let result = fs
            .read_to_string(std::path::Path::new("/agent.md"))
            .unwrap();
        assert!(result.contains(TEMPLATE), "template should be appended");
        assert!(
            result.trim_end().ends_with("step finishes."),
            "should end with template"
        );
    }

    // PC-003: expand no-ops without tracking frontmatter
    #[test]
    fn expand_tracking_noop_no_frontmatter() {
        let fs = InMemoryFileSystem::new();
        let agent = "---\nname: test\n---\n\nNo tracking field.\n";
        fs.write(std::path::Path::new("/agent.md"), agent).unwrap();

        expand_tracking_field(&fs, std::path::Path::new("/agent.md"), TEMPLATE);

        let result = fs
            .read_to_string(std::path::Path::new("/agent.md"))
            .unwrap();
        assert_eq!(result, agent, "content should be unchanged");
    }

    // PC-004: expand no-ops when template missing
    #[test]
    fn expand_tracking_noop_missing_template() {
        let fs = InMemoryFileSystem::new();
        let agent = "---\nname: test\ntracking: todowrite\n---\n\nContent.\n";
        fs.write(std::path::Path::new("/dest/agent.md"), agent)
            .unwrap();

        expand_agents_tracking(
            &fs,
            std::path::Path::new("/ecc_root"),
            std::path::Path::new("/dest"),
        );

        let result = fs
            .read_to_string(std::path::Path::new("/dest/agent.md"))
            .unwrap();
        assert_eq!(
            result, agent,
            "content should be unchanged when template missing"
        );
    }

    // PC-005: expand is idempotent
    #[test]
    fn expand_tracking_idempotent() {
        let fs = InMemoryFileSystem::new();
        let agent = "---\nname: test\ntracking: todowrite\n---\n\nSome content.\n";
        fs.write(std::path::Path::new("/agent.md"), agent).unwrap();

        expand_tracking_field(&fs, std::path::Path::new("/agent.md"), TEMPLATE);
        let after_first = fs
            .read_to_string(std::path::Path::new("/agent.md"))
            .unwrap();

        expand_tracking_field(&fs, std::path::Path::new("/agent.md"), TEMPLATE);
        let after_second = fs
            .read_to_string(std::path::Path::new("/agent.md"))
            .unwrap();

        assert_eq!(
            after_first, after_second,
            "second expansion should be no-op"
        );
    }
}
