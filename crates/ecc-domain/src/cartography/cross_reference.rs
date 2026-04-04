//! Cross-reference matrix for cartography elements, journeys, and flows.
//!
//! Pure domain function — no I/O.

use crate::cartography::element_types::ElementEntry;

/// Build the cross-reference INDEX.md content from element entries and slugs.
///
/// Produces a markdown table mapping each element slug to the journeys and flows
/// that reference it. This is a full-replacement document (not delta-merged).
pub fn build_cross_reference_matrix(
    elements: &[ElementEntry],
    journey_slugs: &[String],
    flow_slugs: &[String],
) -> String {
    let mut lines = vec![
        "# Cartography Elements — Cross-Reference Index".to_string(),
        String::new(),
        "Auto-generated. Do not edit manually.".to_string(),
        String::new(),
        "| Element | Journeys | Flows |".to_string(),
        "|---------|----------|-------|".to_string(),
    ];

    for element in elements {
        let referenced_journeys: Vec<&String> = journey_slugs
            .iter()
            .filter(|slug| {
                element
                    .content
                    .contains(&format!("/{slug}"))
                    || element.content.contains(&format!("[{slug}]"))
            })
            .collect();

        let referenced_flows: Vec<&String> = flow_slugs
            .iter()
            .filter(|slug| {
                element
                    .content
                    .contains(&format!("/{slug}"))
                    || element.content.contains(&format!("[{slug}]"))
            })
            .collect();

        let journeys_str = if referenced_journeys.is_empty() {
            "-".to_string()
        } else {
            referenced_journeys
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        };

        let flows_str = if referenced_flows.is_empty() {
            "-".to_string()
        } else {
            referenced_flows
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        };

        lines.push(format!(
            "| {} | {} | {} |",
            element.slug, journeys_str, flows_str
        ));
    }

    if elements.is_empty() {
        lines.push("| _(none)_ | — | — |".to_string());
    }

    lines.push(String::new());
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_elements_produces_none_row() {
        let result = build_cross_reference_matrix(&[], &[], &[]);
        assert!(result.contains("_(none)_"));
    }

    #[test]
    fn element_with_no_refs_shows_dashes() {
        let entries = vec![ElementEntry {
            slug: "cartographer".to_string(),
            content: "# Cartographer\n\nSome content.".to_string(),
        }];
        let result = build_cross_reference_matrix(&entries, &[], &[]);
        assert!(result.contains("| cartographer | - | - |"));
    }
}
