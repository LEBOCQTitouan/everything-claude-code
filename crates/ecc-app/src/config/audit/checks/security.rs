//! Security-related audit checks — deny rules and hook duplicates.

use ecc_domain::config::audit::{AuditCheckResult, AuditFinding, Severity};
use ecc_ports::fs::FileSystem;
use std::path::Path;

use super::super::read_json_safe;

/// Check that ECC deny rules are present in settings.json.
pub fn check_deny_rules(fs: &dyn FileSystem, settings_path: &Path) -> AuditCheckResult {
    let mut findings = Vec::new();

    let settings = match read_json_safe(fs, settings_path) {
        Ok(Some(s)) => s,
        Ok(None) => {
            findings.push(AuditFinding {
                id: "DENY-001".into(),
                severity: Severity::Critical,
                title: "No settings.json found".into(),
                detail: format!("Expected settings at {}", settings_path.display()),
                fix: "Run `ecc install` to create settings with deny rules.".into(),
            });
            return AuditCheckResult {
                name: "Deny rules".into(),
                passed: false,
                findings,
            };
        }
        Err(msg) => {
            findings.push(AuditFinding {
                id: "DENY-002".into(),
                severity: Severity::High,
                title: "Corrupt settings.json".into(),
                detail: msg.to_string(),
                fix: "Fix or recreate settings.json.".into(),
            });
            return AuditCheckResult {
                name: "Deny rules".into(),
                passed: false,
                findings,
            };
        }
    };

    let deny: Vec<&str> = settings
        .get("permissions")
        .and_then(|p| p.get("deny"))
        .and_then(|d| d.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    let deny_set: std::collections::HashSet<&str> = deny.into_iter().collect();

    let missing: Vec<&&str> = ecc_domain::config::deny_rules::ECC_DENY_RULES
        .iter()
        .filter(|rule| !deny_set.contains(**rule))
        .collect();

    if !missing.is_empty() {
        let preview: Vec<&str> = missing.iter().take(3).map(|r| **r).collect();
        let suffix = if missing.len() > 3 {
            format!(" (and {} more)", missing.len() - 3)
        } else {
            String::new()
        };

        findings.push(AuditFinding {
            id: "DENY-002".into(),
            severity: Severity::Critical,
            title: format!("{} deny rule(s) missing", missing.len()),
            detail: format!("Missing: {}{suffix}", preview.join(", ")),
            fix: "Run `ecc install` to add deny rules, or add them manually to ~/.claude/settings.json.".into(),
        });
    }

    AuditCheckResult {
        name: "Deny rules".into(),
        passed: findings.is_empty(),
        findings,
    }
}

/// Check for duplicate hooks in settings.json.
pub fn check_hook_duplicates(fs: &dyn FileSystem, settings_path: &Path) -> AuditCheckResult {
    let mut findings = Vec::new();

    let settings = match read_json_safe(fs, settings_path) {
        Ok(Some(s)) => s,
        _ => {
            return AuditCheckResult {
                name: "Hook duplicates".into(),
                passed: true,
                findings,
            };
        }
    };

    let hooks_obj = match settings.get("hooks").and_then(|h| h.as_object()) {
        Some(h) => h,
        None => {
            return AuditCheckResult {
                name: "Hook duplicates".into(),
                passed: true,
                findings,
            };
        }
    };

    let mut total_duplicates = 0usize;

    for entries in hooks_obj.values() {
        let arr = match entries.as_array() {
            Some(a) => a,
            None => continue,
        };
        let mut seen = std::collections::HashSet::new();
        for entry in arr {
            let key = match entry.get("hooks") {
                Some(h) => serde_json::to_string(h).unwrap_or_default(),
                None => serde_json::to_string(entry).unwrap_or_default(),
            };
            if !seen.insert(key) {
                total_duplicates += 1;
            }
        }
    }

    if total_duplicates > 0 {
        findings.push(AuditFinding {
            id: "HOOK-001".into(),
            severity: Severity::High,
            title: format!("{total_duplicates} duplicate hook(s) found"),
            detail: "Duplicate hooks fire multiple times per event, wasting resources.".into(),
            fix: "Run `ecc install` to replace hooks section with the clean source.".into(),
        });
    }

    AuditCheckResult {
        name: "Hook duplicates".into(),
        passed: findings.is_empty(),
        findings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_domain::config::audit::Severity;
    use ecc_test_support::InMemoryFileSystem;
    use std::path::Path;

    // --- check_deny_rules ---

    #[test]
    fn check_deny_rules_all_present() {
        let deny_array: Vec<serde_json::Value> = ecc_domain::config::deny_rules::ECC_DENY_RULES
            .iter()
            .map(|r| serde_json::Value::String(r.to_string()))
            .collect();
        let settings = serde_json::json!({
            "permissions": { "deny": deny_array }
        });
        let fs = InMemoryFileSystem::new().with_file("/settings.json", &settings.to_string());

        let result = check_deny_rules(&fs, Path::new("/settings.json"));
        assert!(result.passed);
        assert!(result.findings.is_empty());
    }

    #[test]
    fn check_deny_rules_some_missing() {
        let settings = serde_json::json!({
            "permissions": { "deny": ["Read(//**/.env)"] }
        });
        let fs = InMemoryFileSystem::new().with_file("/settings.json", &settings.to_string());

        let result = check_deny_rules(&fs, Path::new("/settings.json"));
        assert!(!result.passed);
        assert_eq!(result.findings.len(), 1);
        assert_eq!(result.findings[0].id, "DENY-002");
        assert_eq!(result.findings[0].severity, Severity::Critical);
    }

    #[test]
    fn check_deny_rules_no_settings() {
        let fs = InMemoryFileSystem::new();
        let result = check_deny_rules(&fs, Path::new("/settings.json"));
        assert!(!result.passed);
        assert_eq!(result.findings[0].id, "DENY-001");
    }

    #[test]
    fn check_deny_rules_no_permissions_key() {
        let settings = serde_json::json!({"hooks": {}});
        let fs = InMemoryFileSystem::new().with_file("/settings.json", &settings.to_string());

        let result = check_deny_rules(&fs, Path::new("/settings.json"));
        assert!(!result.passed);
        assert_eq!(result.findings[0].id, "DENY-002");
    }

    // --- check_hook_duplicates ---

    #[test]
    fn check_hook_duplicates_no_duplicates() {
        let settings = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {"hooks": [{"command": "ecc-hook a"}]},
                    {"hooks": [{"command": "ecc-hook b"}]}
                ]
            }
        });
        let fs = InMemoryFileSystem::new().with_file("/settings.json", &settings.to_string());

        let result = check_hook_duplicates(&fs, Path::new("/settings.json"));
        assert!(result.passed);
    }

    #[test]
    fn check_hook_duplicates_with_duplicates() {
        let settings = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {"hooks": [{"command": "ecc-hook a"}]},
                    {"hooks": [{"command": "ecc-hook a"}]}
                ]
            }
        });
        let fs = InMemoryFileSystem::new().with_file("/settings.json", &settings.to_string());

        let result = check_hook_duplicates(&fs, Path::new("/settings.json"));
        assert!(!result.passed);
        assert_eq!(result.findings[0].id, "HOOK-001");
    }

    #[test]
    fn check_hook_duplicates_no_settings() {
        let fs = InMemoryFileSystem::new();
        let result = check_hook_duplicates(&fs, Path::new("/settings.json"));
        assert!(result.passed);
    }

    #[test]
    fn check_hook_duplicates_no_hooks_key() {
        let settings = serde_json::json!({"permissions": {}});
        let fs = InMemoryFileSystem::new().with_file("/settings.json", &settings.to_string());

        let result = check_hook_duplicates(&fs, Path::new("/settings.json"));
        assert!(result.passed);
    }
}
