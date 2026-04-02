//! Staleness detection for cartography entries.
//!
//! Compares `last_updated` date in CARTOGRAPHY-META markers against
//! source file modification dates. Pure string logic — no I/O.

/// Result of parsing a CARTOGRAPHY-META marker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CartographyMetaParsed {
    /// Date in YYYY-MM-DD format.
    pub last_updated: String,
    /// Source file paths listed in the marker.
    pub sources: Vec<String>,
}

/// Parse the `<!-- CARTOGRAPHY-META: last_updated=..., sources=... -->` marker
/// from document content.
///
/// Returns `None` if no marker is found.
pub fn parse_cartography_meta(content: &str) -> Option<CartographyMetaParsed> {
    let marker_prefix = "<!-- CARTOGRAPHY-META: ";
    let start = content.find(marker_prefix)?;
    let after_prefix = start + marker_prefix.len();
    let end = content[after_prefix..].find("-->")?;
    let inner = &content[after_prefix..after_prefix + end].trim();

    let mut last_updated = None;
    let mut sources = Vec::new();

    for part in inner.split(", ") {
        if let Some(val) = part.strip_prefix("last_updated=") {
            last_updated = Some(val.to_string());
        } else if let Some(val) = part.strip_prefix("sources=") {
            sources = val.split(',').map(|s| s.trim().to_string()).collect();
        }
    }

    Some(CartographyMetaParsed {
        last_updated: last_updated?,
        sources,
    })
}

/// Check whether a cartography entry is stale relative to source file dates.
///
/// `source_modified_dates` is a slice of `(source_path, modified_date)` pairs
/// where `modified_date` is a `YYYY-MM-DD` string.
///
/// Returns a stale marker string if any source is newer than `last_updated`,
/// or `None` if up-to-date (or if no CARTOGRAPHY-META marker is found).
pub fn check_staleness(
    content: &str,
    source_modified_dates: &[(&str, &str)],
) -> Option<String> {
    let meta = parse_cartography_meta(content)?;

    // Find the most recent modification date among listed sources
    let most_recent = source_modified_dates
        .iter()
        .filter(|(path, _)| meta.sources.contains(&path.to_string()))
        .map(|(_, date)| *date)
        .max()?;

    // Stale if any source was modified after last_updated
    if most_recent > meta.last_updated.as_str() {
        Some(format!(
            "<!-- STALE: last_updated={}, source_modified={} -->",
            meta.last_updated, most_recent
        ))
    } else {
        None
    }
}

/// Strip any `<!-- STALE: ... -->` markers from the given content.
///
/// Returns the content with all stale annotation lines removed.
pub fn remove_stale_marker(content: &str) -> String {
    content
        .lines()
        .filter(|line| !line.trim_start().starts_with("<!-- STALE:"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_cartography_meta ────────────────────────────────────────────────

    #[test]
    fn parses_meta_marker_with_single_source() {
        let content = "# Doc\n<!-- CARTOGRAPHY-META: last_updated=2026-01-01, sources=src/main.rs -->\nsome text";
        let meta = parse_cartography_meta(content).expect("should find meta");
        assert_eq!(meta.last_updated, "2026-01-01");
        assert_eq!(meta.sources, vec!["src/main.rs"]);
    }

    #[test]
    fn parses_meta_marker_with_multiple_sources() {
        let content =
            "<!-- CARTOGRAPHY-META: last_updated=2026-03-15, sources=src/a.rs,src/b.rs,src/c.rs -->";
        let meta = parse_cartography_meta(content).expect("should find meta");
        assert_eq!(meta.last_updated, "2026-03-15");
        assert_eq!(meta.sources, vec!["src/a.rs", "src/b.rs", "src/c.rs"]);
    }

    #[test]
    fn returns_none_when_no_meta_marker() {
        let content = "# Doc\nNo marker here.";
        assert!(parse_cartography_meta(content).is_none());
    }

    // ── check_staleness ───────────────────────────────────────────────────────

    #[test]
    fn returns_stale_marker_when_source_is_newer() {
        let content =
            "<!-- CARTOGRAPHY-META: last_updated=2026-01-01, sources=src/main.rs -->";
        let dates = [("src/main.rs", "2026-03-15")];
        let result = check_staleness(content, &dates);
        assert!(result.is_some(), "expected stale marker");
        let marker = result.unwrap();
        assert!(
            marker.contains("last_updated=2026-01-01"),
            "marker should include last_updated"
        );
        assert!(
            marker.contains("source_modified=2026-03-15"),
            "marker should include source_modified"
        );
        assert!(marker.starts_with("<!-- STALE:"), "marker should be HTML comment");
        assert!(marker.ends_with("-->"), "marker should close HTML comment");
    }

    #[test]
    fn returns_none_when_source_matches_last_updated() {
        let content =
            "<!-- CARTOGRAPHY-META: last_updated=2026-03-15, sources=src/main.rs -->";
        let dates = [("src/main.rs", "2026-03-15")];
        assert!(check_staleness(content, &dates).is_none());
    }

    #[test]
    fn returns_none_when_source_is_older_than_last_updated() {
        let content =
            "<!-- CARTOGRAPHY-META: last_updated=2026-03-15, sources=src/main.rs -->";
        let dates = [("src/main.rs", "2026-01-01")];
        assert!(check_staleness(content, &dates).is_none());
    }

    #[test]
    fn returns_none_when_no_meta_marker_present() {
        let content = "# Doc\nNo marker here.";
        let dates = [("src/main.rs", "2026-03-15")];
        assert!(check_staleness(content, &dates).is_none());
    }

    #[test]
    fn uses_most_recent_modified_date_when_multiple_sources_and_one_is_newer() {
        let content = "<!-- CARTOGRAPHY-META: last_updated=2026-01-01, sources=src/a.rs,src/b.rs -->";
        let dates = [("src/a.rs", "2025-12-01"), ("src/b.rs", "2026-06-01")];
        let result = check_staleness(content, &dates);
        assert!(result.is_some(), "expected stale marker because src/b.rs is newer");
        let marker = result.unwrap();
        assert!(marker.contains("source_modified=2026-06-01"));
    }

    #[test]
    fn returns_none_when_none_of_the_sources_are_listed_in_dates() {
        let content =
            "<!-- CARTOGRAPHY-META: last_updated=2026-01-01, sources=src/other.rs -->";
        let dates = [("src/main.rs", "2026-03-15")];
        assert!(check_staleness(content, &dates).is_none());
    }

    // ── remove_stale_marker ───────────────────────────────────────────────────

    #[test]
    fn removes_stale_marker_from_content() {
        let content = "# Doc\n<!-- STALE: last_updated=2026-01-01, source_modified=2026-03-15 -->\nBody text";
        let cleaned = remove_stale_marker(content);
        assert!(!cleaned.contains("<!-- STALE:"), "stale marker should be removed");
        assert!(cleaned.contains("# Doc"), "non-stale content should be preserved");
        assert!(cleaned.contains("Body text"), "body should be preserved");
    }

    #[test]
    fn noop_when_no_stale_marker_present() {
        let content = "# Doc\nNo stale marker.";
        let cleaned = remove_stale_marker(content);
        assert_eq!(cleaned, content);
    }

    #[test]
    fn removes_multiple_stale_markers() {
        let content = "<!-- STALE: last_updated=2026-01-01, source_modified=2026-03-01 -->\nMiddle\n<!-- STALE: last_updated=2026-02-01, source_modified=2026-04-01 -->";
        let cleaned = remove_stale_marker(content);
        assert!(!cleaned.contains("<!-- STALE:"));
        assert!(cleaned.contains("Middle"));
    }
}
