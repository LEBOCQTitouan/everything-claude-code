use serde::{Deserialize, Serialize};

/// A deny rule that prevents specific tools/patterns in Claude Code settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DenyRule {
    pub tool_name: String,
    pub reason: String,
}

/// Evaluate whether a set of deny rules contains a match for the given tool.
pub fn is_denied<'a>(rules: &'a [DenyRule], tool: &str) -> Option<&'a DenyRule> {
    rules.iter().find(|r| r.tool_name == tool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_matching_rule() {
        let rules = vec![DenyRule {
            tool_name: "dangerous_tool".into(),
            reason: "not allowed".into(),
        }];
        assert!(is_denied(&rules, "dangerous_tool").is_some());
    }

    #[test]
    fn returns_none_when_no_match() {
        let rules = vec![DenyRule {
            tool_name: "other".into(),
            reason: "nope".into(),
        }];
        assert!(is_denied(&rules, "safe_tool").is_none());
    }
}
