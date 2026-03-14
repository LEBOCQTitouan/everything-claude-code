use super::turn::{format_turns, Turn};

/// Export format for session output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Markdown,
    Json,
    Text,
}

impl ExportFormat {
    /// Parse an export format from a string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "md" | "markdown" => Some(Self::Markdown),
            "json" => Some(Self::Json),
            "text" | "txt" => Some(Self::Text),
            _ => None,
        }
    }
}

/// Export turns in the specified format.
pub fn export_turns(session_name: &str, turns: &[Turn], format: ExportFormat) -> String {
    match format {
        ExportFormat::Markdown => export_markdown(session_name, turns),
        ExportFormat::Json => export_json(turns),
        ExportFormat::Text => export_text(turns),
    }
}

fn export_markdown(session_name: &str, turns: &[Turn]) -> String {
    let mut out = format!("# Session: {session_name}\n\n");
    out.push_str(&format_turns(turns));
    out
}

fn export_json(turns: &[Turn]) -> String {
    let entries: Vec<String> = turns
        .iter()
        .map(|t| {
            format!(
                "  {{\n    \"timestamp\": \"{}\",\n    \"role\": \"{}\",\n    \"content\": {}\n  }}",
                t.timestamp,
                t.role.as_str().to_lowercase(),
                escape_json_string(&t.content),
            )
        })
        .collect();

    format!("[\n{}\n]", entries.join(",\n"))
}

fn export_text(turns: &[Turn]) -> String {
    turns
        .iter()
        .map(|t| format!("[{}] {}: {}", t.timestamp, t.role.as_str(), t.content))
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn escape_json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claw::turn::Role;

    fn make_turn(ts: &str, role: Role, content: &str) -> Turn {
        Turn {
            timestamp: ts.to_string(),
            role,
            content: content.to_string(),
        }
    }

    // --- ExportFormat::parse ---

    #[test]
    fn parse_md() {
        assert_eq!(ExportFormat::parse("md"), Some(ExportFormat::Markdown));
    }

    #[test]
    fn parse_markdown() {
        assert_eq!(ExportFormat::parse("markdown"), Some(ExportFormat::Markdown));
    }

    #[test]
    fn parse_json() {
        assert_eq!(ExportFormat::parse("json"), Some(ExportFormat::Json));
    }

    #[test]
    fn parse_text() {
        assert_eq!(ExportFormat::parse("text"), Some(ExportFormat::Text));
        assert_eq!(ExportFormat::parse("txt"), Some(ExportFormat::Text));
    }

    #[test]
    fn parse_case_insensitive() {
        assert_eq!(ExportFormat::parse("MD"), Some(ExportFormat::Markdown));
        assert_eq!(ExportFormat::parse("JSON"), Some(ExportFormat::Json));
    }

    #[test]
    fn parse_invalid() {
        assert_eq!(ExportFormat::parse("xml"), None);
        assert_eq!(ExportFormat::parse(""), None);
    }

    // --- export_turns markdown ---

    #[test]
    fn export_markdown_header() {
        let turns = vec![make_turn("ts1", Role::User, "hello")];
        let result = export_turns("test-session", &turns, ExportFormat::Markdown);
        assert!(result.starts_with("# Session: test-session"));
    }

    #[test]
    fn export_markdown_contains_turns() {
        let turns = vec![
            make_turn("ts1", Role::User, "hello"),
            make_turn("ts2", Role::Assistant, "hi"),
        ];
        let result = export_turns("test", &turns, ExportFormat::Markdown);
        assert!(result.contains("### [ts1] User"));
        assert!(result.contains("### [ts2] Assistant"));
    }

    #[test]
    fn export_markdown_empty() {
        let result = export_turns("empty", &[], ExportFormat::Markdown);
        assert!(result.contains("# Session: empty"));
    }

    // --- export_turns json ---

    #[test]
    fn export_json_structure() {
        let turns = vec![make_turn("ts1", Role::User, "hello")];
        let result = export_turns("test", &turns, ExportFormat::Json);
        assert!(result.starts_with('['));
        assert!(result.ends_with(']'));
        assert!(result.contains("\"role\": \"user\""));
        assert!(result.contains("\"content\": \"hello\""));
        assert!(result.contains("\"timestamp\": \"ts1\""));
    }

    #[test]
    fn export_json_escapes_special_chars() {
        let turns = vec![make_turn("ts1", Role::User, "line1\nline2")];
        let result = export_turns("test", &turns, ExportFormat::Json);
        assert!(result.contains("\\n"));
    }

    #[test]
    fn export_json_empty() {
        let result = export_turns("test", &[], ExportFormat::Json);
        assert_eq!(result.trim(), "[\n\n]");
    }

    // --- export_turns text ---

    #[test]
    fn export_text_format() {
        let turns = vec![make_turn("ts1", Role::User, "hello")];
        let result = export_turns("test", &turns, ExportFormat::Text);
        assert_eq!(result, "[ts1] User: hello");
    }

    #[test]
    fn export_text_multiple() {
        let turns = vec![
            make_turn("ts1", Role::User, "hi"),
            make_turn("ts2", Role::Assistant, "hey"),
        ];
        let result = export_turns("test", &turns, ExportFormat::Text);
        assert!(result.contains("[ts1] User: hi"));
        assert!(result.contains("[ts2] Assistant: hey"));
    }

    // --- escape_json_string ---

    #[test]
    fn escape_quotes() {
        assert_eq!(escape_json_string("say \"hi\""), "\"say \\\"hi\\\"\"");
    }

    #[test]
    fn escape_backslash() {
        assert_eq!(escape_json_string("a\\b"), "\"a\\\\b\"");
    }

    #[test]
    fn escape_newlines() {
        assert_eq!(escape_json_string("a\nb"), "\"a\\nb\"");
    }

    #[test]
    fn escape_tabs() {
        assert_eq!(escape_json_string("a\tb"), "\"a\\tb\"");
    }
}
