//! Config I/O operations — reads/writes config files via FileSystem port.

pub mod detect {
    use ecc_domain::config::detect::{
        extract_frontmatter_name, DetectedAgent, DetectedHook, DetectedSkill, DetectionResult,
    };
    use ecc_ports::fs::FileSystem;
    use std::collections::BTreeMap;
    use std::path::Path;

    /// List .md filenames in a directory, sorted.
    fn list_md_files(fs: &dyn FileSystem, dir: &Path) -> Vec<String> {
        let entries = match fs.read_dir(dir) {
            Ok(e) => e,
            Err(_) => return Vec::new(),
        };

        let mut filenames: Vec<String> = entries
            .iter()
            .filter(|p| {
                p.extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
            })
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .collect();

        filenames.sort();
        filenames
    }

    /// Detect agents in a directory (reads `agents/` subdir).
    pub fn detect_agents(fs: &dyn FileSystem, dir: &Path) -> Vec<DetectedAgent> {
        let agents_dir = dir.join("agents");
        let filenames = list_md_files(fs, &agents_dir);

        filenames
            .into_iter()
            .map(|filename| {
                let name = fs
                    .read_to_string(&agents_dir.join(&filename))
                    .ok()
                    .and_then(|content| extract_frontmatter_name(&content));
                DetectedAgent { filename, name }
            })
            .collect()
    }

    /// Detect commands in a directory (reads `commands/` subdir).
    pub fn detect_commands(fs: &dyn FileSystem, dir: &Path) -> Vec<String> {
        list_md_files(fs, &dir.join("commands"))
    }

    /// Detect skills in a directory (reads `skills/` subdir).
    pub fn detect_skills(fs: &dyn FileSystem, dir: &Path) -> Vec<DetectedSkill> {
        let skills_dir = dir.join("skills");
        let entries = match fs.read_dir(&skills_dir) {
            Ok(e) => e,
            Err(_) => return Vec::new(),
        };

        let mut skills: Vec<DetectedSkill> = entries
            .iter()
            .filter(|p| fs.is_dir(p))
            .filter_map(|p| {
                let dirname = p.file_name()?.to_string_lossy().into_owned();
                let has_skill_md = fs.exists(&skills_dir.join(&dirname).join("SKILL.md"));
                Some(DetectedSkill {
                    dirname,
                    has_skill_md,
                })
            })
            .collect();

        skills.sort_by(|a, b| a.dirname.cmp(&b.dirname));
        skills
    }

    /// Detect rules in a directory, grouped by subdirectory.
    pub fn detect_rules(fs: &dyn FileSystem, dir: &Path) -> BTreeMap<String, Vec<String>> {
        let rules_dir = dir.join("rules");
        let entries = match fs.read_dir(&rules_dir) {
            Ok(e) => e,
            Err(_) => return BTreeMap::new(),
        };

        let mut result = BTreeMap::new();

        for entry in &entries {
            if !fs.is_dir(entry) {
                continue;
            }
            if let Some(group_name) = entry.file_name() {
                let group = group_name.to_string_lossy().into_owned();
                let files = list_md_files(fs, entry);
                if !files.is_empty() {
                    result.insert(group, files);
                }
            }
        }

        result
    }

    /// Detect hooks from settings.json.
    pub fn detect_hooks(fs: &dyn FileSystem, dir: &Path) -> Vec<DetectedHook> {
        let settings_path = dir.join("settings.json");
        let content = match fs.read_to_string(&settings_path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let settings: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => return Vec::new(),
        };

        let hooks_obj = match settings.get("hooks").and_then(|h| h.as_object()) {
            Some(h) => h,
            None => return Vec::new(),
        };

        let mut hooks = Vec::new();

        for (event, entries) in hooks_obj {
            let Some(entries_arr) = entries.as_array() else {
                continue;
            };
            for entry in entries_arr {
                let description = entry
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("")
                    .to_string();
                let matcher = entry
                    .get("matcher")
                    .and_then(|m| m.as_str())
                    .unwrap_or("*")
                    .to_string();
                hooks.push(DetectedHook {
                    event: event.clone(),
                    description,
                    matcher,
                });
            }
        }

        hooks
    }

    /// Detect CLAUDE.md headings in a project directory.
    fn detect_claude_md(fs: &dyn FileSystem, project_dir: &Path) -> Vec<String> {
        let claude_md_path = project_dir.join("CLAUDE.md");
        let content = match fs.read_to_string(&claude_md_path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        content
            .lines()
            .filter(|line| line.starts_with("## "))
            .map(|line| line.trim().to_string())
            .collect()
    }

    /// Run full detection on a Claude Code config directory.
    /// `project_dir` is the project root (for CLAUDE.md detection).
    pub fn detect(
        fs: &dyn FileSystem,
        dir: &Path,
        project_dir: Option<&Path>,
    ) -> DetectionResult {
        let claude_md_headings = match project_dir {
            Some(pd) => detect_claude_md(fs, pd),
            None => Vec::new(),
        };
        let has_claude_md = project_dir.is_some_and(|pd| fs.exists(&pd.join("CLAUDE.md")));

        DetectionResult {
            agents: detect_agents(fs, dir),
            commands: detect_commands(fs, dir),
            skills: detect_skills(fs, dir),
            rules: detect_rules(fs, dir),
            hooks: detect_hooks(fs, dir),
            claude_md_headings,
            has_settings_json: fs.exists(&dir.join("settings.json")),
            has_claude_md,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ecc_test_support::InMemoryFileSystem;
        use std::path::Path;

        // --- detect_agents ---

        #[test]
        fn detect_agents_empty_dir() {
            let fs = InMemoryFileSystem::new();
            let agents = detect_agents(&fs, Path::new("/claude"));
            assert!(agents.is_empty());
        }

        #[test]
        fn detect_agents_with_agents() {
            let fs = InMemoryFileSystem::new()
                .with_file(
                    "/claude/agents/planner.md",
                    "---\nname: Planner\n---\n# Planner agent",
                )
                .with_file("/claude/agents/reviewer.md", "# No frontmatter");

            let agents = detect_agents(&fs, Path::new("/claude"));
            assert_eq!(agents.len(), 2);
            assert_eq!(agents[0].filename, "planner.md");
            assert_eq!(agents[0].name, Some("Planner".to_string()));
            assert_eq!(agents[1].filename, "reviewer.md");
            assert_eq!(agents[1].name, None);
        }

        // --- detect_commands ---

        #[test]
        fn detect_commands_empty_dir() {
            let fs = InMemoryFileSystem::new();
            let commands = detect_commands(&fs, Path::new("/claude"));
            assert!(commands.is_empty());
        }

        #[test]
        fn detect_commands_with_commands() {
            let fs = InMemoryFileSystem::new()
                .with_file("/claude/commands/plan.md", "# Plan")
                .with_file("/claude/commands/verify.md", "# Verify");

            let commands = detect_commands(&fs, Path::new("/claude"));
            assert_eq!(commands, vec!["plan.md", "verify.md"]);
        }

        // --- detect_skills ---

        #[test]
        fn detect_skills_with_skill_md() {
            let fs = InMemoryFileSystem::new()
                .with_dir("/claude/skills/tdd")
                .with_file("/claude/skills/tdd/SKILL.md", "# TDD Skill");

            let skills = detect_skills(&fs, Path::new("/claude"));
            assert_eq!(skills.len(), 1);
            assert_eq!(skills[0].dirname, "tdd");
            assert!(skills[0].has_skill_md);
        }

        #[test]
        fn detect_skills_without_skill_md() {
            let fs = InMemoryFileSystem::new()
                .with_dir("/claude/skills/security")
                .with_file("/claude/skills/security/notes.txt", "notes");

            let skills = detect_skills(&fs, Path::new("/claude"));
            assert_eq!(skills.len(), 1);
            assert_eq!(skills[0].dirname, "security");
            assert!(!skills[0].has_skill_md);
        }

        #[test]
        fn detect_skills_empty_dir() {
            let fs = InMemoryFileSystem::new();
            let skills = detect_skills(&fs, Path::new("/claude"));
            assert!(skills.is_empty());
        }

        // --- detect_rules ---

        #[test]
        fn detect_rules_grouped_by_subdir() {
            let fs = InMemoryFileSystem::new()
                .with_dir("/claude/rules/common")
                .with_file("/claude/rules/common/style.md", "# Style")
                .with_file("/claude/rules/common/security.md", "# Security")
                .with_dir("/claude/rules/typescript")
                .with_file("/claude/rules/typescript/eslint.md", "# ESLint");

            let rules = detect_rules(&fs, Path::new("/claude"));
            assert_eq!(rules.len(), 2);
            assert_eq!(
                rules["common"],
                vec!["security.md".to_string(), "style.md".to_string()]
            );
            assert_eq!(
                rules["typescript"],
                vec!["eslint.md".to_string()]
            );
        }

        #[test]
        fn detect_rules_empty_subdir_excluded() {
            let fs = InMemoryFileSystem::new()
                .with_dir("/claude/rules/empty");

            let rules = detect_rules(&fs, Path::new("/claude"));
            assert!(rules.is_empty());
        }

        // --- detect_hooks ---

        #[test]
        fn detect_hooks_from_settings_json() {
            let settings = r#"{
                "hooks": {
                    "PreToolUse": [
                        {"description": "Format code", "matcher": "Write"},
                        {"description": "Lint check", "matcher": "Edit"}
                    ],
                    "PostToolUse": [
                        {"description": "Auto test"}
                    ]
                }
            }"#;
            let fs = InMemoryFileSystem::new().with_file("/claude/settings.json", settings);

            let hooks = detect_hooks(&fs, Path::new("/claude"));
            assert_eq!(hooks.len(), 3);
            assert_eq!(hooks[0].event, "PostToolUse");
            assert_eq!(hooks[0].description, "Auto test");
            assert_eq!(hooks[0].matcher, "*");
        }

        #[test]
        fn detect_hooks_no_settings() {
            let fs = InMemoryFileSystem::new();
            let hooks = detect_hooks(&fs, Path::new("/claude"));
            assert!(hooks.is_empty());
        }

        // --- detect (full integration) ---

        #[test]
        fn detect_full_integration() {
            let fs = InMemoryFileSystem::new()
                .with_file("/claude/agents/planner.md", "---\nname: Planner\n---\n")
                .with_file("/claude/commands/plan.md", "# Plan")
                .with_dir("/claude/skills/tdd")
                .with_file("/claude/skills/tdd/SKILL.md", "# TDD")
                .with_dir("/claude/rules/common")
                .with_file("/claude/rules/common/style.md", "# Style")
                .with_file(
                    "/claude/settings.json",
                    r#"{"hooks": {"PreToolUse": [{"description": "test", "matcher": "Write"}]}}"#,
                )
                .with_file("/project/CLAUDE.md", "# Title\n## Section One\n## Section Two\n");

            let result = detect(&fs, Path::new("/claude"), Some(Path::new("/project")));

            assert_eq!(result.agents.len(), 1);
            assert_eq!(result.commands.len(), 1);
            assert_eq!(result.skills.len(), 1);
            assert_eq!(result.rules.len(), 1);
            assert_eq!(result.hooks.len(), 1);
            assert_eq!(result.claude_md_headings.len(), 2);
            assert!(result.has_settings_json);
            assert!(result.has_claude_md);
        }

        #[test]
        fn detect_empty_dir() {
            let fs = InMemoryFileSystem::new();
            let result = detect(&fs, Path::new("/claude"), None);

            assert!(result.agents.is_empty());
            assert!(result.commands.is_empty());
            assert!(result.skills.is_empty());
            assert!(result.rules.is_empty());
            assert!(result.hooks.is_empty());
            assert!(result.claude_md_headings.is_empty());
            assert!(!result.has_settings_json);
            assert!(!result.has_claude_md);
        }
    }
}

pub mod audit {
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
}

pub mod clean {
    use ecc_domain::config::clean::{CleanReport, ARTIFACT_DIRS};
    use ecc_domain::config::manifest::{EccManifest, MANIFEST_FILENAME};
    use ecc_ports::fs::FileSystem;
    use std::path::Path;

    fn remove_file(
        fs: &dyn FileSystem,
        path: &Path,
        label: &str,
        dry_run: bool,
        report: &mut CleanReport,
    ) {
        if !fs.exists(path) {
            report.skipped.push(label.to_string());
            return;
        }
        if dry_run {
            report.removed.push(label.to_string());
            return;
        }
        match fs.remove_file(path) {
            Ok(()) => report.removed.push(label.to_string()),
            Err(e) => report.errors.push(format!("{label}: {e}")),
        }
    }

    fn remove_directory(
        fs: &dyn FileSystem,
        path: &Path,
        label: &str,
        dry_run: bool,
        report: &mut CleanReport,
    ) {
        if !fs.exists(path) {
            report.skipped.push(label.to_string());
            return;
        }
        if dry_run {
            report.removed.push(label.to_string());
            return;
        }
        match fs.remove_dir_all(path) {
            Ok(()) => report.removed.push(label.to_string()),
            Err(e) => report.errors.push(format!("{label}: {e}")),
        }
    }

    /// Returns true if a hook entry's `hooks[].command` starts with "ecc-hook " or "ecc-shell-hook ".
    fn is_current_ecc_hook(entry: &serde_json::Value) -> bool {
        let Some(hooks) = entry.get("hooks").and_then(|h| h.as_array()) else {
            return false;
        };

        hooks.iter().any(|hook| {
            let Some(cmd) = hook.get("command").and_then(|c| c.as_str()) else {
                return false;
            };
            cmd.starts_with("ecc-hook ") || cmd.starts_with("ecc-shell-hook ")
        })
    }

    /// Remove ECC hooks from settings.json, preserving user-added hooks.
    /// Returns `Ok(Some(count))` if hooks were removed, `Ok(None)` if no changes, `Err` on failure.
    fn remove_ecc_hooks(
        fs: &dyn FileSystem,
        settings_path: &Path,
        is_legacy_hook: &dyn Fn(&serde_json::Value) -> bool,
        dry_run: bool,
    ) -> Result<Option<usize>, String> {
        let content = fs
            .read_to_string(settings_path)
            .map_err(|e| e.to_string())?;
        let mut settings: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| e.to_string())?;

        let Some(hooks_obj) = settings.get_mut("hooks").and_then(|h| h.as_object_mut()) else {
            return Ok(None);
        };

        let hooks_before: usize = hooks_obj
            .values()
            .filter_map(|v| v.as_array())
            .map(|a| a.len())
            .sum();

        for entries in hooks_obj.values_mut() {
            let Some(arr) = entries.as_array_mut() else {
                continue;
            };
            arr.retain(|entry| !is_legacy_hook(entry) && !is_current_ecc_hook(entry));
        }

        let hooks_after: usize = hooks_obj
            .values()
            .filter_map(|v| v.as_array())
            .map(|a| a.len())
            .sum();

        let removed_count = hooks_before - hooks_after;
        if removed_count == 0 {
            return Ok(None);
        }

        if !dry_run {
            let json =
                serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
            fs.write(settings_path, &format!("{json}\n"))
                .map_err(|e| e.to_string())?;
        }

        Ok(Some(removed_count))
    }

    /// Remove only files listed in the manifest (surgical cleanup).
    /// In `dry_run` mode, records what would be removed without actually removing.
    pub fn clean_from_manifest(
        fs: &dyn FileSystem,
        dir: &Path,
        manifest: &EccManifest,
        dry_run: bool,
    ) -> CleanReport {
        let mut report = CleanReport::new();

        // Remove agent files
        for agent in &manifest.artifacts.agents {
            let file_path = dir.join("agents").join(agent);
            let label = format!("agents/{agent}");
            remove_file(fs, &file_path, &label, dry_run, &mut report);
        }

        // Remove command files
        for command in &manifest.artifacts.commands {
            let file_path = dir.join("commands").join(command);
            let label = format!("commands/{command}");
            remove_file(fs, &file_path, &label, dry_run, &mut report);
        }

        // Remove skill directories
        for skill in &manifest.artifacts.skills {
            let dir_path = dir.join("skills").join(skill);
            let label = format!("skills/{skill}");
            remove_directory(fs, &dir_path, &label, dry_run, &mut report);
        }

        // Remove rule files (grouped by language/group)
        for (group, files) in &manifest.artifacts.rules {
            for file in files {
                let file_path = dir.join("rules").join(group).join(file);
                let label = format!("rules/{group}/{file}");
                remove_file(fs, &file_path, &label, dry_run, &mut report);
            }
        }

        // Remove manifest itself
        let manifest_path = dir.join(MANIFEST_FILENAME);
        remove_file(fs, &manifest_path, MANIFEST_FILENAME, dry_run, &mut report);

        report
    }

    /// Remove entire ECC directories and clean hooks from settings.json (nuclear option).
    /// `is_legacy_hook` is a predicate that identifies legacy ECC hooks to remove.
    pub fn clean_all(
        fs: &dyn FileSystem,
        dir: &Path,
        is_legacy_hook: &dyn Fn(&serde_json::Value) -> bool,
        dry_run: bool,
    ) -> CleanReport {
        let mut report = CleanReport::new();

        // Remove entire artifact directories
        for artifact_dir in ARTIFACT_DIRS {
            let dir_path = dir.join(artifact_dir);
            remove_directory(fs, &dir_path, artifact_dir, dry_run, &mut report);
        }

        // Remove ECC hooks from settings.json
        let settings_path = dir.join("settings.json");
        if fs.exists(&settings_path) {
            match remove_ecc_hooks(fs, &settings_path, is_legacy_hook, dry_run) {
                Ok(Some(count)) => {
                    report
                        .removed
                        .push(format!("settings.json ({count} ECC hook(s))"));
                }
                Ok(None) => {}
                Err(msg) => {
                    report.errors.push(format!("settings.json: {msg}"));
                }
            }
        }

        // Remove manifest
        let manifest_path = dir.join(MANIFEST_FILENAME);
        remove_file(fs, &manifest_path, MANIFEST_FILENAME, dry_run, &mut report);

        report
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ecc_domain::config::manifest::Artifacts;
        use ecc_test_support::InMemoryFileSystem;
        use std::collections::BTreeMap;
        use std::path::Path;

        fn sample_manifest() -> EccManifest {
            let mut rules = BTreeMap::new();
            rules.insert(
                "common".to_string(),
                vec!["style.md".to_string(), "security.md".to_string()],
            );

            EccManifest {
                version: "4.0.0".to_string(),
                installed_at: "2026-03-14T00:00:00Z".to_string(),
                updated_at: "2026-03-14T00:00:00Z".to_string(),
                languages: vec!["rust".to_string()],
                artifacts: Artifacts {
                    agents: vec!["planner.md".to_string()],
                    commands: vec!["plan.md".to_string()],
                    skills: vec!["tdd".to_string()],
                    rules,
                    hook_descriptions: vec![],
                },
            }
        }

        fn build_populated_fs() -> InMemoryFileSystem {
            InMemoryFileSystem::new()
                .with_dir("/claude/agents")
                .with_file("/claude/agents/planner.md", "# Planner")
                .with_dir("/claude/commands")
                .with_file("/claude/commands/plan.md", "# Plan")
                .with_dir("/claude/skills")
                .with_dir("/claude/skills/tdd")
                .with_file("/claude/skills/tdd/SKILL.md", "# TDD")
                .with_dir("/claude/rules")
                .with_dir("/claude/rules/common")
                .with_file("/claude/rules/common/style.md", "# Style")
                .with_file("/claude/rules/common/security.md", "# Security")
                .with_file("/claude/.ecc-manifest.json", "{}")
        }

        // --- clean_from_manifest ---

        #[test]
        fn clean_from_manifest_removes_listed_files() {
            let fs = build_populated_fs();
            let manifest = sample_manifest();
            let dir = Path::new("/claude");

            let report = clean_from_manifest(&fs, dir, &manifest, false);

            assert!(report.errors.is_empty());
            assert!(!report.removed.is_empty());
            assert!(!fs.exists(&dir.join("agents/planner.md")));
            assert!(!fs.exists(&dir.join("commands/plan.md")));
            assert!(!fs.exists(&dir.join("rules/common/style.md")));
            assert!(!fs.exists(&dir.join("rules/common/security.md")));
            assert!(!fs.exists(&dir.join(".ecc-manifest.json")));
        }

        #[test]
        fn clean_from_manifest_skips_missing() {
            let fs = InMemoryFileSystem::new();
            let manifest = sample_manifest();
            let dir = Path::new("/claude");

            let report = clean_from_manifest(&fs, dir, &manifest, false);

            assert!(report.removed.is_empty());
            assert!(!report.skipped.is_empty());
            assert!(report.errors.is_empty());
            // agents/planner.md, commands/plan.md, skills/tdd, rules/common/style.md,
            // rules/common/security.md, .ecc-manifest.json = 6 skipped
            assert_eq!(report.skipped.len(), 6);
        }

        #[test]
        fn clean_from_manifest_dry_run_does_not_remove() {
            let fs = build_populated_fs();
            let manifest = sample_manifest();
            let dir = Path::new("/claude");

            let report = clean_from_manifest(&fs, dir, &manifest, true);

            assert!(!report.removed.is_empty());
            // Files should still exist
            assert!(fs.exists(&dir.join("agents/planner.md")));
            assert!(fs.exists(&dir.join("commands/plan.md")));
            assert!(fs.exists(&dir.join(".ecc-manifest.json")));
        }

        // --- clean_all ---

        #[test]
        fn clean_all_removes_directories() {
            let fs = build_populated_fs();
            let dir = Path::new("/claude");
            let no_legacy = |_: &serde_json::Value| false;

            let report = clean_all(&fs, dir, &no_legacy, false);

            assert!(report.errors.is_empty());
            assert!(!fs.exists(&dir.join("agents/planner.md")));
            assert!(!fs.exists(&dir.join("commands/plan.md")));
            assert!(!fs.exists(&dir.join("skills/tdd/SKILL.md")));
        }

        #[test]
        fn clean_all_cleans_hooks() {
            let settings = r#"{
                "hooks": {
                    "PreToolUse": [
                        {"description": "ECC format", "hooks": [{"command": "ecc-hook format"}]},
                        {"description": "User hook", "hooks": [{"command": "my-custom-hook"}]}
                    ]
                },
                "other": "preserved"
            }"#;
            let fs = InMemoryFileSystem::new()
                .with_file("/claude/settings.json", settings)
                .with_file("/claude/.ecc-manifest.json", "{}");
            let dir = Path::new("/claude");
            let no_legacy = |_: &serde_json::Value| false;

            let report = clean_all(&fs, dir, &no_legacy, false);

            // Should have removed 1 ECC hook
            assert!(
                report
                    .removed
                    .iter()
                    .any(|r| r.contains("1 ECC hook(s)"))
            );

            // Verify settings.json was rewritten with user hook preserved
            let updated = fs
                .read_to_string(Path::new("/claude/settings.json"))
                .unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&updated).unwrap();
            let pre_hooks = parsed["hooks"]["PreToolUse"].as_array().unwrap();
            assert_eq!(pre_hooks.len(), 1);
            assert_eq!(pre_hooks[0]["description"], "User hook");
            // Other fields preserved
            assert_eq!(parsed["other"], "preserved");
        }

        #[test]
        fn clean_all_with_legacy_hook_predicate() {
            let settings = r#"{
                "hooks": {
                    "PreToolUse": [
                        {"description": "legacy", "type": "legacy"},
                        {"description": "keep", "hooks": [{"command": "safe"}]}
                    ]
                }
            }"#;
            let fs = InMemoryFileSystem::new()
                .with_file("/claude/settings.json", settings)
                .with_file("/claude/.ecc-manifest.json", "{}");
            let dir = Path::new("/claude");
            let is_legacy = |v: &serde_json::Value| {
                v.get("type").and_then(|t| t.as_str()) == Some("legacy")
            };

            let report = clean_all(&fs, dir, &is_legacy, false);

            assert!(
                report
                    .removed
                    .iter()
                    .any(|r| r.contains("1 ECC hook(s)"))
            );
        }
    }
}

pub mod merge {
    use ecc_domain::config::merge::{contents_differ, FileToReview};
    use ecc_ports::fs::FileSystem;
    use std::path::Path;

    /// Pre-scan a directory to identify files that need review (new or changed).
    /// Returns `(files_to_review, unchanged_filenames)`.
    pub fn pre_scan_directory(
        fs: &dyn FileSystem,
        src_dir: &Path,
        dest_dir: &Path,
        ext: &str,
    ) -> (Vec<FileToReview>, Vec<String>) {
        let mut files_to_review = Vec::new();
        let mut unchanged = Vec::new();

        let entries = match fs.read_dir(src_dir) {
            Ok(e) => e,
            Err(_) => return (files_to_review, unchanged),
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
                files_to_review.push(FileToReview {
                    filename,
                    src_path,
                    dest_path,
                    is_new: true,
                });
            } else {
                let src_content = fs.read_to_string(&src_path).unwrap_or_default();
                let dest_content = fs.read_to_string(&dest_path).unwrap_or_default();

                if contents_differ(&src_content, &dest_content) {
                    files_to_review.push(FileToReview {
                        filename,
                        src_path,
                        dest_path,
                        is_new: false,
                    });
                } else {
                    unchanged.push(filename);
                }
            }
        }

        (files_to_review, unchanged)
    }

    /// Copy a file from source to destination.
    /// In dry-run mode, the copy is skipped.
    pub fn apply_accept(
        fs: &dyn FileSystem,
        src_path: &Path,
        dest_path: &Path,
        dry_run: bool,
    ) -> Result<(), String> {
        if dry_run {
            return Ok(());
        }

        if let Some(parent) = dest_path.parent() {
            fs.create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {e}"))?;
        }

        fs.copy(src_path, dest_path)
            .map_err(|e| format!("Failed to copy file: {e}"))
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ecc_test_support::InMemoryFileSystem;
        use std::path::Path;

        // --- pre_scan_directory ---

        #[test]
        fn pre_scan_directory_new_files() {
            let fs = InMemoryFileSystem::new()
                .with_file("/src/a.md", "content a")
                .with_file("/src/b.md", "content b");

            let (to_review, unchanged) =
                pre_scan_directory(&fs, Path::new("/src"), Path::new("/dest"), ".md");
            assert_eq!(to_review.len(), 2);
            assert!(unchanged.is_empty());
            assert!(to_review[0].is_new);
            assert!(to_review[1].is_new);
        }

        #[test]
        fn pre_scan_directory_changed_files() {
            let fs = InMemoryFileSystem::new()
                .with_file("/src/a.md", "new content")
                .with_file("/dest/a.md", "old content");

            let (to_review, unchanged) =
                pre_scan_directory(&fs, Path::new("/src"), Path::new("/dest"), ".md");
            assert_eq!(to_review.len(), 1);
            assert!(unchanged.is_empty());
            assert!(!to_review[0].is_new);
        }

        #[test]
        fn pre_scan_directory_unchanged_files() {
            let fs = InMemoryFileSystem::new()
                .with_file("/src/a.md", "same content")
                .with_file("/dest/a.md", "same content");

            let (to_review, unchanged) =
                pre_scan_directory(&fs, Path::new("/src"), Path::new("/dest"), ".md");
            assert!(to_review.is_empty());
            assert_eq!(unchanged, vec!["a.md"]);
        }

        #[test]
        fn pre_scan_directory_filters_by_ext() {
            let fs = InMemoryFileSystem::new()
                .with_file("/src/a.md", "content")
                .with_file("/src/b.txt", "content");

            let (to_review, _) =
                pre_scan_directory(&fs, Path::new("/src"), Path::new("/dest"), ".md");
            assert_eq!(to_review.len(), 1);
            assert_eq!(to_review[0].filename, "a.md");
        }

        #[test]
        fn pre_scan_directory_mixed() {
            let fs = InMemoryFileSystem::new()
                .with_file("/src/new.md", "brand new")
                .with_file("/src/changed.md", "updated")
                .with_file("/dest/changed.md", "original")
                .with_file("/src/same.md", "same")
                .with_file("/dest/same.md", "same");

            let (to_review, unchanged) =
                pre_scan_directory(&fs, Path::new("/src"), Path::new("/dest"), ".md");
            assert_eq!(to_review.len(), 2);
            assert_eq!(unchanged, vec!["same.md"]);
        }

        #[test]
        fn pre_scan_directory_empty_src() {
            let fs = InMemoryFileSystem::new();
            let (to_review, unchanged) =
                pre_scan_directory(&fs, Path::new("/src"), Path::new("/dest"), ".md");
            assert!(to_review.is_empty());
            assert!(unchanged.is_empty());
        }

        // --- apply_accept ---

        #[test]
        fn apply_accept_copies_file() {
            let fs = InMemoryFileSystem::new()
                .with_file("/src/a.md", "content");

            let result = apply_accept(
                &fs,
                Path::new("/src/a.md"),
                Path::new("/dest/a.md"),
                false,
            );
            assert!(result.is_ok());
            assert_eq!(
                fs.read_to_string(Path::new("/dest/a.md")).unwrap(),
                "content"
            );
        }

        #[test]
        fn apply_accept_dry_run_skips() {
            let fs = InMemoryFileSystem::new()
                .with_file("/src/a.md", "content");

            let result = apply_accept(
                &fs,
                Path::new("/src/a.md"),
                Path::new("/dest/a.md"),
                true,
            );
            assert!(result.is_ok());
            assert!(!fs.exists(Path::new("/dest/a.md")));
        }

        #[test]
        fn apply_accept_creates_parent_dirs() {
            let fs = InMemoryFileSystem::new()
                .with_file("/src/a.md", "content");

            let result = apply_accept(
                &fs,
                Path::new("/src/a.md"),
                Path::new("/dest/sub/dir/a.md"),
                false,
            );
            assert!(result.is_ok());
            assert!(fs.exists(Path::new("/dest/sub/dir/a.md")));
        }

        #[test]
        fn apply_accept_error_on_missing_src() {
            let fs = InMemoryFileSystem::new();

            let result = apply_accept(
                &fs,
                Path::new("/nonexistent.md"),
                Path::new("/dest/a.md"),
                false,
            );
            assert!(result.is_err());
        }
    }
}

pub mod manifest {
    use ecc_domain::config::manifest::{EccManifest, MANIFEST_FILENAME};
    use ecc_ports::fs::FileSystem;
    use std::path::Path;

    /// Read an existing manifest from a directory via the FileSystem port.
    /// Returns None if not found or corrupted.
    pub fn read_manifest(fs: &dyn FileSystem, dir: &Path) -> Option<EccManifest> {
        let manifest_path = dir.join(MANIFEST_FILENAME);
        let content = fs.read_to_string(&manifest_path).ok()?;
        let parsed: serde_json::Value = serde_json::from_str(&content).ok()?;
        if parsed.get("version").is_none() || parsed.get("artifacts").is_none() {
            return None;
        }
        serde_json::from_value(parsed).ok()
    }

    /// Write a manifest to a directory via the FileSystem port.
    pub fn write_manifest(
        fs: &dyn FileSystem,
        dir: &Path,
        manifest: &EccManifest,
    ) -> Result<(), ecc_ports::fs::FsError> {
        fs.create_dir_all(dir)?;
        let manifest_path = dir.join(MANIFEST_FILENAME);
        let json = serde_json::to_string_pretty(manifest)
            .expect("manifest serialization should not fail");
        fs.write(&manifest_path, &format!("{json}\n"))
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ecc_domain::config::manifest::{create_manifest, Artifacts};
        use ecc_test_support::InMemoryFileSystem;
        use std::collections::BTreeMap;
        use std::path::Path;

        fn sample_artifacts() -> Artifacts {
            Artifacts {
                agents: vec!["agent1.md".into(), "agent2.md".into()],
                commands: vec!["cmd1.md".into()],
                skills: vec!["skill1".into()],
                rules: {
                    let mut m = BTreeMap::new();
                    m.insert("common".into(), vec!["rule1.md".into()]);
                    m
                },
                hook_descriptions: vec!["hook1".into()],
            }
        }

        #[test]
        fn read_manifest_not_found() {
            let fs = InMemoryFileSystem::new();
            assert!(read_manifest(&fs, Path::new("/project/.claude")).is_none());
        }

        #[test]
        fn read_manifest_invalid_json() {
            let fs = InMemoryFileSystem::new()
                .with_file("/project/.claude/.ecc-manifest.json", "not json");
            assert!(read_manifest(&fs, Path::new("/project/.claude")).is_none());
        }

        #[test]
        fn read_manifest_missing_version() {
            let fs = InMemoryFileSystem::new().with_file(
                "/project/.claude/.ecc-manifest.json",
                r#"{"artifacts": {}}"#,
            );
            assert!(read_manifest(&fs, Path::new("/project/.claude")).is_none());
        }

        #[test]
        fn write_and_read_manifest_roundtrip() {
            let fs = InMemoryFileSystem::new();
            let dir = Path::new("/project/.claude");
            let original = create_manifest(
                "4.0.0",
                "2026-03-14T00:00:00Z",
                &["rust".into()],
                sample_artifacts(),
            );
            write_manifest(&fs, dir, &original).unwrap();
            let read_back = read_manifest(&fs, dir).unwrap();
            assert_eq!(read_back, original);
        }

        #[test]
        fn json_uses_camel_case() {
            let m = create_manifest("4.0.0", "now", &[], Artifacts::default());
            let json = serde_json::to_string(&m).unwrap();
            assert!(json.contains("installedAt"));
            assert!(json.contains("updatedAt"));
            assert!(json.contains("hookDescriptions"));
            assert!(!json.contains("installed_at"));
        }
    }
}

pub mod gitignore {
    use ecc_domain::config::gitignore::{
        build_gitignore_section, parse_gitignore_patterns, GitignoreEntry, GitignoreResult,
        ECC_GITIGNORE_ENTRIES,
    };
    use ecc_ports::fs::FileSystem;
    use ecc_ports::shell::ShellExecutor;
    use std::path::Path;

    /// Check if a directory is inside a git repository.
    pub fn is_git_repo(shell: &dyn ShellExecutor, dir: &Path) -> bool {
        shell
            .run_command_in_dir("git", &["rev-parse", "--git-dir"], dir)
            .is_ok_and(|out| out.success())
    }

    /// Ensure ECC entries are present in .gitignore.
    /// Creates .gitignore if it doesn't exist (only in git repos).
    pub fn ensure_gitignore_entries(
        fs: &dyn FileSystem,
        shell: &dyn ShellExecutor,
        dir: &Path,
        entries: Option<&[GitignoreEntry]>,
    ) -> GitignoreResult {
        let entries = entries.unwrap_or(ECC_GITIGNORE_ENTRIES);

        if !is_git_repo(shell, dir) {
            return GitignoreResult {
                added: vec![],
                already_present: vec![],
                skipped: true,
            };
        }

        let gitignore_path = dir.join(".gitignore");
        let existing_content = fs
            .read_to_string(&gitignore_path)
            .unwrap_or_default();

        let existing_patterns = parse_gitignore_patterns(&existing_content);
        let mut added = Vec::new();
        let mut already_present = Vec::new();
        let mut to_add = Vec::new();

        for entry in entries {
            if existing_patterns.contains(entry.pattern) {
                already_present.push(entry.pattern.to_string());
            } else {
                to_add.push(entry);
                added.push(entry.pattern.to_string());
            }
        }

        if to_add.is_empty() {
            return GitignoreResult {
                added,
                already_present,
                skipped: false,
            };
        }

        let section = build_gitignore_section(&to_add);
        let new_content = format!("{}\n{}", existing_content.trim_end(), section);
        let _ = fs.write(&gitignore_path, &new_content);

        GitignoreResult {
            added,
            already_present,
            skipped: false,
        }
    }

    /// Find ECC-generated files that are currently tracked by git.
    pub fn find_tracked_ecc_files(
        shell: &dyn ShellExecutor,
        fs: &dyn FileSystem,
        dir: &Path,
    ) -> Vec<String> {
        if !is_git_repo(shell, dir) {
            return vec![];
        }

        let mut tracked = Vec::new();
        for entry in ECC_GITIGNORE_ENTRIES {
            if entry.pattern.ends_with('/') {
                // Directory — check if any files inside are tracked
                let full_path = dir.join(entry.pattern);
                if fs.exists(&full_path)
                    && let Ok(out) =
                        shell.run_command_in_dir("git", &["ls-files", entry.pattern], dir)
                    && out.success()
                    && !out.stdout.trim().is_empty()
                {
                    tracked.push(entry.pattern.to_string());
                }
            } else if let Ok(out) = shell.run_command_in_dir(
                "git",
                &["ls-files", "--error-unmatch", entry.pattern],
                dir,
            )
                && out.success()
            {
                tracked.push(entry.pattern.to_string());
            }
        }
        tracked
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ecc_ports::shell::CommandOutput;
        use ecc_test_support::{InMemoryFileSystem, MockExecutor};

        fn git_success() -> CommandOutput {
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            }
        }

        fn git_executor() -> MockExecutor {
            MockExecutor::new().on("git", git_success())
        }

        // --- is_git_repo ---

        #[test]
        fn is_git_repo_true() {
            let shell = git_executor();
            assert!(is_git_repo(&shell, Path::new("/project")));
        }

        #[test]
        fn is_git_repo_false() {
            let shell = MockExecutor::new().on(
                "git",
                CommandOutput {
                    stdout: String::new(),
                    stderr: "not a git repo".into(),
                    exit_code: 128,
                },
            );
            assert!(!is_git_repo(&shell, Path::new("/project")));
        }

        // --- ensure_gitignore_entries ---

        #[test]
        fn ensure_skips_non_git_repo() {
            let fs = InMemoryFileSystem::new();
            let shell = MockExecutor::new().on(
                "git",
                CommandOutput {
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: 128,
                },
            );
            let result = ensure_gitignore_entries(&fs, &shell, Path::new("/project"), None);
            assert!(result.skipped);
            assert!(result.added.is_empty());
        }

        #[test]
        fn ensure_adds_all_entries_to_new_gitignore() {
            let fs = InMemoryFileSystem::new();
            let shell = git_executor();
            let result = ensure_gitignore_entries(&fs, &shell, Path::new("/project"), None);
            assert!(!result.skipped);
            assert_eq!(result.added.len(), ECC_GITIGNORE_ENTRIES.len());
            assert!(result.already_present.is_empty());
        }

        #[test]
        fn ensure_detects_already_present() {
            let fs = InMemoryFileSystem::new()
                .with_file("/project/.gitignore", ".claude/settings.local.json\n");
            let shell = git_executor();
            let result = ensure_gitignore_entries(&fs, &shell, Path::new("/project"), None);
            assert_eq!(result.already_present.len(), 1);
            assert_eq!(result.added.len(), ECC_GITIGNORE_ENTRIES.len() - 1);
        }

        #[test]
        fn ensure_all_present_adds_nothing() {
            let content = ECC_GITIGNORE_ENTRIES
                .iter()
                .map(|e| e.pattern)
                .collect::<Vec<_>>()
                .join("\n");
            let fs = InMemoryFileSystem::new().with_file("/project/.gitignore", &content);
            let shell = git_executor();
            let result = ensure_gitignore_entries(&fs, &shell, Path::new("/project"), None);
            assert!(result.added.is_empty());
            assert_eq!(result.already_present.len(), ECC_GITIGNORE_ENTRIES.len());
        }
    }
}
