use ecc_domain::config::validate::REQUIRED_PATTERN_SECTIONS;

/// Check that all required sections are present and have non-empty bodies.
pub(super) fn validate_sections(content: &str, label: &str) -> (String, bool) {
    let mut errors = String::new();
    let mut has_errors = false;

    for &section in REQUIRED_PATTERN_SECTIONS {
        let heading = format!("## {section}");
        if !content.contains(&heading) {
            errors.push_str(&format!(
                "ERROR: {label} - Missing required section '{section}'\n"
            ));
            has_errors = true;
        } else if section_body_is_empty(content, section) {
            errors.push_str(&format!(
                "ERROR: {label} - Section '{section}' has empty body\n"
            ));
            has_errors = true;
        }
    }

    (errors, !has_errors)
}

/// Returns true if the `## <section>` heading is present but its body is empty.
///
/// "Empty" means no non-whitespace content before the next `## ` heading or end of file.
pub(super) fn section_body_is_empty(content: &str, section: &str) -> bool {
    let heading = format!("## {section}");
    let Some(start) = content.find(&heading) else {
        return false; // not present — separate check handles missing sections
    };
    // Advance past the heading line
    let after_heading = &content[start + heading.len()..];
    // Find the end of the section: next `## ` or end of string
    let body = match after_heading.find("\n## ") {
        Some(next) => &after_heading[..next],
        None => after_heading,
    };
    body.trim().is_empty()
}
