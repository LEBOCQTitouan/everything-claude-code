//! Index generation — BACKLOG.md table and stats from parsed entries.

use super::entry::BacklogEntry;
use std::collections::BTreeMap;

/// Generate a markdown index table from entries, sorted by numeric ID.
pub fn generate_index_table(entries: &[BacklogEntry]) -> String {
    let mut sorted: Vec<&BacklogEntry> = entries.iter().collect();
    sorted.sort_by_key(|e| {
        e.id
            .strip_prefix("BL-")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0)
    });

    let mut lines = Vec::new();
    lines.push("| ID | Title | Tier | Scope | Target | Status | Created |".to_string());
    lines.push("|----|-------|------|-------|--------|--------|---------|".to_string());

    for entry in &sorted {
        let tier = entry.tier.as_deref().unwrap_or("—");
        let scope = entry.scope.as_deref().unwrap_or("—");
        let target = entry.effective_target();
        lines.push(format!(
            "| {} | {} | {} | {} | {} | {} | {} |",
            entry.id, entry.title, tier, scope, target, entry.status, entry.created
        ));
    }

    lines.join("\n")
}

/// Generate a stats section from entries.
pub fn generate_stats(entries: &[BacklogEntry]) -> String {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for entry in entries {
        *counts.entry(entry.status.to_lowercase()).or_insert(0) += 1;
    }

    let total = entries.len();
    let mut lines = Vec::new();
    lines.push("## Stats".to_string());
    lines.push(String::new());
    lines.push(format!("- **Total:** {total}"));

    // Standard order for known statuses
    let ordered = ["open", "in-progress", "implemented", "archived", "promoted"];
    for status in &ordered {
        if let Some(count) = counts.get(*status) {
            let label = capitalize(status);
            lines.push(format!("- **{label}:** {count}"));
        }
    }
    // Any remaining statuses not in the standard order
    for (status, count) in &counts {
        if !ordered.contains(&status.as_str()) {
            let label = capitalize(status);
            lines.push(format!("- **{label}:** {count}"));
        }
    }

    lines.join("\n")
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

/// Extract the `## Dependency Graph` section from existing BACKLOG.md content.
///
/// Returns `None` if the section is not found.
pub fn extract_dependency_graph(content: &str) -> Option<String> {
    let marker = "## Dependency Graph";
    let start = content.find(marker)?;
    let section = &content[start..];
    // Find the next `##` after the marker line
    let after_marker = &section[marker.len()..];
    let end = after_marker
        .find("\n## ")
        .map(|pos| marker.len() + pos)
        .unwrap_or(section.len());
    let result = section[..end].trim_end().to_string();
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, title: &str, status: &str, created: &str) -> BacklogEntry {
        BacklogEntry {
            id: id.to_string(),
            title: title.to_string(),
            status: status.to_string(),
            created: created.to_string(),
            tier: None,
            scope: Some("MEDIUM".to_string()),
            target: None,
            target_command: Some("/spec dev".to_string()),
            tags: vec![],
        }
    }

    #[test]
    fn generate_index_table_sorted() {
        let entries = vec![
            entry("BL-003", "Third", "open", "2026-03-03"),
            entry("BL-001", "First", "implemented", "2026-03-01"),
            entry("BL-002", "Second", "open", "2026-03-02"),
        ];
        let table = generate_index_table(&entries);
        let lines: Vec<&str> = table.lines().collect();
        assert_eq!(lines.len(), 5); // header + separator + 3 rows
        assert!(lines[0].contains("| ID |"));
        assert!(lines[2].contains("BL-001"));
        assert!(lines[3].contains("BL-002"));
        assert!(lines[4].contains("BL-003"));
    }

    #[test]
    fn generate_stats_counts() {
        let entries = vec![
            entry("BL-001", "A", "open", "2026-01-01"),
            entry("BL-002", "B", "open", "2026-01-02"),
            entry("BL-003", "C", "implemented", "2026-01-03"),
            entry("BL-004", "D", "archived", "2026-01-04"),
            entry("BL-005", "E", "open", "2026-01-05"),
        ];
        let stats = generate_stats(&entries);
        assert!(stats.contains("**Total:** 5"));
        assert!(stats.contains("**Open:** 3"));
        assert!(stats.contains("**Implemented:** 1"));
        assert!(stats.contains("**Archived:** 1"));
    }

    #[test]
    fn extract_dependency_graph_present() {
        let content = "# Backlog\n\n| table |\n\n## Dependency Graph\n\n```\nBL-001 → BL-002\n```\n\n## Stats\n\n- Total: 2\n";
        let graph = extract_dependency_graph(content);
        assert!(graph.is_some());
        let graph = graph.unwrap();
        assert!(graph.contains("BL-001 → BL-002"));
        assert!(!graph.contains("## Stats"));
    }

    #[test]
    fn extract_dependency_graph_absent() {
        let content = "# Backlog\n\n| table |\n\n## Stats\n\n- Total: 2\n";
        assert!(extract_dependency_graph(content).is_none());
    }
}
