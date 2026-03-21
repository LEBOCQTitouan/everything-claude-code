use super::turn::Turn;

/// Default number of turns to keep during compaction.
const DEFAULT_KEEP: usize = 10;

/// Compact a conversation by keeping only the last N turns.
/// Returns a new Vec with an optional compaction header turn prepended.
pub fn compact_turns(turns: &[Turn], keep: Option<usize>) -> Vec<Turn> {
    let n = keep.unwrap_or(DEFAULT_KEEP);

    if turns.len() <= n {
        return turns.to_vec();
    }

    let dropped = turns.len() - n;
    let mut result = Vec::with_capacity(n + 1);

    // Add compaction header
    result.push(Turn {
        timestamp: String::new(),
        role: super::turn::Role::System,
        content: format!("[Compacted: {dropped} earlier turns removed]"),
    });

    // Keep last N turns
    result.extend_from_slice(&turns[turns.len() - n..]);
    result
}

/// Build a compaction summary message.
pub fn compaction_summary(original_count: usize, kept_count: usize) -> String {
    let dropped = original_count.saturating_sub(kept_count);
    format!("Compacted session: kept {kept_count} turns, removed {dropped} turns")
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
    fn compact_under_limit_returns_same() {
        let turns = vec![make_turn("a"), make_turn("b")];
        let result = compact_turns(&turns, Some(5));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].content, "a");
    }

    #[test]
    fn compact_at_limit_returns_same() {
        let turns = vec![make_turn("a"), make_turn("b")];
        let result = compact_turns(&turns, Some(2));
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn compact_over_limit_adds_header() {
        let turns: Vec<Turn> = (0..5).map(|i| make_turn(&format!("turn-{i}"))).collect();
        let result = compact_turns(&turns, Some(2));
        // Header + 2 kept turns
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].role, Role::System);
        assert!(result[0].content.contains("3 earlier turns removed"));
        assert_eq!(result[1].content, "turn-3");
        assert_eq!(result[2].content, "turn-4");
    }

    #[test]
    fn compact_default_keep() {
        let turns: Vec<Turn> = (0..15).map(|i| make_turn(&format!("t{i}"))).collect();
        let result = compact_turns(&turns, None);
        // Header + 10 kept
        assert_eq!(result.len(), 11);
        assert!(result[0].content.contains("5 earlier turns removed"));
    }

    #[test]
    fn compact_empty() {
        let result = compact_turns(&[], Some(5));
        assert!(result.is_empty());
    }

    #[test]
    fn compact_keep_one() {
        let turns = vec![make_turn("a"), make_turn("b"), make_turn("c")];
        let result = compact_turns(&turns, Some(1));
        assert_eq!(result.len(), 2); // header + 1
        assert!(result[0].content.contains("2 earlier turns removed"));
        assert_eq!(result[1].content, "c");
    }

    #[test]
    fn compaction_summary_message() {
        let msg = compaction_summary(20, 10);
        assert_eq!(msg, "Compacted session: kept 10 turns, removed 10 turns");
    }

    #[test]
    fn compaction_summary_no_removal() {
        let msg = compaction_summary(5, 5);
        assert_eq!(msg, "Compacted session: kept 5 turns, removed 0 turns");
    }

    #[test]
    fn compact_header_has_system_role() {
        let turns: Vec<Turn> = (0..5).map(|i| make_turn(&format!("t{i}"))).collect();
        let result = compact_turns(&turns, Some(2));
        assert_eq!(result[0].role, Role::System);
    }
}
