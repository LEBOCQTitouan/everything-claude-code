//! Module summary marker-based insertion for MODULE-SUMMARIES.md.

const START_MARKER: &str = "<!-- IMPLEMENT-GENERATED -->";
const END_MARKER: &str = "<!-- END IMPLEMENT-GENERATED -->";

/// Extract unique crate names from changed file paths.
pub fn identify_crate_paths(changed_files: &[String]) -> Vec<String> {
    let mut crates: Vec<String> = changed_files
        .iter()
        .filter_map(|f| {
            if f.starts_with("crates/") {
                f.strip_prefix("crates/")
                    .and_then(|rest| rest.split('/').next())
                    .map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    crates.sort();
    crates
}

/// Format a single module summary entry.
pub fn format_entry(crate_name: &str, feature: &str) -> String {
    format!(
        "### {crate_name}\n\
         Updated as part of {feature}.\n\
         **Key Functions / Types:** (see source)\n\
         **Spec Cross-Link:** {feature}\n\
         **Modified in:** {feature}\n"
    )
}

/// Insert/update entries between markers in MODULE-SUMMARIES.md content.
pub fn insert_entries(
    content: &str,
    entries: &[(String, String)], // (crate_name, formatted_entry)
) -> String {
    let start = content.find(START_MARKER);
    let end = content.find(END_MARKER);

    match (start, end) {
        (Some(s), Some(e)) => {
            let before = &content[..s + START_MARKER.len()];
            let after = &content[e..];
            let mut block = String::from("\n\n");
            for (_, entry) in entries {
                block.push_str(entry);
                block.push('\n');
            }
            format!("{before}{block}{after}")
        }
        _ => {
            // No markers — append with markers
            let mut result = content.to_string();
            result.push_str(&format!("\n{START_MARKER}\n\n"));
            for (_, entry) in entries {
                result.push_str(entry);
                result.push('\n');
            }
            result.push_str(&format!("{END_MARKER}\n"));
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identify_crate_paths_filters() {
        let files = vec![
            "crates/ecc-domain/src/drift/mod.rs".to_string(),
            "crates/ecc-app/src/lib.rs".to_string(),
            "agents/drift-checker.md".to_string(),
            "docs/README.md".to_string(),
        ];
        let crates = identify_crate_paths(&files);
        assert_eq!(crates, vec!["ecc-app", "ecc-domain"]);
    }

    #[test]
    fn insert_between_markers() {
        let content = "# Header\n<!-- IMPLEMENT-GENERATED -->\nold content\n<!-- END IMPLEMENT-GENERATED -->\n# Footer";
        let entries = vec![("ecc-domain".to_string(), format_entry("ecc-domain", "BL-126"))];
        let result = insert_entries(content, &entries);
        assert!(result.contains("### ecc-domain"));
        assert!(result.contains("# Footer"));
        assert!(!result.contains("old content"));
    }

    #[test]
    fn insert_creates_markers_when_missing() {
        let content = "# MODULE SUMMARIES\n\nSome existing content.";
        let entries = vec![("ecc-cli".to_string(), format_entry("ecc-cli", "BL-126"))];
        let result = insert_entries(content, &entries);
        assert!(result.contains(START_MARKER));
        assert!(result.contains(END_MARKER));
        assert!(result.contains("### ecc-cli"));
    }

    #[test]
    fn empty_changed_files() {
        let crates = identify_crate_paths(&[]);
        assert!(crates.is_empty());
    }
}
