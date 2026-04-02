//! Pure CONTEXT.md formatting for memory entries.

use crate::memory::entry::MemoryEntry;

const DEFAULT_TOP_N: usize = 10;
const MAX_LINES: usize = 200;
const CONTENT_PREVIEW_CHARS: usize = 200;

/// Format a slice of MemoryEntry values as a CONTEXT.md markdown document.
///
/// - If `entries` is empty, returns "No memories stored".
/// - Selects top `max_lines`-aware subset (at most `DEFAULT_TOP_N` entries).
/// - Each entry shows tier, title, relevance score, and truncated content.
/// - Total output is capped at `max_lines` lines.
pub fn format_context_md(entries: &[MemoryEntry], max_lines: usize) -> String {
    if entries.is_empty() {
        return "No memories stored".to_owned();
    }

    let effective_max = max_lines.min(MAX_LINES);
    let top_n = DEFAULT_TOP_N.min(entries.len());
    let selected = &entries[..top_n];

    let mut lines: Vec<String> = vec!["# Relevant Memory Context".to_owned(), String::new()];

    for entry in selected {
        let content_preview = if entry.content.len() > CONTENT_PREVIEW_CHARS {
            format!("{}…", &entry.content[..CONTENT_PREVIEW_CHARS])
        } else {
            entry.content.clone()
        };

        lines.push(format!(
            "## [{}] {} (score: {:.2})",
            entry.tier, entry.title, entry.relevance_score
        ));
        lines.push(String::new());
        lines.push(content_preview);
        lines.push(String::new());

        if lines.len() >= effective_max {
            break;
        }
    }

    // Trim to max_lines
    if lines.len() > effective_max {
        lines.truncate(effective_max);
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::entry::{MemoryEntry, MemoryId};
    use crate::memory::tier::MemoryTier;

    fn make_entry(id: i64, title: &str, score: f64, tier: MemoryTier) -> MemoryEntry {
        MemoryEntry::new(
            MemoryId(id),
            tier,
            title,
            "Some content for this memory entry.",
            vec!["rust".to_owned()],
            None,
            None,
            score,
            "2026-01-01T00:00:00Z",
            "2026-01-01T00:00:00Z",
            false,
            vec![],
            None,
        )
    }

    // PC-016: format_context_md with 0 entries returns "No memories stored"
    #[test]
    fn test_format_context_md_empty_entries() {
        let result = format_context_md(&[], 200);
        assert_eq!(result, "No memories stored");
    }

    // PC-017: format_context_md with 15 entries truncates to top-10 within 200 lines
    #[test]
    fn test_format_context_md_15_entries_truncates_to_10() {
        let entries: Vec<MemoryEntry> = (0..15)
            .map(|i| make_entry(i, &format!("Entry {i}"), 1.0, MemoryTier::Episodic))
            .collect();
        let result = format_context_md(&entries, 200);
        // Should contain entries 0-9, not 10-14
        assert!(result.contains("Entry 0"));
        assert!(result.contains("Entry 9"));
        assert!(!result.contains("Entry 10"));
        assert!(!result.contains("Entry 14"));
    }

    #[test]
    fn test_format_context_md_respects_max_lines() {
        let entries: Vec<MemoryEntry> = (0..10)
            .map(|i| make_entry(i, &format!("Entry {i}"), 1.0, MemoryTier::Episodic))
            .collect();
        // With max_lines=5, output should be very short
        let result = format_context_md(&entries, 5);
        let line_count = result.lines().count();
        assert!(line_count <= 5, "expected <= 5 lines, got {line_count}");
    }

    // PC-018: format_context_md entries show tier, title, relevance score, truncated content
    #[test]
    fn test_format_context_md_shows_tier() {
        let entries = vec![make_entry(1, "My Entry", 0.75, MemoryTier::Semantic)];
        let result = format_context_md(&entries, 200);
        assert!(result.contains("semantic"), "should show tier");
    }

    #[test]
    fn test_format_context_md_shows_title() {
        let entries = vec![make_entry(
            1,
            "Important Knowledge",
            1.0,
            MemoryTier::Semantic,
        )];
        let result = format_context_md(&entries, 200);
        assert!(result.contains("Important Knowledge"), "should show title");
    }

    #[test]
    fn test_format_context_md_shows_relevance_score() {
        let entries = vec![make_entry(1, "Entry", 0.75, MemoryTier::Episodic)];
        let result = format_context_md(&entries, 200);
        assert!(result.contains("0.75"), "should show relevance score");
    }

    #[test]
    fn test_format_context_md_truncates_long_content() {
        let long_content = "x".repeat(300);
        let entry = MemoryEntry::new(
            MemoryId(1),
            MemoryTier::Episodic,
            "Long Content Entry",
            long_content,
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
        let result = format_context_md(&[entry], 200);
        // The x string should be truncated with ellipsis
        assert!(result.contains('…'), "should contain truncation marker");
    }

    #[test]
    fn test_format_context_md_short_content_not_truncated() {
        let entries = vec![make_entry(1, "Short", 1.0, MemoryTier::Working)];
        let result = format_context_md(&entries, 200);
        assert!(
            result.contains("Some content for this memory entry."),
            "short content should appear verbatim"
        );
    }
}
