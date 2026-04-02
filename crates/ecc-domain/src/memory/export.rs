//! Pure export formatting for memory entries.

use crate::memory::entry::MemoryEntry;

/// Format a MemoryEntry as exportable markdown.
///
/// The output can be re-imported to reconstruct the entry (lossless round-trip).
pub fn format_entry_as_md(entry: &MemoryEntry) -> String {
    let mut lines = Vec::new();

    lines.push(format!("# {}", entry.title));
    lines.push(String::new());
    lines.push(format!("**Tier**: {}", entry.tier));
    lines.push(format!("**ID**: {}", entry.id));
    lines.push(format!("**Relevance Score**: {:.4}", entry.relevance_score));
    lines.push(format!("**Created**: {}", entry.created_at));
    lines.push(format!("**Updated**: {}", entry.updated_at));
    lines.push(format!("**Stale**: {}", entry.stale));

    if !entry.tags.is_empty() {
        lines.push(format!("**Tags**: {}", entry.tags.join(", ")));
    }

    if let Some(proj) = &entry.project_id {
        lines.push(format!("**Project**: {proj}"));
    }

    if !entry.related_work_items.is_empty() {
        lines.push(format!(
            "**Related Work Items**: {}",
            entry.related_work_items.join(", ")
        ));
    }

    if let Some(src) = &entry.source_path {
        lines.push(format!("**Source Path**: {src}"));
    }

    lines.push(String::new());
    lines.push("## Content".to_owned());
    lines.push(String::new());
    lines.push(entry.content.clone());

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::entry::{MemoryEntry, MemoryId};
    use crate::memory::tier::MemoryTier;

    fn make_entry() -> MemoryEntry {
        MemoryEntry::new(
            MemoryId(1),
            MemoryTier::Semantic,
            "Test Export Entry",
            "This is the content of the memory.",
            vec!["rust".to_owned(), "ddd".to_owned()],
            Some("proj-abc".to_owned()),
            None,
            0.95,
            "2026-01-01T00:00:00Z",
            "2026-01-02T00:00:00Z",
            false,
            vec!["BL-001".to_owned()],
            Some("/path/to/source.md".to_owned()),
        )
    }

    // PC-015: format_entry_as_md produces valid markdown with tier, title, tags, content
    #[test]
    fn test_format_entry_as_md_contains_title() {
        let entry = make_entry();
        let md = format_entry_as_md(&entry);
        assert!(
            md.contains("# Test Export Entry"),
            "should contain title as H1"
        );
    }

    #[test]
    fn test_format_entry_as_md_contains_tier() {
        let entry = make_entry();
        let md = format_entry_as_md(&entry);
        assert!(md.contains("semantic"), "should contain tier");
    }

    #[test]
    fn test_format_entry_as_md_contains_tags() {
        let entry = make_entry();
        let md = format_entry_as_md(&entry);
        assert!(md.contains("rust"), "should contain tags");
        assert!(md.contains("ddd"), "should contain all tags");
    }

    #[test]
    fn test_format_entry_as_md_contains_content() {
        let entry = make_entry();
        let md = format_entry_as_md(&entry);
        assert!(
            md.contains("This is the content of the memory."),
            "should contain content"
        );
    }

    #[test]
    fn test_format_entry_as_md_contains_related_work_items() {
        let entry = make_entry();
        let md = format_entry_as_md(&entry);
        assert!(md.contains("BL-001"), "should contain work item refs");
    }

    #[test]
    fn test_format_entry_as_md_no_tags_section_when_empty() {
        let entry = MemoryEntry::new(
            MemoryId(2),
            MemoryTier::Working,
            "No Tags",
            "content",
            vec![],
            None,
            None,
            1.0,
            "2026-01-01T00:00:00Z",
            "2026-01-01T00:00:00Z",
            false,
            vec![],
            None,
        );
        let md = format_entry_as_md(&entry);
        assert!(
            !md.contains("**Tags**"),
            "should not have Tags line when empty"
        );
    }

    #[test]
    fn test_format_entry_as_md_is_valid_markdown() {
        let entry = make_entry();
        let md = format_entry_as_md(&entry);
        // Basic markdown validity: starts with heading
        assert!(md.starts_with("# "), "should start with H1 heading");
    }
}
