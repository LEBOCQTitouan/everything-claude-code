//! Knowledge sources registry use cases — list, add, check, reindex.
//!
//! Orchestrates domain logic through the FileSystem and ShellExecutor ports.

use ecc_domain::sources::entry::{
    Quadrant, SourceEntry, SourceError, SourceType, SourceUrl, validate_title,
};
use ecc_domain::sources::parser::parse_sources;
use ecc_domain::sources::registry::SourcesRegistry;
use ecc_domain::sources::serializer::serialize_sources;
use ecc_ports::fs::FileSystem;
use ecc_ports::shell::ShellExecutor;
use std::path::Path;
use std::str::FromStr;

/// App-layer error type — wraps domain errors and I/O concerns.
#[derive(Debug, thiserror::Error)]
pub enum SourcesAppError {
    #[error("domain error: {0}")]
    Domain(#[from] SourceError),
    #[error("I/O error: {0}")]
    Io(String),
    #[error("shell command failed: {0}")]
    ShellFailed(String),
}

/// Status of a reachability/freshness check for a single source entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckStatus {
    /// Entry is reachable and up-to-date.
    Ok,
    /// HTTP non-200 response.
    Stale,
    /// curl timeout or connection error.
    Unreachable,
    /// last_checked > 90 days ago (days since check).
    WarnAge(u64),
    /// last_checked > 180 days ago (days since check).
    ErrorAge(u64),
}

/// Result of checking a single source entry.
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub title: String,
    pub url: String,
    pub status: CheckStatus,
}

/// Load registry from file, or return empty registry if file is missing.
fn load_registry(fs: &dyn FileSystem, path: &Path) -> Result<SourcesRegistry, SourcesAppError> {
    if !fs.exists(path) {
        return Ok(SourcesRegistry::default());
    }
    let content = fs
        .read_to_string(path)
        .map_err(|e| SourcesAppError::Io(e.to_string()))?;
    parse_sources(&content).map_err(|errors| {
        SourcesAppError::Io(format!(
            "parse errors: {}",
            errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        ))
    })
}

/// Atomic write: write to temp file, then rename.
fn atomic_write(fs: &dyn FileSystem, path: &Path, content: &str) -> Result<(), SourcesAppError> {
    let tmp_path = path.with_extension("md.tmp");
    fs.write(&tmp_path, content)
        .map_err(|e| SourcesAppError::Io(format!("failed to write temp file: {e}")))?;
    fs.rename(&tmp_path, path)
        .map_err(|e| SourcesAppError::Io(format!("failed to rename temp file: {e}")))?;
    Ok(())
}

/// Convert a YYYY-MM-DD date string to days since a fixed epoch (2000-01-01).
///
/// Returns None if the string cannot be parsed.
fn date_to_days(date: &str) -> Option<u64> {
    let parts: Vec<&str> = date.splitn(3, '-').collect();
    if parts.len() != 3 {
        return None;
    }
    let year: i64 = parts[0].parse().ok()?;
    let month: i64 = parts[1].parse().ok()?;
    let day: i64 = parts[2].parse().ok()?;

    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    // Days in each month for non-leap year
    let month_days: [i64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut total: i64 = year * 365 + year / 4 - year / 100 + year / 400;
    for days in month_days.iter().take((month - 1) as usize) {
        total += days;
    }
    total += day;

    Some(total as u64)
}

/// Compute days between two YYYY-MM-DD strings.
///
/// Returns None if either string cannot be parsed.
fn days_between(from: &str, to: &str) -> Option<u64> {
    let from_days = date_to_days(from)?;
    let to_days = date_to_days(to)?;
    Some(to_days.saturating_sub(from_days))
}

/// List knowledge source entries, optionally filtered by quadrant and subject.
///
/// Returns all matching entries as owned values.
pub fn list(
    fs: &dyn FileSystem,
    sources_path: &Path,
    quadrant: Option<&str>,
    subject: Option<&str>,
) -> Result<Vec<SourceEntry>, SourcesAppError> {
    let registry = load_registry(fs, sources_path)?;
    let q = quadrant
        .map(Quadrant::from_str)
        .transpose()
        .map_err(SourcesAppError::Domain)?;
    let entries = registry.list(q.as_ref(), subject);
    Ok(entries.into_iter().cloned().collect())
}

/// Add a new source entry to the registry.
///
/// Reads the existing file (or creates empty registry if missing), validates the
/// entry, checks for duplicates, then atomically writes the updated file.
#[allow(clippy::too_many_arguments)]
pub fn add(
    fs: &dyn FileSystem,
    sources_path: &Path,
    url: &str,
    title: &str,
    source_type: &str,
    quadrant: &str,
    subject: &str,
    added_by: &str,
    date: &str,
) -> Result<(), SourcesAppError> {
    let source_url = SourceUrl::parse(url).map_err(SourcesAppError::Domain)?;
    validate_title(title).map_err(SourcesAppError::Domain)?;

    let source_type = SourceType::from_str(source_type).map_err(SourcesAppError::Domain)?;
    let quadrant = Quadrant::from_str(quadrant).map_err(SourcesAppError::Domain)?;

    let registry = load_registry(fs, sources_path)?;

    let entry = SourceEntry {
        url: source_url,
        title: title.to_owned(),
        source_type,
        quadrant,
        subject: subject.to_owned(),
        added_by: added_by.to_owned(),
        added_date: date.to_owned(),
        last_checked: None,
        deprecation_reason: None,
        stale: false,
    };

    let new_registry = registry.add(entry).map_err(SourcesAppError::Domain)?;
    let content = serialize_sources(&new_registry);
    atomic_write(fs, sources_path, &content)
}

/// Reindex the sources file: move inbox entries into quadrant sections.
///
/// If `dry_run` is true, returns the generated content without writing.
/// Otherwise, atomically writes and returns None.
pub fn reindex(
    fs: &dyn FileSystem,
    sources_path: &Path,
    dry_run: bool,
) -> Result<Option<String>, SourcesAppError> {
    let registry = load_registry(fs, sources_path)?;
    let reindexed = registry.reindex();
    let content = serialize_sources(&reindexed);

    if dry_run {
        return Ok(Some(content));
    }

    atomic_write(fs, sources_path, &content)?;
    Ok(None)
}

/// Check reachability and freshness of all source entries.
///
/// For each entry, issues a curl request and records the outcome.
/// Atomically writes the updated registry with new `stale` flags and `last_checked` dates.
pub fn check(
    fs: &dyn FileSystem,
    shell: &dyn ShellExecutor,
    sources_path: &Path,
    today: &str,
) -> Result<Vec<CheckResult>, SourcesAppError> {
    let registry = load_registry(fs, sources_path)?;
    let mut results: Vec<CheckResult> = Vec::new();
    let mut updated_entries: Vec<SourceEntry> = Vec::new();

    let all_entries: Vec<SourceEntry> = registry
        .entries
        .iter()
        .chain(registry.inbox.iter())
        .cloned()
        .collect();

    for entry in all_entries {
        let curl_result = shell.run_command(
            "curl",
            &[
                "-sL",
                "-o",
                "/dev/null",
                "-w",
                "%{http_code}",
                "--max-time",
                "10",
                entry.url.as_str(),
            ],
        );

        let status = match curl_result {
            Err(_) => {
                // curl timeout or I/O error → unreachable
                let updated = SourceEntry {
                    stale: true,
                    last_checked: Some(today.to_owned()),
                    ..entry.clone()
                };
                updated_entries.push(updated);
                CheckStatus::Unreachable
            }
            Ok(output) => {
                let http_code = output.stdout.trim().parse::<u16>().unwrap_or(0);

                if http_code == 200 {
                    // Reachable — check age, clear stale if was set
                    let updated = SourceEntry {
                        stale: false,
                        last_checked: Some(today.to_owned()),
                        ..entry.clone()
                    };
                    let age_status = if let Some(ref last) = entry.last_checked {
                        let days = days_between(last, today).unwrap_or(0);
                        if days > 180 {
                            Some(CheckStatus::ErrorAge(days))
                        } else if days > 90 {
                            Some(CheckStatus::WarnAge(days))
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    updated_entries.push(updated);
                    age_status.unwrap_or(CheckStatus::Ok)
                } else {
                    // Non-200 → stale
                    let updated = SourceEntry {
                        stale: true,
                        last_checked: Some(today.to_owned()),
                        ..entry.clone()
                    };
                    updated_entries.push(updated);
                    CheckStatus::Stale
                }
            }
        };

        results.push(CheckResult {
            title: entry.title.clone(),
            url: entry.url.as_str().to_owned(),
            status,
        });
    }

    // Rebuild registry with updated entries
    let new_registry = SourcesRegistry {
        inbox: updated_entries
            .iter()
            .filter(|e| registry.inbox.iter().any(|i| i.url == e.url))
            .cloned()
            .collect(),
        entries: updated_entries
            .iter()
            .filter(|e| registry.entries.iter().any(|r| r.url == e.url))
            .cloned()
            .collect(),
        module_mappings: registry.module_mappings.clone(),
    };

    let content = serialize_sources(&new_registry);
    atomic_write(fs, sources_path, &content)?;

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_ports::shell::{CommandOutput, ShellError};
    use ecc_test_support::InMemoryFileSystem;
    use std::collections::HashMap;

    // --- Helpers ---

    fn sample_sources_md() -> String {
        "# Knowledge Sources\n\n## Inbox\n\n\n## Adopt\n\n### testing\n- [Rust Testing Guide](https://example.com/rust-testing) — type: doc | subject: testing | added: 2026-01-01 | by: human\n\n## Trial\n\n## Assess\n\n## Hold\n\n## Module Mapping\n\n| Module | Subjects |\n|--------|----------|\n".to_owned()
    }

    fn sample_with_inbox() -> String {
        "# Knowledge Sources\n\n## Inbox\n\n- [New Entry](https://example.com/new) — type: repo | quadrant: adopt | subject: cli | added: 2026-03-29 | by: human\n\n## Adopt\n\n## Trial\n\n## Assess\n\n## Hold\n\n## Module Mapping\n\n| Module | Subjects |\n|--------|----------|\n".to_owned()
    }

    struct MockShellExecutor {
        responses: HashMap<String, Result<CommandOutput, ShellError>>,
    }

    impl MockShellExecutor {
        fn new() -> Self {
            Self {
                responses: HashMap::new(),
            }
        }

        fn with_response(mut self, url: &str, output: CommandOutput) -> Self {
            self.responses.insert(url.to_owned(), Ok(output));
            self
        }

        fn with_error(mut self, url: &str, error: ShellError) -> Self {
            self.responses.insert(url.to_owned(), Err(error));
            self
        }
    }

    impl ShellExecutor for MockShellExecutor {
        fn run_command(&self, _cmd: &str, args: &[&str]) -> Result<CommandOutput, ShellError> {
            let url = args.last().unwrap_or(&"");
            match self.responses.get(*url) {
                Some(Ok(output)) => Ok(output.clone()),
                Some(Err(_)) => Err(ShellError::Io("mock error".into())),
                None => Err(ShellError::Io("no mock response".into())),
            }
        }

        fn run_command_in_dir(
            &self,
            _cmd: &str,
            _args: &[&str],
            _dir: &Path,
        ) -> Result<CommandOutput, ShellError> {
            unimplemented!("not needed in sources tests")
        }

        fn command_exists(&self, _cmd: &str) -> bool {
            true
        }

        fn spawn_with_stdin(
            &self,
            _cmd: &str,
            _args: &[&str],
            _stdin: &str,
        ) -> Result<CommandOutput, ShellError> {
            unimplemented!("not needed in sources tests")
        }
    }

    fn ok_200() -> CommandOutput {
        CommandOutput {
            stdout: "200".to_owned(),
            stderr: String::new(),
            exit_code: 0,
        }
    }

    fn non_200(code: &str) -> CommandOutput {
        CommandOutput {
            stdout: code.to_owned(),
            stderr: String::new(),
            exit_code: 0,
        }
    }

    // --- PC-018: list use case returns filtered entries ---

    #[test]
    fn list_with_filters() {
        let fs = InMemoryFileSystem::new().with_file("/sources.md", &sample_sources_md());

        let all = list(&fs, Path::new("/sources.md"), None, None).unwrap();
        assert!(!all.is_empty(), "expected at least one entry");

        let adopt_entries = list(&fs, Path::new("/sources.md"), Some("adopt"), None).unwrap();
        assert_eq!(adopt_entries.len(), 1);
        assert_eq!(
            adopt_entries[0].url.as_str(),
            "https://example.com/rust-testing"
        );

        let subject_entries = list(&fs, Path::new("/sources.md"), None, Some("testing")).unwrap();
        assert_eq!(subject_entries.len(), 1);

        let empty = list(&fs, Path::new("/sources.md"), Some("hold"), None).unwrap();
        assert!(empty.is_empty(), "hold quadrant should be empty");
    }

    // --- PC-012: add() uses injected date parameter ---

    #[test]
    fn add_uses_injected_date() {
        let fs = InMemoryFileSystem::new().with_file("/sources.md", &sample_sources_md());

        add(
            &fs,
            Path::new("/sources.md"),
            "https://example.com/dated-entry",
            "Dated Entry",
            "doc",
            "adopt",
            "testing",
            "human",
            "2026-01-15",
        )
        .unwrap();

        let content = fs.read_to_string(Path::new("/sources.md")).unwrap();
        let registry = parse_sources(&content).unwrap();
        let entry = registry
            .find_by_url("https://example.com/dated-entry")
            .expect("dated entry must exist");
        assert_eq!(
            entry.added_date, "2026-01-15",
            "entry must use the injected date"
        );
    }

    // --- PC-019: add use case appends entry, atomic write ---

    #[test]
    fn add_entry() {
        let fs = InMemoryFileSystem::new().with_file("/sources.md", &sample_sources_md());

        add(
            &fs,
            Path::new("/sources.md"),
            "https://example.com/new-tool",
            "New Tool",
            "repo",
            "adopt",
            "testing",
            "human",
            "2026-03-29",
        )
        .unwrap();

        // Temp file should be cleaned up
        assert!(!fs.exists(Path::new("/sources.md.tmp")));
        // Main file should exist and contain new entry
        let content = fs.read_to_string(Path::new("/sources.md")).unwrap();
        assert!(content.contains("https://example.com/new-tool"));
        assert!(content.contains("New Tool"));
    }

    // --- PC-020: add creates file when missing ---

    #[test]
    fn add_creates_file() {
        let fs = InMemoryFileSystem::new();

        add(
            &fs,
            Path::new("/sources.md"),
            "https://example.com/new",
            "New Source",
            "doc",
            "assess",
            "testing",
            "human",
            "2026-03-29",
        )
        .unwrap();

        assert!(fs.exists(Path::new("/sources.md")));
        let content = fs.read_to_string(Path::new("/sources.md")).unwrap();
        assert!(content.contains("https://example.com/new"));
    }

    // --- PC-021: add rejects duplicate URL ---

    #[test]
    fn add_duplicate_rejected() {
        let fs = InMemoryFileSystem::new().with_file("/sources.md", &sample_sources_md());

        let result = add(
            &fs,
            Path::new("/sources.md"),
            "https://example.com/rust-testing",
            "Duplicate",
            "doc",
            "adopt",
            "testing",
            "human",
            "2026-03-29",
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, SourcesAppError::Domain(SourceError::DuplicateUrl(_))),
            "expected DuplicateUrl error, got: {err:?}"
        );
    }

    // --- PC-022: reindex moves inbox entries to quadrant sections ---

    #[test]
    fn reindex_moves_inbox() {
        let fs = InMemoryFileSystem::new().with_file("/sources.md", &sample_with_inbox());

        reindex(&fs, Path::new("/sources.md"), false).unwrap();

        let content = fs.read_to_string(Path::new("/sources.md")).unwrap();
        // Entry should now be in a quadrant section, not inbox
        assert!(content.contains("https://example.com/new"));
        // Inbox section should be empty
        let registry = parse_sources(&content).unwrap();
        assert!(
            registry.inbox.is_empty(),
            "inbox should be empty after reindex"
        );
        assert!(
            !registry.entries.is_empty(),
            "entries should contain the moved entry"
        );
    }

    // --- PC-023: reindex dry-run returns content without writing ---

    #[test]
    fn reindex_dry_run() {
        let fs = InMemoryFileSystem::new().with_file("/sources.md", &sample_with_inbox());

        // Record original content
        let original_content = fs.read_to_string(Path::new("/sources.md")).unwrap();

        let result = reindex(&fs, Path::new("/sources.md"), true).unwrap();

        assert!(result.is_some(), "dry-run should return content");
        let returned_content = result.unwrap();
        assert!(returned_content.contains("https://example.com/new"));

        // File should NOT be modified
        let current_content = fs.read_to_string(Path::new("/sources.md")).unwrap();
        assert_eq!(
            original_content, current_content,
            "dry-run must not write to disk"
        );
    }

    // --- PC-024: check flags stale on non-200 curl response ---

    #[test]
    fn check_stale_on_non_200() {
        let fs = InMemoryFileSystem::new().with_file("/sources.md", &sample_sources_md());

        let shell = MockShellExecutor::new()
            .with_response("https://example.com/rust-testing", non_200("404"));

        let results = check(&fs, &shell, Path::new("/sources.md"), "2026-03-29").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "https://example.com/rust-testing");
        assert_eq!(results[0].status, CheckStatus::Stale);

        // Entry should now have stale=true
        let content = fs.read_to_string(Path::new("/sources.md")).unwrap();
        let registry = parse_sources(&content).unwrap();
        let entry = registry
            .find_by_url("https://example.com/rust-testing")
            .unwrap();
        assert!(entry.stale, "entry should be marked stale");
    }

    // --- PC-025: check WARN on >90 days since last_checked ---

    #[test]
    fn check_warn_90_days() {
        // Use a date that is definitely >90 days before 2026-03-29
        let old_date = "2025-12-20"; // ~99 days before 2026-03-29
        let sources_md = format!(
            "# Knowledge Sources\n\n## Inbox\n\n\n## Adopt\n\n### testing\n- [Old Entry](https://example.com/old) — type: doc | subject: testing | added: 2025-01-01 | by: human | checked: {old_date}\n\n## Trial\n\n## Assess\n\n## Hold\n\n## Module Mapping\n\n| Module | Subjects |\n|--------|----------|\n"
        );

        let fs = InMemoryFileSystem::new().with_file("/sources.md", &sources_md);
        let shell = MockShellExecutor::new().with_response("https://example.com/old", ok_200());

        let results = check(&fs, &shell, Path::new("/sources.md"), "2026-03-29").unwrap();
        assert_eq!(results.len(), 1);
        assert!(
            matches!(results[0].status, CheckStatus::WarnAge(days) if days > 90),
            "expected WarnAge for >90 days, got: {:?}",
            results[0].status
        );
    }

    // --- PC-026: check ERROR on >180 days since last_checked ---

    #[test]
    fn check_error_180_days() {
        // Use a date that is definitely >180 days before 2026-03-29
        let old_date = "2025-09-20"; // ~190 days before 2026-03-29
        let sources_md = format!(
            "# Knowledge Sources\n\n## Inbox\n\n\n## Adopt\n\n### testing\n- [Very Old Entry](https://example.com/very-old) — type: doc | subject: testing | added: 2025-01-01 | by: human | checked: {old_date}\n\n## Trial\n\n## Assess\n\n## Hold\n\n## Module Mapping\n\n| Module | Subjects |\n|--------|----------|\n"
        );

        let fs = InMemoryFileSystem::new().with_file("/sources.md", &sources_md);
        let shell =
            MockShellExecutor::new().with_response("https://example.com/very-old", ok_200());

        let results = check(&fs, &shell, Path::new("/sources.md"), "2026-03-29").unwrap();
        assert_eq!(results.len(), 1);
        assert!(
            matches!(results[0].status, CheckStatus::ErrorAge(days) if days > 180),
            "expected ErrorAge for >180 days, got: {:?}",
            results[0].status
        );
    }

    // --- PC-027: check treats curl timeout as unreachable ---

    #[test]
    fn check_curl_timeout() {
        let fs = InMemoryFileSystem::new().with_file("/sources.md", &sample_sources_md());

        let shell = MockShellExecutor::new().with_error(
            "https://example.com/rust-testing",
            ShellError::Io("timeout".into()),
        );

        let results = check(&fs, &shell, Path::new("/sources.md"), "2026-03-29").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, CheckStatus::Unreachable);
    }

    // --- PC-028: check clears stale flag on successful recheck ---

    #[test]
    fn check_clears_stale() {
        let sources_md = "# Knowledge Sources\n\n## Inbox\n\n\n## Adopt\n\n### testing\n- [Stale Entry](https://example.com/stale) \u{2014} type: doc | subject: testing | added: 2026-01-01 | by: human | stale\n\n## Trial\n\n## Assess\n\n## Hold\n\n## Module Mapping\n\n| Module | Subjects |\n|--------|----------|\n";

        let fs = InMemoryFileSystem::new().with_file("/sources.md", sources_md);
        let shell = MockShellExecutor::new().with_response("https://example.com/stale", ok_200());

        let results = check(&fs, &shell, Path::new("/sources.md"), "2026-03-29").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, CheckStatus::Ok);

        // Stale flag should be cleared
        let content = fs.read_to_string(Path::new("/sources.md")).unwrap();
        let registry = parse_sources(&content).unwrap();
        let entry = registry.find_by_url("https://example.com/stale").unwrap();
        assert!(!entry.stale, "stale flag should be cleared on 200 response");
    }

    // --- PC-029: check atomic write after updates ---

    #[test]
    fn check_atomic_write() {
        let fs = InMemoryFileSystem::new().with_file("/sources.md", &sample_sources_md());

        let shell =
            MockShellExecutor::new().with_response("https://example.com/rust-testing", ok_200());

        check(&fs, &shell, Path::new("/sources.md"), "2026-03-29").unwrap();

        // Temp file should be cleaned up
        assert!(!fs.exists(Path::new("/sources.md.tmp")));
        // Main file should exist
        assert!(fs.exists(Path::new("/sources.md")));
    }
}
