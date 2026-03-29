//! Knowledge sources registry use cases — list, add, check, reindex.
//!
//! Orchestrates registry parsing and I/O through the FileSystem and ShellExecutor ports.

use ecc_ports::fs::FileSystem;
use ecc_ports::shell::ShellExecutor;
use std::path::Path;

/// A single knowledge source entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceEntry {
    /// URL of the source.
    pub url: String,
    /// Human-readable title.
    pub title: String,
    /// Type of source (blog, repo, docs, etc.).
    pub source_type: String,
    /// Subject/topic category.
    pub subject: String,
    /// Quadrant (Adopt, Trial, Assess, Hold, Inbox).
    pub quadrant: String,
    /// Date the entry was added (ISO 8601 string).
    pub added_date: String,
    /// Who added the entry.
    pub added_by: String,
}

/// Result of a URL reachability check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckReport {
    /// Entries flagged as stale or unreachable.
    pub stale: Vec<SourceEntry>,
    /// Entries confirmed reachable.
    pub reachable: Vec<SourceEntry>,
}

/// Errors produced by sources use cases.
#[derive(Debug, thiserror::Error)]
pub enum SourcesError {
    /// The registry file could not be read.
    #[error("failed to read registry: {0}")]
    IoError(String),
    /// A duplicate URL was detected.
    #[error("duplicate URL: {0}")]
    DuplicateUrl(String),
    /// The registry file could not be parsed.
    #[error("parse error: {0}")]
    ParseError(String),
}

/// Parse the registry Markdown content into a list of entries.
///
/// # Registry Format
///
/// ```markdown
/// ## Adopt
///
/// - url: https://example.com
///   title: Example Site
///   type: blog
///   subject: CLI
///   added-date: 2026-01-01
///   added-by: alice
/// ```
fn parse_registry(content: &str) -> Result<Vec<SourceEntry>, SourcesError> {
    let mut entries = Vec::new();
    let mut current_quadrant = String::new();

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Detect quadrant section headers: "## <QuadrantName>"
        if let Some(header) = line.strip_prefix("## ") {
            current_quadrant = header.trim().to_string();
            i += 1;
            continue;
        }

        // Detect entry start: "- url: <url>"
        if let Some(url_val) = line.strip_prefix("- url: ") {
            let url = url_val.trim().to_string();
            let mut title = String::new();
            let mut source_type = String::new();
            let mut subject = String::new();
            let mut added_date = String::new();
            let mut added_by = String::new();

            i += 1;
            while i < lines.len() {
                let field_line = lines[i].trim();
                if field_line.is_empty()
                    || field_line.starts_with("- url: ")
                    || field_line.starts_with("## ")
                {
                    break;
                }
                if let Some(v) = field_line.strip_prefix("title: ") {
                    title = v.trim().to_string();
                } else if let Some(v) = field_line.strip_prefix("type: ") {
                    source_type = v.trim().to_string();
                } else if let Some(v) = field_line.strip_prefix("subject: ") {
                    subject = v.trim().to_string();
                } else if let Some(v) = field_line.strip_prefix("added-date: ") {
                    added_date = v.trim().to_string();
                } else if let Some(v) = field_line.strip_prefix("added-by: ") {
                    added_by = v.trim().to_string();
                }
                i += 1;
            }

            if !url.is_empty() {
                entries.push(SourceEntry {
                    url,
                    title,
                    source_type,
                    subject,
                    quadrant: current_quadrant.clone(),
                    added_date,
                    added_by,
                });
            }
            continue;
        }

        i += 1;
    }

    Ok(entries)
}

/// Render a registry from entries, grouped by quadrant in canonical order.
fn render_registry(entries: &[SourceEntry]) -> String {
    let quadrant_order = ["Adopt", "Trial", "Assess", "Hold", "Inbox"];
    let mut output = String::new();

    for quadrant in &quadrant_order {
        let section: Vec<&SourceEntry> =
            entries.iter().filter(|e| e.quadrant == *quadrant).collect();
        if section.is_empty() {
            continue;
        }
        output.push_str(&format!("## {quadrant}\n\n"));
        for entry in section {
            output.push_str(&format!("- url: {}\n", entry.url));
            output.push_str(&format!("  title: {}\n", entry.title));
            output.push_str(&format!("  type: {}\n", entry.source_type));
            output.push_str(&format!("  subject: {}\n", entry.subject));
            output.push_str(&format!("  added-date: {}\n", entry.added_date));
            output.push_str(&format!("  added-by: {}\n", entry.added_by));
            output.push('\n');
        }
    }

    output
}

/// List sources, optionally filtered by quadrant and/or subject.
pub fn list(
    fs: &dyn FileSystem,
    path: &Path,
    quadrant: Option<&str>,
    subject: Option<&str>,
) -> Result<Vec<SourceEntry>, SourcesError> {
    let content = fs
        .read_to_string(path)
        .map_err(|e| SourcesError::IoError(e.to_string()))?;

    let entries = parse_registry(&content)?;

    let filtered = entries
        .into_iter()
        .filter(|e| {
            quadrant.is_none_or(|q| e.quadrant.eq_ignore_ascii_case(q))
                && subject.is_none_or(|s| e.subject.eq_ignore_ascii_case(s))
        })
        .collect();

    Ok(filtered)
}

/// Add a new source entry to the registry.
///
/// Rejects duplicate URLs. Adds the entry to the section matching its quadrant.
/// Uses atomic write (temp file + rename).
pub fn add(fs: &dyn FileSystem, path: &Path, entry: SourceEntry) -> Result<(), SourcesError> {
    let content = if fs.exists(path) {
        fs.read_to_string(path)
            .map_err(|e| SourcesError::IoError(e.to_string()))?
    } else {
        String::new()
    };

    let mut entries = parse_registry(&content)?;

    // Check for duplicate URL
    if entries.iter().any(|e| e.url == entry.url) {
        return Err(SourcesError::DuplicateUrl(entry.url));
    }

    entries.push(entry);

    let rendered = render_registry(&entries);

    // Atomic write: temp file + rename
    let tmp_path = path.with_extension("md.tmp");
    fs.write(&tmp_path, &rendered)
        .map_err(|e| SourcesError::IoError(format!("failed to write temp file: {e}")))?;
    if let Err(e) = fs.rename(&tmp_path, path) {
        let _ = fs.remove_file(&tmp_path);
        return Err(SourcesError::IoError(format!(
            "failed to rename temp file: {e}"
        )));
    }

    Ok(())
}

/// Check all sources for reachability using curl.
///
/// For each URL, runs `curl -sI -o /dev/null -w "%{http_code}" <url>`.
/// Entries returning non-200 status are flagged as stale.
pub fn check(
    fs: &dyn FileSystem,
    shell: &dyn ShellExecutor,
    path: &Path,
) -> Result<CheckReport, SourcesError> {
    let content = fs
        .read_to_string(path)
        .map_err(|e| SourcesError::IoError(e.to_string()))?;

    let entries = parse_registry(&content)?;
    let mut stale = Vec::new();
    let mut reachable = Vec::new();

    for entry in entries {
        let result = shell.run_command(
            "curl",
            &["-sI", "-o", "/dev/null", "-w", "%{http_code}", &entry.url],
        );
        let is_ok = match result {
            Ok(output) => output.stdout.trim() == "200",
            Err(_) => false,
        };
        if is_ok {
            reachable.push(entry);
        } else {
            stale.push(entry);
        }
    }

    Ok(CheckReport { stale, reachable })
}

/// Reindex the registry: re-render in canonical quadrant order deterministically.
///
/// Reads the registry, parses all entries, then rewrites the file with entries
/// grouped by quadrant in canonical order (Adopt, Trial, Assess, Hold, Inbox).
pub fn reindex(fs: &dyn FileSystem, path: &Path) -> Result<(), SourcesError> {
    let content = fs
        .read_to_string(path)
        .map_err(|e| SourcesError::IoError(e.to_string()))?;

    let entries = parse_registry(&content)?;
    let rendered = render_registry(&entries);

    let tmp_path = path.with_extension("md.tmp");
    fs.write(&tmp_path, &rendered)
        .map_err(|e| SourcesError::IoError(format!("failed to write temp file: {e}")))?;
    if let Err(e) = fs.rename(&tmp_path, path) {
        let _ = fs.remove_file(&tmp_path);
        return Err(SourcesError::IoError(format!(
            "failed to rename temp file: {e}"
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::{InMemoryFileSystem, MockExecutor};
    use ecc_ports::shell::CommandOutput;

    /// Build a sample registry content with multiple quadrants.
    fn sample_registry() -> String {
        concat!(
            "## Adopt\n\n",
            "- url: https://adopt.example.com\n",
            "  title: Adopt Example\n",
            "  type: blog\n",
            "  subject: CLI\n",
            "  added-date: 2026-01-01\n",
            "  added-by: alice\n",
            "\n",
            "## Trial\n\n",
            "- url: https://trial.example.com\n",
            "  title: Trial Example\n",
            "  type: docs\n",
            "  subject: Rust\n",
            "  added-date: 2026-01-02\n",
            "  added-by: bob\n",
            "\n",
            "## Inbox\n\n",
            "- url: https://inbox.example.com\n",
            "  title: Inbox Example\n",
            "  type: repo\n",
            "  subject: CLI\n",
            "  added-date: 2026-01-03\n",
            "  added-by: carol\n",
            "\n",
        ).to_string()
    }

    fn sample_entry(url: &str, quadrant: &str) -> SourceEntry {
        SourceEntry {
            url: url.to_string(),
            title: format!("Title for {url}"),
            source_type: "blog".to_string(),
            subject: "Rust".to_string(),
            quadrant: quadrant.to_string(),
            added_date: "2026-03-28".to_string(),
            added_by: "tester".to_string(),
        }
    }

    // --- PC-019: list() with quadrant filter ---

    #[test]
    fn list_with_quadrant_filter() {
        let fs = InMemoryFileSystem::new()
            .with_file("/sources.md", &sample_registry());

        let result = list(&fs, Path::new("/sources.md"), Some("Adopt"), None).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].url, "https://adopt.example.com");
        assert_eq!(result[0].quadrant, "Adopt");
    }

    // --- PC-020: list() with subject filter ---

    #[test]
    fn list_with_subject_filter() {
        let fs = InMemoryFileSystem::new()
            .with_file("/sources.md", &sample_registry());

        let result = list(&fs, Path::new("/sources.md"), None, Some("Rust")).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].url, "https://trial.example.com");
        assert_eq!(result[0].subject, "Rust");
    }

    // --- PC-021: list() no filters returns all ---

    #[test]
    fn list_no_filters_returns_all() {
        let fs = InMemoryFileSystem::new()
            .with_file("/sources.md", &sample_registry());

        let result = list(&fs, Path::new("/sources.md"), None, None).unwrap();

        assert_eq!(result.len(), 3);
    }

    // --- PC-022: add() to correct quadrant, atomic write ---

    #[test]
    fn add_to_correct_quadrant_atomic_write() {
        let fs = InMemoryFileSystem::new()
            .with_file("/sources.md", &sample_registry());

        let entry = sample_entry("https://new.example.com", "Trial");
        add(&fs, Path::new("/sources.md"), entry).unwrap();

        // Temp file should not remain
        assert!(!fs.exists(Path::new("/sources.md.tmp")));

        let entries = list(&fs, Path::new("/sources.md"), Some("Trial"), None).unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().any(|e| e.url == "https://new.example.com"));
    }

    // --- PC-023: add() rejects duplicate URL ---

    #[test]
    fn add_rejects_duplicate_url() {
        let fs = InMemoryFileSystem::new()
            .with_file("/sources.md", &sample_registry());

        let entry = sample_entry("https://adopt.example.com", "Trial");
        let result = add(&fs, Path::new("/sources.md"), entry);

        assert!(matches!(result, Err(SourcesError::DuplicateUrl(url)) if url == "https://adopt.example.com"));
    }

    // --- PC-024: check() flags stale URLs ---

    #[test]
    fn check_flags_stale_urls() {
        let fs = InMemoryFileSystem::new()
            .with_file("/sources.md", &sample_registry());

        let shell = MockExecutor::new()
            .on_args(
                "curl",
                &["-sI", "-o", "/dev/null", "-w", "%{http_code}", "https://adopt.example.com"],
                CommandOutput { stdout: "200".to_string(), stderr: String::new(), exit_code: 0 },
            )
            .on_args(
                "curl",
                &["-sI", "-o", "/dev/null", "-w", "%{http_code}", "https://trial.example.com"],
                CommandOutput { stdout: "404".to_string(), stderr: String::new(), exit_code: 0 },
            )
            .on_args(
                "curl",
                &["-sI", "-o", "/dev/null", "-w", "%{http_code}", "https://inbox.example.com"],
                CommandOutput { stdout: "200".to_string(), stderr: String::new(), exit_code: 0 },
            );

        let report = check(&fs, &shell, Path::new("/sources.md")).unwrap();

        assert_eq!(report.stale.len(), 1);
        assert_eq!(report.stale[0].url, "https://trial.example.com");
        assert_eq!(report.reachable.len(), 2);
    }

    // --- PC-025: check() uses ShellExecutor for curl ---

    #[test]
    fn check_uses_shell_executor_for_curl() {
        let fs = InMemoryFileSystem::new()
            .with_file(
                "/sources.md",
                concat!(
                    "## Adopt\n\n",
                    "- url: https://check.example.com\n",
                    "  title: Check Test\n",
                    "  type: blog\n",
                    "  subject: Test\n",
                    "  added-date: 2026-01-01\n",
                    "  added-by: alice\n",
                    "\n",
                ),
            );

        // Shell executor returns 200 → should be in reachable, not stale
        let shell = MockExecutor::new().on(
            "curl",
            CommandOutput { stdout: "200".to_string(), stderr: String::new(), exit_code: 0 },
        );

        let report = check(&fs, &shell, Path::new("/sources.md")).unwrap();

        assert_eq!(report.reachable.len(), 1);
        assert_eq!(report.stale.len(), 0);
        assert_eq!(report.reachable[0].url, "https://check.example.com");
    }

    // --- PC-026: reindex() classifies inbox entries ---

    #[test]
    fn reindex_classifies_inbox_entries() {
        let content = concat!(
            "## Inbox\n\n",
            "- url: https://inbox.example.com\n",
            "  title: Inbox Entry\n",
            "  type: blog\n",
            "  subject: Rust\n",
            "  added-date: 2026-01-01\n",
            "  added-by: alice\n",
            "\n",
        );
        let fs = InMemoryFileSystem::new().with_file("/sources.md", content);

        reindex(&fs, Path::new("/sources.md")).unwrap();

        let entries = list(&fs, Path::new("/sources.md"), None, None).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].quadrant, "Inbox");
        assert_eq!(entries[0].url, "https://inbox.example.com");
    }

    // --- PC-027: reindex() deterministic rewrite ---

    #[test]
    fn reindex_deterministic_rewrite() {
        let content = concat!(
            "## Trial\n\n",
            "- url: https://trial.example.com\n",
            "  title: Trial\n",
            "  type: docs\n",
            "  subject: Rust\n",
            "  added-date: 2026-01-01\n",
            "  added-by: alice\n",
            "\n",
            "## Adopt\n\n",
            "- url: https://adopt.example.com\n",
            "  title: Adopt\n",
            "  type: blog\n",
            "  subject: CLI\n",
            "  added-date: 2026-01-01\n",
            "  added-by: bob\n",
            "\n",
        );
        let fs = InMemoryFileSystem::new().with_file("/sources.md", content);

        reindex(&fs, Path::new("/sources.md")).unwrap();

        let written = fs.read_to_string(Path::new("/sources.md")).unwrap();

        let adopt_pos = written.find("## Adopt").unwrap();
        let trial_pos = written.find("## Trial").unwrap();
        assert!(adopt_pos < trial_pos, "Adopt should appear before Trial after reindex");

        assert!(!fs.exists(Path::new("/sources.md.tmp")));
    }

    // --- PC-028: add() sets metadata (added-date, added-by) ---

    #[test]
    fn add_sets_metadata() {
        let fs = InMemoryFileSystem::new()
            .with_file("/sources.md", "");

        let entry = SourceEntry {
            url: "https://meta.example.com".to_string(),
            title: "Meta Test".to_string(),
            source_type: "blog".to_string(),
            subject: "Rust".to_string(),
            quadrant: "Adopt".to_string(),
            added_date: "2026-03-28".to_string(),
            added_by: "tester".to_string(),
        };

        add(&fs, Path::new("/sources.md"), entry).unwrap();

        let entries = list(&fs, Path::new("/sources.md"), None, None).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].added_date, "2026-03-28");
        assert_eq!(entries[0].added_by, "tester");
    }
}
