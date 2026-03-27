/// Convert a description string into a URL-safe slug.
///
/// - Lowercases the string
/// - Removes non-alphanumeric characters except spaces
/// - Collapses multiple spaces into one
/// - Replaces spaces with hyphens
/// - Truncates to at most 40 characters
/// - Trims any trailing hyphen
pub fn make_slug(desc: &str) -> String {
    let lower = desc.to_lowercase();

    // Keep only alphanumeric chars and spaces
    let filtered: String = lower
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == ' ')
        .collect();

    // Collapse multiple spaces into one
    let collapsed = collapse_spaces(&filtered);

    // Replace spaces with hyphens
    let hyphenated = collapsed.replace(' ', "-");

    // Truncate to 40 chars and trim trailing hyphen
    let truncated = if hyphenated.len() > 40 {
        &hyphenated[..40]
    } else {
        &hyphenated
    };

    truncated.trim_end_matches('-').to_string()
}

fn collapse_spaces(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_space = false;
    for c in s.chars() {
        if c == ' ' {
            if !prev_space {
                result.push(c);
            }
            prev_space = true;
        } else {
            result.push(c);
            prev_space = false;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_slug() {
        assert_eq!(make_slug("Hello World"), "hello-world");
    }

    #[test]
    fn strips_special_chars() {
        assert_eq!(make_slug("My Feature (v2)!"), "my-feature-v2");
    }

    #[test]
    fn collapses_spaces() {
        assert_eq!(make_slug("a  b   c"), "a-b-c");
    }

    #[test]
    fn truncates_to_40() {
        let long = "a".repeat(50);
        let slug = make_slug(&long);
        assert!(slug.len() <= 40);
    }

    #[test]
    fn trims_trailing_hyphen() {
        // "aaaa---" after truncation at 40 — trailing hyphen trimmed
        let s = "a".repeat(39) + " b";
        let slug = make_slug(&s);
        assert!(!slug.ends_with('-'));
    }
}
