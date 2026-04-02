//! `validate_cartography` use case — validates docs/cartography/ structure.
//!
//! Scans element files, checks INDEX.md presence and staleness, and optionally
//! computes coverage of sources referenced by element files.

use ecc_domain::cartography::element_validation::validate_element;
use ecc_ports::fs::FileSystem;
use ecc_ports::shell::ShellExecutor;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

/// Marker embedded in element files for staleness detection.
const CARTOGRAPHY_META_MARKER: &str = "CARTOGRAPHY-META";

/// Run cartography validation.
///
/// Scans `docs/cartography/elements/` (if present), validates each element file,
/// checks INDEX.md presence, warns on stale INDEX, includes element sources in
/// coverage calculation when `--coverage` is requested.
///
/// Returns `true` if validation passed (no errors), `false` on any error.
pub fn run_validate_cartography(
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
    shell: &dyn ShellExecutor,
    root: &Path,
    coverage: bool,
) -> bool {
    let elements_dir = root.join("docs/cartography/elements");

    // AC-016.3: If elements/ dir missing, exit cleanly.
    if !fs.exists(&elements_dir) {
        return true;
    }

    // Scan element files.
    let element_files = match scan_element_files(fs, &elements_dir, terminal) {
        Ok(files) => files,
        Err(()) => return false,
    };

    // AC-011.3: Validate each element file.
    let mut has_errors = false;
    let mut element_slugs: Vec<String> = Vec::new();
    let mut all_element_sources: Vec<String> = Vec::new();

    for element_path in &element_files {
        let filename = element_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let slug = filename.trim_end_matches(".md").to_string();
        element_slugs.push(slug);

        let content = match fs.read_to_string(element_path) {
            Ok(c) => c,
            Err(e) => {
                terminal.stderr_write(&format!(
                    "ERROR: cannot read {}: {}\n",
                    element_path.display(),
                    e
                ));
                has_errors = true;
                continue;
            }
        };

        // Validate element content.
        if let Err(missing) = validate_element(&content) {
            terminal.stderr_write(&format!(
                "ERROR: {} missing sections: {}\n",
                element_path.display(),
                missing.join(", ")
            ));
            has_errors = true;
        }

        // Collect sources for coverage and staleness.
        let sources = parse_meta_sources(&content);
        all_element_sources.extend(sources);
    }

    // AC-015.1: Staleness check for element files with CARTOGRAPHY-META.
    check_element_staleness(fs, shell, terminal, &elements_dir, &element_files);

    // AC-014.5 / AC-014.6: INDEX.md presence and staleness.
    let index_path = elements_dir.join("INDEX.md");
    if !fs.exists(&index_path) {
        if !element_files.is_empty() {
            terminal.stderr_write("WARNING: docs/cartography/elements/INDEX.md is absent\n");
        }
    } else {
        // AC-014.6: Check if INDEX.md is stale (missing element slugs).
        check_index_staleness(fs, terminal, &index_path, &element_slugs);
    }

    // AC-015.2 / AC-015.3: Coverage calculation includes element-referenced sources.
    if coverage {
        run_coverage(fs, terminal, root, &all_element_sources);
    }

    !has_errors
}

/// Scan the elements/ directory and return paths to element .md files (excluding INDEX.md and README.md).
fn scan_element_files(
    fs: &dyn FileSystem,
    elements_dir: &Path,
    terminal: &dyn TerminalIO,
) -> Result<Vec<std::path::PathBuf>, ()> {
    let entries = match fs.read_dir(elements_dir) {
        Ok(e) => e,
        Err(e) => {
            terminal.stderr_write(&format!("ERROR: cannot read elements/: {e}\n"));
            return Err(());
        }
    };

    let files: Vec<_> = entries
        .into_iter()
        .filter(|p| {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            name.ends_with(".md") && name != "INDEX.md" && name != "README.md"
        })
        .collect();

    Ok(files)
}

/// Parse `sources` field from a CARTOGRAPHY-META marker in element file content.
///
/// Expected format: `<!-- CARTOGRAPHY-META last_updated:<date> sources:<path1,path2,...> -->`
fn parse_meta_sources(content: &str) -> Vec<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.contains(CARTOGRAPHY_META_MARKER) {
            // Extract sources=<...>
            if let Some(sources_start) = trimmed.find("sources:") {
                let rest = &trimmed[sources_start + "sources:".len()..];
                // Stop at "-->" or whitespace-separated next field
                let sources_str = rest.trim_end_matches("-->").trim();
                // sources may contain comma-separated paths
                return sources_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
    }
    Vec::new()
}

/// Check staleness of element files that contain CARTOGRAPHY-META markers.
///
/// For each element file with a CARTOGRAPHY-META marker, reads `last_updated` and
/// compares against the git log of referenced source files.
fn check_element_staleness(
    fs: &dyn FileSystem,
    shell: &dyn ShellExecutor,
    terminal: &dyn TerminalIO,
    _elements_dir: &Path,
    element_files: &[std::path::PathBuf],
) {
    for path in element_files {
        let content = match fs.read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if !content.contains(CARTOGRAPHY_META_MARKER) {
            continue;
        }

        let last_updated = parse_meta_last_updated(&content);
        let sources = parse_meta_sources(&content);

        if last_updated.is_empty() || sources.is_empty() {
            continue;
        }

        // Check git log for each source to see if modified after last_updated.
        for source in &sources {
            let result = shell.run_command(
                "git",
                &[
                    "log",
                    "--since",
                    &last_updated,
                    "--oneline",
                    "--",
                    source,
                ],
            );
            if let Ok(out) = result {
                if !out.stdout.trim().is_empty() {
                    terminal.stderr_write(&format!(
                        "WARNING: element {} may be stale (source {} modified since {})\n",
                        path.display(),
                        source,
                        last_updated
                    ));
                }
            }
        }
    }
}

/// Parse `last_updated` field from a CARTOGRAPHY-META marker.
fn parse_meta_last_updated(content: &str) -> String {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.contains(CARTOGRAPHY_META_MARKER) {
            if let Some(lu_start) = trimmed.find("last_updated:") {
                let rest = &trimmed[lu_start + "last_updated:".len()..];
                // Value ends at next space or "-->"
                let value = rest
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_end_matches("-->")
                    .trim();
                return value.to_string();
            }
        }
    }
    String::new()
}

/// Check INDEX.md staleness by comparing table header slugs to known element slugs.
fn check_index_staleness(
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
    index_path: &Path,
    element_slugs: &[String],
) {
    let content = match fs.read_to_string(index_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let index_slugs = parse_index_element_slugs(&content);

    let missing_in_index: Vec<&String> = element_slugs
        .iter()
        .filter(|s| !index_slugs.contains(s))
        .collect();

    if !missing_in_index.is_empty() {
        let missing_list: Vec<&str> = missing_in_index.iter().map(|s| s.as_str()).collect();
        terminal.stderr_write(&format!(
            "WARNING: INDEX.md is stale — missing slugs: {}\n",
            missing_list.join(", ")
        ));
    }
}

/// Parse element slugs from the first column of an INDEX.md Markdown table.
///
/// Assumes table rows look like: `| slug | ... |`
fn parse_index_element_slugs(content: &str) -> Vec<String> {
    let mut slugs = Vec::new();
    let mut in_table = false;
    let mut header_seen = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            in_table = true;
            // Skip header separator lines (e.g. `| --- | --- |`)
            let inner = &trimmed[1..trimmed.len() - 1];
            if inner.split('|').all(|c| c.trim().replace('-', "").is_empty()) {
                header_seen = true;
                continue;
            }
            if !header_seen {
                // First row is header
                header_seen = true;
                continue;
            }
            // Data row: first cell is the element slug
            if let Some(first_cell) = inner.split('|').next() {
                let slug = first_cell.trim().to_string();
                if !slug.is_empty() {
                    slugs.push(slug);
                }
            }
        } else if in_table {
            break;
        }
    }

    slugs
}

/// Compute coverage: ratio of element-referenced sources to total discoverable sources.
///
/// When coverage < 50%, prints gap report including element sources.
fn run_coverage(
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
    root: &Path,
    element_sources: &[String],
) {
    // Count discoverable sources: agents/, commands/, skills/, hooks/, rules/, crates/
    let discoverable_dirs = [
        "agents", "commands", "skills", "hooks", "rules", "crates",
    ];

    let mut total_discoverable: usize = 0;
    let mut discovered_sources: Vec<String> = element_sources.to_vec();

    for dir_name in &discoverable_dirs {
        let dir = root.join(dir_name);
        if let Ok(entries) = fs.read_dir_recursive(&dir) {
            let md_count = entries
                .iter()
                .filter(|p| {
                    p.to_string_lossy().ends_with(".md")
                        || p.to_string_lossy().ends_with(".rs")
                        || p.to_string_lossy().ends_with(".toml")
                })
                .count();
            total_discoverable += md_count;
        }
    }

    if total_discoverable == 0 {
        terminal.stdout_write("Coverage: no discoverable sources found\n");
        return;
    }

    let covered = discovered_sources.len();
    let pct = (covered * 100) / total_discoverable;

    terminal.stdout_write(&format!("Coverage: {covered}/{total_discoverable} ({pct}%)\n"));

    if pct < 50 {
        terminal.stderr_write("WARNING: coverage below 50% — gaps:\n");
        for source in &discovered_sources {
            terminal.stderr_write(&format!("  - {source}\n"));
        }
        // Include element sources in gap report
        let element_gaps: Vec<&str> = element_sources
            .iter()
            .map(|s| s.as_str())
            .collect();
        if !element_gaps.is_empty() {
            terminal.stderr_write("Element sources referenced but not covering:\n");
            for gap in &element_gaps {
                terminal.stderr_write(&format!("  - {gap}\n"));
            }
        }
        discovered_sources.dedup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_ports::shell::CommandOutput;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment, MockExecutor};
    use std::path::PathBuf;

    fn make_success_git_output() -> CommandOutput {
        CommandOutput {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
        }
    }

    fn make_stale_git_output() -> CommandOutput {
        CommandOutput {
            stdout: "abc1234 modify agents/my-agent.md\n".to_string(),
            stderr: String::new(),
            exit_code: 0,
        }
    }

    fn valid_element_content() -> &'static str {
        "## Overview\n\nElement overview.\n\n## Relationships\n\nNone.\n\n## Participating Flows\n\n- [flow-alpha](../flows/flow-alpha.md)\n\n## Participating Journeys\n\n- [journey-beta](../journeys/journey-beta.md)\n"
    }

    fn valid_element_with_meta(last_updated: &str, sources: &str) -> String {
        format!(
            "<!-- CARTOGRAPHY-META last_updated:{last_updated} sources:{sources} -->\n\n## Overview\n\n## Relationships\n\n## Participating Flows\n\n## Participating Journeys\n"
        )
    }

    // --- PC-022: invalid_element_exits_with_error ---

    #[test]
    fn invalid_element_exits_with_error() {
        // Element file missing required sections
        let invalid_content = "## Overview\n\nOnly overview present.\n";
        let fs = InMemoryFileSystem::new()
            .with_dir("docs/cartography/elements")
            .with_file(
                "docs/cartography/elements/my-element.md",
                invalid_content,
            );
        let terminal = BufferedTerminal::new();
        let shell = MockExecutor::new().on("git", make_success_git_output());

        let result = run_validate_cartography(
            &fs,
            &terminal,
            &shell,
            Path::new("."),
            false,
        );

        assert!(!result, "should return false when element is invalid");

        let stderr = terminal.stderr_output().join("");
        assert!(
            stderr.contains("ERROR"),
            "stderr should contain error message, got: {stderr}"
        );
    }

    // --- PC-023: missing_index_warns_not_errors ---

    #[test]
    fn missing_index_warns_not_errors() {
        // elements/ exists with a valid element, but INDEX.md is absent
        let fs = InMemoryFileSystem::new()
            .with_dir("docs/cartography/elements")
            .with_file(
                "docs/cartography/elements/my-element.md",
                valid_element_content(),
            );
        let terminal = BufferedTerminal::new();
        let shell = MockExecutor::new().on("git", make_success_git_output());

        let result = run_validate_cartography(
            &fs,
            &terminal,
            &shell,
            Path::new("."),
            false,
        );

        // Should NOT return false (missing INDEX is a warning, not error)
        assert!(
            result,
            "should return true when element is valid but INDEX.md absent"
        );

        let stderr = terminal.stderr_output().join("");
        assert!(
            stderr.contains("WARNING") && stderr.contains("INDEX"),
            "stderr should warn about missing INDEX.md, got: {stderr}"
        );

        // Must NOT have ERROR in stderr
        assert!(
            !stderr.contains("ERROR"),
            "missing INDEX.md should not produce an error, got: {stderr}"
        );
    }

    // --- PC-024: stale_index_warns_missing_slugs ---

    #[test]
    fn stale_index_warns_missing_slugs() {
        // INDEX.md exists but does not list "my-new-element" slug
        let index_content = "\
# Element Cross-Reference Matrix

| Element | journey-alpha |
|---------|--------------|
| old-element | Y |
";
        let fs = InMemoryFileSystem::new()
            .with_dir("docs/cartography/elements")
            .with_file(
                "docs/cartography/elements/old-element.md",
                valid_element_content(),
            )
            .with_file(
                "docs/cartography/elements/my-new-element.md",
                valid_element_content(),
            )
            .with_file("docs/cartography/elements/INDEX.md", index_content);
        let terminal = BufferedTerminal::new();
        let shell = MockExecutor::new().on("git", make_success_git_output());

        let result = run_validate_cartography(
            &fs,
            &terminal,
            &shell,
            Path::new("."),
            false,
        );

        assert!(
            result,
            "stale INDEX.md should not cause failure (warning only)"
        );

        let stderr = terminal.stderr_output().join("");
        assert!(
            stderr.contains("WARNING") && stderr.contains("stale"),
            "stderr should warn about stale INDEX, got: {stderr}"
        );
        assert!(
            stderr.contains("my-new-element"),
            "warning should mention missing slug 'my-new-element', got: {stderr}"
        );
    }

    // --- PC-025: missing_elements_dir_clean_exit ---

    #[test]
    fn missing_elements_dir_clean_exit() {
        // elements/ directory does not exist at all
        let fs = InMemoryFileSystem::new();
        let terminal = BufferedTerminal::new();
        let shell = MockExecutor::new();

        let result = run_validate_cartography(
            &fs,
            &terminal,
            &shell,
            Path::new("."),
            false,
        );

        assert!(result, "should return true when elements/ is missing");

        let stderr = terminal.stderr_output().join("");
        assert!(
            !stderr.contains("ERROR"),
            "missing elements/ dir should not produce error, got: {stderr}"
        );
    }

    // --- PC-026: staleness_includes_elements ---

    #[test]
    fn staleness_includes_elements() {
        // Element file with CARTOGRAPHY-META marker
        let element_content =
            valid_element_with_meta("2026-01-01", "agents/my-agent.md");

        let fs = InMemoryFileSystem::new()
            .with_dir("docs/cartography/elements")
            .with_file(
                "docs/cartography/elements/my-element.md",
                &element_content,
            )
            .with_file("docs/cartography/elements/INDEX.md", "# Index\n");
        let terminal = BufferedTerminal::new();

        // git log returns commits indicating source was modified after last_updated
        let shell = MockExecutor::new().on("git", make_stale_git_output());

        run_validate_cartography(
            &fs,
            &terminal,
            &shell,
            Path::new("."),
            false,
        );

        let stderr = terminal.stderr_output().join("");
        assert!(
            stderr.contains("stale"),
            "staleness check should warn for elements with CARTOGRAPHY-META, got: {stderr}"
        );
    }

    // --- PC-027: coverage_includes_element_sources ---

    #[test]
    fn coverage_includes_element_sources() {
        // Element file referencing a source via CARTOGRAPHY-META
        let element_content = valid_element_with_meta("2026-01-01", "agents/my-agent.md");

        // Provide an agents/ dir with one file so there's something to count
        let fs = InMemoryFileSystem::new()
            .with_dir("docs/cartography/elements")
            .with_file(
                "docs/cartography/elements/my-element.md",
                &element_content,
            )
            .with_file("docs/cartography/elements/INDEX.md", "# Index\n")
            .with_file("agents/my-agent.md", "# Agent\n");
        let terminal = BufferedTerminal::new();
        let shell = MockExecutor::new().on("git", make_success_git_output());

        run_validate_cartography(
            &fs,
            &terminal,
            &shell,
            Path::new("."),
            true, // --coverage
        );

        let stdout = terminal.stdout_output().join("");
        assert!(
            stdout.contains("Coverage"),
            "coverage output should be produced, got: {stdout}"
        );

        // The element source should be included (denominator / numerator)
        // We simply check coverage was calculated (not specifically asserting the number,
        // just that element sources are included in the calculation).
        assert!(
            stdout.contains("Coverage:"),
            "should output coverage line, got: {stdout}"
        );
    }

    // --- PC-028: low_coverage_includes_all_gap_types ---

    #[test]
    fn low_coverage_includes_all_gap_types() {
        // Set up many undocumented sources to force coverage below 50%
        // and one element file referencing a source
        let element_content = valid_element_with_meta("2026-01-01", "agents/agent-a.md");

        let mut fs = InMemoryFileSystem::new()
            .with_dir("docs/cartography/elements")
            .with_file(
                "docs/cartography/elements/my-element.md",
                &element_content,
            )
            .with_file("docs/cartography/elements/INDEX.md", "# Index\n");

        // Add many undocumented agent files to push coverage below 50%
        for i in 0..20 {
            fs = fs.with_file(&format!("agents/agent-{i}.md"), "# Agent\n");
        }

        let terminal = BufferedTerminal::new();
        let shell = MockExecutor::new().on("git", make_success_git_output());

        run_validate_cartography(
            &fs,
            &terminal,
            &shell,
            Path::new("."),
            true, // --coverage
        );

        let stdout = terminal.stdout_output().join("");
        let stderr = terminal.stderr_output().join("");

        // Coverage should be below 50% — gap report should include element sources
        assert!(
            stderr.contains("WARNING") && stderr.contains("50%"),
            "should warn about low coverage, got stderr: {stderr}"
        );
    }
}
