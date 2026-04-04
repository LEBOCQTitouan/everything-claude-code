//! Cross-reference matrix builder for element × journey × flow relationships.
//!
//! Zero I/O — returns a Markdown table string.

use crate::cartography::element_types::ElementEntry;

/// Build a Markdown cross-reference matrix.
///
/// Rows = one per element, columns = journey slugs (sorted) then flow slugs (sorted).
/// Cell = "Y" when the element participates, blank otherwise.
/// Returns a complete Markdown table string.
pub fn build_cross_reference_matrix(
    elements: &[ElementEntry],
    journey_slugs: &[String],
    flow_slugs: &[String],
) -> String {
    // Sort slugs alphabetically within each group; journeys first, then flows.
    let mut sorted_journeys: Vec<&str> = journey_slugs.iter().map(String::as_str).collect();
    sorted_journeys.sort_unstable();
    let mut sorted_flows: Vec<&str> = flow_slugs.iter().map(String::as_str).collect();
    sorted_flows.sort_unstable();

    let all_columns: Vec<&str> = sorted_journeys
        .iter()
        .chain(sorted_flows.iter())
        .copied()
        .collect();

    // Build header row
    let header = format!(
        "| Element | {} |",
        all_columns.join(" | ")
    );

    // Build separator row
    let separator = format!(
        "|---------|{}|",
        all_columns.iter().map(|c| format!("-{}-|", "-".repeat(c.len()))).collect::<String>()
    );

    let mut lines = vec![header, separator];

    for element in elements {
        let cells: Vec<String> = all_columns
            .iter()
            .map(|col| {
                let in_journeys = element.participating_journeys.iter().any(|j| j == col);
                let in_flows = element.participating_flows.iter().any(|f| f == col);
                if in_journeys || in_flows {
                    "Y".to_string()
                } else {
                    String::new()
                }
            })
            .collect();

        let row = format!(
            "| {} | {} |",
            element.slug,
            cells.join(" | ")
        );
        lines.push(row);
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cartography::element_types::{ElementEntry, ElementType};

    fn make_entry(slug: &str, journeys: &[&str], flows: &[&str]) -> ElementEntry {
        ElementEntry {
            slug: slug.to_string(),
            element_type: ElementType::Agent,
            purpose: String::new(),
            uses: vec![],
            used_by: vec![],
            participating_flows: flows.iter().map(|s| s.to_string()).collect(),
            participating_journeys: journeys.iter().map(|s| s.to_string()).collect(),
            sources: vec![],
            last_updated: String::new(),
        }
    }

    #[test]
    fn matrix_structure() {
        let elements = vec![
            make_entry("elem-1", &["journey-a"], &["flow-x"]),
            make_entry("elem-2", &["journey-b"], &["flow-y"]),
        ];
        let journeys = vec!["journey-a".to_string(), "journey-b".to_string()];
        let flows = vec!["flow-x".to_string(), "flow-y".to_string()];

        let table = build_cross_reference_matrix(&elements, &journeys, &flows);

        // Header row must contain "Element" as first column
        assert!(
            table.contains("| Element |"),
            "expected 'Element' column in header, got:\n{table}"
        );
        // Journey and flow columns present
        assert!(
            table.contains("journey-a"),
            "expected 'journey-a' column, got:\n{table}"
        );
        assert!(
            table.contains("journey-b"),
            "expected 'journey-b' column, got:\n{table}"
        );
        assert!(
            table.contains("flow-x"),
            "expected 'flow-x' column, got:\n{table}"
        );
        assert!(
            table.contains("flow-y"),
            "expected 'flow-y' column, got:\n{table}"
        );
        // elem-1 has Y for journey-a and flow-x
        let lines: Vec<&str> = table.lines().collect();
        let elem1_line = lines
            .iter()
            .find(|l| l.contains("elem-1"))
            .expect("elem-1 row not found");
        assert!(
            elem1_line.contains('Y'),
            "expected 'Y' in elem-1 row, got: {elem1_line}"
        );
    }

    #[test]
    fn journey_columns_before_flow() {
        let elements = vec![make_entry("elem-1", &["j-first"], &["f-last"])];
        let journeys = vec!["j-first".to_string()];
        let flows = vec!["f-last".to_string()];

        let table = build_cross_reference_matrix(&elements, &journeys, &flows);

        let header_line = table.lines().next().expect("table should have at least one line");
        let j_pos = header_line.find("j-first").expect("j-first not in header");
        let f_pos = header_line.find("f-last").expect("f-last not in header");
        assert!(
            j_pos < f_pos,
            "journey column should appear before flow column; header: {header_line}"
        );
    }

    #[test]
    fn empty_element_list() {
        let journeys = vec!["journey-a".to_string()];
        let flows = vec!["flow-x".to_string()];

        let table = build_cross_reference_matrix(&[], &journeys, &flows);

        // Must have header row
        assert!(
            table.contains("| Element |"),
            "expected header-only table with 'Element' column, got:\n{table}"
        );
        // Must have separator row
        let line_count = table.lines().count();
        assert!(
            line_count >= 2,
            "expected at least 2 lines (header + separator), got {line_count}"
        );
        // Should have no element rows (only header + separator)
        assert_eq!(
            line_count,
            2,
            "expected exactly 2 lines for empty element list, got {line_count}"
        );
    }
}
