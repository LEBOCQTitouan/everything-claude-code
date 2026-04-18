//! Backlog entry parsing — frontmatter extraction, ID parsing, error types.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Errors that can occur during backlog operations.
#[derive(Debug, thiserror::Error)]
pub enum BacklogError {
    /// No frontmatter delimiter found in the markdown file.
    #[error("no frontmatter delimiter found")]
    NoFrontmatter,

    /// YAML frontmatter could not be parsed.
    #[error("YAML parse error: {0}")]
    MalformedYaml(String),

    /// The backlog directory does not exist.
    #[error("backlog directory not found: {0}")]
    DirectoryNotFound(PathBuf),

    /// A query string was empty when it should not be.
    #[error("query must not be empty")]
    EmptyQuery,

    /// An I/O error occurred at a specific path.
    #[error("I/O error at {path}: {message}")]
    Io {
        /// The file path where the error occurred.
        path: String,
        /// The error message.
        message: String,
    },

    /// A reindex safety constraint was violated.
    #[error("reindex safety block: {0}")]
    SafetyBlock(String),
}

/// All valid kebab-case status strings for backlog entries.
pub const VALID_STATUSES: &[&str] = &["open", "in-progress", "implemented", "archived", "promoted"];

/// Backlog entry status with typed variants and Unknown fallback.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BacklogStatus {
    /// Entry is open and ready to start.
    Open,
    /// Entry is currently being worked on.
    #[serde(alias = "in-progress")]
    InProgress,
    /// Entry is implemented and completed.
    Implemented,
    /// Entry has been archived.
    Archived,
    /// Entry has been promoted.
    Promoted,
    /// A custom status that doesn't match the standard variants.
    #[serde(untagged)]
    Unknown(String),
}

impl BacklogStatus {
    /// Returns the display string for the status.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Open => "open",
            Self::InProgress => "in-progress",
            Self::Implemented => "implemented",
            Self::Archived => "archived",
            Self::Promoted => "promoted",
            Self::Unknown(s) => s,
        }
    }

    /// Whether this status represents an active entry (eligible for duplicate checking).
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Open | Self::InProgress)
    }

    /// Parse a kebab-case status string into a typed variant.
    ///
    /// Returns `None` for any string not in the 5 known valid statuses.
    /// The `Unknown` variant is intentionally not constructible via this method.
    pub fn from_kebab(s: &str) -> Option<Self> {
        match s {
            "open" => Some(Self::Open),
            "in-progress" => Some(Self::InProgress),
            "implemented" => Some(Self::Implemented),
            "archived" => Some(Self::Archived),
            "promoted" => Some(Self::Promoted),
            _ => None,
        }
    }
}

/// A parsed backlog entry from a BL-*.md file's YAML frontmatter.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BacklogEntry {
    /// Backlog entry ID, e.g., "BL-075".
    pub id: String,
    /// Short title describing the work.
    pub title: String,
    /// Current status (open, in-progress, implemented, archived, promoted).
    pub status: BacklogStatus,
    /// ISO 8601 date when the entry was created.
    pub created: String,
    /// Optional priority tier.
    #[serde(default)]
    pub tier: Option<String>,
    /// Optional scope indicator (e.g., HIGH, MEDIUM, LOW).
    #[serde(default)]
    pub scope: Option<String>,
    /// Optional target identifier (e.g., version, milestone).
    #[serde(default)]
    pub target: Option<String>,
    /// Optional target command (e.g., /spec dev).
    #[serde(default)]
    pub target_command: Option<String>,
    /// Optional list of tags for categorization.
    #[serde(default)]
    pub tags: Vec<String>,
}

impl BacklogEntry {
    /// Returns the effective target field, preferring `target_command` over `target`.
    pub fn effective_target(&self) -> &str {
        self.target_command
            .as_deref()
            .or(self.target.as_deref())
            .unwrap_or("—")
    }
}

/// Extract the numeric ID from a backlog filename like "BL-075-some-title.md".
///
/// Returns `None` if the filename doesn't match the BL-NNN pattern.
pub fn extract_id_from_filename(filename: &str) -> Option<u32> {
    let stripped = filename.strip_prefix("BL-")?;
    let end = stripped
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(stripped.len());
    let digits = &stripped[..end];
    if digits.is_empty() {
        return None;
    }
    digits.parse().ok()
}

/// Returns true if `filename` is the backlog file for the given numeric ID.
///
/// Uses `BL-{:03}` padding for IDs ≤ 999 and `BL-{id}` for IDs ≥ 1000.
/// Matches both `BL-NNN.md` (no slug) and `BL-NNN-<slug>.md` (with slug).
pub fn matches_backlog_filename(filename: &str, id: u32) -> bool {
    let prefix = if id <= 999 {
        format!("BL-{id:03}")
    } else {
        format!("BL-{id}")
    };
    filename == format!("{prefix}.md") || filename.starts_with(&format!("{prefix}-"))
}

/// Parse YAML frontmatter from markdown content.
///
/// Returns `Err(BacklogError)` if no frontmatter found or YAML is malformed.
pub fn parse_frontmatter(content: &str) -> Result<BacklogEntry, BacklogError> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return Err(BacklogError::NoFrontmatter);
    }
    let after_first = &trimmed["---".len()..];
    let end_pos = after_first
        .find("\n---")
        .ok_or(BacklogError::NoFrontmatter)?;
    let yaml_str = &after_first[..end_pos];
    serde_saphyr::from_str(yaml_str).map_err(|e| BacklogError::MalformedYaml(e.to_string()))
}

/// Update the `status:` field within YAML frontmatter in place.
///
/// Only modifies the first `status:` line within the frontmatter block (between the opening
/// `---` and the closing `\n---`). Everything outside that range — including the body — is
/// preserved character-for-character.
///
/// Returns the original content unchanged when the current status already equals `new_status`
/// (no-op guard). Strips YAML quotes from the existing value before comparison.
///
/// # Errors
///
/// - `BacklogError::NoFrontmatter` if the content has no valid frontmatter delimiters.
/// - `BacklogError::MalformedYaml("status field not found")` if no `status:` key is present
///   inside the frontmatter block.
pub fn replace_frontmatter_status(content: &str, new_status: &str) -> Result<String, BacklogError> {
    // Defense-in-depth: reject status strings with characters that could corrupt YAML
    if new_status.is_empty()
        || new_status
            .bytes()
            .any(|b| !b.is_ascii_alphanumeric() && b != b'-')
    {
        return Err(BacklogError::MalformedYaml(format!(
            "invalid status string: {new_status:?}"
        )));
    }
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return Err(BacklogError::NoFrontmatter);
    }
    let after_open = &trimmed["---".len()..];
    let close_pos = after_open
        .find("\n---")
        .ok_or(BacklogError::NoFrontmatter)?;
    let frontmatter = &after_open[..close_pos];

    // Locate first status: line inside frontmatter
    let status_line_offset = frontmatter
        .lines()
        .scan(0usize, |pos, line| {
            let start = *pos;
            *pos += line.len() + 1; // +1 for \n
            Some((start, line))
        })
        .find(|(_, line)| {
            let l = line.trim_start();
            l == "status:" || l.starts_with("status: ") || l.starts_with("status:\t")
        })
        .map(|(offset, _)| offset)
        .ok_or_else(|| BacklogError::MalformedYaml("status field not found".into()))?;

    // The line within after_open (skip past the leading \n after ---)
    let fm_start_in_after_open = 0usize;
    let line_start = fm_start_in_after_open + status_line_offset;
    let line_end = after_open[line_start..]
        .find('\n')
        .map(|n| line_start + n)
        .unwrap_or(after_open.len());
    let existing_line = &after_open[line_start..line_end];

    // No-op guard: return unchanged only when line is already exactly `status: {new_status}`
    // (unquoted). If the value matches but is quoted, we still rewrite to normalize quoting
    // per AC-002.4.
    let expected_line = format!("status: {new_status}");
    if existing_line == expected_line {
        return Ok(content.to_owned());
    }

    // Reconstruct: prefix + replacement line + suffix
    let leading_whitespace = content.len() - trimmed.len();
    let open_delim_len = "---".len();
    let prefix_len = leading_whitespace + open_delim_len + line_start;
    let suffix_start = leading_whitespace + open_delim_len + line_end;

    let mut result = String::with_capacity(content.len());
    result.push_str(&content[..prefix_len]);
    result.push_str("status: ");
    result.push_str(new_status);
    result.push_str(&content[suffix_start..]);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_id_from_filename_valid() {
        assert_eq!(extract_id_from_filename("BL-075-some-title.md"), Some(75));
        assert_eq!(extract_id_from_filename("BL-001-first.md"), Some(1));
        assert_eq!(extract_id_from_filename("BL-100.md"), Some(100));
    }

    #[test]
    fn extract_id_from_filename_non_bl() {
        assert_eq!(extract_id_from_filename("README.md"), None);
        assert_eq!(extract_id_from_filename("BACKLOG.md"), None);
        assert_eq!(extract_id_from_filename("notes.txt"), None);
        assert_eq!(extract_id_from_filename("BL-.md"), None);
    }

    #[test]
    fn parse_frontmatter_valid() {
        let content = "---\nid: BL-066\ntitle: Deterministic backlog\nstatus: open\ncreated: 2026-03-26\ntier: '9'\nscope: MEDIUM\ntarget_command: /spec dev\ntags: [deterministic, backlog]\n---\n\n# Body";
        let entry = parse_frontmatter(content).unwrap();
        assert_eq!(entry.id, "BL-066");
        assert_eq!(entry.title, "Deterministic backlog");
        assert_eq!(entry.status, BacklogStatus::Open);
        assert_eq!(entry.created, "2026-03-26");
        assert_eq!(entry.scope.as_deref(), Some("MEDIUM"));
        assert_eq!(entry.target_command.as_deref(), Some("/spec dev"));
        assert_eq!(entry.tags, vec!["deterministic", "backlog"]);
    }

    #[test]
    fn parse_frontmatter_malformed() {
        let content = "---\n{{{invalid yaml\n---\n";
        assert!(matches!(
            parse_frontmatter(content),
            Err(BacklogError::MalformedYaml(_))
        ));

        let content_no_delimiters = "just some text";
        assert!(matches!(
            parse_frontmatter(content_no_delimiters),
            Err(BacklogError::NoFrontmatter)
        ));
    }

    #[test]
    fn parse_frontmatter_optional_fields_missing() {
        let content =
            "---\nid: BL-001\ntitle: Minimal entry\nstatus: open\ncreated: 2026-03-20\n---\n";
        let entry = parse_frontmatter(content).unwrap();
        assert_eq!(entry.id, "BL-001");
        assert!(entry.tier.is_none());
        assert!(entry.scope.is_none());
        assert!(entry.target.is_none());
        assert!(entry.target_command.is_none());
        assert!(entry.tags.is_empty());
    }

    #[test]
    fn backlog_error_variants() {
        let errors: Vec<BacklogError> = vec![
            BacklogError::NoFrontmatter,
            BacklogError::MalformedYaml("test".into()),
            BacklogError::DirectoryNotFound(PathBuf::from("/tmp")),
            BacklogError::EmptyQuery,
            BacklogError::Io {
                path: "/tmp/test".into(),
                message: "not found".into(),
            },
        ];
        assert_eq!(errors.len(), 5);
        for err in &errors {
            assert!(!format!("{err}").is_empty());
        }
    }

    #[test]
    fn backlog_error_io_variant() {
        let err = BacklogError::Io {
            path: "/docs/backlog".into(),
            message: "permission denied".into(),
        };
        let display = format!("{err}");
        assert!(display.contains("/docs/backlog"));
        assert!(display.contains("permission denied"));
    }

    #[test]
    fn serialize_backlog_entry() {
        let entry = BacklogEntry {
            id: "BL-001".into(),
            title: "Test entry".into(),
            status: BacklogStatus::Open,
            created: "2026-04-07".into(),
            tier: None,
            scope: Some("LOW".into()),
            target: None,
            target_command: None,
            tags: vec!["test".into()],
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("BL-001"));
        assert!(json.contains("Test entry"));
        let roundtrip: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip["id"], "BL-001");
    }

    #[test]
    fn backlog_status_serde() {
        let yaml = "open";
        let status: BacklogStatus = serde_saphyr::from_str(yaml).unwrap();
        assert_eq!(status, BacklogStatus::Open);

        let yaml = "implemented";
        let status: BacklogStatus = serde_saphyr::from_str(yaml).unwrap();
        assert_eq!(status, BacklogStatus::Implemented);

        let yaml = "in-progress";
        let status: BacklogStatus = serde_saphyr::from_str(yaml).unwrap();
        assert_eq!(status, BacklogStatus::InProgress);
    }

    #[test]
    fn backlog_status_unknown_fallback() {
        let yaml = "custom-status";
        let status: BacklogStatus = serde_saphyr::from_str(yaml).unwrap();
        assert_eq!(status, BacklogStatus::Unknown("custom-status".into()));
        assert_eq!(status.as_str(), "custom-status");
        assert!(!status.is_active());
    }

    // PC-001: replace_frontmatter_status updates status line, preserves body
    #[test]
    fn replace_frontmatter_status_updates_status() {
        let content = "---\nid: BL-001\nstatus: open\ncreated: 2026-01-01\n---\n\n# Body text\nsome content here";
        let result = replace_frontmatter_status(content, "implemented").unwrap();
        assert!(result.contains("status: implemented"));
        assert!(!result.contains("status: open"));
        // Body preserved character-for-character
        assert!(result.ends_with("\n\n# Body text\nsome content here"));
    }

    // PC-002: No-op when status already matches
    #[test]
    fn replace_frontmatter_status_noop_same_status() {
        let content = "---\nid: BL-001\nstatus: open\ncreated: 2026-01-01\n---\n\n# Body";
        let result = replace_frontmatter_status(content, "open").unwrap();
        assert_eq!(result, content);
    }

    // PC-003: Error when no status: field in frontmatter
    #[test]
    fn replace_frontmatter_status_missing_status_field() {
        let content = "---\nid: BL-001\ncreated: 2026-01-01\n---\n\n# Body";
        let err = replace_frontmatter_status(content, "implemented").unwrap_err();
        assert!(matches!(err, BacklogError::MalformedYaml(_)));
        if let BacklogError::MalformedYaml(msg) = err {
            assert!(msg.contains("status field not found"));
        }
    }

    // PC-004: Updates only first status: line when duplicates exist
    #[test]
    fn replace_frontmatter_status_duplicate_keys() {
        let content =
            "---\nid: BL-001\nstatus: open\nstatus: archived\ncreated: 2026-01-01\n---\n\n# Body";
        let result = replace_frontmatter_status(content, "implemented").unwrap();
        // First occurrence updated
        let first_status_pos = result.find("status: ").unwrap();
        let first_status_line: &str = result[first_status_pos..].lines().next().unwrap();
        assert_eq!(first_status_line, "status: implemented");
        // Second occurrence unchanged
        assert!(result.contains("status: archived"));
    }

    // PC-005: Strips YAML quotes from status value
    #[test]
    fn replace_frontmatter_status_strips_quotes() {
        let content =
            "---\nid: BL-001\nstatus: \"implemented\"\ncreated: 2026-01-01\n---\n\n# Body";
        let result = replace_frontmatter_status(content, "implemented").unwrap();
        // No-op because quoted "implemented" == unquoted implemented
        assert_eq!(result, content.replace("\"implemented\"", "implemented"));
        // Also test double-quoted -> different status
        let content2 = "---\nid: BL-001\nstatus: \"open\"\ncreated: 2026-01-01\n---\n\n# Body";
        let result2 = replace_frontmatter_status(content2, "implemented").unwrap();
        assert!(result2.contains("status: implemented"));
        assert!(!result2.contains("\""));
    }

    // matches_backlog_filename tests

    // PC-006: Predicate matches BL-{:03} for IDs ≤ 999
    #[test]
    fn matches_backlog_filename_padded() {
        assert!(matches_backlog_filename("BL-001-foo.md", 1));
        assert!(matches_backlog_filename("BL-100.md", 100));
        assert!(matches_backlog_filename("BL-100-bar.md", 100));
        assert!(matches_backlog_filename("BL-099-zzz.md", 99));
        assert!(!matches_backlog_filename("BL-001-foo.md", 2));
        assert!(!matches_backlog_filename("BL-100-foo.md", 1));
        assert!(!matches_backlog_filename("foo.md", 1));
        assert!(!matches_backlog_filename("BL-001.txt", 1));
        assert!(!matches_backlog_filename("BL-001", 1));
    }

    // PC-007: Predicate matches BL-{id} for IDs ≥ 1000
    #[test]
    fn matches_backlog_filename_unpadded() {
        assert!(matches_backlog_filename("BL-1000-foo.md", 1000));
        assert!(matches_backlog_filename("BL-1000.md", 1000));
        assert!(matches_backlog_filename("BL-9999-zzz.md", 9999));
        assert!(matches_backlog_filename("BL-10000-foo.md", 10000));
        assert!(!matches_backlog_filename("BL-100-foo.md", 1000));
        assert!(!matches_backlog_filename("BL-1000-foo.md", 100));
    }

    // PC-006 (original numbering): Does not modify status: lines in body after closing ---
    #[test]
    fn replace_frontmatter_status_ignores_body_status() {
        let content = "---\nid: BL-001\nstatus: open\ncreated: 2026-01-01\n---\n\nstatus: this is in the body\n# Body";
        let result = replace_frontmatter_status(content, "implemented").unwrap();
        assert!(result.contains("status: implemented"));
        // Body status line unchanged
        assert!(result.contains("status: this is in the body"));
    }

    // PC-007: from_kebab returns Some for 5 valid statuses, None for unknown
    #[test]
    fn from_kebab_valid_and_invalid() {
        assert_eq!(BacklogStatus::from_kebab("open"), Some(BacklogStatus::Open));
        assert_eq!(
            BacklogStatus::from_kebab("in-progress"),
            Some(BacklogStatus::InProgress)
        );
        assert_eq!(
            BacklogStatus::from_kebab("implemented"),
            Some(BacklogStatus::Implemented)
        );
        assert_eq!(
            BacklogStatus::from_kebab("archived"),
            Some(BacklogStatus::Archived)
        );
        assert_eq!(
            BacklogStatus::from_kebab("promoted"),
            Some(BacklogStatus::Promoted)
        );
        assert_eq!(BacklogStatus::from_kebab("unknown"), None);
        assert_eq!(BacklogStatus::from_kebab("custom-status"), None);
        assert_eq!(BacklogStatus::from_kebab(""), None);
    }
}
