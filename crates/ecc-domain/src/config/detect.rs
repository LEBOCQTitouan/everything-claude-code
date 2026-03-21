use std::collections::BTreeMap;

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
///
/// Delegates to the canonical `extract_frontmatter` in `config::validate`,
/// then extracts and unquotes the `name` value.
pub fn extract_frontmatter_name(content: &str) -> Option<String> {
    let fm = super::validate::extract_frontmatter(content)?;
    let value = fm.get("name")?;
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
    Some(stripped.to_string())
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
    use std::collections::BTreeMap;

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
        rules.insert(
            "common".into(),
            vec!["style.md".into(), "security.md".into()],
        );
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
