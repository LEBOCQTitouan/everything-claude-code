//! Backlog entry parsing — frontmatter extraction and ID parsing.

use serde::Deserialize;

/// A parsed backlog entry from a BL-*.md file's YAML frontmatter.
#[derive(Debug, Clone, Deserialize)]
pub struct BacklogEntry {
    pub id: String,
    pub title: String,
    pub status: String,
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
    let digits: String = stripped.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse().ok()
}

/// Parse YAML frontmatter from markdown content.
///
/// Expects content with `---` delimiters:
/// ```text
/// ---
/// id: BL-001
/// title: Example
/// status: open
/// created: 2026-03-20
/// ---
/// ```
///
/// Returns `Err` if no frontmatter found or YAML is malformed.
pub fn parse_frontmatter(content: &str) -> Result<BacklogEntry, String> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return Err("no frontmatter delimiter found".to_string());
    }
    let after_first = &trimmed[3..];
    let end_pos = after_first
        .find("\n---")
        .ok_or_else(|| "no closing frontmatter delimiter found".to_string())?;
    let yaml_str = &after_first[..end_pos];
    serde_yaml::from_str(yaml_str).map_err(|e| format!("YAML parse error: {e}"))
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
        assert_eq!(entry.status, "open");
        assert_eq!(entry.created, "2026-03-26");
        assert_eq!(entry.scope.as_deref(), Some("MEDIUM"));
        assert_eq!(entry.target_command.as_deref(), Some("/spec dev"));
        assert_eq!(entry.tags, vec!["deterministic", "backlog"]);
    }

    #[test]
    fn parse_frontmatter_malformed() {
        let content = "---\n{{{invalid yaml\n---\n";
        assert!(parse_frontmatter(content).is_err());

        let content_no_delimiters = "just some text";
        assert!(parse_frontmatter(content_no_delimiters).is_err());
    }

    #[test]
    fn parse_frontmatter_optional_fields_missing() {
        let content = "---\nid: BL-001\ntitle: Minimal entry\nstatus: open\ncreated: 2026-03-20\n---\n";
        let entry = parse_frontmatter(content).unwrap();
        assert_eq!(entry.id, "BL-001");
        assert!(entry.tier.is_none());
        assert!(entry.scope.is_none());
        assert!(entry.target.is_none());
        assert!(entry.target_command.is_none());
        assert!(entry.tags.is_empty());
    }
}
