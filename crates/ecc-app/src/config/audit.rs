use ecc_domain::config::audit::{
    compute_audit_score, exists_in_settings, exists_in_source, is_ecc_managed_hook,
    parse_frontmatter, ArtifactAudit, AuditCheckResult, AuditFinding, AuditReport,
    ConfigAudit, HookDiffEntry, HooksDiff, Severity,
};
use ecc_ports::fs::FileSystem;
use std::path::Path;

/// Read a JSON file, returning None on any error.
fn read_json_safe(
    fs: &dyn FileSystem,
    path: &Path,
) -> Option<serde_json::Value> {
    let content = fs.read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Check that ECC deny rules are present in settings.json.
pub fn check_deny_rules(
    fs: &dyn FileSystem,
    settings_path: &Path,
) -> AuditCheckResult {
    let mut findings = Vec::new();

    let settings = match read_json_safe(fs, settings_path) {
        Some(s) => s,
        None => {
            findings.push(AuditFinding {
                id: "DENY-001".into(),
                severity: Severity::Critical,
                title: "No settings.json found".into(),
                detail: format!("Expected settings at {}", settings_path.display()),
                fix: "Run `ecc install` to create settings with deny rules.".into(),
            });
            return AuditCheckResult {
                name: "Deny rules".into(),
                passed: false,
                findings,
            };
        }
    };

    let deny: Vec<&str> = settings
        .get("permissions")
        .and_then(|p| p.get("deny"))
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .collect()
        })
        .unwrap_or_default();

    let deny_set: std::collections::HashSet<&str> =
        deny.into_iter().collect();

    let missing: Vec<&&str> = ecc_domain::config::deny_rules::ECC_DENY_RULES
        .iter()
        .filter(|rule| !deny_set.contains(**rule))
        .collect();

    if !missing.is_empty() {
        let preview: Vec<&str> =
            missing.iter().take(3).map(|r| **r).collect();
        let suffix = if missing.len() > 3 {
            format!(" (and {} more)", missing.len() - 3)
        } else {
            String::new()
        };

        findings.push(AuditFinding {
            id: "DENY-002".into(),
            severity: Severity::Critical,
            title: format!("{} deny rule(s) missing", missing.len()),
            detail: format!("Missing: {}{suffix}", preview.join(", ")),
            fix: "Run `ecc install` to add deny rules, or add them manually to ~/.claude/settings.json.".into(),
        });
    }

    AuditCheckResult {
        name: "Deny rules".into(),
        passed: findings.is_empty(),
        findings,
    }
}

/// Check that ECC gitignore entries are present in .gitignore.
pub fn check_gitignore(
    fs: &dyn FileSystem,
    project_dir: &Path,
) -> AuditCheckResult {
    let mut findings = Vec::new();
    let gitignore_path = project_dir.join(".gitignore");

    if !fs.exists(&gitignore_path) {
        findings.push(AuditFinding {
            id: "GIT-001".into(),
            severity: Severity::Medium,
            title: "No .gitignore file found".into(),
            detail: "Project has no .gitignore — local configs may be committed accidentally."
                .into(),
            fix: "Run `ecc init` to create .gitignore with ECC entries.".into(),
        });
        return AuditCheckResult {
            name: "Gitignore".into(),
            passed: false,
            findings,
        };
    }

    let content = fs.read_to_string(&gitignore_path).unwrap_or_default();

    let patterns: std::collections::HashSet<String> = content
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();

    let missing: Vec<&str> = ecc_domain::config::gitignore::ECC_GITIGNORE_ENTRIES
        .iter()
        .filter(|e| !patterns.contains(e.pattern))
        .map(|e| e.pattern)
        .collect();

    if !missing.is_empty() {
        findings.push(AuditFinding {
            id: "GIT-002".into(),
            severity: Severity::High,
            title: format!("{} gitignore entry/ies missing", missing.len()),
            detail: format!("Missing: {}", missing.join(", ")),
            fix: "Run `ecc init` to add missing entries.".into(),
        });
    }

    AuditCheckResult {
        name: "Gitignore".into(),
        passed: findings.is_empty(),
        findings,
    }
}

/// Check for duplicate hooks in settings.json.
pub fn check_hook_duplicates(
    fs: &dyn FileSystem,
    settings_path: &Path,
) -> AuditCheckResult {
    let mut findings = Vec::new();

    let settings = match read_json_safe(fs, settings_path) {
        Some(s) => s,
        None => {
            return AuditCheckResult {
                name: "Hook duplicates".into(),
                passed: true,
                findings,
            };
        }
    };

    let hooks_obj = match settings.get("hooks").and_then(|h| h.as_object()) {
        Some(h) => h,
        None => {
            return AuditCheckResult {
                name: "Hook duplicates".into(),
                passed: true,
                findings,
            };
        }
    };

    let mut total_duplicates = 0usize;

    for entries in hooks_obj.values() {
        let arr = match entries.as_array() {
            Some(a) => a,
            None => continue,
        };
        let mut seen = std::collections::HashSet::new();
        for entry in arr {
            let key = match entry.get("hooks") {
                Some(h) => serde_json::to_string(h).unwrap_or_default(),
                None => serde_json::to_string(entry).unwrap_or_default(),
            };
            if !seen.insert(key) {
                total_duplicates += 1;
            }
        }
    }

    if total_duplicates > 0 {
        findings.push(AuditFinding {
            id: "HOOK-001".into(),
            severity: Severity::High,
            title: format!("{total_duplicates} duplicate hook(s) found"),
            detail: "Duplicate hooks fire multiple times per event, wasting resources."
                .into(),
            fix: "Run `ecc install` to replace hooks section with the clean source."
                .into(),
        });
    }

    AuditCheckResult {
        name: "Hook duplicates".into(),
        passed: findings.is_empty(),
        findings,
    }
}

/// Check if global CLAUDE.md exists.
pub fn check_global_claude_md(
    fs: &dyn FileSystem,
    claude_dir: &Path,
) -> AuditCheckResult {
    let mut findings = Vec::new();
    let claude_md_path = claude_dir.join("CLAUDE.md");

    if !fs.exists(&claude_md_path) {
        findings.push(AuditFinding {
            id: "CMD-001".into(),
            severity: Severity::Medium,
            title: "No global ~/.claude/CLAUDE.md".into(),
            detail:
                "Critical cross-project instructions only load when rules match file paths."
                    .into(),
            fix: "Create ~/.claude/CLAUDE.md with a 50-80 line summary of key rules."
                .into(),
        });
    }

    AuditCheckResult {
        name: "Global CLAUDE.md".into(),
        passed: findings.is_empty(),
        findings,
    }
}

/// Check if agents have `skills:` frontmatter.
pub fn check_agent_skills(
    fs: &dyn FileSystem,
    agents_dir: &Path,
) -> AuditCheckResult {
    let mut findings = Vec::new();

    if !fs.exists(agents_dir) {
        return AuditCheckResult {
            name: "Agent skills".into(),
            passed: true,
            findings,
        };
    }

    let entries = match fs.read_dir(agents_dir) {
        Ok(e) => e,
        Err(_) => {
            return AuditCheckResult {
                name: "Agent skills".into(),
                passed: true,
                findings,
            };
        }
    };

    let agents: Vec<_> = entries
        .iter()
        .filter(|p| {
            p.extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        })
        .collect();

    let mut with_skills = 0usize;
    let mut without_skills = 0usize;

    for agent_path in &agents {
        if let Ok(content) = fs.read_to_string(agent_path) {
            let fm = parse_frontmatter(&content);
            if fm.contains_key("skills") {
                with_skills += 1;
            } else {
                without_skills += 1;
            }
        }
    }

    if without_skills > 0 && agents.len() > 5 {
        let ratio = (with_skills * 100) / agents.len();
        if ratio < 50 {
            findings.push(AuditFinding {
                id: "AGT-001".into(),
                severity: Severity::Low,
                title: format!(
                    "Only {with_skills}/{} agents use skills: preloading",
                    agents.len()
                ),
                detail: "Agents without skills: must discover skills at runtime — slower and less reliable.".into(),
                fix: "Add skills: frontmatter to agents that reference specific skills.".into(),
            });
        }
    }

    AuditCheckResult {
        name: "Agent skills".into(),
        passed: findings.is_empty(),
        findings,
    }
}

/// Check if commands have `description:` frontmatter.
pub fn check_command_descriptions(
    fs: &dyn FileSystem,
    commands_dir: &Path,
) -> AuditCheckResult {
    let mut findings = Vec::new();

    if !fs.exists(commands_dir) {
        return AuditCheckResult {
            name: "Command descriptions".into(),
            passed: true,
            findings,
        };
    }

    let entries = match fs.read_dir(commands_dir) {
        Ok(e) => e,
        Err(_) => {
            return AuditCheckResult {
                name: "Command descriptions".into(),
                passed: true,
                findings,
            };
        }
    };

    let commands: Vec<_> = entries
        .iter()
        .filter(|p| {
            p.extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
                && !p
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with('_'))
        })
        .collect();

    let mut missing_desc = Vec::new();

    for cmd_path in &commands {
        if let Ok(content) = fs.read_to_string(cmd_path) {
            let fm = parse_frontmatter(&content);
            if !fm.contains_key("description")
                && let Some(name) = cmd_path.file_name()
            {
                missing_desc.push(name.to_string_lossy().into_owned());
            }
        }
    }

    if !missing_desc.is_empty() {
        findings.push(AuditFinding {
            id: "CMD-002".into(),
            severity: Severity::Low,
            title: format!(
                "{} command(s) missing description frontmatter",
                missing_desc.len()
            ),
            detail: format!("Missing: {}", missing_desc.join(", ")),
            fix: "Add description: field to YAML frontmatter in each command file."
                .into(),
        });
    }

    AuditCheckResult {
        name: "Command descriptions".into(),
        passed: findings.is_empty(),
        findings,
    }
}

/// Check project CLAUDE.md line count.
pub fn check_project_claude_md(
    fs: &dyn FileSystem,
    project_dir: &Path,
) -> AuditCheckResult {
    let mut findings = Vec::new();
    let claude_md_path = project_dir.join("CLAUDE.md");

    if !fs.exists(&claude_md_path) {
        return AuditCheckResult {
            name: "Project CLAUDE.md".into(),
            passed: true,
            findings,
        };
    }

    let content = match fs.read_to_string(&claude_md_path) {
        Ok(c) => c,
        Err(_) => {
            return AuditCheckResult {
                name: "Project CLAUDE.md".into(),
                passed: true,
                findings,
            };
        }
    };

    let lines = content.lines().count();

    if lines > 200 {
        findings.push(AuditFinding {
            id: "PCM-001".into(),
            severity: Severity::Medium,
            title: format!("CLAUDE.md is {lines} lines (recommended < 200)"),
            detail:
                "Large CLAUDE.md files consume context budget on every conversation."
                    .into(),
            fix: "Move detailed instructions to rules/ or skills/ and keep CLAUDE.md lean."
                .into(),
        });
    }

    AuditCheckResult {
        name: "Project CLAUDE.md".into(),
        passed: findings.is_empty(),
        findings,
    }
}

/// Read hooks from a settings.json file.
fn read_hooks_from_settings(
    fs: &dyn FileSystem,
    settings_path: &Path,
) -> serde_json::Value {
    read_json_safe(fs, settings_path)
        .and_then(|s| s.get("hooks").cloned())
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()))
}

/// Read hooks from a hooks.json source file.
fn read_hooks_from_source(
    fs: &dyn FileSystem,
    hooks_json_path: &Path,
) -> serde_json::Value {
    read_json_safe(fs, hooks_json_path)
        .and_then(|s| s.get("hooks").cloned())
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()))
}

/// Compare hooks in settings.json against the source hooks.json.
pub fn diff_hooks(
    fs: &dyn FileSystem,
    settings_path: &Path,
    hooks_json_path: &Path,
) -> HooksDiff {
    let settings_hooks = read_hooks_from_settings(fs, settings_path);
    let source_hooks = read_hooks_from_source(fs, hooks_json_path);

    let mut stale = Vec::new();
    let mut matching = Vec::new();
    let mut user_hooks = Vec::new();

    if let Some(settings_obj) = settings_hooks.as_object() {
        for (event, entries) in settings_obj {
            let arr = match entries.as_array() {
                Some(a) => a,
                None => continue,
            };
            for entry in arr {
                if is_ecc_managed_hook(entry, &source_hooks) {
                    if exists_in_source(event, entry, &source_hooks) {
                        matching.push(HookDiffEntry {
                            event: event.clone(),
                            entry: entry.clone(),
                        });
                    } else {
                        stale.push(HookDiffEntry {
                            event: event.clone(),
                            entry: entry.clone(),
                        });
                    }
                } else {
                    user_hooks.push(HookDiffEntry {
                        event: event.clone(),
                        entry: entry.clone(),
                    });
                }
            }
        }
    }

    let mut missing = Vec::new();
    if let Some(source_obj) = source_hooks.as_object() {
        for (event, entries) in source_obj {
            let arr = match entries.as_array() {
                Some(a) => a,
                None => continue,
            };
            for entry in arr {
                if !exists_in_settings(event, entry, &settings_hooks) {
                    missing.push(HookDiffEntry {
                        event: event.clone(),
                        entry: entry.clone(),
                    });
                }
            }
        }
    }

    HooksDiff {
        stale,
        missing,
        matching,
        user_hooks,
    }
}

/// Compare files in a source directory against an installed directory.
pub fn audit_artifact_dir(
    fs: &dyn FileSystem,
    src_dir: &Path,
    dest_dir: &Path,
    ext: &str,
) -> ArtifactAudit {
    let mut matching = Vec::new();
    let mut outdated = Vec::new();
    let mut missing = Vec::new();

    if !fs.exists(src_dir) {
        return ArtifactAudit {
            matching,
            outdated,
            missing,
        };
    }

    let entries = match fs.read_dir(src_dir) {
        Ok(e) => e,
        Err(_) => {
            return ArtifactAudit {
                matching,
                outdated,
                missing,
            };
        }
    };

    let src_files: Vec<String> = entries
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

    for filename in src_files {
        let src_path = src_dir.join(&filename);
        let dest_path = dest_dir.join(&filename);

        if !fs.exists(&dest_path) {
            missing.push(filename);
        } else {
            let src_content = fs
                .read_to_string(&src_path)
                .unwrap_or_default();
            let dest_content = fs
                .read_to_string(&dest_path)
                .unwrap_or_default();
            if src_content.trim() != dest_content.trim() {
                outdated.push(filename);
            } else {
                matching.push(filename);
            }
        }
    }

    ArtifactAudit {
        matching,
        outdated,
        missing,
    }
}

/// Full ECC config audit comparing installed artifacts against source.
pub fn audit_ecc_config(
    fs: &dyn FileSystem,
    ecc_root: &Path,
    claude_dir: &Path,
) -> ConfigAudit {
    let agents = audit_artifact_dir(
        fs,
        &ecc_root.join("agents"),
        &claude_dir.join("agents"),
        ".md",
    );

    let commands = audit_artifact_dir(
        fs,
        &ecc_root.join("commands"),
        &claude_dir.join("commands"),
        ".md",
    );

    let hooks_json_path = ecc_root.join("hooks").join("hooks.json");
    let settings_json_path = claude_dir.join("settings.json");
    let hooks = diff_hooks(fs, &settings_json_path, &hooks_json_path);

    let has_differences = !agents.outdated.is_empty()
        || !agents.missing.is_empty()
        || !commands.outdated.is_empty()
        || !commands.missing.is_empty()
        || !hooks.stale.is_empty()
        || !hooks.missing.is_empty();

    ConfigAudit {
        agents,
        commands,
        hooks,
        has_differences,
    }
}

/// Run all audit checks and compute a score and grade.
pub fn run_all_checks(
    fs: &dyn FileSystem,
    claude_dir: &Path,
    project_dir: &Path,
    ecc_root: &Path,
) -> AuditReport {
    let settings_path = claude_dir.join("settings.json");
    let agents_dir = ecc_root.join("agents");
    let commands_dir = ecc_root.join("commands");

    let checks = vec![
        check_deny_rules(fs, &settings_path),
        check_gitignore(fs, project_dir),
        check_hook_duplicates(fs, &settings_path),
        check_global_claude_md(fs, claude_dir),
        check_agent_skills(fs, &agents_dir),
        check_command_descriptions(fs, &commands_dir),
        check_project_claude_md(fs, project_dir),
    ];

    let (score, grade) = compute_audit_score(&checks);

    AuditReport {
        checks,
        score,
        grade,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::InMemoryFileSystem;
    use std::path::Path;

    // --- check_deny_rules ---

    #[test]
    fn check_deny_rules_all_present() {
        let deny_array: Vec<serde_json::Value> =
            ecc_domain::config::deny_rules::ECC_DENY_RULES
                .iter()
                .map(|r| serde_json::Value::String(r.to_string()))
                .collect();
        let settings = serde_json::json!({
            "permissions": { "deny": deny_array }
        });
        let fs = InMemoryFileSystem::new()
            .with_file("/settings.json", &settings.to_string());

        let result = check_deny_rules(&fs, Path::new("/settings.json"));
        assert!(result.passed);
        assert!(result.findings.is_empty());
    }

    #[test]
    fn check_deny_rules_some_missing() {
        let settings = serde_json::json!({
            "permissions": { "deny": ["Read(//**/.env)"] }
        });
        let fs = InMemoryFileSystem::new()
            .with_file("/settings.json", &settings.to_string());

        let result = check_deny_rules(&fs, Path::new("/settings.json"));
        assert!(!result.passed);
        assert_eq!(result.findings.len(), 1);
        assert_eq!(result.findings[0].id, "DENY-002");
        assert_eq!(result.findings[0].severity, Severity::Critical);
    }

    #[test]
    fn check_deny_rules_no_settings() {
        let fs = InMemoryFileSystem::new();
        let result = check_deny_rules(&fs, Path::new("/settings.json"));
        assert!(!result.passed);
        assert_eq!(result.findings[0].id, "DENY-001");
    }

    #[test]
    fn check_deny_rules_no_permissions_key() {
        let settings = serde_json::json!({"hooks": {}});
        let fs = InMemoryFileSystem::new()
            .with_file("/settings.json", &settings.to_string());

        let result = check_deny_rules(&fs, Path::new("/settings.json"));
        assert!(!result.passed);
        assert_eq!(result.findings[0].id, "DENY-002");
    }

    // --- check_gitignore ---

    #[test]
    fn check_gitignore_all_present() {
        let content: String = ecc_domain::config::gitignore::ECC_GITIGNORE_ENTRIES
            .iter()
            .map(|e| e.pattern)
            .collect::<Vec<_>>()
            .join("\n");
        let fs = InMemoryFileSystem::new()
            .with_file("/project/.gitignore", &content);

        let result = check_gitignore(&fs, Path::new("/project"));
        assert!(result.passed);
    }

    #[test]
    fn check_gitignore_missing_entries() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/.gitignore", "node_modules\n");

        let result = check_gitignore(&fs, Path::new("/project"));
        assert!(!result.passed);
        assert_eq!(result.findings[0].id, "GIT-002");
        assert_eq!(result.findings[0].severity, Severity::High);
    }

    #[test]
    fn check_gitignore_no_file() {
        let fs = InMemoryFileSystem::new();
        let result = check_gitignore(&fs, Path::new("/project"));
        assert!(!result.passed);
        assert_eq!(result.findings[0].id, "GIT-001");
    }

    // --- check_hook_duplicates ---

    #[test]
    fn check_hook_duplicates_no_duplicates() {
        let settings = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {"hooks": [{"command": "ecc-hook a"}]},
                    {"hooks": [{"command": "ecc-hook b"}]}
                ]
            }
        });
        let fs = InMemoryFileSystem::new()
            .with_file("/settings.json", &settings.to_string());

        let result =
            check_hook_duplicates(&fs, Path::new("/settings.json"));
        assert!(result.passed);
    }

    #[test]
    fn check_hook_duplicates_with_duplicates() {
        let settings = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {"hooks": [{"command": "ecc-hook a"}]},
                    {"hooks": [{"command": "ecc-hook a"}]}
                ]
            }
        });
        let fs = InMemoryFileSystem::new()
            .with_file("/settings.json", &settings.to_string());

        let result =
            check_hook_duplicates(&fs, Path::new("/settings.json"));
        assert!(!result.passed);
        assert_eq!(result.findings[0].id, "HOOK-001");
    }

    #[test]
    fn check_hook_duplicates_no_settings() {
        let fs = InMemoryFileSystem::new();
        let result =
            check_hook_duplicates(&fs, Path::new("/settings.json"));
        assert!(result.passed);
    }

    #[test]
    fn check_hook_duplicates_no_hooks_key() {
        let settings = serde_json::json!({"permissions": {}});
        let fs = InMemoryFileSystem::new()
            .with_file("/settings.json", &settings.to_string());

        let result =
            check_hook_duplicates(&fs, Path::new("/settings.json"));
        assert!(result.passed);
    }

    // --- check_global_claude_md ---

    #[test]
    fn check_global_claude_md_exists() {
        let fs = InMemoryFileSystem::new()
            .with_file("/claude/CLAUDE.md", "# Global instructions");

        let result =
            check_global_claude_md(&fs, Path::new("/claude"));
        assert!(result.passed);
    }

    #[test]
    fn check_global_claude_md_missing() {
        let fs = InMemoryFileSystem::new();
        let result =
            check_global_claude_md(&fs, Path::new("/claude"));
        assert!(!result.passed);
        assert_eq!(result.findings[0].id, "CMD-001");
    }

    // --- check_agent_skills ---

    #[test]
    fn check_agent_skills_all_have_skills() {
        let fs = InMemoryFileSystem::new();
        // Need more than 5 agents to trigger the check
        let fs = (0..6).fold(fs, |fs, i| {
            fs.with_file(
                &format!("/agents/agent{i}.md"),
                "---\nskills: tdd-workflow\n---\n# Agent",
            )
        });

        let result = check_agent_skills(&fs, Path::new("/agents"));
        assert!(result.passed);
    }

    #[test]
    fn check_agent_skills_few_agents_no_finding() {
        // With 3 agents and none having skills, no finding (< 5 threshold)
        let fs = InMemoryFileSystem::new()
            .with_file("/agents/a.md", "---\nname: a\n---\n")
            .with_file("/agents/b.md", "---\nname: b\n---\n")
            .with_file("/agents/c.md", "---\nname: c\n---\n");

        let result = check_agent_skills(&fs, Path::new("/agents"));
        assert!(result.passed);
    }

    #[test]
    fn check_agent_skills_no_dir() {
        let fs = InMemoryFileSystem::new();
        let result = check_agent_skills(&fs, Path::new("/nonexistent"));
        assert!(result.passed);
    }

    // --- check_command_descriptions ---

    #[test]
    fn check_command_descriptions_all_have_desc() {
        let fs = InMemoryFileSystem::new()
            .with_file(
                "/commands/plan.md",
                "---\ndescription: Plan stuff\n---\n# Plan",
            )
            .with_file(
                "/commands/verify.md",
                "---\ndescription: Verify stuff\n---\n# Verify",
            );

        let result =
            check_command_descriptions(&fs, Path::new("/commands"));
        assert!(result.passed);
    }

    #[test]
    fn check_command_descriptions_missing_desc() {
        let fs = InMemoryFileSystem::new()
            .with_file(
                "/commands/plan.md",
                "---\ndescription: Plan stuff\n---\n# Plan",
            )
            .with_file("/commands/verify.md", "---\nname: verify\n---\n# Verify");

        let result =
            check_command_descriptions(&fs, Path::new("/commands"));
        assert!(!result.passed);
        assert_eq!(result.findings[0].id, "CMD-002");
    }

    #[test]
    fn check_command_descriptions_skips_underscore() {
        let fs = InMemoryFileSystem::new()
            .with_file("/commands/_archive.md", "# No frontmatter");

        let result =
            check_command_descriptions(&fs, Path::new("/commands"));
        assert!(result.passed);
    }

    // --- check_project_claude_md ---

    #[test]
    fn check_project_claude_md_small() {
        let content = "# Title\nSome content\n";
        let fs = InMemoryFileSystem::new()
            .with_file("/project/CLAUDE.md", content);

        let result =
            check_project_claude_md(&fs, Path::new("/project"));
        assert!(result.passed);
    }

    #[test]
    fn check_project_claude_md_large() {
        let content = (0..250)
            .map(|i| format!("Line {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let fs = InMemoryFileSystem::new()
            .with_file("/project/CLAUDE.md", &content);

        let result =
            check_project_claude_md(&fs, Path::new("/project"));
        assert!(!result.passed);
        assert_eq!(result.findings[0].id, "PCM-001");
    }

    #[test]
    fn check_project_claude_md_missing() {
        let fs = InMemoryFileSystem::new();
        let result =
            check_project_claude_md(&fs, Path::new("/project"));
        assert!(result.passed);
    }

    // --- audit_artifact_dir ---

    #[test]
    fn audit_artifact_dir_matching_files() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/a.md", "content a")
            .with_file("/dest/a.md", "content a");

        let result =
            audit_artifact_dir(&fs, Path::new("/src"), Path::new("/dest"), ".md");
        assert_eq!(result.matching, vec!["a.md"]);
        assert!(result.outdated.is_empty());
        assert!(result.missing.is_empty());
    }

    #[test]
    fn audit_artifact_dir_outdated_files() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/a.md", "new content")
            .with_file("/dest/a.md", "old content");

        let result =
            audit_artifact_dir(&fs, Path::new("/src"), Path::new("/dest"), ".md");
        assert!(result.matching.is_empty());
        assert_eq!(result.outdated, vec!["a.md"]);
    }

    #[test]
    fn audit_artifact_dir_missing_files() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/a.md", "content");

        let result =
            audit_artifact_dir(&fs, Path::new("/src"), Path::new("/dest"), ".md");
        assert!(result.matching.is_empty());
        assert_eq!(result.missing, vec!["a.md"]);
    }

    #[test]
    fn audit_artifact_dir_no_src() {
        let fs = InMemoryFileSystem::new();
        let result =
            audit_artifact_dir(&fs, Path::new("/src"), Path::new("/dest"), ".md");
        assert!(result.matching.is_empty());
        assert!(result.outdated.is_empty());
        assert!(result.missing.is_empty());
    }

    #[test]
    fn audit_artifact_dir_filters_by_ext() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/a.md", "content")
            .with_file("/src/b.txt", "content");

        let result =
            audit_artifact_dir(&fs, Path::new("/src"), Path::new("/dest"), ".md");
        assert_eq!(result.missing, vec!["a.md"]);
    }

    // --- diff_hooks ---

    #[test]
    fn diff_hooks_matching() {
        let settings = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {"hooks": [{"command": "ecc-hook format"}]}
                ]
            }
        });
        let source = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {"hooks": [{"command": "ecc-hook format"}]}
                ]
            }
        });
        let fs = InMemoryFileSystem::new()
            .with_file("/settings.json", &settings.to_string())
            .with_file("/hooks.json", &source.to_string());

        let diff = diff_hooks(
            &fs,
            Path::new("/settings.json"),
            Path::new("/hooks.json"),
        );
        assert_eq!(diff.matching.len(), 1);
        assert!(diff.stale.is_empty());
        assert!(diff.missing.is_empty());
    }

    #[test]
    fn diff_hooks_missing_in_settings() {
        let settings = serde_json::json!({"hooks": {}});
        let source = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {"hooks": [{"command": "ecc-hook format"}]}
                ]
            }
        });
        let fs = InMemoryFileSystem::new()
            .with_file("/settings.json", &settings.to_string())
            .with_file("/hooks.json", &source.to_string());

        let diff = diff_hooks(
            &fs,
            Path::new("/settings.json"),
            Path::new("/hooks.json"),
        );
        assert_eq!(diff.missing.len(), 1);
        assert!(diff.matching.is_empty());
    }

    #[test]
    fn diff_hooks_stale_in_settings() {
        let settings = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {"hooks": [{"command": "ecc-hook old-format"}]}
                ]
            }
        });
        let source = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {"hooks": [{"command": "ecc-hook new-format"}]}
                ]
            }
        });
        let fs = InMemoryFileSystem::new()
            .with_file("/settings.json", &settings.to_string())
            .with_file("/hooks.json", &source.to_string());

        let diff = diff_hooks(
            &fs,
            Path::new("/settings.json"),
            Path::new("/hooks.json"),
        );
        assert_eq!(diff.stale.len(), 1);
        assert_eq!(diff.missing.len(), 1);
    }

    #[test]
    fn diff_hooks_user_hooks_preserved() {
        let settings = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {"hooks": [{"command": "my-custom-hook"}]}
                ]
            }
        });
        let source = serde_json::json!({"hooks": {}});
        let fs = InMemoryFileSystem::new()
            .with_file("/settings.json", &settings.to_string())
            .with_file("/hooks.json", &source.to_string());

        let diff = diff_hooks(
            &fs,
            Path::new("/settings.json"),
            Path::new("/hooks.json"),
        );
        assert_eq!(diff.user_hooks.len(), 1);
    }

    // --- run_all_checks integration ---

    #[test]
    fn run_all_checks_clean_setup() {
        let deny_array: Vec<serde_json::Value> =
            ecc_domain::config::deny_rules::ECC_DENY_RULES
                .iter()
                .map(|r| serde_json::Value::String(r.to_string()))
                .collect();
        let settings = serde_json::json!({
            "permissions": { "deny": deny_array },
            "hooks": {}
        });
        let gitignore_content: String =
            ecc_domain::config::gitignore::ECC_GITIGNORE_ENTRIES
                .iter()
                .map(|e| e.pattern)
                .collect::<Vec<_>>()
                .join("\n");

        let fs = InMemoryFileSystem::new()
            .with_file("/claude/settings.json", &settings.to_string())
            .with_file("/claude/CLAUDE.md", "# Global\nShort file\n")
            .with_file("/project/.gitignore", &gitignore_content)
            .with_file("/project/CLAUDE.md", "# Project\nSmall file\n");

        let report = run_all_checks(
            &fs,
            Path::new("/claude"),
            Path::new("/project"),
            Path::new("/ecc"),
        );

        assert_eq!(report.checks.len(), 7);
        assert!(report.score >= 90);
        assert_eq!(report.grade, "A");
    }

    #[test]
    fn run_all_checks_empty_setup() {
        let fs = InMemoryFileSystem::new();

        let report = run_all_checks(
            &fs,
            Path::new("/claude"),
            Path::new("/project"),
            Path::new("/ecc"),
        );

        assert_eq!(report.checks.len(), 7);
        // Should have findings: no settings, no gitignore, no CLAUDE.md
        let total_findings: usize =
            report.checks.iter().map(|c| c.findings.len()).sum();
        assert!(total_findings >= 2);
        assert!(report.score < 90);
    }
}
