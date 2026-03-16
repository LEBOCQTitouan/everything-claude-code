use ecc_domain::config::audit::{
    parse_frontmatter, AuditCheckResult, AuditFinding, Severity,
};
use ecc_ports::fs::FileSystem;
use std::path::Path;

use super::read_json_safe;

/// Check that ECC deny rules are present in settings.json.
pub fn check_deny_rules(
    fs: &dyn FileSystem,
    settings_path: &Path,
) -> AuditCheckResult {
    let mut findings = Vec::new();

    let settings = match read_json_safe(fs, settings_path) {
        Ok(Some(s)) => s,
        Ok(None) => {
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
        Err(msg) => {
            findings.push(AuditFinding {
                id: "DENY-002".into(),
                severity: Severity::High,
                title: "Corrupt settings.json".into(),
                detail: msg,
                fix: "Fix or recreate settings.json.".into(),
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
        Ok(Some(s)) => s,
        _ => {
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

/// Check that the ECC statusline script is installed and settings reference it.
pub fn check_statusline(
    fs: &dyn FileSystem,
    claude_dir: &Path,
) -> AuditCheckResult {
    let mut findings = Vec::new();

    let script_path = claude_dir.join(ecc_domain::config::statusline::STATUSLINE_SCRIPT_FILENAME);
    if !fs.exists(&script_path) {
        findings.push(AuditFinding {
            id: "SL-001".into(),
            severity: Severity::Low,
            title: "Statusline script not installed".into(),
            detail: format!("Expected at {}", script_path.display()),
            fix: "Run `ecc install` to install the statusline script.".into(),
        });
    }

    let settings_path = claude_dir.join("settings.json");
    match read_json_safe(fs, &settings_path) {
        Ok(Some(settings)) => {
            let has_ecc_statusline = settings
                .get("statusLine")
                .and_then(|sl| sl.get("command"))
                .and_then(|c| c.as_str())
                .is_some_and(|cmd| {
                    cmd.contains(ecc_domain::config::statusline::STATUSLINE_SCRIPT_FILENAME)
                });

            if !has_ecc_statusline && findings.is_empty() {
                // Script exists but settings don't reference it
                findings.push(AuditFinding {
                    id: "SL-002".into(),
                    severity: Severity::Low,
                    title: "Statusline not configured in settings.json".into(),
                    detail: "Script exists but settings.json doesn't reference it.".into(),
                    fix: "Run `ecc install` to configure the statusline.".into(),
                });
            }
        }
        _ => {
            // Settings missing/corrupt — already reported by check_deny_rules
        }
    }

    AuditCheckResult {
        name: "Statusline".into(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_domain::config::audit::Severity;
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

        let result = check_hook_duplicates(&fs, Path::new("/settings.json"));
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

        let result = check_hook_duplicates(&fs, Path::new("/settings.json"));
        assert!(!result.passed);
        assert_eq!(result.findings[0].id, "HOOK-001");
    }

    #[test]
    fn check_hook_duplicates_no_settings() {
        let fs = InMemoryFileSystem::new();
        let result = check_hook_duplicates(&fs, Path::new("/settings.json"));
        assert!(result.passed);
    }

    #[test]
    fn check_hook_duplicates_no_hooks_key() {
        let settings = serde_json::json!({"permissions": {}});
        let fs = InMemoryFileSystem::new()
            .with_file("/settings.json", &settings.to_string());

        let result = check_hook_duplicates(&fs, Path::new("/settings.json"));
        assert!(result.passed);
    }

    // --- check_global_claude_md ---

    #[test]
    fn check_global_claude_md_exists() {
        let fs = InMemoryFileSystem::new()
            .with_file("/claude/CLAUDE.md", "# Global instructions");

        let result = check_global_claude_md(&fs, Path::new("/claude"));
        assert!(result.passed);
    }

    #[test]
    fn check_global_claude_md_missing() {
        let fs = InMemoryFileSystem::new();
        let result = check_global_claude_md(&fs, Path::new("/claude"));
        assert!(!result.passed);
        assert_eq!(result.findings[0].id, "CMD-001");
    }

    // --- check_agent_skills ---

    #[test]
    fn check_agent_skills_all_have_skills() {
        let fs = InMemoryFileSystem::new();
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

        let result = check_command_descriptions(&fs, Path::new("/commands"));
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

        let result = check_command_descriptions(&fs, Path::new("/commands"));
        assert!(!result.passed);
        assert_eq!(result.findings[0].id, "CMD-002");
    }

    #[test]
    fn check_command_descriptions_skips_underscore() {
        let fs = InMemoryFileSystem::new()
            .with_file("/commands/_archive.md", "# No frontmatter");

        let result = check_command_descriptions(&fs, Path::new("/commands"));
        assert!(result.passed);
    }

    // --- check_statusline ---

    #[test]
    fn check_statusline_all_present() {
        let settings = serde_json::json!({
            "statusLine": {"command": "/home/user/.claude/statusline-command.sh"}
        });
        let fs = InMemoryFileSystem::new()
            .with_file("/claude/statusline-command.sh", "#!/bin/bash\necho ok")
            .with_file("/claude/settings.json", &settings.to_string());

        let result = check_statusline(&fs, Path::new("/claude"));
        assert!(result.passed);
    }

    #[test]
    fn check_statusline_script_missing() {
        let settings = serde_json::json!({
            "statusLine": {"command": "/claude/statusline-command.sh"}
        });
        let fs = InMemoryFileSystem::new()
            .with_file("/claude/settings.json", &settings.to_string());

        let result = check_statusline(&fs, Path::new("/claude"));
        assert!(!result.passed);
        assert_eq!(result.findings[0].id, "SL-001");
    }

    #[test]
    fn check_statusline_not_in_settings() {
        let fs = InMemoryFileSystem::new()
            .with_file("/claude/statusline-command.sh", "#!/bin/bash")
            .with_file("/claude/settings.json", "{}");

        let result = check_statusline(&fs, Path::new("/claude"));
        assert!(!result.passed);
        assert_eq!(result.findings[0].id, "SL-002");
    }

    #[test]
    fn check_statusline_custom_command_no_finding() {
        // User has custom statusline, script missing — only SL-001
        let settings = serde_json::json!({
            "statusLine": {"command": "my-custom.sh"}
        });
        let fs = InMemoryFileSystem::new()
            .with_file("/claude/settings.json", &settings.to_string());

        let result = check_statusline(&fs, Path::new("/claude"));
        assert!(!result.passed);
        // Only SL-001 (script missing), not SL-002
        assert_eq!(result.findings.len(), 1);
        assert_eq!(result.findings[0].id, "SL-001");
    }

    // --- check_project_claude_md ---

    #[test]
    fn check_project_claude_md_small() {
        let content = "# Title\nSome content\n";
        let fs = InMemoryFileSystem::new()
            .with_file("/project/CLAUDE.md", content);

        let result = check_project_claude_md(&fs, Path::new("/project"));
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

        let result = check_project_claude_md(&fs, Path::new("/project"));
        assert!(!result.passed);
        assert_eq!(result.findings[0].id, "PCM-001");
    }

    #[test]
    fn check_project_claude_md_missing() {
        let fs = InMemoryFileSystem::new();
        let result = check_project_claude_md(&fs, Path::new("/project"));
        assert!(result.passed);
    }
}
