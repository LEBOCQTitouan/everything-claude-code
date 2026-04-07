//! Drift check use case — orchestrates spec-vs-implementation drift detection.

use ecc_domain::drift::{self, DriftReport};
use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

/// Run drift check. Returns true if no high drift, false on error.
pub fn run_drift_check(
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
    spec_dir: &Path,
    json: bool,
) -> bool {
    // Try spec.md, fall back to plan.md
    let spec_content = read_first_existing(fs, spec_dir, &["spec.md", "plan.md"]);
    let design_content = read_first_existing(fs, spec_dir, &["design.md", "solution.md"]);
    let done_path = Path::new(".claude/workflow/implement-done.md");
    let _done_content = fs.read_to_string(done_path).ok();

    if spec_content.is_none() {
        terminal.stderr_write("ERROR: No spec.md or plan.md found\n");
        return false;
    }

    let spec = spec_content.unwrap();
    let acs = drift::extract_ac_ids(&spec);

    let pc_coverage = design_content
        .as_deref()
        .map(drift::extract_pc_ac_coverage)
        .unwrap_or_default();

    // Extract file expectations from design (simplified — looks for file paths in tables)
    let expected_files: Vec<String> = Vec::new(); // TODO: parse from design
    let actual_files: Vec<String> = Vec::new(); // TODO: parse from implement-done.md

    let report = drift::compute_drift(&acs, &pc_coverage, &expected_files, &actual_files);

    if json {
        let json_out = format!(
            "{{\"level\":\"{}\",\"unimplemented_acs\":{},\"total_acs\":{},\"covered_acs\":{}}}",
            report.level.map(|l| l.to_string()).unwrap_or_default(),
            report.unimplemented_acs.len(),
            report.total_acs,
            report.covered_acs,
        );
        terminal.stdout_write(&format!("{json_out}\n"));
    } else {
        output_report(terminal, &report);
    }

    // Write drift-report.md
    let report_content = format_report_md(&report);
    let report_path = Path::new(".claude/workflow/drift-report.md");
    if let Some(parent) = report_path.parent() {
        let _ = fs.create_dir_all(parent);
    }
    let _ = fs.write(report_path, &report_content);

    !matches!(report.level, Some(drift::DriftLevel::High))
}

fn read_first_existing(fs: &dyn FileSystem, dir: &Path, names: &[&str]) -> Option<String> {
    for name in names {
        let path = dir.join(name);
        if let Ok(content) = fs.read_to_string(&path) {
            return Some(content);
        }
    }
    None
}

fn output_report(terminal: &dyn TerminalIO, report: &DriftReport) {
    let level = report.level.map(|l| l.to_string()).unwrap_or_default();
    terminal.stdout_write(&format!("Drift level: {level}\n"));
    terminal.stdout_write(&format!(
        "ACs: {}/{} covered\n",
        report.covered_acs, report.total_acs
    ));
    if !report.unimplemented_acs.is_empty() {
        terminal.stdout_write("Unimplemented ACs:\n");
        for ac in &report.unimplemented_acs {
            terminal.stdout_write(&format!("  - {ac}\n"));
        }
    }
}

fn format_report_md(report: &DriftReport) -> String {
    let level = report.level.map(|l| l.to_string()).unwrap_or_default();
    let mut md = format!("# Drift Report\n\nLevel: **{level}**\n\n");
    md.push_str(&format!(
        "ACs: {}/{} covered\n\n",
        report.covered_acs, report.total_acs
    ));
    if !report.unimplemented_acs.is_empty() {
        md.push_str("## Unimplemented ACs\n\n");
        for ac in &report.unimplemented_acs {
            md.push_str(&format!("- {ac}\n"));
        }
    }
    md
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem};

    #[test]
    fn drift_check_missing_spec_returns_false() {
        let fs = InMemoryFileSystem::new();
        let term = BufferedTerminal::new();
        let result = run_drift_check(&fs, &term, Path::new("/specs"), false);
        assert!(!result);
        assert!(term.stderr_output().join("").contains("ERROR"));
    }

    #[test]
    fn drift_check_with_spec_produces_report() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/specs")
            .with_dir(".claude/workflow")
            .with_file("/specs/spec.md", "- AC-001.1: test\n- AC-001.2: test2")
            .with_file(
                "/specs/design.md",
                "| PC-001 | unit | test | AC-001.1 | cmd | PASS |",
            );
        let term = BufferedTerminal::new();
        let result = run_drift_check(&fs, &term, Path::new("/specs"), false);
        assert!(result); // Not HIGH drift
        let output = term.stdout_output().join("");
        assert!(output.contains("Drift level:"));
    }

    #[test]
    fn drift_check_json_output() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/specs")
            .with_dir(".claude/workflow")
            .with_file("/specs/spec.md", "- AC-001.1: test");
        let term = BufferedTerminal::new();
        run_drift_check(&fs, &term, Path::new("/specs"), true);
        let output = term.stdout_output().join("");
        assert!(output.contains("\"level\""));
    }
}
