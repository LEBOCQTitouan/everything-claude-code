//! Structural audit checks — gitignore, CLAUDE.md, statusline.

use ecc_domain::config::audit::{AuditCheckResult, AuditFinding, Severity};
use ecc_ports::fs::FileSystem;
use std::path::Path;

use super::super::read_json_safe;

/// Check that ECC gitignore entries are present in .gitignore.
pub fn check_gitignore(fs: &dyn FileSystem, project_dir: &Path) -> AuditCheckResult {
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

/// Check if global CLAUDE.md exists.
pub fn check_global_claude_md(fs: &dyn FileSystem, claude_dir: &Path) -> AuditCheckResult {
    let mut findings = Vec::new();
    let claude_md_path = claude_dir.join("CLAUDE.md");

    if !fs.exists(&claude_md_path) {
        findings.push(AuditFinding {
            id: "CMD-001".into(),
            severity: Severity::Medium,
            title: "No global ~/.claude/CLAUDE.md".into(),
            detail: "Critical cross-project instructions only load when rules match file paths."
                .into(),
            fix: "Create ~/.claude/CLAUDE.md with a 50-80 line summary of key rules.".into(),
        });
    }

    AuditCheckResult {
        name: "Global CLAUDE.md".into(),
        passed: findings.is_empty(),
        findings,
    }
}

/// Check that the ECC statusline script is installed and settings reference it.
pub fn check_statusline(fs: &dyn FileSystem, claude_dir: &Path) -> AuditCheckResult {
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
    if let Ok(Some(settings)) = read_json_safe(fs, &settings_path) {
        let has_ecc_statusline = settings
            .get("statusLine")
            .and_then(|sl| sl.get("command"))
            .and_then(|c| c.as_str())
            .is_some_and(|cmd| {
                cmd.contains(ecc_domain::config::statusline::STATUSLINE_SCRIPT_FILENAME)
            });

        if !has_ecc_statusline && findings.is_empty() {
            findings.push(AuditFinding {
                id: "SL-002".into(),
                severity: Severity::Low,
                title: "Statusline not configured in settings.json".into(),
                detail: "Script exists but settings.json doesn't reference it.".into(),
                fix: "Run `ecc install` to configure the statusline.".into(),
            });
        }
    }

    AuditCheckResult {
        name: "Statusline".into(),
        passed: findings.is_empty(),
        findings,
    }
}

/// Check project CLAUDE.md line count.
pub fn check_project_claude_md(fs: &dyn FileSystem, project_dir: &Path) -> AuditCheckResult {
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
            detail: "Large CLAUDE.md files consume context budget on every conversation.".into(),
            fix: "Move detailed instructions to rules/ or skills/ and keep CLAUDE.md lean.".into(),
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

    // --- check_gitignore ---

    #[test]
    fn check_gitignore_all_present() {
        let content: String = ecc_domain::config::gitignore::ECC_GITIGNORE_ENTRIES
            .iter()
            .map(|e| e.pattern)
            .collect::<Vec<_>>()
            .join("\n");
        let fs = InMemoryFileSystem::new().with_file("/project/.gitignore", &content);

        let result = check_gitignore(&fs, Path::new("/project"));
        assert!(result.passed);
    }

    #[test]
    fn check_gitignore_missing_entries() {
        let fs = InMemoryFileSystem::new().with_file("/project/.gitignore", "node_modules\n");

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

    // --- check_global_claude_md ---

    #[test]
    fn check_global_claude_md_exists() {
        let fs = InMemoryFileSystem::new().with_file("/claude/CLAUDE.md", "# Global instructions");

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
        let fs =
            InMemoryFileSystem::new().with_file("/claude/settings.json", &settings.to_string());

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
        let settings = serde_json::json!({
            "statusLine": {"command": "my-custom.sh"}
        });
        let fs =
            InMemoryFileSystem::new().with_file("/claude/settings.json", &settings.to_string());

        let result = check_statusline(&fs, Path::new("/claude"));
        assert!(!result.passed);
        assert_eq!(result.findings.len(), 1);
        assert_eq!(result.findings[0].id, "SL-001");
    }

    // --- check_project_claude_md ---

    #[test]
    fn check_project_claude_md_small() {
        let content = "# Title\nSome content\n";
        let fs = InMemoryFileSystem::new().with_file("/project/CLAUDE.md", content);

        let result = check_project_claude_md(&fs, Path::new("/project"));
        assert!(result.passed);
    }

    #[test]
    fn check_project_claude_md_large() {
        let content = (0..250)
            .map(|i| format!("Line {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let fs = InMemoryFileSystem::new().with_file("/project/CLAUDE.md", &content);

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
