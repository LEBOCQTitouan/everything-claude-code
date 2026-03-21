/// Commands available in the Claw REPL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClawCommand {
    /// Display help text
    Help,
    /// Clear the current session history
    Clear,
    /// Show conversation history
    History,
    /// List or switch sessions
    Sessions(Option<String>),
    /// Switch model
    Model(Option<String>),
    /// Load a skill by name
    Load(String),
    /// Branch current session into a new name
    Branch(String),
    /// Search history for a keyword
    Search(String),
    /// Compact history to last N turns
    Compact(Option<usize>),
    /// Export session in a format (md, json, text)
    Export(Option<String>),
    /// Show session metrics
    Metrics,
    /// Exit the REPL
    Exit,
    /// A regular user message (not a command)
    UserMessage(String),
}

/// Parse user input into a ClawCommand.
pub fn parse_command(input: &str) -> ClawCommand {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return ClawCommand::UserMessage(String::new());
    }

    // Check for exit keywords
    if trimmed == "exit" || trimmed == "quit" {
        return ClawCommand::Exit;
    }

    // Commands start with /
    if !trimmed.starts_with('/') {
        return ClawCommand::UserMessage(trimmed.to_string());
    }

    let without_slash = &trimmed[1..];
    let (cmd, arg) = match without_slash.find(' ') {
        Some(pos) => (&without_slash[..pos], Some(without_slash[pos + 1..].trim())),
        None => (without_slash, None),
    };

    match cmd.to_lowercase().as_str() {
        "help" | "h" => ClawCommand::Help,
        "clear" => ClawCommand::Clear,
        "history" | "hist" => ClawCommand::History,
        "sessions" | "sess" => ClawCommand::Sessions(arg.map(|s| s.to_string())),
        "model" | "m" => ClawCommand::Model(arg.map(|s| s.to_string())),
        "load" | "l" => match arg {
            Some(name) if !name.is_empty() => ClawCommand::Load(name.to_string()),
            _ => ClawCommand::Help, // /load without arg shows help
        },
        "branch" | "br" => match arg {
            Some(name) if !name.is_empty() => ClawCommand::Branch(name.to_string()),
            _ => ClawCommand::Help,
        },
        "search" | "s" => match arg {
            Some(query) if !query.is_empty() => ClawCommand::Search(query.to_string()),
            _ => ClawCommand::Help,
        },
        "compact" => {
            let n = arg.and_then(|s| s.parse::<usize>().ok());
            ClawCommand::Compact(n)
        }
        "export" | "x" => ClawCommand::Export(arg.map(|s| s.to_string())),
        "metrics" | "stats" => ClawCommand::Metrics,
        "exit" | "quit" | "q" => ClawCommand::Exit,
        _ => ClawCommand::UserMessage(trimmed.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_help() {
        assert_eq!(parse_command("/help"), ClawCommand::Help);
        assert_eq!(parse_command("/h"), ClawCommand::Help);
    }

    #[test]
    fn parse_clear() {
        assert_eq!(parse_command("/clear"), ClawCommand::Clear);
    }

    #[test]
    fn parse_history() {
        assert_eq!(parse_command("/history"), ClawCommand::History);
        assert_eq!(parse_command("/hist"), ClawCommand::History);
    }

    #[test]
    fn parse_sessions_no_arg() {
        assert_eq!(parse_command("/sessions"), ClawCommand::Sessions(None));
    }

    #[test]
    fn parse_sessions_with_name() {
        assert_eq!(
            parse_command("/sessions my-session"),
            ClawCommand::Sessions(Some("my-session".to_string()))
        );
    }

    #[test]
    fn parse_sessions_alias() {
        assert_eq!(parse_command("/sess"), ClawCommand::Sessions(None));
    }

    #[test]
    fn parse_model_no_arg() {
        assert_eq!(parse_command("/model"), ClawCommand::Model(None));
    }

    #[test]
    fn parse_model_with_name() {
        assert_eq!(
            parse_command("/model opus"),
            ClawCommand::Model(Some("opus".to_string()))
        );
    }

    #[test]
    fn parse_model_alias() {
        assert_eq!(parse_command("/m"), ClawCommand::Model(None));
    }

    #[test]
    fn parse_load() {
        assert_eq!(
            parse_command("/load tdd-workflow"),
            ClawCommand::Load("tdd-workflow".to_string())
        );
    }

    #[test]
    fn parse_load_alias() {
        assert_eq!(
            parse_command("/l tdd-workflow"),
            ClawCommand::Load("tdd-workflow".to_string())
        );
    }

    #[test]
    fn parse_load_no_arg_shows_help() {
        assert_eq!(parse_command("/load"), ClawCommand::Help);
    }

    #[test]
    fn parse_branch() {
        assert_eq!(
            parse_command("/branch experiment"),
            ClawCommand::Branch("experiment".to_string())
        );
    }

    #[test]
    fn parse_branch_alias() {
        assert_eq!(
            parse_command("/br experiment"),
            ClawCommand::Branch("experiment".to_string())
        );
    }

    #[test]
    fn parse_branch_no_arg_shows_help() {
        assert_eq!(parse_command("/branch"), ClawCommand::Help);
    }

    #[test]
    fn parse_search() {
        assert_eq!(
            parse_command("/search keyword"),
            ClawCommand::Search("keyword".to_string())
        );
    }

    #[test]
    fn parse_search_alias() {
        assert_eq!(
            parse_command("/s keyword"),
            ClawCommand::Search("keyword".to_string())
        );
    }

    #[test]
    fn parse_search_no_arg_shows_help() {
        assert_eq!(parse_command("/search"), ClawCommand::Help);
    }

    #[test]
    fn parse_compact_no_arg() {
        assert_eq!(parse_command("/compact"), ClawCommand::Compact(None));
    }

    #[test]
    fn parse_compact_with_number() {
        assert_eq!(parse_command("/compact 10"), ClawCommand::Compact(Some(10)));
    }

    #[test]
    fn parse_compact_invalid_number() {
        assert_eq!(parse_command("/compact abc"), ClawCommand::Compact(None));
    }

    #[test]
    fn parse_export_no_arg() {
        assert_eq!(parse_command("/export"), ClawCommand::Export(None));
    }

    #[test]
    fn parse_export_with_format() {
        assert_eq!(
            parse_command("/export md"),
            ClawCommand::Export(Some("md".to_string()))
        );
    }

    #[test]
    fn parse_export_alias() {
        assert_eq!(parse_command("/x"), ClawCommand::Export(None));
    }

    #[test]
    fn parse_metrics() {
        assert_eq!(parse_command("/metrics"), ClawCommand::Metrics);
        assert_eq!(parse_command("/stats"), ClawCommand::Metrics);
    }

    #[test]
    fn parse_exit_slash() {
        assert_eq!(parse_command("/exit"), ClawCommand::Exit);
        assert_eq!(parse_command("/quit"), ClawCommand::Exit);
        assert_eq!(parse_command("/q"), ClawCommand::Exit);
    }

    #[test]
    fn parse_exit_keyword() {
        assert_eq!(parse_command("exit"), ClawCommand::Exit);
        assert_eq!(parse_command("quit"), ClawCommand::Exit);
    }

    #[test]
    fn parse_user_message() {
        assert_eq!(
            parse_command("hello world"),
            ClawCommand::UserMessage("hello world".to_string())
        );
    }

    #[test]
    fn parse_empty_input() {
        assert_eq!(parse_command(""), ClawCommand::UserMessage(String::new()));
    }

    #[test]
    fn parse_whitespace_only() {
        assert_eq!(
            parse_command("   "),
            ClawCommand::UserMessage(String::new())
        );
    }

    #[test]
    fn parse_unknown_command_is_user_message() {
        assert_eq!(
            parse_command("/unknown"),
            ClawCommand::UserMessage("/unknown".to_string())
        );
    }

    #[test]
    fn parse_preserves_arg_whitespace() {
        let cmd = parse_command("/search  multi word query ");
        assert_eq!(cmd, ClawCommand::Search("multi word query".to_string()));
    }

    #[test]
    fn parse_case_insensitive_commands() {
        assert_eq!(parse_command("/HELP"), ClawCommand::Help);
        assert_eq!(parse_command("/CLEAR"), ClawCommand::Clear);
        assert_eq!(parse_command("/EXIT"), ClawCommand::Exit);
    }
}
