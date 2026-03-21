use super::turn::{Turn, format_turns};

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
///
/// For `Json` format, returns the result of calling `json_serializer` with
/// the turns slice. This keeps serialization out of the domain layer.
pub fn export_turns(
    session_name: &str,
    turns: &[Turn],
    format: ExportFormat,
    json_serializer: impl FnOnce(&[Turn]) -> String,
) -> String {
    match format {
        ExportFormat::Markdown => export_markdown(session_name, turns),
        ExportFormat::Json => json_serializer(turns),
        ExportFormat::Text => export_text(turns),
    }
}

fn export_markdown(session_name: &str, turns: &[Turn]) -> String {
    let mut out = format!("# Session: {session_name}\n\n");
    out.push_str(&format_turns(turns));
    out
}

fn export_text(turns: &[Turn]) -> String {
    turns
        .iter()
        .map(|t| format!("[{}] {}: {}", t.timestamp, t.role.as_str(), t.content))
        .collect::<Vec<_>>()
        .join("\n\n")
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

    fn test_json_serializer(turns: &[Turn]) -> String {
        serde_json::to_string_pretty(turns).unwrap_or_else(|_| "[]".to_string())
    }

    // --- ExportFormat::parse ---

    #[test]
    fn parse_md() {
        assert_eq!(ExportFormat::parse("md"), Some(ExportFormat::Markdown));
    }

    #[test]
    fn parse_markdown() {
        assert_eq!(
            ExportFormat::parse("markdown"),
            Some(ExportFormat::Markdown)
        );
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
        let result = export_turns(
            "test-session",
            &turns,
            ExportFormat::Markdown,
            test_json_serializer,
        );
        assert!(result.starts_with("# Session: test-session"));
    }

    #[test]
    fn export_markdown_contains_turns() {
        let turns = vec![
            make_turn("ts1", Role::User, "hello"),
            make_turn("ts2", Role::Assistant, "hi"),
        ];
        let result = export_turns("test", &turns, ExportFormat::Markdown, test_json_serializer);
        assert!(result.contains("### [ts1] User"));
        assert!(result.contains("### [ts2] Assistant"));
    }

    #[test]
    fn export_markdown_empty() {
        let result = export_turns("empty", &[], ExportFormat::Markdown, test_json_serializer);
        assert!(result.contains("# Session: empty"));
    }

    // --- export_turns json ---

    #[test]
    fn export_json_structure() {
        let turns = vec![make_turn("ts1", Role::User, "hello")];
        let result = export_turns("test", &turns, ExportFormat::Json, test_json_serializer);
        assert!(result.starts_with('['));
        assert!(result.ends_with(']'));
        assert!(result.contains("\"role\": \"user\""));
        assert!(result.contains("\"content\": \"hello\""));
        assert!(result.contains("\"timestamp\": \"ts1\""));
    }

    #[test]
    fn export_json_escapes_special_chars() {
        let turns = vec![make_turn("ts1", Role::User, "line1\nline2")];
        let result = export_turns("test", &turns, ExportFormat::Json, test_json_serializer);
        assert!(result.contains("\\n"));
    }

    #[test]
    fn export_json_empty() {
        let result = export_turns("test", &[], ExportFormat::Json, test_json_serializer);
        assert_eq!(result.trim(), "[]");
    }

    // --- export_turns text ---

    #[test]
    fn export_text_format() {
        let turns = vec![make_turn("ts1", Role::User, "hello")];
        let result = export_turns("test", &turns, ExportFormat::Text, test_json_serializer);
        assert_eq!(result, "[ts1] User: hello");
    }

    #[test]
    fn export_text_multiple() {
        let turns = vec![
            make_turn("ts1", Role::User, "hi"),
            make_turn("ts2", Role::Assistant, "hey"),
        ];
        let result = export_turns("test", &turns, ExportFormat::Text, test_json_serializer);
        assert!(result.contains("[ts1] User: hi"));
        assert!(result.contains("[ts2] Assistant: hey"));
    }
}
