//! Validation logic for element markdown files.
//!
//! Zero I/O — operates on `&str` content only.

/// Required sections for a valid element markdown file.
const REQUIRED_ELEMENT_SECTIONS: &[&str] = &[
    "## Overview",
    "## Relationships",
    "## Participating Flows",
    "## Participating Journeys",
];

/// Validate element markdown content, checking for all required sections.
///
/// Returns `Ok(())` when all required sections are present.
/// Returns `Err` with a list of missing section names when any are absent.
/// Section order is irrelevant — all lines are scanned.
pub fn validate_element(content: &str) -> Result<(), Vec<String>> {
    let mut missing: Vec<String> = REQUIRED_ELEMENT_SECTIONS
        .iter()
        .filter(|&&section| !content.lines().any(|line| line.trim() == section))
        .map(|&section| {
            // Return just the section name without the "## " prefix
            section.trim_start_matches("## ").to_string()
        })
        .collect();

    if missing.is_empty() {
        Ok(())
    } else {
        missing.sort();
        Err(missing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_element_all_sections() {
        let content = "\
## Overview

Some overview text.

## Relationships

Some relationships.

## Participating Flows

- flow-alpha

## Participating Journeys

- journey-beta
";
        assert_eq!(validate_element(content), Ok(()));
    }

    #[test]
    fn missing_sections_reported() {
        let content = "\
## Overview

Only the overview section is present.
";
        let result = validate_element(content);
        assert!(result.is_err(), "expected Err for missing sections");
        let missing = result.unwrap_err();
        assert!(
            missing.contains(&"Relationships".to_string()),
            "expected 'Relationships' in missing list, got: {missing:?}"
        );
        assert!(
            missing.contains(&"Participating Flows".to_string()),
            "expected 'Participating Flows' in missing list, got: {missing:?}"
        );
        assert!(
            missing.contains(&"Participating Journeys".to_string()),
            "expected 'Participating Journeys' in missing list, got: {missing:?}"
        );
    }

    #[test]
    fn section_order_independent() {
        let content = "\
## Participating Journeys

- journey-one

## Participating Flows

- flow-one

## Relationships

Uses nothing.

## Overview

Reversed order element.
";
        assert_eq!(validate_element(content), Ok(()));
    }
}
