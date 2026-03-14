use super::turn::{format_turns, Turn};

/// Assemble a full prompt for `claude -p` from system context, history, and user message.
///
/// Sections:
/// - SYSTEM: optional system instructions (skills, context)
/// - HISTORY: prior conversation turns
/// - USER: the current user message
pub fn assemble_prompt(
    system_context: Option<&str>,
    history: &[Turn],
    user_message: &str,
) -> String {
    let mut parts = Vec::new();

    if let Some(ctx) = system_context
        && !ctx.is_empty()
    {
        parts.push(format!("## SYSTEM\n\n{ctx}"));
    }

    if !history.is_empty() {
        let history_md = format_turns(history);
        parts.push(format!("## HISTORY\n\n{history_md}"));
    }

    parts.push(format!("## USER\n\n{user_message}"));

    parts.join("\n\n")
}

/// Build system context from loaded skill content.
pub fn build_system_context(skills: &[String]) -> Option<String> {
    if skills.is_empty() {
        return None;
    }

    let combined = skills.join("\n\n---\n\n");
    Some(combined)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claw::turn::Role;

    fn make_turn(role: Role, content: &str) -> Turn {
        Turn {
            timestamp: "ts".to_string(),
            role,
            content: content.to_string(),
        }
    }

    // --- assemble_prompt ---

    #[test]
    fn prompt_with_all_sections() {
        let history = vec![
            make_turn(Role::User, "hello"),
            make_turn(Role::Assistant, "hi"),
        ];
        let result = assemble_prompt(Some("Be helpful."), &history, "What is 1+1?");
        assert!(result.starts_with("## SYSTEM"));
        assert!(result.contains("Be helpful."));
        assert!(result.contains("## HISTORY"));
        assert!(result.contains("## USER"));
        assert!(result.contains("What is 1+1?"));
    }

    #[test]
    fn prompt_without_system() {
        let result = assemble_prompt(None, &[], "hello");
        assert!(!result.contains("## SYSTEM"));
        assert!(result.contains("## USER"));
        assert!(result.contains("hello"));
    }

    #[test]
    fn prompt_empty_system_is_omitted() {
        let result = assemble_prompt(Some(""), &[], "hello");
        assert!(!result.contains("## SYSTEM"));
    }

    #[test]
    fn prompt_without_history() {
        let result = assemble_prompt(Some("context"), &[], "question");
        assert!(result.contains("## SYSTEM"));
        assert!(!result.contains("## HISTORY"));
        assert!(result.contains("## USER"));
    }

    #[test]
    fn prompt_with_history() {
        let history = vec![make_turn(Role::User, "first")];
        let result = assemble_prompt(None, &history, "second");
        assert!(result.contains("## HISTORY"));
        assert!(result.contains("first"));
        assert!(result.contains("## USER"));
        assert!(result.contains("second"));
    }

    #[test]
    fn prompt_sections_in_order() {
        let history = vec![make_turn(Role::User, "prev")];
        let result = assemble_prompt(Some("sys"), &history, "msg");
        let sys_pos = result.find("## SYSTEM").unwrap();
        let hist_pos = result.find("## HISTORY").unwrap();
        let user_pos = result.find("## USER").unwrap();
        assert!(sys_pos < hist_pos);
        assert!(hist_pos < user_pos);
    }

    #[test]
    fn prompt_user_message_at_end() {
        let result = assemble_prompt(None, &[], "final message");
        assert!(result.ends_with("final message"));
    }

    #[test]
    fn prompt_multiline_user_message() {
        let result = assemble_prompt(None, &[], "line1\nline2");
        assert!(result.contains("line1\nline2"));
    }

    // --- build_system_context ---

    #[test]
    fn system_context_empty_skills() {
        assert_eq!(build_system_context(&[]), None);
    }

    #[test]
    fn system_context_single_skill() {
        let skills = vec!["TDD workflow instructions".to_string()];
        let result = build_system_context(&skills);
        assert_eq!(result, Some("TDD workflow instructions".to_string()));
    }

    #[test]
    fn system_context_multiple_skills() {
        let skills = vec!["Skill A".to_string(), "Skill B".to_string()];
        let result = build_system_context(&skills).unwrap();
        assert!(result.contains("Skill A"));
        assert!(result.contains("---"));
        assert!(result.contains("Skill B"));
    }

    #[test]
    fn system_context_preserves_content() {
        let skills = vec!["Full content\nwith lines".to_string()];
        let result = build_system_context(&skills).unwrap();
        assert!(result.contains("Full content\nwith lines"));
    }

    #[test]
    fn system_context_separator_between_skills() {
        let skills = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let result = build_system_context(&skills).unwrap();
        assert_eq!(result, "A\n\n---\n\nB\n\n---\n\nC");
    }
}
