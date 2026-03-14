//! Detection use case — thin wrapper around domain detect + report.

use ecc_domain::config::detect::{self, DetectionResult};
use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

/// Detect existing Claude Code configuration and print a report.
///
/// Returns the detection result for further use by callers.
pub fn detect_and_report(
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
    claude_dir: &Path,
    project_dir: Option<&Path>,
) -> DetectionResult {
    let result = detect::detect(fs, claude_dir, project_dir);
    let report = detect::generate_report(&result);
    terminal.stdout_write(&format!("{report}\n"));
    result
}

/// Check if the detection result indicates an empty (first-time) setup.
pub fn is_empty_setup(result: &DetectionResult) -> bool {
    result.agents.is_empty()
        && result.commands.is_empty()
        && result.skills.is_empty()
        && result.rules.is_empty()
        && result.hooks.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem};
    use std::path::Path;

    #[test]
    fn detect_and_report_empty_setup() {
        let fs = InMemoryFileSystem::new().with_dir("/claude");
        let terminal = BufferedTerminal::new();

        let result = detect_and_report(&fs, &terminal, Path::new("/claude"), None);

        assert!(is_empty_setup(&result));
        let output = terminal.stdout_output().join("");
        assert!(output.contains("no existing configuration found"));
    }

    #[test]
    fn detect_and_report_with_agents() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/claude/agents")
            .with_file(
                "/claude/agents/planner.md",
                "---\nname: planner\n---\n# Planner",
            );
        let terminal = BufferedTerminal::new();

        let result = detect_and_report(&fs, &terminal, Path::new("/claude"), None);

        assert!(!is_empty_setup(&result));
        assert_eq!(result.agents.len(), 1);
        let output = terminal.stdout_output().join("");
        assert!(output.contains("Agents:"));
        assert!(output.contains("1 found"));
    }

    #[test]
    fn detect_and_report_with_project_dir() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/claude")
            .with_file("/project/CLAUDE.md", "# Project\n## Architecture\n## Testing");
        let terminal = BufferedTerminal::new();

        let result = detect_and_report(
            &fs,
            &terminal,
            Path::new("/claude"),
            Some(Path::new("/project")),
        );

        assert!(result.has_claude_md);
        let output = terminal.stdout_output().join("");
        assert!(output.contains("CLAUDE.md: exists"));
    }

    #[test]
    fn is_empty_setup_false_with_rules() {
        let mut rules = std::collections::BTreeMap::new();
        rules.insert("common".to_string(), vec!["style.md".to_string()]);

        let result = DetectionResult {
            agents: vec![],
            commands: vec![],
            skills: vec![],
            rules,
            hooks: vec![],
            claude_md_headings: vec![],
            has_settings_json: false,
            has_claude_md: false,
        };

        assert!(!is_empty_setup(&result));
    }

    #[test]
    fn detect_and_report_prints_output() {
        let fs = InMemoryFileSystem::new().with_dir("/claude");
        let terminal = BufferedTerminal::new();

        detect_and_report(&fs, &terminal, Path::new("/claude"), None);

        let output = terminal.stdout_output();
        assert!(!output.is_empty());
    }
}
