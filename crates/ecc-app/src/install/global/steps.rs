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

    // NOTE: if a third agent-frontmatter transformation is added, extract a Transformer trait (BL-150)
    // Expand tool-set references: replace `tool-set: X` with `tools: [A, B, C]` inline
    expand_agents_tool_sets(ctx.fs, &claude_dir.join("agents"), ecc_root);

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

/// Expand `tool-set: <preset>` frontmatter in all agent files at `dest_dir`.
///
/// For each `.md` file in `dest_dir`, if the frontmatter has a `tool-set:` field,
/// resolve the preset via the tool manifest at `ecc_root/manifest/tool-manifest.yaml`,
/// then rewrite the frontmatter replacing `tool-set: X` with `tools: [A, B, C]`.
/// Uses atomic write (write to temp path, then rename). Skips symlinks.
fn expand_agents_tool_sets(fs: &dyn ecc_ports::fs::FileSystem, dest_dir: &Path, ecc_root: &Path) {
    use ecc_domain::config::tool_manifest::ToolManifest;

    // Load the manifest; if missing or invalid, skip all expansion
    let manifest_path = ecc_root.join("manifest").join("tool-manifest.yaml");
    let manifest: ToolManifest = match fs.read_to_string(&manifest_path) {
        Ok(content) => match ecc_domain::config::tool_manifest::parse_tool_manifest(&content) {
            Ok(m) => m,
            Err(e) => {
                tracing::debug!(error = %e, "tool manifest parse failed, skipping tool-set expansion");
                return;
            }
        },
        Err(_) => {
            tracing::debug!("tool manifest not found, skipping tool-set expansion");
            return;
        }
    };

    let Ok(entries) = fs.read_dir(dest_dir) else {
        return;
    };

    for entry in entries {
        if entry.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        // Skip symlinks
        if fs.is_symlink(&entry) {
            tracing::debug!(path = %entry.display(), "skipping symlink during tool-set expansion");
            continue;
        }
        expand_tool_set_field(fs, &entry, &manifest);
    }
}

/// Expand a single agent file's `tool-set: <preset>` into `tools: [A, B, C]`.
///
/// Writes atomically: first to a temp path, then renames over the original.
fn expand_tool_set_field(
    fs: &dyn ecc_ports::fs::FileSystem,
    agent_path: &Path,
    manifest: &ecc_domain::config::tool_manifest::ToolManifest,
) {
    let Ok(content) = fs.read_to_string(agent_path) else {
        return;
    };

    // Extract preset name from frontmatter
    let preset_name = match extract_tool_set_from_frontmatter(&content) {
        Some(name) => name,
        None => return, // no tool-set: field, nothing to do
    };

    // Resolve preset tools from manifest
    let tools = match manifest.presets.get(&preset_name) {
        Some(t) => t.clone(),
        None => {
            tracing::warn!(
                path = %agent_path.display(),
                preset = %preset_name,
                "tool-set preset not found in manifest, skipping expansion"
            );
            return;
        }
    };

    // Build the inline tools string: `tools: [A, B, C]`
    let tools_str = format!("tools: [{}]", tools.join(", "));

    // Replace the `tool-set: <name>` line with `tools: [...]`
    let new_content = replace_tool_set_with_tools(&content, &tools_str);
    if new_content == content {
        // Nothing changed (shouldn't happen if extract succeeded, but be safe)
        return;
    }

    // Atomic write: write to temp path, then rename
    let temp_path = agent_path.with_extension("md.tmp");
    if let Err(e) = fs.write(&temp_path, &new_content) {
        tracing::warn!(path = %agent_path.display(), error = %e, "failed to write temp file for tool-set expansion");
        return;
    }
    if let Err(e) = fs.rename(&temp_path, agent_path) {
        tracing::warn!(path = %agent_path.display(), error = %e, "failed to rename temp file for tool-set expansion");
        // Attempt cleanup of temp file (best-effort)
        let _ = fs.remove_file(&temp_path);
    }
}

/// Extract the `tool-set:` value from YAML frontmatter, if present.
fn extract_tool_set_from_frontmatter(content: &str) -> Option<String> {
    let rest = content.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    let frontmatter = &rest[..end];
    for line in frontmatter.lines() {
        let trimmed = line.trim();
        if let Some(val) = trimmed.strip_prefix("tool-set:") {
            let name = val.trim().trim_matches('"').trim_matches('\'').to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    None
}

/// Replace `tool-set: <preset>` line in frontmatter with `tools: [...]`.
fn replace_tool_set_with_tools(content: &str, tools_str: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut in_frontmatter = false;
    let mut frontmatter_done = false;
    let mut separator_count = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        if !frontmatter_done && trimmed == "---" {
            separator_count += 1;
            if separator_count == 1 {
                in_frontmatter = true;
                result.push_str(line);
                result.push('\n');
                continue;
            } else if separator_count == 2 {
                in_frontmatter = false;
                frontmatter_done = true;
                result.push_str(line);
                result.push('\n');
                continue;
            }
        }

        if in_frontmatter && trimmed.starts_with("tool-set:") {
            result.push_str(tools_str);
            result.push('\n');
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    // Preserve trailing newline if original had one (lines() strips it)
    // The loop above always adds '\n' after each line, so content ends with '\n'
    result
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

    // Helper: write a minimal tool manifest with readonly-analyzer preset
    fn with_tool_manifest(fs: &InMemoryFileSystem, ecc_root: &std::path::Path) {
        let yaml = "tools:\n  - Read\n  - Grep\n  - Glob\npresets:\n  readonly-analyzer:\n    - Read\n    - Grep\n    - Glob\n";
        fs.write(&ecc_root.join("manifest/tool-manifest.yaml"), yaml)
            .unwrap();
    }

    // PC-033: expand_tool_sets_inlines_spec_adversary
    // Install pipeline writes expanded inline `tools: [Read, Grep, Glob]` in
    // installed spec-adversary.md (no `tool-set:`).
    #[test]
    fn expand_tool_sets_inlines_spec_adversary() {
        let fs = InMemoryFileSystem::new();
        let ecc_root = std::path::Path::new("/ecc");
        let dest_dir = std::path::Path::new("/dest");

        with_tool_manifest(&fs, ecc_root);

        // Write spec-adversary.md with tool-set: readonly-analyzer (source form)
        let source_content = "---\nname: spec-adversary\nmodel: opus\ntool-set: readonly-analyzer\n---\n# Spec Adversary\n";
        fs.write(&dest_dir.join("spec-adversary.md"), source_content)
            .unwrap();

        expand_agents_tool_sets(&fs, dest_dir, ecc_root);

        let result = fs
            .read_to_string(&dest_dir.join("spec-adversary.md"))
            .unwrap();

        // Must contain expanded tools list
        assert!(
            result.contains("tools:"),
            "installed file must have tools: field; got:\n{result}"
        );
        assert!(
            result.contains("Read") && result.contains("Grep") && result.contains("Glob"),
            "installed file must have expanded tools; got:\n{result}"
        );
        // Must NOT contain tool-set:
        assert!(
            !result.contains("tool-set:"),
            "installed file must not contain tool-set:; got:\n{result}"
        );
    }

    // PC-035: pre_post_effective_tools_byte_identical
    // After expansion, the effective tools list is byte-identical to the preset contents.
    #[test]
    fn pre_post_effective_tools_byte_identical() {
        let fs = InMemoryFileSystem::new();
        let ecc_root = std::path::Path::new("/ecc");
        let dest_dir = std::path::Path::new("/dest");

        with_tool_manifest(&fs, ecc_root);

        let source_content =
            "---\nname: test-agent\nmodel: sonnet\ntool-set: readonly-analyzer\n---\n# Agent\n";
        fs.write(&dest_dir.join("test-agent.md"), source_content)
            .unwrap();

        expand_agents_tool_sets(&fs, dest_dir, ecc_root);

        let result = fs.read_to_string(&dest_dir.join("test-agent.md")).unwrap();

        // The inline tools must exactly match the preset's members (Read, Grep, Glob)
        let expected_tools = ["Read", "Grep", "Glob"];
        for tool in &expected_tools {
            assert!(
                result.contains(tool),
                "expanded output must contain tool '{tool}'; got:\n{result}"
            );
        }
        // No extra tools beyond the preset
        assert!(
            !result.contains("Write") && !result.contains("Edit") && !result.contains("Bash"),
            "expanded output must not contain tools outside the preset; got:\n{result}"
        );
    }

    // PC-073: write_atomic_and_rejects_symlinks
    // Atomic write: write to temp, then rename. Symlink paths are skipped.
    #[test]
    fn write_atomic_and_rejects_symlinks() {
        let fs = InMemoryFileSystem::new();
        let ecc_root = std::path::Path::new("/ecc");
        let dest_dir = std::path::Path::new("/dest");

        with_tool_manifest(&fs, ecc_root);

        // Write a real agent file that will be expanded
        let real_content =
            "---\nname: real-agent\nmodel: sonnet\ntool-set: readonly-analyzer\n---\n# Agent\n";
        fs.write(&dest_dir.join("real-agent.md"), real_content)
            .unwrap();

        // Create a symlink file — expansion should skip it
        fs.create_symlink(
            std::path::Path::new("/some/other/target.md"),
            &dest_dir.join("symlink-agent.md"),
        )
        .unwrap();

        // Run expansion — should not panic or error on symlink
        expand_agents_tool_sets(&fs, dest_dir, ecc_root);

        // The real file should be expanded
        let real_result = fs.read_to_string(&dest_dir.join("real-agent.md")).unwrap();
        assert!(
            !real_result.contains("tool-set:"),
            "real file should be expanded (no tool-set:); got:\n{real_result}"
        );
        assert!(
            real_result.contains("tools:"),
            "real file should have tools:; got:\n{real_result}"
        );
    }
}
