use super::turn::{Role, Turn};

/// Metrics for a Claw session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionMetrics {
    pub total_turns: usize,
    pub user_turns: usize,
    pub assistant_turns: usize,
    pub system_turns: usize,
    pub total_chars: usize,
    pub estimated_tokens: usize,
}

/// Approximate token count: ~4 characters per token.
const CHARS_PER_TOKEN: usize = 4;

/// Compute metrics for a set of turns.
pub fn compute_metrics(turns: &[Turn]) -> SessionMetrics {
    let mut user_turns = 0;
    let mut assistant_turns = 0;
    let mut system_turns = 0;
    let mut total_chars = 0;

    for turn in turns {
        match turn.role {
            Role::User => user_turns += 1,
            Role::Assistant => assistant_turns += 1,
            Role::System => system_turns += 1,
        }
        total_chars += turn.content.len();
    }

    SessionMetrics {
        total_turns: turns.len(),
        user_turns,
        assistant_turns,
        system_turns,
        total_chars,
        estimated_tokens: total_chars / CHARS_PER_TOKEN,
    }
}

/// Format metrics as a human-readable string.
pub fn format_metrics(metrics: &SessionMetrics) -> String {
    format!(
        "Turns: {} (user: {}, assistant: {}, system: {})\n\
         Characters: {}\n\
         Estimated tokens: ~{}",
        metrics.total_turns,
        metrics.user_turns,
        metrics.assistant_turns,
        metrics.system_turns,
        metrics.total_chars,
        metrics.estimated_tokens,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_turn(role: Role, content: &str) -> Turn {
        Turn {
            timestamp: "ts".to_string(),
            role,
            content: content.to_string(),
        }
    }

    #[test]
    fn metrics_empty() {
        let m = compute_metrics(&[]);
        assert_eq!(m.total_turns, 0);
        assert_eq!(m.user_turns, 0);
        assert_eq!(m.assistant_turns, 0);
        assert_eq!(m.system_turns, 0);
        assert_eq!(m.total_chars, 0);
        assert_eq!(m.estimated_tokens, 0);
    }

    #[test]
    fn metrics_single_turn() {
        let turns = vec![make_turn(Role::User, "hello")];
        let m = compute_metrics(&turns);
        assert_eq!(m.total_turns, 1);
        assert_eq!(m.user_turns, 1);
        assert_eq!(m.total_chars, 5);
        assert_eq!(m.estimated_tokens, 1); // 5/4 = 1
    }

    #[test]
    fn metrics_mixed_roles() {
        let turns = vec![
            make_turn(Role::User, "hi"),
            make_turn(Role::Assistant, "hello"),
            make_turn(Role::System, "sys"),
            make_turn(Role::User, "ok"),
        ];
        let m = compute_metrics(&turns);
        assert_eq!(m.total_turns, 4);
        assert_eq!(m.user_turns, 2);
        assert_eq!(m.assistant_turns, 1);
        assert_eq!(m.system_turns, 1);
    }

    #[test]
    fn metrics_token_estimation() {
        // 80 chars should be ~20 tokens
        let content = "a".repeat(80);
        let turns = vec![make_turn(Role::User, &content)];
        let m = compute_metrics(&turns);
        assert_eq!(m.total_chars, 80);
        assert_eq!(m.estimated_tokens, 20);
    }

    #[test]
    fn metrics_char_count_accumulates() {
        let turns = vec![
            make_turn(Role::User, "1234"),       // 4
            make_turn(Role::Assistant, "12345"), // 5
        ];
        let m = compute_metrics(&turns);
        assert_eq!(m.total_chars, 9);
    }

    #[test]
    fn format_metrics_output() {
        let m = SessionMetrics {
            total_turns: 5,
            user_turns: 3,
            assistant_turns: 2,
            system_turns: 0,
            total_chars: 500,
            estimated_tokens: 125,
        };
        let output = format_metrics(&m);
        assert!(output.contains("Turns: 5"));
        assert!(output.contains("user: 3"));
        assert!(output.contains("assistant: 2"));
        assert!(output.contains("Characters: 500"));
        assert!(output.contains("~125"));
    }

    #[test]
    fn metrics_empty_content() {
        let turns = vec![make_turn(Role::User, "")];
        let m = compute_metrics(&turns);
        assert_eq!(m.total_turns, 1);
        assert_eq!(m.total_chars, 0);
        assert_eq!(m.estimated_tokens, 0);
    }

    #[test]
    fn metrics_multiline_content() {
        let turns = vec![make_turn(Role::User, "line1\nline2\nline3")];
        let m = compute_metrics(&turns);
        assert_eq!(m.total_chars, 17);
    }
}
