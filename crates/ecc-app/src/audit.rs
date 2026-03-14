//! Audit use case — runs domain checks and formats colored report.

use ecc_domain::ansi;
use ecc_domain::config::audit::{AuditFinding, AuditReport, Severity};
use ecc_ports::env::Environment;
use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

/// Options for running an audit.
#[derive(Debug, Clone)]
pub struct AuditOptions<'a> {
    pub claude_dir: &'a Path,
    pub project_dir: &'a Path,
    pub ecc_root: &'a Path,
}

/// Run a full ECC configuration audit.
///
/// Returns `false` if any critical findings were detected.
pub fn run_audit(
    fs: &dyn FileSystem,
    env: &dyn Environment,
    terminal: &dyn TerminalIO,
    options: &AuditOptions,
) -> bool {
    let colored = env.var("NO_COLOR").is_none();

    terminal.stdout_write(&format!(
        "\n{}\n\n",
        ansi::bold("ECC Configuration Audit", colored)
    ));

    let report = crate::config::audit::run_all_checks(
        fs,
        options.claude_dir,
        options.project_dir,
        options.ecc_root,
    );

    print_report(terminal, &report, colored);

    !has_critical(&report)
}

/// Format a single finding for display.
pub fn format_finding(finding: &AuditFinding, colored: bool) -> String {
    let severity_label = match finding.severity {
        Severity::Critical => ansi::red("CRITICAL", colored),
        Severity::High => ansi::yellow("HIGH", colored),
        Severity::Medium => ansi::cyan("MEDIUM", colored),
        Severity::Low => ansi::dim("LOW", colored),
    };

    format!(
        "  [{severity_label}] {}\n    {}\n    Fix: {}",
        finding.title, finding.detail, finding.fix
    )
}

fn print_report(terminal: &dyn TerminalIO, report: &AuditReport, colored: bool) {
    for check in &report.checks {
        let status = if check.passed {
            ansi::green("PASS", colored)
        } else {
            ansi::red("FAIL", colored)
        };
        terminal.stdout_write(&format!("[{status}] {}\n", check.name));

        for finding in &check.findings {
            terminal.stdout_write(&format!("{}\n", format_finding(finding, colored)));
        }
    }

    let grade_colored = match report.grade.as_str() {
        "A" => ansi::green(&report.grade, colored),
        "B" => ansi::green(&report.grade, colored),
        "C" => ansi::yellow(&report.grade, colored),
        "D" => ansi::yellow(&report.grade, colored),
        _ => ansi::red(&report.grade, colored),
    };

    terminal.stdout_write(&format!(
        "\nScore: {}/100  Grade: {grade_colored}\n",
        report.score
    ));
}

fn has_critical(report: &AuditReport) -> bool {
    report
        .checks
        .iter()
        .flat_map(|c| &c.findings)
        .any(|f| f.severity == Severity::Critical)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_domain::ansi::strip_ansi;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment};
    use std::path::Path;

    fn setup_passing_fs() -> InMemoryFileSystem {
        let settings = serde_json::json!({
            "permissions": {
                "deny": ecc_domain::config::deny_rules::ECC_DENY_RULES
            }
        });
        let gitignore_entries: Vec<&str> = ecc_domain::config::gitignore::ECC_GITIGNORE_ENTRIES
            .iter()
            .map(|e| e.pattern)
            .collect();

        InMemoryFileSystem::new()
            .with_file(
                "/claude/settings.json",
                &serde_json::to_string(&settings).unwrap(),
            )
            .with_file("/claude/CLAUDE.md", "# Global\n## Instructions\nContent here")
            .with_file(
                "/project/.gitignore",
                &gitignore_entries.join("\n"),
            )
            .with_file("/project/CLAUDE.md", "# Project\n## Architecture\nContent")
            .with_dir("/ecc/agents")
            .with_dir("/ecc/commands")
    }

    #[test]
    fn run_audit_passing_returns_true() {
        let fs = setup_passing_fs();
        let env = MockEnvironment::new();
        let terminal = BufferedTerminal::new();
        let options = AuditOptions {
            claude_dir: Path::new("/claude"),
            project_dir: Path::new("/project"),
            ecc_root: Path::new("/ecc"),
        };

        let result = run_audit(&fs, &env, &terminal, &options);
        assert!(result);

        let output = terminal.stdout_output().join("");
        assert!(output.contains("Score:"));
        assert!(output.contains("Grade:"));
    }

    #[test]
    fn run_audit_critical_findings_returns_false() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/ecc/agents")
            .with_dir("/ecc/commands");
        let env = MockEnvironment::new();
        let terminal = BufferedTerminal::new();
        let options = AuditOptions {
            claude_dir: Path::new("/claude"),
            project_dir: Path::new("/project"),
            ecc_root: Path::new("/ecc"),
        };

        let result = run_audit(&fs, &env, &terminal, &options);
        assert!(!result);

        let output = terminal.stdout_output().join("");
        assert!(output.contains("FAIL"));
        assert!(output.contains("CRITICAL"));
    }

    #[test]
    fn run_audit_respects_no_color() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/ecc/agents")
            .with_dir("/ecc/commands");
        let env = MockEnvironment::new().with_var("NO_COLOR", "1");
        let terminal = BufferedTerminal::new();
        let options = AuditOptions {
            claude_dir: Path::new("/claude"),
            project_dir: Path::new("/project"),
            ecc_root: Path::new("/ecc"),
        };

        run_audit(&fs, &env, &terminal, &options);

        let output = terminal.stdout_output().join("");
        // No ANSI escape sequences when NO_COLOR is set
        assert_eq!(output, strip_ansi(&output));
    }

    #[test]
    fn format_finding_critical() {
        let finding = AuditFinding {
            id: "TEST-001".into(),
            severity: Severity::Critical,
            title: "Something bad".into(),
            detail: "Details here".into(),
            fix: "Fix it".into(),
        };
        let output = format_finding(&finding, false);
        assert!(output.contains("CRITICAL"));
        assert!(output.contains("Something bad"));
        assert!(output.contains("Details here"));
        assert!(output.contains("Fix: Fix it"));
    }

    #[test]
    fn format_finding_high() {
        let finding = AuditFinding {
            id: "TEST-002".into(),
            severity: Severity::High,
            title: "Warning".into(),
            detail: "Info".into(),
            fix: "Do this".into(),
        };
        let output = format_finding(&finding, false);
        assert!(output.contains("HIGH"));
    }

    #[test]
    fn format_finding_medium() {
        let finding = AuditFinding {
            id: "TEST-003".into(),
            severity: Severity::Medium,
            title: "Note".into(),
            detail: "Info".into(),
            fix: "Optional".into(),
        };
        let output = format_finding(&finding, false);
        assert!(output.contains("MEDIUM"));
    }

    #[test]
    fn format_finding_low() {
        let finding = AuditFinding {
            id: "TEST-004".into(),
            severity: Severity::Low,
            title: "Minor".into(),
            detail: "Info".into(),
            fix: "Maybe".into(),
        };
        let output = format_finding(&finding, false);
        assert!(output.contains("LOW"));
    }

    #[test]
    fn grade_output_shown() {
        let fs = setup_passing_fs();
        let env = MockEnvironment::new();
        let terminal = BufferedTerminal::new();
        let options = AuditOptions {
            claude_dir: Path::new("/claude"),
            project_dir: Path::new("/project"),
            ecc_root: Path::new("/ecc"),
        };

        run_audit(&fs, &env, &terminal, &options);

        let output = terminal.stdout_output().join("");
        // Should show score and grade
        assert!(output.contains("/100"));
    }

    #[test]
    fn audit_header_shown() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/ecc/agents")
            .with_dir("/ecc/commands");
        let env = MockEnvironment::new();
        let terminal = BufferedTerminal::new();
        let options = AuditOptions {
            claude_dir: Path::new("/claude"),
            project_dir: Path::new("/project"),
            ecc_root: Path::new("/ecc"),
        };

        run_audit(&fs, &env, &terminal, &options);

        let output = terminal.stdout_output().join("");
        assert!(output.contains("ECC Configuration Audit"));
    }
}
