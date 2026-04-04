//! Schema validation for cartography element files.
//!
//! Pure string validation -- checks for required markdown headers.
//! Zero I/O.

/// Validates an element file's required sections.
///
/// Required sections: `## Overview`, `## Responsibilities`, `## Interfaces`, `## Related Journeys`
///
/// Returns `Ok(())` if all required sections are present, or `Err(missing)` where
/// `missing` is a list of section names that are absent.
pub fn validate_element(content: &str) -> Result<(), Vec<String>> {
    let required = [
        ("## Overview", "Overview"),
        ("## Responsibilities", "Responsibilities"),
        ("## Interfaces", "Interfaces"),
        ("## Related Journeys", "Related Journeys"),
    ];

    let missing: Vec<String> = required
        .iter()
        .filter(|(header, _)| !content.contains(header))
        .map(|(_, name)| (*name).to_owned())
        .collect();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn element_passes_when_all_required_sections_present() {
        let content = "\
# Element: auth-service

## Overview
The authentication service.

## Responsibilities
- Handle login
- Issue tokens

## Interfaces
- POST /auth/login

## Related Journeys
- user-login-journey
";
        assert!(validate_element(content).is_ok());
    }

    #[test]
    fn element_reports_missing_sections_when_absent() {
        let content = "# Element: bad\n\n## Overview\nSome text.\n";
        let err = validate_element(content).unwrap_err();
        assert!(err.iter().any(|s| s.contains("Responsibilities")));
        assert!(err.iter().any(|s| s.contains("Interfaces")));
        assert!(err.iter().any(|s| s.contains("Related Journeys")));
    }
}
