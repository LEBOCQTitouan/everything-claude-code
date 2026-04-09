use ecc_domain::config::validate::{
    VALID_PATTERN_DIFFICULTIES, VALID_PATTERN_LANGUAGES, parse_tool_list,
};
use std::collections::HashMap;

/// Check required frontmatter fields and category match.
pub(super) fn validate_frontmatter_fields(
    fm: &HashMap<String, String>,
    label: &str,
    expected_category: &str,
) -> (String, bool) {
    let mut errors = String::new();
    let mut has_errors = false;

    for field in &["name", "category", "tags", "languages", "difficulty"] {
        match fm.get(*field) {
            Some(v) if !v.trim().is_empty() => {}
            _ => {
                errors.push_str(&format!(
                    "ERROR: {label} - Missing required frontmatter field '{field}'\n"
                ));
                has_errors = true;
            }
        }
    }

    if let Some(cat) = fm.get("category")
        && !cat.trim().is_empty()
        && cat.trim() != expected_category
    {
        errors.push_str(&format!(
            "ERROR: {label} - category frontmatter '{cat}' does not match directory '{expected_category}'\n"
        ));
        has_errors = true;
    }

    (errors, !has_errors)
}

/// Validate the languages list and implementation heading matching.
pub(super) fn validate_languages(
    fm: &HashMap<String, String>,
    content: &str,
    label: &str,
) -> (String, bool) {
    let mut errors = String::new();
    let mut has_errors = false;

    let Some(raw_langs) = fm.get("languages") else {
        return (errors, true);
    };

    let langs = parse_tool_list(raw_langs.trim());
    if langs.is_empty() {
        errors.push_str(&format!("ERROR: {label} - languages list is empty\n"));
        return (errors, false);
    }

    let is_all = langs.len() == 1 && langs[0] == "all";
    let mut lang_err = false;
    for lang in &langs {
        if !VALID_PATTERN_LANGUAGES.contains(&lang.as_str()) {
            errors.push_str(&format!(
                "ERROR: {label} - unrecognized language '{lang}'\n"
            ));
            lang_err = true;
            has_errors = true;
        }
    }

    if !is_all && !lang_err {
        let impl_headings = extract_impl_headings(content);
        for heading_lang in &impl_headings {
            let heading_lower = heading_lang.to_lowercase();
            if !langs.iter().any(|l| l.to_lowercase() == heading_lower) {
                errors.push_str(&format!(
                    "ERROR: {label} - Language Implementations heading '### {heading_lang}' not listed in frontmatter languages\n"
                ));
                has_errors = true;
            }
        }
    }

    (errors, !has_errors)
}

/// Validate the difficulty field value.
pub(super) fn validate_difficulty(fm: &HashMap<String, String>, label: &str) -> (String, bool) {
    if let Some(diff) = fm.get("difficulty")
        && !diff.trim().is_empty()
        && !VALID_PATTERN_DIFFICULTIES.contains(&diff.trim())
    {
        return (
            format!("ERROR: {label} - unrecognized difficulty '{diff}'\n"),
            false,
        );
    }
    (String::new(), true)
}

/// Extract all `### <Name>` headings found under `## Language Implementations`.
pub(super) fn extract_impl_headings(content: &str) -> Vec<String> {
    let section_heading = "## Language Implementations";
    let Some(start) = content.find(section_heading) else {
        return Vec::new();
    };
    let after = &content[start + section_heading.len()..];
    // Scope to this section only (up to next ## heading)
    let section_body = match after.find("\n## ") {
        Some(end) => &after[..end],
        None => after,
    };
    section_body
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            line.strip_prefix("### ")
                .map(|rest| rest.trim().to_string())
        })
        .collect()
}
