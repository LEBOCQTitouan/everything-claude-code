use ecc_domain::config::validate::parse_tool_list;
use std::collections::HashMap;

/// Validate cross-references in related-patterns.
pub(crate) fn validate_cross_refs(
    fm: &HashMap<String, String>,
    stem: &str,
    all_stems: &[String],
    label: &str,
) -> (String, bool) {
    let mut errors = String::new();
    let mut has_errors = false;

    let Some(raw_refs) = fm.get("related-patterns") else {
        return (errors, true);
    };

    let refs = parse_tool_list(raw_refs.trim());
    for ref_name in &refs {
        if ref_name == stem {
            errors.push_str(&format!(
                "WARN: {label} - self-reference in related-patterns: '{ref_name}'\n"
            ));
        } else if !all_stems.iter().any(|s| s == ref_name) {
            errors.push_str(&format!(
                "ERROR: {label} - cross-reference to non-existent pattern '{ref_name}'\n"
            ));
            has_errors = true;
        }
    }

    (errors, !has_errors)
}
