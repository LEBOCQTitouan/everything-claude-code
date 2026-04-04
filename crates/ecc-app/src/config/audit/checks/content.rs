//! Content audit checks — agent skills and command descriptions.

use ecc_domain::config::audit::{AuditCheckResult, AuditFinding, Severity, parse_frontmatter};
use ecc_ports::fs::FileSystem;
use std::path::Path;

/// Check if agents have `skills:` frontmatter.
pub fn check_agent_skills(fs: &dyn FileSystem, agents_dir: &Path) -> AuditCheckResult {
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
pub fn check_command_descriptions(fs: &dyn FileSystem, commands_dir: &Path) -> AuditCheckResult {
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
            fix: "Add description: field to YAML frontmatter in each command file.".into(),
        });
    }

    AuditCheckResult {
        name: "Command descriptions".into(),
        passed: findings.is_empty(),
        findings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::InMemoryFileSystem;
    use std::path::Path;

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
        let fs = InMemoryFileSystem::new().with_file("/commands/_archive.md", "# No frontmatter");

        let result = check_command_descriptions(&fs, Path::new("/commands"));
        assert!(result.passed);
    }

    // --- check_pattern_count ---

    #[test]
    fn check_pattern_count_reports() {
        // patterns/testing has 1 md file → check should produce a finding or at least report
        let fs = InMemoryFileSystem::new()
            .with_file("/patterns/testing/pattern1.md", "# Pattern 1")
            .with_file("/patterns/testing/pattern2.md", "# Pattern 2");

        let result = check_pattern_count(&fs, Path::new("/patterns"));
        // The check runs without panic, returns a valid result
        assert_eq!(result.name, "Pattern count");
        // With patterns present the check should pass (findings only on problems)
        assert!(result.passed, "expected passed=true for populated patterns dir");
    }
}
