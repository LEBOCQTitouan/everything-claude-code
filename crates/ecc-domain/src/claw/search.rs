use super::turn::Turn;

/// Search turns for a keyword (case-insensitive).
/// Returns indices of matching turns.
pub fn search_turns(turns: &[Turn], keyword: &str) -> Vec<usize> {
    let lower_keyword = keyword.to_lowercase();
    turns
        .iter()
        .enumerate()
        .filter(|(_, turn)| turn.content.to_lowercase().contains(&lower_keyword))
        .map(|(i, _)| i)
        .collect()
}

/// Format search results for display.
pub fn format_search_results(turns: &[Turn], indices: &[usize]) -> String {
    if indices.is_empty() {
        return "No matches found.".to_string();
    }

    let mut lines = Vec::new();
    lines.push(format!("Found {} match(es):", indices.len()));

    for &idx in indices {
        if let Some(turn) = turns.get(idx) {
            let preview = truncate_preview(&turn.content, 80);
            lines.push(format!(
                "  [{}] {} ({}): {}",
                idx + 1,
                turn.role.as_str(),
                turn.timestamp,
                preview,
            ));
        }
    }

    lines.join("\n")
}

fn truncate_preview(s: &str, max_len: usize) -> String {
    let first_line = s.lines().next().unwrap_or("");
    if first_line.len() <= max_len {
        first_line.to_string()
    } else {
        let boundary = first_line
            .char_indices()
            .take_while(|(i, _)| *i < max_len)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);
        format!("{}...", &first_line[..boundary])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claw::turn::Role;

    fn make_turn(content: &str) -> Turn {
        Turn {
            timestamp: "ts".to_string(),
            role: Role::User,
            content: content.to_string(),
        }
    }

    #[test]
    fn search_finds_exact_match() {
        let turns = vec![make_turn("hello world"), make_turn("goodbye")];
        let results = search_turns(&turns, "hello");
        assert_eq!(results, vec![0]);
    }

    #[test]
    fn search_case_insensitive() {
        let turns = vec![make_turn("Hello World")];
        let results = search_turns(&turns, "hello");
        assert_eq!(results, vec![0]);
    }

    #[test]
    fn search_multiple_matches() {
        let turns = vec![
            make_turn("the quick fox"),
            make_turn("the slow turtle"),
            make_turn("the quick rabbit"),
        ];
        let results = search_turns(&turns, "quick");
        assert_eq!(results, vec![0, 2]);
    }

    #[test]
    fn search_no_matches() {
        let turns = vec![make_turn("hello")];
        let results = search_turns(&turns, "xyz");
        assert!(results.is_empty());
    }

    #[test]
    fn search_empty_turns() {
        let results = search_turns(&[], "keyword");
        assert!(results.is_empty());
    }

    // --- format_search_results ---

    #[test]
    fn format_no_results() {
        let output = format_search_results(&[], &[]);
        assert_eq!(output, "No matches found.");
    }

    #[test]
    fn format_with_results() {
        let turns = vec![make_turn("hello world")];
        let output = format_search_results(&turns, &[0]);
        assert!(output.contains("Found 1 match(es)"));
        assert!(output.contains("hello world"));
    }

    #[test]
    fn format_truncates_long_content() {
        let long_content = "a".repeat(200);
        let turns = vec![make_turn(&long_content)];
        let output = format_search_results(&turns, &[0]);
        assert!(output.contains("..."));
    }

    // --- truncate_preview ---

    #[test]
    fn truncate_short_string() {
        assert_eq!(truncate_preview("hello", 80), "hello");
    }

    #[test]
    fn truncate_long_string() {
        let long = "a".repeat(100);
        let result = truncate_preview(&long, 80);
        assert!(result.ends_with("..."));
        assert_eq!(result.len(), 83); // 80 + "..."
    }

    #[test]
    fn truncate_uses_first_line() {
        let result = truncate_preview("first line\nsecond line", 80);
        assert_eq!(result, "first line");
    }
}
