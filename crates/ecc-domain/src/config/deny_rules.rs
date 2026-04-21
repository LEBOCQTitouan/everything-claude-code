use serde::{Deserialize, Serialize};

/// A deny rule that prevents specific tools/patterns in Claude Code settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DenyRule {
    /// Name of the tool or pattern to deny.
    pub tool_name: String,
    /// Explanation of why this tool is denied.
    pub reason: String,
}

/// Evaluate whether a set of deny rules contains a match for the given tool.
pub fn is_denied<'a>(rules: &'a [DenyRule], tool: &str) -> Option<&'a DenyRule> {
    rules.iter().find(|r| r.tool_name == tool)
}

/// Default deny rules ECC should add to settings.json.
/// Protects against reading/writing secrets, destructive git commands, and shell exploits.
pub const ECC_DENY_RULES: &[&str] = &[
    "Read(//**/.env)",
    "Read(//**/.env.*)",
    "Write(//**/.env)",
    "Write(//**/.env.*)",
    "Read(//Users/*/.ssh/**)",
    "Read(//Users/*/.aws/**)",
    "Read(//Users/*/.gnupg/**)",
    "Read(//**/*.pem)",
    "Read(//**/*.key)",
    "Write(//**/*.pem)",
    "Write(//**/*.key)",
    "Bash(rm -rf:*)",
    "Bash(chmod 777:*)",
    "Bash(git push*--force*)",
];

/// Result of ensuring deny rules are present.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DenyRulesResult {
    /// Number of rules added.
    pub added: usize,
    /// Number of rules already present.
    pub existing: usize,
}

/// Ensure ECC deny rules are present in an existing deny list.
/// Returns the merged list and a result summary.
/// Non-destructive: only adds rules that don't already exist.
pub fn ensure_deny_rules(existing: &[String]) -> (Vec<String>, DenyRulesResult) {
    let mut existing_set = std::collections::HashSet::new();
    for rule in existing {
        existing_set.insert(rule.as_str());
    }

    let mut new_list: Vec<String> = existing.to_vec();
    let mut added = 0usize;
    let mut already_present = 0usize;

    for &rule in ECC_DENY_RULES {
        if existing_set.contains(rule) {
            already_present += 1;
        } else {
            new_list.push(rule.to_string());
            added += 1;
        }
    }

    (
        new_list,
        DenyRulesResult {
            added,
            existing: already_present,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- is_denied ---

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

    // --- ECC_DENY_RULES ---

    #[test]
    fn ecc_deny_rules_has_expected_count() {
        assert_eq!(ECC_DENY_RULES.len(), 14);
    }

    #[test]
    fn ecc_deny_rules_contains_env_protection() {
        assert!(ECC_DENY_RULES.contains(&"Read(//**/.env)"));
        assert!(ECC_DENY_RULES.contains(&"Write(//**/.env)"));
    }

    #[test]
    fn ecc_deny_rules_contains_ssh_protection() {
        assert!(ECC_DENY_RULES.contains(&"Read(//Users/*/.ssh/**)"));
    }

    #[test]
    fn ecc_deny_rules_contains_destructive_bash() {
        assert!(ECC_DENY_RULES.contains(&"Bash(rm -rf:*)"));
        assert!(ECC_DENY_RULES.contains(&"Bash(chmod 777:*)"));
        assert!(ECC_DENY_RULES.contains(&"Bash(git push*--force*)"));
    }

    // --- ensure_deny_rules ---

    #[test]
    fn ensure_deny_rules_empty_existing() {
        let (list, result) = ensure_deny_rules(&[]);
        assert_eq!(result.added, 14);
        assert_eq!(result.existing, 0);
        assert_eq!(list.len(), 14);
    }

    #[test]
    fn ensure_deny_rules_all_present() {
        let existing: Vec<String> = ECC_DENY_RULES.iter().map(|s| s.to_string()).collect();
        let (list, result) = ensure_deny_rules(&existing);
        assert_eq!(result.added, 0);
        assert_eq!(result.existing, 14);
        assert_eq!(list.len(), 14);
    }

    #[test]
    fn ensure_deny_rules_partial_overlap() {
        let existing = vec![
            "Read(//**/.env)".to_string(),
            "Write(//**/.env)".to_string(),
            "CustomRule(something)".to_string(),
        ];
        let (list, result) = ensure_deny_rules(&existing);
        assert_eq!(result.added, 12);
        assert_eq!(result.existing, 2);
        // Original rules preserved + new ones added
        assert_eq!(list.len(), 3 + 12);
        // Custom rule still present
        assert!(list.contains(&"CustomRule(something)".to_string()));
    }

    #[test]
    fn ensure_deny_rules_preserves_order() {
        let existing = vec!["CustomFirst".to_string()];
        let (list, _) = ensure_deny_rules(&existing);
        assert_eq!(list[0], "CustomFirst");
    }

    #[test]
    fn ensure_deny_rules_no_duplicates() {
        let existing: Vec<String> = ECC_DENY_RULES.iter().map(|s| s.to_string()).collect();
        let (list, _) = ensure_deny_rules(&existing);
        let mut seen = std::collections::HashSet::new();
        for rule in &list {
            assert!(seen.insert(rule.clone()), "duplicate rule: {rule}");
        }
    }
}
