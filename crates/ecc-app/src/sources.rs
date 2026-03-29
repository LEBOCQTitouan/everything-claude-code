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

/// List sources, optionally filtered by quadrant and/or subject.
pub fn list(
    _fs: &dyn FileSystem,
    _path: &Path,
    _quadrant: Option<&str>,
    _subject: Option<&str>,
) -> Result<Vec<SourceEntry>, SourcesError> {
    todo!("not implemented")
}

/// Add a new source entry to the registry.
pub fn add(_fs: &dyn FileSystem, _path: &Path, _entry: SourceEntry) -> Result<(), SourcesError> {
    todo!("not implemented")
}

/// Check all sources for reachability using curl.
pub fn check(
    _fs: &dyn FileSystem,
    _shell: &dyn ShellExecutor,
    _path: &Path,
) -> Result<CheckReport, SourcesError> {
    todo!("not implemented")
}

/// Reindex the registry: re-render in canonical quadrant order deterministically.
pub fn reindex(_fs: &dyn FileSystem, _path: &Path) -> Result<(), SourcesError> {
    todo!("not implemented")
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
