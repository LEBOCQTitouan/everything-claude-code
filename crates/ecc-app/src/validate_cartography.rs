//! `validate_cartography` use case — validates docs/cartography/ structure.
//!
//! Scans element files, checks INDEX.md presence and staleness, and optionally
//! computes coverage of sources referenced by element files.

use ecc_domain::cartography::element_validation::validate_element;
use ecc_ports::fs::FileSystem;
use ecc_ports::shell::ShellExecutor;
use ecc_ports::terminal::TerminalIO;
use std::path::{Path, PathBuf};

/// Marker embedded in element files for staleness detection.
const CARTOGRAPHY_META_MARKER: &str = "CARTOGRAPHY-META";

/// Construct a path by joining root to a sub-path, normalizing the `.` prefix.
///
/// When root is `"."`, returns `PathBuf::from(sub_path)` instead of `"./sub_path"`.
/// This ensures InMemoryFileSystem lookups match keys stored without a `./` prefix.
fn normalize_path(root: &Path, sub_path: &str) -> PathBuf {
    if root == Path::new(".") {
        PathBuf::from(sub_path)
    } else {
        root.join(sub_path)
    }
}

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
    let elements_dir = normalize_path(root, "docs/cartography/elements");

    // AC-016.3: If elements/ dir missing, exit cleanly.
    if !fs.exists(&elements_dir) {
        return true;
    }

    let (element_errors, element_slugs, element_sources) =
        validate_elements_dir(fs, shell, terminal, &elements_dir);

    // AC-014.5 / AC-014.6: INDEX.md presence and staleness.
    let index_path = elements_dir.join("INDEX.md");
    if !fs.exists(&index_path) {
        if !element_slugs.is_empty() {
            terminal.stderr_write("WARNING: docs/cartography/elements/INDEX.md is absent\n");
        }
    } else {
        // AC-014.6: Check if INDEX.md is stale (missing element slugs).
        check_index_staleness(fs, terminal, &index_path, &element_slugs);
    }

    // AC-015.2 / AC-015.3: Coverage calculation includes element-referenced sources.
    if coverage {
        run_coverage(fs, terminal, root, &element_sources);
    }

    !element_errors
}

/// Validate all element files in the elements directory.
///
/// Returns `(has_errors, element_slugs, all_element_sources)`.
fn validate_elements_dir(
    fs: &dyn FileSystem,
    shell: &dyn ShellExecutor,
    terminal: &dyn TerminalIO,
    elements_dir: &Path,
) -> (bool, Vec<String>, Vec<String>) {
    let element_files = match scan_element_files(fs, elements_dir, terminal) {
        Ok(files) => files,
        Err(()) => return (true, Vec::new(), Vec::new()),
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
    check_element_staleness(fs, shell, terminal, &element_files);

    (has_errors, element_slugs, all_element_sources)
}

/// Scan the elements/ directory, returning paths to element `.md` files.
///
/// Excludes `INDEX.md` and `README.md`.
fn scan_element_files(
    fs: &dyn FileSystem,
    elements_dir: &Path,
    terminal: &dyn TerminalIO,
) -> Result<Vec<PathBuf>, ()> {
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
        if trimmed.contains(CARTOGRAPHY_META_MARKER)
            && let Some(sources_start) = trimmed.find("sources:")
        {
            let rest = &trimmed[sources_start + "sources:".len()..];
            let sources_str = rest.trim_end_matches("-->").trim();
            return sources_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }
    Vec::new()
}

/// Parse `last_updated` field from a CARTOGRAPHY-META marker.
fn parse_meta_last_updated(content: &str) -> String {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.contains(CARTOGRAPHY_META_MARKER)
            && let Some(lu_start) = trimmed.find("last_updated:")
        {
            let rest = &trimmed[lu_start + "last_updated:".len()..];
            let value = rest
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_end_matches("-->")
                .trim();
            return value.to_string();
        }
    }
    String::new()
}

/// Check staleness of element files that contain CARTOGRAPHY-META markers.
///
/// For each element file with a CARTOGRAPHY-META marker, reads `last_updated` and
/// compares against the git log of referenced source files.
fn check_element_staleness(
    fs: &dyn FileSystem,
    shell: &dyn ShellExecutor,
    terminal: &dyn TerminalIO,
    element_files: &[PathBuf],
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

        for source in &sources {
            let result = shell.run_command(
                "git",
                &["log", "--since", &last_updated, "--oneline", "--", source],
            );
            if let Ok(out) = result
                && !out.stdout.trim().is_empty()
            {
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

/// Check INDEX.md staleness by comparing table row slugs to known element slugs.
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

    let missing_in_index: Vec<&str> = element_slugs
        .iter()
        .filter(|s| !index_slugs.contains(s))
        .map(|s| s.as_str())
        .collect();

    if !missing_in_index.is_empty() {
        terminal.stderr_write(&format!(
            "WARNING: INDEX.md is stale — missing slugs: {}\n",
            missing_in_index.join(", ")
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
            let inner = &trimmed[1..trimmed.len() - 1];
            // Skip header separator lines (e.g. `| --- | --- |`)
            if inner.split('|').all(|c| c.trim().replace('-', "").is_empty()) {
                header_seen = true;
                continue;
            }
            if !header_seen {
                // First row is the column header
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
/// When coverage < 50%, prints gap report including element-referenced sources.
fn run_coverage(
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
    root: &Path,
    element_sources: &[String],
) {
    let discoverable_dirs = ["agents", "commands", "skills", "hooks", "rules", "crates"];

    let total_discoverable: usize = discoverable_dirs
        .iter()
        .map(|dir_name| {
            let dir = normalize_path(root, dir_name);
            fs.read_dir_recursive(&dir)
                .map(|entries| {
                    entries
                        .iter()
                        .filter(|p| {
                            let s = p.to_string_lossy();
                            s.ends_with(".md") || s.ends_with(".rs") || s.ends_with(".toml")
                        })
                        .count()
                })
                .unwrap_or(0)
        })
        .sum();

    if total_discoverable == 0 {
        terminal.stdout_write("Coverage: no discoverable sources found\n");
        return;
    }

    let covered = element_sources.len();
    let pct = (covered * 100) / total_discoverable;

    terminal.stdout_write(&format!("Coverage: {covered}/{total_discoverable} ({pct}%)\n"));

    if pct < 50 {
        terminal.stderr_write("WARNING: coverage below 50% — gaps:\n");
        for source in element_sources {
            terminal.stderr_write(&format!("  - {source}\n"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_ports::shell::CommandOutput;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockExecutor};

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
        let invalid_content = "## Overview\n\nOnly overview present.\n";
        let fs = InMemoryFileSystem::new()
            .with_dir("docs/cartography/elements")
            .with_file(
                "docs/cartography/elements/my-element.md",
                invalid_content,
            );
        let terminal = BufferedTerminal::new();
        let shell = MockExecutor::new().on("git", make_success_git_output());

        let result = run_validate_cartography(&fs, &terminal, &shell, Path::new("."), false);

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
        let fs = InMemoryFileSystem::new()
            .with_dir("docs/cartography/elements")
            .with_file(
                "docs/cartography/elements/my-element.md",
                valid_element_content(),
            );
        let terminal = BufferedTerminal::new();
        let shell = MockExecutor::new().on("git", make_success_git_output());

        let result = run_validate_cartography(&fs, &terminal, &shell, Path::new("."), false);

        assert!(
            result,
            "should return true when element is valid but INDEX.md absent"
        );
        let stderr = terminal.stderr_output().join("");
        assert!(
            stderr.contains("WARNING") && stderr.contains("INDEX"),
            "stderr should warn about missing INDEX.md, got: {stderr}"
        );
        assert!(
            !stderr.contains("ERROR"),
            "missing INDEX.md should not produce an error, got: {stderr}"
        );
    }

    // --- PC-024: stale_index_warns_missing_slugs ---

    #[test]
    fn stale_index_warns_missing_slugs() {
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

        let result = run_validate_cartography(&fs, &terminal, &shell, Path::new("."), false);

        assert!(result, "stale INDEX.md should not cause failure");
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
        let fs = InMemoryFileSystem::new();
        let terminal = BufferedTerminal::new();
        let shell = MockExecutor::new();

        let result = run_validate_cartography(&fs, &terminal, &shell, Path::new("."), false);

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
        let element_content = valid_element_with_meta("2026-01-01", "agents/my-agent.md");
        let fs = InMemoryFileSystem::new()
            .with_dir("docs/cartography/elements")
            .with_file(
                "docs/cartography/elements/my-element.md",
                &element_content,
            )
            .with_file("docs/cartography/elements/INDEX.md", "# Index\n");
        let terminal = BufferedTerminal::new();
        let shell = MockExecutor::new().on("git", make_stale_git_output());

        run_validate_cartography(&fs, &terminal, &shell, Path::new("."), false);

        let stderr = terminal.stderr_output().join("");
        assert!(
            stderr.contains("stale"),
            "staleness check should warn for elements with CARTOGRAPHY-META, got: {stderr}"
        );
    }

    // --- PC-027: coverage_includes_element_sources ---

    #[test]
    fn coverage_includes_element_sources() {
        let element_content = valid_element_with_meta("2026-01-01", "agents/my-agent.md");
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

        run_validate_cartography(&fs, &terminal, &shell, Path::new("."), true);

        let stdout = terminal.stdout_output().join("");
        assert!(
            stdout.contains("Coverage:"),
            "should output coverage line, got: {stdout}"
        );
    }

    // --- PC-028: low_coverage_includes_all_gap_types ---

    #[test]
    fn low_coverage_includes_all_gap_types() {
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

        run_validate_cartography(&fs, &terminal, &shell, Path::new("."), true);

        let stderr = terminal.stderr_output().join("");
        assert!(
            stderr.contains("WARNING") && stderr.contains("50%"),
            "should warn about low coverage, got stderr: {stderr}"
        );
    }
}
