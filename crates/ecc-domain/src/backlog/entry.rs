//! Backlog entry parsing — frontmatter extraction, ID parsing, error types.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Errors that can occur during backlog operations.
#[derive(Debug, thiserror::Error)]
pub enum BacklogError {
    #[error("no frontmatter delimiter found")]
    NoFrontmatter,

    #[error("YAML parse error: {0}")]
    MalformedYaml(String),

    #[error("backlog directory not found: {0}")]
    DirectoryNotFound(PathBuf),

    #[error("query must not be empty")]
    EmptyQuery,

    #[error("I/O error: {0}")]
    IoError(String),
}

/// Backlog entry status with typed variants and Unknown fallback.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BacklogStatus {
    Open,
    #[serde(alias = "in-progress")]
    InProgress,
    Implemented,
    Archived,
    Promoted,
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
}

/// A parsed backlog entry from a BL-*.md file's YAML frontmatter.
#[derive(Debug, Clone, Deserialize)]
pub struct BacklogEntry {
    pub id: String,
    pub title: String,
    pub status: BacklogStatus,
    pub created: String,
    #[serde(default)]
    pub tier: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub target_command: Option<String>,
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
    serde_yml::from_str(yaml_str).map_err(|e| BacklogError::MalformedYaml(e.to_string()))
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
        // Verify all 5 variants exist and have Display
        let errors: Vec<BacklogError> = vec![
            BacklogError::NoFrontmatter,
            BacklogError::MalformedYaml("test".into()),
            BacklogError::DirectoryNotFound(PathBuf::from("/tmp")),
            BacklogError::EmptyQuery,
            BacklogError::IoError("test".into()),
        ];
        assert_eq!(errors.len(), 5);
        for err in &errors {
            assert!(!format!("{err}").is_empty());
        }
    }

    #[test]
    fn backlog_status_serde() {
        let yaml = "open";
        let status: BacklogStatus = serde_yml::from_str(yaml).unwrap();
        assert_eq!(status, BacklogStatus::Open);

        let yaml = "implemented";
        let status: BacklogStatus = serde_yml::from_str(yaml).unwrap();
        assert_eq!(status, BacklogStatus::Implemented);

        let yaml = "in-progress";
        let status: BacklogStatus = serde_yml::from_str(yaml).unwrap();
        assert_eq!(status, BacklogStatus::InProgress);
    }

    #[test]
    fn backlog_status_unknown_fallback() {
        let yaml = "custom-status";
        let status: BacklogStatus = serde_yml::from_str(yaml).unwrap();
        assert_eq!(status, BacklogStatus::Unknown("custom-status".into()));
        assert_eq!(status.as_str(), "custom-status");
        assert!(!status.is_active());
    }
}
