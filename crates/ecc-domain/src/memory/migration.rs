//! Pure migration parsing functions for legacy memory files.

use crate::memory::entry::{MemoryEntry, MemoryId};
use crate::memory::tier::MemoryTier;
use regex::Regex;
use std::sync::LazyLock;

static WORK_ITEM_REF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"BL-\d{3,}").expect("BUG: invalid WORK_ITEM_REF_RE regex"));

/// Extract all BL-NNN references from content.
pub fn extract_work_item_refs(content: &str) -> Vec<String> {
    let mut refs: Vec<String> = WORK_ITEM_REF_RE
        .find_iter(content)
        .map(|m| m.as_str().to_owned())
        .collect();
    refs.dedup();
    refs
}

/// Parse a work-item markdown file into a MemoryEntry.
///
/// Returns `None` if the content cannot be parsed meaningfully.
pub fn parse_work_item_md(content: &str) -> Option<MemoryEntry> {
    // Extract title from first `# ...` heading
    let title = content
        .lines()
        .find(|l| l.starts_with("# "))
        .map(|l| l.trim_start_matches("# ").trim().to_owned())?;

    if title.is_empty() {
        return None;
    }

    let related = extract_work_item_refs(content);

    Some(MemoryEntry::new(
        MemoryId(0),
        MemoryTier::Episodic,
        title,
        content.to_owned(),
        vec![],
        None,
        None,
        1.0,
        "2026-01-01T00:00:00Z",
        "2026-01-01T00:00:00Z",
        false,
        related,
        None,
    ))
}

/// Parse a single action-log JSON entry into a MemoryEntry.
///
/// Returns `None` for malformed or missing required fields.
pub fn parse_action_log_entry(json: &serde_json::Value) -> Option<MemoryEntry> {
    let title = json.get("action")?.as_str()?.to_owned();
    if title.is_empty() {
        return None;
    }

    let content = json
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_owned();

    let timestamp = json
        .get("timestamp")
        .and_then(|v| v.as_str())
        .unwrap_or("2026-01-01T00:00:00Z")
        .to_owned();

    let related = extract_work_item_refs(&content);

    Some(MemoryEntry::new(
        MemoryId(0),
        MemoryTier::Episodic,
        title,
        content,
        vec![],
        None,
        None,
        1.0,
        timestamp.clone(),
        timestamp,
        false,
        related,
        None,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // PC-012: parse_work_item_md extracts title, content, BL-NNN references
    #[test]
    fn test_parse_work_item_md_basic() {
        let md = "# Implement Feature BL-042\n\nThis is the content for BL-042 and BL-043.\n";
        let entry = parse_work_item_md(md).expect("should parse");
        assert_eq!(entry.title, "Implement Feature BL-042");
        assert!(entry.content.contains("BL-042"));
        assert!(entry.related_work_items.contains(&"BL-042".to_owned()));
        assert!(entry.related_work_items.contains(&"BL-043".to_owned()));
    }

    #[test]
    fn test_parse_work_item_md_tier_is_episodic() {
        let md = "# My Work Item\n\nContent here.\n";
        let entry = parse_work_item_md(md).expect("should parse");
        assert_eq!(entry.tier, MemoryTier::Episodic);
    }

    #[test]
    fn test_parse_work_item_md_no_heading_returns_none() {
        let md = "Just some content without a heading.";
        let result = parse_work_item_md(md);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_work_item_md_no_bl_refs() {
        let md = "# Clean Entry\n\nNo references here.";
        let entry = parse_work_item_md(md).expect("should parse");
        assert!(entry.related_work_items.is_empty());
    }

    // PC-013: parse_action_log_entry extracts action fields; malformed JSON returns None
    #[test]
    fn test_parse_action_log_entry_basic() {
        let v = json!({
            "action": "implement feature",
            "description": "Implemented BL-010 feature",
            "timestamp": "2026-01-15T12:00:00Z"
        });
        let entry = parse_action_log_entry(&v).expect("should parse");
        assert_eq!(entry.title, "implement feature");
        assert!(entry.content.contains("BL-010"));
        assert_eq!(entry.created_at, "2026-01-15T12:00:00Z");
    }

    #[test]
    fn test_parse_action_log_entry_missing_action_returns_none() {
        let v = json!({ "description": "no action field" });
        assert!(parse_action_log_entry(&v).is_none());
    }

    #[test]
    fn test_parse_action_log_entry_empty_action_returns_none() {
        let v = json!({ "action": "", "description": "empty" });
        assert!(parse_action_log_entry(&v).is_none());
    }

    #[test]
    fn test_parse_action_log_entry_null_returns_none() {
        let v = serde_json::Value::Null;
        assert!(parse_action_log_entry(&v).is_none());
    }

    #[test]
    fn test_parse_action_log_entry_tier_is_episodic() {
        let v = json!({ "action": "test action", "description": "desc" });
        let entry = parse_action_log_entry(&v).expect("should parse");
        assert_eq!(entry.tier, MemoryTier::Episodic);
    }

    // PC-014: extract_work_item_refs finds all BL-NNN patterns
    #[test]
    fn test_extract_refs_single() {
        let refs = extract_work_item_refs("Working on BL-042 today");
        assert_eq!(refs, vec!["BL-042"]);
    }

    #[test]
    fn test_extract_refs_multiple() {
        let refs = extract_work_item_refs("BL-001 and BL-002 and BL-003");
        assert_eq!(refs.len(), 3);
        assert!(refs.contains(&"BL-001".to_owned()));
        assert!(refs.contains(&"BL-002".to_owned()));
        assert!(refs.contains(&"BL-003".to_owned()));
    }

    #[test]
    fn test_extract_refs_none() {
        let refs = extract_work_item_refs("No references here.");
        assert!(refs.is_empty());
    }

    #[test]
    fn test_extract_refs_dedup() {
        let refs = extract_work_item_refs("BL-001 and BL-001 again");
        // dedup removes consecutive duplicates
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0], "BL-001");
    }

    #[test]
    fn test_extract_refs_long_number() {
        let refs = extract_work_item_refs("Fixed in BL-1234");
        assert_eq!(refs, vec!["BL-1234"]);
    }
}
