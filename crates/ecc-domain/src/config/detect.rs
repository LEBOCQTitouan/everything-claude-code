use ecc_ports::fs::FileSystem;
use std::collections::BTreeMap;
use std::path::Path;

/// An agent file detected during setup scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedAgent {
    pub filename: String,
    pub name: Option<String>,
}

/// A skill directory detected during setup scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedSkill {
    pub dirname: String,
    pub has_skill_md: bool,
}

/// A hook detected from settings.json.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedHook {
    pub event: String,
    pub description: String,
    pub matcher: String,
}

/// Complete detection result for a Claude Code config directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectionResult {
    pub agents: Vec<DetectedAgent>,
    pub commands: Vec<String>,
    pub skills: Vec<DetectedSkill>,
    pub rules: BTreeMap<String, Vec<String>>,
    pub hooks: Vec<DetectedHook>,
    pub claude_md_headings: Vec<String>,
    pub has_settings_json: bool,
    pub has_claude_md: bool,
}

/// Extract `name` from YAML frontmatter in markdown content.
/// Returns `None` if no frontmatter or no `name:` field.
pub fn extract_frontmatter_name(content: &str) -> Option<String> {
    if !content.starts_with("---") {
        return None;
    }

    let after_opening = &content[3..];
    let end_idx = after_opening.find("---")?;
    let frontmatter = &after_opening[..end_idx];

    for line in frontmatter.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("name:") {
            let value = value.trim();
            if value.is_empty() {
                return None;
            }
            // Strip surrounding quotes (single or double)
            let stripped = value
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
                .unwrap_or(value);
            return Some(stripped.to_string());
        }
    }

    None
}

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

/// Generate a human-readable report from detection results.
pub fn generate_report(result: &DetectionResult) -> String {
    let mut lines = vec!["Existing Claude Code configuration:".to_string()];

    if !result.agents.is_empty() {
        lines.push(format!("  Agents:   {} found", result.agents.len()));
        for a in &result.agents {
            let suffix = match &a.name {
                Some(name) => format!(" ({name})"),
                None => String::new(),
            };
            lines.push(format!("    - {}{suffix}", a.filename));
        }
    }

    if !result.commands.is_empty() {
        lines.push(format!("  Commands: {} found", result.commands.len()));
    }

    if !result.skills.is_empty() {
        lines.push(format!("  Skills:   {} found", result.skills.len()));
    }

    if !result.rules.is_empty() {
        let total_rules: usize = result.rules.values().map(Vec::len).sum();
        let groups: Vec<&str> = result.rules.keys().map(String::as_str).collect();
        lines.push(format!(
            "  Rules:    {total_rules} across {} group(s) [{}]",
            groups.len(),
            groups.join(", ")
        ));
    }

    if !result.hooks.is_empty() {
        lines.push(format!("  Hooks:    {} found", result.hooks.len()));
    }

    if result.has_claude_md {
        lines.push(format!(
            "  CLAUDE.md: exists ({} sections)",
            result.claude_md_headings.len()
        ));
    }

    if result.has_settings_json {
        lines.push("  settings.json: exists".to_string());
    }

    if result.agents.is_empty()
        && result.commands.is_empty()
        && result.skills.is_empty()
        && result.rules.is_empty()
        && result.hooks.is_empty()
    {
        lines.push("  (no existing configuration found)".to_string());
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::InMemoryFileSystem;
    use std::path::Path;

    // --- extract_frontmatter_name ---

    #[test]
    fn extract_frontmatter_name_valid() {
        let content = "---\nname: my-agent\ndescription: test\n---\n# Content";
        assert_eq!(
            extract_frontmatter_name(content),
            Some("my-agent".to_string())
        );
    }

    #[test]
    fn extract_frontmatter_name_no_frontmatter() {
        let content = "# Just markdown\nNo frontmatter here.";
        assert_eq!(extract_frontmatter_name(content), None);
    }

    #[test]
    fn extract_frontmatter_name_no_name_field() {
        let content = "---\ndescription: test\nmodel: opus\n---\n# Content";
        assert_eq!(extract_frontmatter_name(content), None);
    }

    #[test]
    fn extract_frontmatter_name_double_quoted() {
        let content = "---\nname: \"Quoted Agent\"\n---\n";
        assert_eq!(
            extract_frontmatter_name(content),
            Some("Quoted Agent".to_string())
        );
    }

    #[test]
    fn extract_frontmatter_name_single_quoted() {
        let content = "---\nname: 'Single Quoted'\n---\n";
        assert_eq!(
            extract_frontmatter_name(content),
            Some("Single Quoted".to_string())
        );
    }

    #[test]
    fn extract_frontmatter_name_unclosed_frontmatter() {
        let content = "---\nname: broken\nno closing delimiter";
        assert_eq!(extract_frontmatter_name(content), None);
    }

    #[test]
    fn extract_frontmatter_name_empty_value() {
        let content = "---\nname:\n---\n";
        assert_eq!(extract_frontmatter_name(content), None);
    }

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

    // --- generate_report ---

    #[test]
    fn generate_report_empty() {
        let result = DetectionResult {
            agents: vec![],
            commands: vec![],
            skills: vec![],
            rules: BTreeMap::new(),
            hooks: vec![],
            claude_md_headings: vec![],
            has_settings_json: false,
            has_claude_md: false,
        };

        let report = generate_report(&result);
        assert!(report.contains("(no existing configuration found)"));
    }

    #[test]
    fn generate_report_with_agents() {
        let result = DetectionResult {
            agents: vec![
                DetectedAgent {
                    filename: "planner.md".into(),
                    name: Some("Planner".into()),
                },
                DetectedAgent {
                    filename: "reviewer.md".into(),
                    name: None,
                },
            ],
            commands: vec!["plan.md".into()],
            skills: vec![],
            rules: BTreeMap::new(),
            hooks: vec![],
            claude_md_headings: vec![],
            has_settings_json: true,
            has_claude_md: false,
        };

        let report = generate_report(&result);
        assert!(report.contains("Agents:   2 found"));
        assert!(report.contains("planner.md (Planner)"));
        assert!(report.contains("reviewer.md"));
        assert!(report.contains("Commands: 1 found"));
        assert!(report.contains("settings.json: exists"));
    }

    #[test]
    fn generate_report_with_rules_and_claude_md() {
        let mut rules = BTreeMap::new();
        rules.insert("common".into(), vec!["style.md".into(), "security.md".into()]);
        rules.insert("typescript".into(), vec!["eslint.md".into()]);

        let result = DetectionResult {
            agents: vec![],
            commands: vec![],
            skills: vec![],
            rules,
            hooks: vec![],
            claude_md_headings: vec!["## Section 1".into(), "## Section 2".into()],
            has_settings_json: false,
            has_claude_md: true,
        };

        let report = generate_report(&result);
        assert!(report.contains("Rules:    3 across 2 group(s) [common, typescript]"));
        assert!(report.contains("CLAUDE.md: exists (2 sections)"));
    }
}
