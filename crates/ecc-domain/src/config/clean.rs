/// Artifact directory names relative to the Claude config directory.
pub const ARTIFACT_DIRS: &[&str] = &["agents", "commands", "skills", "rules", "teams"];

/// Report of what was removed, skipped, or errored during cleanup.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CleanReport {
    /// Items successfully removed.
    pub removed: Vec<String>,
    /// Items skipped (not found or protected).
    pub skipped: Vec<String>,
    /// Error messages from failed removal attempts.
    pub errors: Vec<String>,
}

impl CleanReport {
    /// Create a new empty clean report.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Format a clean report for display.
pub fn format_clean_report(report: &CleanReport, dry_run: bool) -> String {
    let mut lines = Vec::new();
    let prefix = if dry_run {
        "[DRY RUN] Would remove"
    } else {
        "Removed"
    };

    if !report.removed.is_empty() {
        lines.push(format!("\n{prefix}:"));
        for item in &report.removed {
            lines.push(format!("  - {item}"));
        }
    }

    if !report.skipped.is_empty() {
        lines.push("\nSkipped (not found):".to_string());
        for item in &report.skipped {
            lines.push(format!("  - {item}"));
        }
    }

    if !report.errors.is_empty() {
        lines.push("\nErrors:".to_string());
        for item in &report.errors {
            lines.push(format!("  - {item}"));
        }
    }

    lines.push(format!(
        "\nClean summary: {} removed, {} skipped, {} errors",
        report.removed.len(),
        report.skipped.len(),
        report.errors.len()
    ));

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- format_clean_report ---

    #[test]
    fn format_clean_report_normal() {
        let report = CleanReport {
            removed: vec!["agents/planner.md".into(), "commands/plan.md".into()],
            skipped: vec!["skills/missing".into()],
            errors: vec![],
        };

        let output = format_clean_report(&report, false);
        assert!(output.contains("Removed:"));
        assert!(output.contains("agents/planner.md"));
        assert!(output.contains("Skipped (not found):"));
        assert!(output.contains("skills/missing"));
        assert!(output.contains("Clean summary: 2 removed, 1 skipped, 0 errors"));
    }

    #[test]
    fn format_clean_report_dry_run() {
        let report = CleanReport {
            removed: vec!["agents/planner.md".into()],
            skipped: vec![],
            errors: vec![],
        };

        let output = format_clean_report(&report, true);
        assert!(output.contains("[DRY RUN] Would remove:"));
    }

    #[test]
    fn format_clean_report_with_errors() {
        let report = CleanReport {
            removed: vec![],
            skipped: vec![],
            errors: vec!["settings.json: parse error".into()],
        };

        let output = format_clean_report(&report, false);
        assert!(output.contains("Errors:"));
        assert!(output.contains("settings.json: parse error"));
        assert!(output.contains("0 removed, 0 skipped, 1 errors"));
    }
}
