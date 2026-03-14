/// A single conversation turn in a Claw session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Turn {
    pub timestamp: String,
    pub role: Role,
    pub content: String,
}

/// The role of a conversation participant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
    System,
}

impl Role {
    /// Parse a role from a string (case-insensitive).
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "user" => Some(Self::User),
            "assistant" => Some(Self::Assistant),
            "system" => Some(Self::System),
            _ => None,
        }
    }

    /// Display name for the role.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "User",
            Self::Assistant => "Assistant",
            Self::System => "System",
        }
    }
}

/// Format a turn to markdown:
/// ```text
/// ### [timestamp] Role
/// content
/// ---
/// ```
pub fn format_turn(turn: &Turn) -> String {
    format!(
        "### [{}] {}\n{}\n---",
        turn.timestamp,
        turn.role.as_str(),
        turn.content,
    )
}

/// Format multiple turns to markdown, separated by blank lines.
pub fn format_turns(turns: &[Turn]) -> String {
    turns.iter().map(format_turn).collect::<Vec<_>>().join("\n\n")
}

/// Parse turns from markdown. Each turn is delimited by `---`.
/// Format: `### [timestamp] Role\ncontent\n---`
pub fn parse_turns(markdown: &str) -> Vec<Turn> {
    let mut turns = Vec::new();
    let mut current_timestamp = String::new();
    let mut current_role: Option<Role> = None;
    let mut content_lines: Vec<&str> = Vec::new();

    for line in markdown.lines() {
        if line == "---" {
            if let Some(role) = current_role.take() {
                let content = content_lines.join("\n");
                turns.push(Turn {
                    timestamp: current_timestamp.clone(),
                    role,
                    content,
                });
            }
            current_timestamp.clear();
            content_lines.clear();
        } else if let Some(rest) = line.strip_prefix("### [")
            && let Some(bracket_end) = rest.find(']')
        {
            current_timestamp = rest[..bracket_end].to_string();
            let role_str = rest[bracket_end + 1..].trim();
            current_role = Role::parse(role_str);
            content_lines.clear();
        } else if current_role.is_some() {
            content_lines.push(line);
        }
    }

    // Handle final turn without trailing ---
    if let Some(role) = current_role.take() {
        let content = content_lines.join("\n");
        turns.push(Turn {
            timestamp: current_timestamp,
            role,
            content,
        });
    }

    turns
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_turn(ts: &str, role: Role, content: &str) -> Turn {
        Turn {
            timestamp: ts.to_string(),
            role,
            content: content.to_string(),
        }
    }

    // --- Role ---

    #[test]
    fn role_parse_user() {
        assert_eq!(Role::parse("user"), Some(Role::User));
    }

    #[test]
    fn role_parse_assistant() {
        assert_eq!(Role::parse("assistant"), Some(Role::Assistant));
    }

    #[test]
    fn role_parse_system() {
        assert_eq!(Role::parse("system"), Some(Role::System));
    }

    #[test]
    fn role_parse_case_insensitive() {
        assert_eq!(Role::parse("USER"), Some(Role::User));
        assert_eq!(Role::parse("Assistant"), Some(Role::Assistant));
        assert_eq!(Role::parse("SYSTEM"), Some(Role::System));
    }

    #[test]
    fn role_parse_with_whitespace() {
        assert_eq!(Role::parse("  user  "), Some(Role::User));
    }

    #[test]
    fn role_parse_invalid() {
        assert_eq!(Role::parse("unknown"), None);
        assert_eq!(Role::parse(""), None);
    }

    #[test]
    fn role_as_str() {
        assert_eq!(Role::User.as_str(), "User");
        assert_eq!(Role::Assistant.as_str(), "Assistant");
        assert_eq!(Role::System.as_str(), "System");
    }

    // --- format_turn ---

    #[test]
    fn format_turn_basic() {
        let turn = make_turn("2026-03-14 10:00:00", Role::User, "hello");
        let formatted = format_turn(&turn);
        assert_eq!(formatted, "### [2026-03-14 10:00:00] User\nhello\n---");
    }

    #[test]
    fn format_turn_multiline_content() {
        let turn = make_turn("2026-03-14 10:00:00", Role::Assistant, "line1\nline2\nline3");
        let formatted = format_turn(&turn);
        assert_eq!(
            formatted,
            "### [2026-03-14 10:00:00] Assistant\nline1\nline2\nline3\n---"
        );
    }

    #[test]
    fn format_turn_empty_content() {
        let turn = make_turn("2026-03-14 10:00:00", Role::System, "");
        let formatted = format_turn(&turn);
        assert_eq!(formatted, "### [2026-03-14 10:00:00] System\n\n---");
    }

    // --- format_turns ---

    #[test]
    fn format_turns_multiple() {
        let turns = vec![
            make_turn("ts1", Role::User, "hi"),
            make_turn("ts2", Role::Assistant, "hello"),
        ];
        let result = format_turns(&turns);
        assert!(result.contains("### [ts1] User\nhi\n---"));
        assert!(result.contains("### [ts2] Assistant\nhello\n---"));
    }

    #[test]
    fn format_turns_empty() {
        let result = format_turns(&[]);
        assert_eq!(result, "");
    }

    // --- parse_turns ---

    #[test]
    fn parse_turns_single() {
        let md = "### [2026-03-14 10:00:00] User\nhello world\n---";
        let turns = parse_turns(md);
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].timestamp, "2026-03-14 10:00:00");
        assert_eq!(turns[0].role, Role::User);
        assert_eq!(turns[0].content, "hello world");
    }

    #[test]
    fn parse_turns_multiple() {
        let md = "### [ts1] User\nhi\n---\n\n### [ts2] Assistant\nhello\n---";
        let turns = parse_turns(md);
        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0].role, Role::User);
        assert_eq!(turns[0].content, "hi");
        assert_eq!(turns[1].role, Role::Assistant);
        assert_eq!(turns[1].content, "hello");
    }

    #[test]
    fn parse_turns_multiline_content() {
        let md = "### [ts1] User\nline1\nline2\nline3\n---";
        let turns = parse_turns(md);
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].content, "line1\nline2\nline3");
    }

    #[test]
    fn parse_turns_empty_string() {
        let turns = parse_turns("");
        assert!(turns.is_empty());
    }

    #[test]
    fn parse_turns_no_valid_turns() {
        let turns = parse_turns("random text\nno headers here\n---");
        assert!(turns.is_empty());
    }

    #[test]
    fn parse_turns_without_trailing_separator() {
        let md = "### [ts1] User\nhello";
        let turns = parse_turns(md);
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].content, "hello");
    }

    // --- roundtrip ---

    #[test]
    fn roundtrip_single_turn() {
        let original = make_turn("2026-03-14 10:00:00", Role::User, "test content");
        let md = format_turn(&original);
        let parsed = parse_turns(&md);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0], original);
    }

    #[test]
    fn roundtrip_multiple_turns() {
        let originals = vec![
            make_turn("ts1", Role::User, "hello"),
            make_turn("ts2", Role::Assistant, "hi there"),
            make_turn("ts3", Role::User, "thanks"),
        ];
        let md = format_turns(&originals);
        let parsed = parse_turns(&md);
        assert_eq!(parsed, originals);
    }

    #[test]
    fn roundtrip_multiline_content() {
        let original = make_turn("ts1", Role::User, "line1\nline2\nline3");
        let md = format_turn(&original);
        let parsed = parse_turns(&md);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0], original);
    }

    #[test]
    fn parse_turns_system_role() {
        let md = "### [ts1] System\nsystem message\n---";
        let turns = parse_turns(md);
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].role, Role::System);
    }
}
