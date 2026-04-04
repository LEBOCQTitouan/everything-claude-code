//! `validate_cartography` use case.
//!
//! Scans `docs/cartography/journeys/*.md` and `docs/cartography/flows/*.md`,
//! validates each file's schema, checks staleness via `git log`, and
//! optionally outputs a coverage dashboard.

use ecc_domain::cartography::{
    calculate_coverage, check_staleness, parse_cartography_meta,
};
use ecc_domain::cartography::validation::{validate_flow, validate_journey};
use ecc_ports::fs::FileSystem;
use ecc_ports::shell::ShellExecutor;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

/// Run the validate-cartography use case.
///
/// - Scans `docs/cartography/journeys/` and `docs/cartography/flows/`
/// - Validates each `*.md` file against its schema (journey or flow)
/// - Checks staleness for files with `CARTOGRAPHY-META` markers
/// - When `coverage` is true, outputs a coverage dashboard
///
/// Returns `true` if all files are schema-valid, `false` if any errors found.
pub fn run_validate_cartography(
    fs: &dyn FileSystem,
    shell: &dyn ShellExecutor,
    terminal: &dyn TerminalIO,
    project_root: &Path,
    coverage: bool,
) -> bool {
    let journeys_dir = project_root.join("docs/cartography/journeys");
    let flows_dir = project_root.join("docs/cartography/flows");

    let mut has_errors = false;
    let mut all_content: Vec<(String, String)> = Vec::new(); // (path_str, content)

    // Validate journey files
    if let Ok(entries) = fs.read_dir(&journeys_dir) {
        for entry in entries {
            let path_str = entry.to_string_lossy().to_string();
            if !path_str.ends_with(".md") {
                continue;
            }
            if let Ok(content) = fs.read_to_string(&entry) {
                if let Err(missing) = validate_journey(&content) {
                    let file_name = entry
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| path_str.clone());
                    let sections: Vec<String> = missing;
                    terminal.stdout_write(&format!(
                        "ERROR [journey] {file_name}: missing sections: {}\n",
                        sections.join(", ")
                    ));
                    has_errors = true;
                }
                all_content.push((path_str, content));
            }
        }
    }

    // Validate flow files
    if let Ok(entries) = fs.read_dir(&flows_dir) {
        for entry in entries {
            let path_str = entry.to_string_lossy().to_string();
            if !path_str.ends_with(".md") {
                continue;
            }
            if let Ok(content) = fs.read_to_string(&entry) {
                if let Err(missing) = validate_flow(&content) {
                    let file_name = entry
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| path_str.clone());
                    let sections: Vec<String> = missing;
                    terminal.stdout_write(&format!(
                        "ERROR [flow] {file_name}: missing sections: {}\n",
                        sections.join(", ")
                    ));
                    has_errors = true;
                }
                all_content.push((path_str, content));
            }
        }
    }

    // Check staleness for files with CARTOGRAPHY-META markers
    report_staleness(shell, terminal, &all_content);

    // Coverage dashboard
    if coverage {
        report_coverage(fs, terminal, project_root, &all_content);
    }

    !has_errors
}

/// Check staleness for all cartography files that have a CARTOGRAPHY-META marker.
fn report_staleness(
    shell: &dyn ShellExecutor,
    terminal: &dyn TerminalIO,
    all_content: &[(String, String)],
) {
    for (_path_str, content) in all_content {
        let Some(meta) = parse_cartography_meta(content) else {
            continue;
        };

        // Gather source modification dates via git log
        let mut source_dates: Vec<(String, String)> = Vec::new();
        for source in &meta.sources {
            let result = shell.run_command("git", &["log", "-1", "--format=%Y-%m-%d", source]);
            if let Ok(output) = result {
                let date = output.stdout.trim().to_string();
                if !date.is_empty() {
                    source_dates.push((source.clone(), date));
                }
            }
        }

        let dates_ref: Vec<(&str, &str)> = source_dates
            .iter()
            .map(|(p, d)| (p.as_str(), d.as_str()))
            .collect();

        if let Some(stale_marker) = check_staleness(content, &dates_ref) {
            // Parse the delta in days from last_updated and source_modified
            let delta_days = compute_staleness_days(&meta.last_updated, &source_dates);
            terminal.stdout_write(&format!(
                "STALE: {stale_marker} ({delta_days} days)\n"
            ));
        }
    }
}

/// Compute the number of days between last_updated and the most recent source modification.
fn compute_staleness_days(last_updated: &str, source_dates: &[(String, String)]) -> i64 {
    let Some(most_recent) = source_dates.iter().map(|(_, d)| d.as_str()).max() else {
        return 0;
    };

    let lu = parse_date(last_updated);
    let sm = parse_date(most_recent);
    (sm - lu).max(0)
}

/// Parse a YYYY-MM-DD date string into days since an arbitrary epoch.
///
/// Returns an integer suitable for subtraction (not a real calendar days count
/// for non-Gregorian edge cases, but sufficient for day-delta reporting).
fn parse_date(date: &str) -> i64 {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return 0;
    }
    let y: i64 = parts[0].parse().unwrap_or(0);
    let m: i64 = parts[1].parse().unwrap_or(0);
    let d: i64 = parts[2].parse().unwrap_or(0);
    // Approximate Julian Day Number (sufficient for delta reporting)
    let a = (14 - m) / 12;
    let y2 = y + 4800 - a;
    let m2 = m + 12 * a - 3;
    d + (153 * m2 + 2) / 5 + 365 * y2 + y2 / 4 - y2 / 100 + y2 / 400 - 32045
}

/// Compute and print the coverage dashboard.
fn report_coverage(
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
    project_root: &Path,
    cartography_content: &[(String, String)],
) {
    let src_dir = project_root.join("src");
    let source_files = collect_source_files(fs, &src_dir);

    // Extract referenced files from cartography content
    let referenced = extract_referenced_files(cartography_content);

    let report = calculate_coverage(&source_files, &referenced, &[]);

    terminal.stdout_write(&format!(
        "Coverage: {}/{} source files referenced ({:.1}%)\n",
        report.referenced, report.total, report.percentage
    ));

    if !report.priority_gaps.is_empty() {
        terminal.stdout_write("Priority gaps (top unreferenced files by change frequency):\n");
        for gap in &report.priority_gaps {
            terminal.stdout_write(&format!("  - {gap}\n"));
        }
    }
}

/// Collect source files recursively from a directory.
/// Recognises `.rs`, `.ts`, `.js`, `.py` extensions.
fn collect_source_files(fs: &dyn FileSystem, src_dir: &Path) -> Vec<String> {
    let source_extensions = [".rs", ".ts", ".js", ".py"];

    match fs.read_dir_recursive(src_dir) {
        Ok(paths) => paths
            .into_iter()
            .filter(|p| {
                let s = p.to_string_lossy();
                source_extensions.iter().any(|ext| s.ends_with(ext))
            })
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Extract file references from cartography markdown content.
///
/// Looks for backtick-quoted paths (`path/to/file.rs`) in the content.
fn extract_referenced_files(cartography_content: &[(String, String)]) -> Vec<String> {
    let mut referenced = Vec::new();
    for (_path, content) in cartography_content {
        // Find backtick-quoted paths like `src/main.rs`
        let mut chars = content.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '`' {
                let candidate: String = chars.by_ref().take_while(|&c| c != '`').collect();
                if looks_like_file_path(&candidate) {
                    let canonical = candidate.to_string();
                    if !referenced.contains(&canonical) {
                        referenced.push(canonical);
                    }
                }
            }
        }
    }
    referenced
}

/// Heuristic: a string looks like a file path if it contains a dot and
/// no whitespace, and ends with a known source extension.
fn looks_like_file_path(s: &str) -> bool {
    if s.contains(' ') {
        return false;
    }
    let source_extensions = [".rs", ".ts", ".js", ".py"];
    source_extensions.iter().any(|ext| s.ends_with(ext))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_ports::shell::CommandOutput;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockExecutor};

    fn git_date_output(date: &str) -> CommandOutput {
        CommandOutput {
            stdout: format!("{date}\n"),
            stderr: String::new(),
            exit_code: 0,
        }
    }

    fn valid_journey() -> &'static str {
        "\
# Journey

## Overview
Actor.

## Mermaid Diagram

## Steps
1. Step

## Related Flows
- none
"
    }

    fn valid_flow() -> &'static str {
        "\
# Flow

## Overview
Data.

## Mermaid Diagram

## Source-Destination
Source: A

## Transformation Steps
1. T

## Error Paths
- e
"
    }

    #[test]
    fn returns_true_when_all_files_valid() {
        let fs = InMemoryFileSystem::new()
            .with_file("/p/docs/cartography/journeys/j.md", valid_journey())
            .with_file("/p/docs/cartography/flows/f.md", valid_flow());
        let shell = MockExecutor::new();
        let terminal = BufferedTerminal::new();
        let result = run_validate_cartography(&fs, &shell, &terminal, Path::new("/p"), false);
        assert!(result);
    }

    #[test]
    fn returns_false_when_journey_missing_sections() {
        let fs = InMemoryFileSystem::new()
            .with_file("/p/docs/cartography/journeys/j.md", "# Journey\n\n## Steps\n1. S\n");
        let shell = MockExecutor::new();
        let terminal = BufferedTerminal::new();
        let result = run_validate_cartography(&fs, &shell, &terminal, Path::new("/p"), false);
        assert!(!result);
        let out = terminal.stdout_output().join("");
        assert!(out.contains("Overview") || out.contains("Mermaid Diagram") || out.contains("Related Flows"));
    }

    #[test]
    fn returns_true_when_no_cartography_dirs() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let terminal = BufferedTerminal::new();
        let result = run_validate_cartography(&fs, &shell, &terminal, Path::new("/p"), false);
        assert!(result, "no dirs means no errors");
    }

    fn valid_element() -> &'static str {
        "\
# Element: auth-service

## Overview
The authentication service.

## Responsibilities
- Handle login
- Issue tokens

## Interfaces
- POST /auth/login

## Related Journeys
- user-login-journey
"
    }

    // PC-009
    #[test]
    fn invalid_element_exits_with_error() {
        // Element file missing required sections → validate returns false + prints ERROR
        let fs = InMemoryFileSystem::new()
            .with_file("/p/docs/cartography/elements/bad-element.md", "# Element: bad\n\n## Overview\nSome text.\n");
        let shell = MockExecutor::new();
        let terminal = BufferedTerminal::new();
        let result = run_validate_cartography(&fs, &shell, &terminal, Path::new("/p"), false);
        assert!(!result, "invalid element should cause false return");
        let out = terminal.stdout_output().join("");
        assert!(out.contains("ERROR") && out.contains("element"), "expected element error in output, got: {out}");
    }

    // PC-010
    #[test]
    fn missing_index_warns_not_errors() {
        // elements/ has files but no INDEX.md → warn, not error
        let fs = InMemoryFileSystem::new()
            .with_file("/p/docs/cartography/elements/auth.md", valid_element());
        let shell = MockExecutor::new();
        let terminal = BufferedTerminal::new();
        let result = run_validate_cartography(&fs, &shell, &terminal, Path::new("/p"), false);
        assert!(result, "missing INDEX.md should not cause error");
        let out = terminal.stdout_output().join("");
        assert!(out.contains("WARN") || out.contains("warn") || out.contains("index") || out.contains("INDEX"),
            "expected warning about missing INDEX.md, got: {out}");
    }

    // PC-011
    #[test]
    fn stale_index_warns_missing_slugs() {
        // INDEX.md present but missing a slug → warn with list
        let index_content = "\
# Elements Index

| Slug | Description |
|------|-------------|
| other-element | Some other element |
";
        let fs = InMemoryFileSystem::new()
            .with_file("/p/docs/cartography/elements/auth.md", valid_element())
            .with_file("/p/docs/cartography/elements/INDEX.md", index_content);
        let shell = MockExecutor::new();
        let terminal = BufferedTerminal::new();
        let result = run_validate_cartography(&fs, &shell, &terminal, Path::new("/p"), false);
        assert!(result, "stale INDEX should not cause error");
        let out = terminal.stdout_output().join("");
        assert!(out.contains("WARN") || out.contains("warn") || out.contains("auth"),
            "expected warning about missing slug in INDEX, got: {out}");
    }

    // PC-012
    #[test]
    fn missing_elements_dir_clean_exit() {
        // No elements/ dir at all → returns true, no output about elements
        let fs = InMemoryFileSystem::new()
            .with_file("/p/docs/cartography/journeys/j.md", valid_journey())
            .with_file("/p/docs/cartography/flows/f.md", valid_flow());
        let shell = MockExecutor::new();
        let terminal = BufferedTerminal::new();
        let result = run_validate_cartography(&fs, &shell, &terminal, Path::new("/p"), false);
        assert!(result, "missing elements dir should be clean exit");
    }

    // PC-013
    #[test]
    fn staleness_includes_elements() {
        // Element file with CARTOGRAPHY-META marker → staleness check runs
        let element_with_meta = "\
# Element: auth-service

## Overview
The authentication service.

## Responsibilities
- Handle login

## Interfaces
- POST /auth/login

## Related Journeys
- user-login-journey

<!-- CARTOGRAPHY-META: last_updated=2026-01-01, sources=src/auth.rs -->
";
        let fs = InMemoryFileSystem::new()
            .with_file("/p/docs/cartography/elements/auth.md", element_with_meta);
        let shell = MockExecutor::new()
            .on_args("git", &["log", "-1", "--format=%Y-%m-%d", "src/auth.rs"], git_date_output("2026-06-01"));
        let terminal = BufferedTerminal::new();
        run_validate_cartography(&fs, &shell, &terminal, Path::new("/p"), false);
        let out = terminal.stdout_output().join("");
        assert!(out.contains("STALE"), "expected STALE output for element file, got: {out}");
    }

    // PC-014
    #[test]
    fn coverage_includes_element_sources() {
        // Element referencing a source file → coverage counts it
        let element = "\
# Element: auth-service

## Overview
The authentication service. See `src/auth.rs` for implementation.

## Responsibilities
- Handle login

## Interfaces
- POST /auth/login

## Related Journeys
- user-login-journey
";
        let fs = InMemoryFileSystem::new()
            .with_file("/p/docs/cartography/elements/auth.md", element)
            .with_file("/p/src/auth.rs", "// auth");
        let shell = MockExecutor::new();
        let terminal = BufferedTerminal::new();
        run_validate_cartography(&fs, &shell, &terminal, Path::new("/p"), true);
        let out = terminal.stdout_output().join("");
        // auth.rs is referenced in element and exists in src/ → referenced count >= 1
        assert!(out.contains("Coverage:"), "expected coverage output, got: {out}");
        // With 1 file and 1 referenced → 100%
        assert!(out.contains("100") || out.contains("1/1"), "expected 100% coverage, got: {out}");
    }

    // PC-015
    #[test]
    fn low_coverage_includes_all_gap_types() {
        // Multiple src files, only one referenced across journeys+flows+elements
        // → coverage < 50% → priority gaps printed
        let journey_ref = "\
# Journey

## Overview
See `src/a.rs`.

## Mermaid Diagram

## Steps
1. Step

## Related Flows
- none
";
        let fs = InMemoryFileSystem::new()
            .with_file("/p/docs/cartography/journeys/j.md", journey_ref)
            .with_file("/p/src/a.rs", "// a")
            .with_file("/p/src/b.rs", "// b")
            .with_file("/p/src/c.rs", "// c")
            .with_file("/p/src/d.rs", "// d");
        let shell = MockExecutor::new();
        let terminal = BufferedTerminal::new();
        run_validate_cartography(&fs, &shell, &terminal, Path::new("/p"), true);
        let out = terminal.stdout_output().join("");
        assert!(out.contains("Coverage:"), "expected coverage output");
        // 1/4 = 25% < 50% → priority gaps shown
        assert!(out.contains("Priority gaps") || out.contains("gap") || out.contains("src/b.rs") || out.contains("src/c.rs"),
            "expected priority gaps output for low coverage, got: {out}");
    }
}
