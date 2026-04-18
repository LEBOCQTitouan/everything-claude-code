use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Severity level for an audit finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Critical: immediate attention required.
    Critical,
    /// High: significant issue that should be addressed soon.
    High,
    /// Medium: issue that should be addressed during regular maintenance.
    Medium,
    /// Low: minor issue with minimal impact.
    Low,
}

/// A single issue discovered during an audit check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditFinding {
    /// Unique identifier for the finding.
    pub id: String,
    /// Severity of the finding.
    pub severity: Severity,
    /// Short title of the issue.
    pub title: String,
    /// Detailed description of the issue.
    pub detail: String,
    /// Recommended fix for the issue.
    pub fix: String,
}

/// Result of a single named audit check (pass/fail with findings).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditCheckResult {
    /// Name of the check.
    pub name: String,
    /// True if the check passed, false otherwise.
    pub passed: bool,
    /// Issues found by this check.
    pub findings: Vec<AuditFinding>,
}

/// Aggregated audit report with scored grade.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditReport {
    /// Results from all audit checks.
    pub checks: Vec<AuditCheckResult>,
    /// Numeric score (0-100).
    pub score: i32,
    /// Letter grade (A-F).
    pub grade: String,
}

/// Hooks diff between settings and source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HooksDiff {
    /// Hooks in settings that are outdated (source has been updated).
    pub stale: Vec<HookDiffEntry>,
    /// Hooks in source that are missing from settings.
    pub missing: Vec<HookDiffEntry>,
    /// Hooks that match between source and settings.
    pub matching: Vec<HookDiffEntry>,
    /// Hooks in settings that are not ECC-managed.
    pub user_hooks: Vec<HookDiffEntry>,
}

/// A single hook entry in a diff comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookDiffEntry {
    /// The hook event type (e.g., `pre-tool-use`).
    pub event: String,
    /// The hook entry as a JSON value.
    pub entry: serde_json::Value,
}

/// A typed hook entry in a diff comparison (no serde_json::Value).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedHookDiffEntry {
    /// The hook event type (e.g., `pre-tool-use`).
    pub event: String,
    /// The hook entry as a typed structure.
    pub entry: super::hook_types::HookEntry,
}

/// Audit summary for a single artifact type (agents, commands, etc.).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactAudit {
    /// Artifacts that match source and are up-to-date.
    pub matching: Vec<String>,
    /// Artifacts that are installed but outdated.
    pub outdated: Vec<String>,
    /// Artifacts in source but not installed.
    pub missing: Vec<String>,
}

/// Full configuration audit comparing installed artifacts and hooks against source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigAudit {
    /// Audit results for agents.
    pub agents: ArtifactAudit,
    /// Audit results for commands.
    pub commands: ArtifactAudit,
    /// Hooks diff results.
    pub hooks: HooksDiff,
    /// True if any differences were found.
    pub has_differences: bool,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

// Re-export from shared location for backwards compatibility.
pub use super::ECC_PACKAGE_IDENTIFIERS;

// ---------------------------------------------------------------------------
// Pure functions
// ---------------------------------------------------------------------------

/// Check if a command string matches legacy hook patterns.
///
/// # Arguments
///
/// * `cmd` — The command string to check.
///
/// # Returns
///
/// True if the command matches any known legacy pattern.
pub fn is_legacy_pattern(cmd: &str) -> bool {
    // Old-style scripts/hooks/ direct paths
    if cmd.contains("scripts/hooks/") && !cmd.contains("run-with-flags-shell.sh") {
        return true;
    }

    // Unresolved placeholder commands
    if cmd.contains("${ECC_ROOT}") || cmd.contains("${CLAUDE_PLUGIN_ROOT}") {
        return true;
    }

    // Resolved absolute-path run-with-flags.js (not via ecc-hook wrapper)
    if cmd.contains("/dist/hooks/run-with-flags.js") && !cmd.starts_with("ecc-hook") {
        return true;
    }

    // Resolved absolute-path shell hook commands
    if cmd.contains("/scripts/hooks/run-with-flags-shell.sh") && !cmd.starts_with("ecc-shell-hook")
    {
        return true;
    }

    // Inline node -e one-liners from pre-hook-runner era
    if cmd.contains("node -e")
        && (cmd.contains("dev-server")
            || cmd.contains("tmux")
            || cmd.contains("git push")
            || cmd.contains("console.log")
            || cmd.contains("check-console")
            || cmd.contains("pr-created")
            || cmd.contains("build-complete"))
    {
        return true;
    }

    false
}

/// Check if a hook entry is ECC-managed by examining its hooks array.
///
/// A hook is ECC-managed if any of its commands:
/// 1. Start with `ecc-hook` or `ecc-shell-hook` (current format)
/// 2. Contain a known ECC package identifier
/// 3. Match any entry in the provided source hooks
/// 4. Match legacy patterns
///
/// # Arguments
///
/// * `entry` — The hook entry JSON value to check.
/// * `source_hooks` — The source hooks to compare against.
///
/// # Returns
///
/// True if the hook is ECC-managed.
pub fn is_ecc_managed_hook(entry: &serde_json::Value, source_hooks: &serde_json::Value) -> bool {
    let hooks = match entry.get("hooks").and_then(|h| h.as_array()) {
        Some(h) => h,
        None => return false,
    };

    if hooks.is_empty() {
        return false;
    }

    for hook in hooks {
        let cmd = match hook.get("command").and_then(|c| c.as_str()) {
            Some(c) => c,
            None => continue,
        };

        // 1. Current ecc-hook / ecc-shell-hook wrappers
        if cmd.starts_with("ecc-hook ") || cmd.starts_with("ecc-shell-hook ") {
            return true;
        }

        // 2. Absolute path containing ECC package identifier
        for identifier in ECC_PACKAGE_IDENTIFIERS {
            if cmd.contains(identifier) {
                return true;
            }
        }

        // 3. Check if this hook matches any entry in the source hooks
        if matches_source_hook(entry, source_hooks) {
            return true;
        }

        // 4. Legacy patterns
        if is_legacy_pattern(cmd) {
            return true;
        }
    }

    false
}

/// Check if an entry matches any hook in the source hooks by comparing
/// their serialized `hooks` arrays.
///
/// # Arguments
///
/// * `entry` — The hook entry to match.
/// * `source_hooks` — The source hooks to check against.
///
/// # Returns
///
/// True if the entry matches any source hook.
fn matches_source_hook(entry: &serde_json::Value, source_hooks: &serde_json::Value) -> bool {
    let entry_key = match entry.get("hooks") {
        Some(h) => serde_json::to_string(h).unwrap_or_default(),
        None => return false,
    };

    let source_obj = match source_hooks.as_object() {
        Some(o) => o,
        None => return false,
    };

    for entries in source_obj.values() {
        let arr = match entries.as_array() {
            Some(a) => a,
            None => continue,
        };
        for source_entry in arr {
            if let Some(sh) = source_entry.get("hooks") {
                let source_key = serde_json::to_string(sh).unwrap_or_default();
                if source_key == entry_key {
                    return true;
                }
            }
        }
    }

    false
}

/// Compute an audit score and grade from check results.
///
/// Starts at 100, deducting:
/// - 20 per critical finding
/// - 10 per high finding
/// - 5 per medium finding
/// - 2 per low finding
///
/// Grade: A (90+), B (80+), C (70+), D (60+), F (<60)
///
/// # Arguments
///
/// * `checks` — All audit check results.
///
/// # Returns
///
/// A tuple of (score, grade).
pub fn compute_audit_score(checks: &[AuditCheckResult]) -> (i32, String) {
    let all_findings: Vec<&AuditFinding> = checks.iter().flat_map(|c| &c.findings).collect();

    let mut score: i32 = 100;
    for finding in &all_findings {
        match finding.severity {
            Severity::Critical => score -= 20,
            Severity::High => score -= 10,
            Severity::Medium => score -= 5,
            Severity::Low => score -= 2,
        }
    }
    score = score.clamp(0, 100);

    let grade = match score {
        90..=100 => "A",
        80..=89 => "B",
        70..=79 => "C",
        60..=69 => "D",
        _ => "F",
    }
    .to_string();

    (score, grade)
}

/// Simple YAML frontmatter parser.
/// Returns key-value pairs from the frontmatter block.
///
/// Delegates to the canonical `extract_frontmatter` in `config::validate`,
/// converting the result to a `BTreeMap` (empty on missing frontmatter).
///
/// # Arguments
///
/// * `content` — The content to parse for frontmatter.
///
/// # Returns
///
/// A map of frontmatter keys and values.
pub fn parse_frontmatter(content: &str) -> BTreeMap<String, String> {
    match super::validate::extract_frontmatter(content) {
        Some(map) => map.into_iter().collect(),
        None => BTreeMap::new(),
    }
}

/// Check if a typed hook entry is ECC-managed.
///
/// # Arguments
///
/// * `entry` — The typed hook entry to check.
/// * `source_hooks` — The source hooks to compare against.
///
/// # Returns
///
/// True if the hook is ECC-managed.
pub fn is_ecc_managed_hook_typed(
    entry: &super::hook_types::HookEntry,
    source_hooks: &super::hook_types::HooksMap,
) -> bool {
    let hooks = match &entry.hooks {
        Some(h) if !h.is_empty() => h,
        _ => return false,
    };

    for hook in hooks {
        let cmd = match &hook.command {
            Some(super::hook_types::HookCommandValue::Single(c)) => c.as_str(),
            _ => continue,
        };

        if cmd.starts_with("ecc-hook ") || cmd.starts_with("ecc-shell-hook ") {
            return true;
        }

        for identifier in ECC_PACKAGE_IDENTIFIERS {
            if cmd.contains(identifier) {
                return true;
            }
        }

        // Check if entry matches any source hook
        for source_entries in source_hooks.values() {
            if source_entries.contains(entry) {
                return true;
            }
        }

        if is_legacy_pattern(cmd) {
            return true;
        }
    }

    false
}

/// Check if a typed hook entry exists in the source hooks.
///
/// # Arguments
///
/// * `event` — The hook event type.
/// * `entry` — The hook entry to check.
/// * `source_hooks` — The source hooks to search.
///
/// # Returns
///
/// True if the entry exists in the source hooks.
pub fn exists_in_source_typed(
    event: &str,
    entry: &super::hook_types::HookEntry,
    source_hooks: &super::hook_types::HooksMap,
) -> bool {
    match source_hooks.get(event) {
        Some(entries) => entries.contains(entry),
        None => false,
    }
}

/// Check if a typed source hook entry exists in the settings hooks.
///
/// # Arguments
///
/// * `event` — The hook event type.
/// * `entry` — The hook entry to check.
/// * `settings_hooks` — The settings hooks to search.
///
/// # Returns
///
/// True if the entry exists in the settings hooks.
pub fn exists_in_settings_typed(
    event: &str,
    entry: &super::hook_types::HookEntry,
    settings_hooks: &super::hook_types::HooksMap,
) -> bool {
    match settings_hooks.get(event) {
        Some(entries) => entries.contains(entry),
        None => false,
    }
}

/// Check if a settings hook entry exists in the source hooks (by serialized hooks array).
///
/// # Arguments
///
/// * `event` — The hook event type.
/// * `settings_entry` — The settings hook entry to check.
/// * `source_hooks` — The source hooks to search.
///
/// # Returns
///
/// True if the entry exists in the source hooks.
pub fn exists_in_source(
    event: &str,
    settings_entry: &serde_json::Value,
    source_hooks: &serde_json::Value,
) -> bool {
    let entries = match source_hooks.get(event).and_then(|e| e.as_array()) {
        Some(a) => a,
        None => return false,
    };
    let key = match settings_entry.get("hooks") {
        Some(h) => serde_json::to_string(h).unwrap_or_default(),
        None => return false,
    };
    entries.iter().any(|e| {
        e.get("hooks")
            .map(|h| serde_json::to_string(h).unwrap_or_default())
            .is_some_and(|k| k == key)
    })
}

/// Check if a source hook entry exists in the settings hooks.
///
/// # Arguments
///
/// * `event` — The hook event type.
/// * `source_entry` — The source hook entry to check.
/// * `settings_hooks` — The settings hooks to search.
///
/// # Returns
///
/// True if the entry exists in the settings hooks.
pub fn exists_in_settings(
    event: &str,
    source_entry: &serde_json::Value,
    settings_hooks: &serde_json::Value,
) -> bool {
    let entries = match settings_hooks.get(event).and_then(|e| e.as_array()) {
        Some(a) => a,
        None => return false,
    };
    let key = match source_entry.get("hooks") {
        Some(h) => serde_json::to_string(h).unwrap_or_default(),
        None => return false,
    };
    entries.iter().any(|e| {
        e.get("hooks")
            .map(|h| serde_json::to_string(h).unwrap_or_default())
            .is_some_and(|k| k == key)
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- is_legacy_pattern ---

    #[test]
    fn is_legacy_pattern_scripts_hooks() {
        assert!(is_legacy_pattern("node scripts/hooks/check.js"));
    }

    #[test]
    fn is_legacy_pattern_not_shell_wrapper() {
        assert!(!is_legacy_pattern("scripts/hooks/run-with-flags-shell.sh"));
    }

    #[test]
    fn is_legacy_pattern_ecc_root_placeholder() {
        assert!(is_legacy_pattern("${ECC_ROOT}/hooks/run.js"));
    }

    #[test]
    fn is_legacy_pattern_claude_plugin_root() {
        assert!(is_legacy_pattern("${CLAUDE_PLUGIN_ROOT}/hooks/run.js"));
    }

    #[test]
    fn is_legacy_pattern_dist_hooks_not_wrapper() {
        assert!(is_legacy_pattern(
            "node /home/user/.ecc/dist/hooks/run-with-flags.js"
        ));
    }

    #[test]
    fn is_legacy_pattern_dist_hooks_via_wrapper() {
        assert!(!is_legacy_pattern(
            "ecc-hook /home/user/.ecc/dist/hooks/run-with-flags.js"
        ));
    }

    #[test]
    fn is_legacy_pattern_shell_hook_not_wrapper() {
        assert!(is_legacy_pattern(
            "bash /abs/path/scripts/hooks/run-with-flags-shell.sh"
        ));
    }

    #[test]
    fn is_legacy_pattern_shell_hook_via_wrapper() {
        assert!(!is_legacy_pattern(
            "ecc-shell-hook /abs/path/scripts/hooks/run-with-flags-shell.sh"
        ));
    }

    #[test]
    fn is_legacy_pattern_node_e_dev_server() {
        assert!(is_legacy_pattern("node -e 'require(\"dev-server\")'"));
    }

    #[test]
    fn is_legacy_pattern_node_e_tmux() {
        assert!(is_legacy_pattern("node -e 'tmux split-window'"));
    }

    #[test]
    fn is_legacy_pattern_node_e_git_push() {
        assert!(is_legacy_pattern("node -e 'exec(\"git push\")'"));
    }

    #[test]
    fn is_legacy_pattern_node_e_console_log() {
        assert!(is_legacy_pattern("node -e 'console.log(1)'"));
    }

    #[test]
    fn is_legacy_pattern_node_e_check_console() {
        assert!(is_legacy_pattern("node -e 'check-console()'"));
    }

    #[test]
    fn is_legacy_pattern_node_e_pr_created() {
        assert!(is_legacy_pattern("node -e 'pr-created()'"));
    }

    #[test]
    fn is_legacy_pattern_node_e_build_complete() {
        assert!(is_legacy_pattern("node -e 'build-complete()'"));
    }

    #[test]
    fn is_legacy_pattern_not_legacy() {
        assert!(!is_legacy_pattern("ecc-hook pre-tool-use format"));
    }

    #[test]
    fn is_legacy_pattern_normal_node_e() {
        assert!(!is_legacy_pattern("node -e 'process.exit(0)'"));
    }

    // --- is_ecc_managed_hook ---

    #[test]
    fn is_ecc_managed_hook_ecc_hook_prefix() {
        let entry = serde_json::json!({
            "hooks": [{"command": "ecc-hook pre-tool-use format"}]
        });
        let source = serde_json::json!({});
        assert!(is_ecc_managed_hook(&entry, &source));
    }

    #[test]
    fn is_ecc_managed_hook_ecc_shell_hook_prefix() {
        let entry = serde_json::json!({
            "hooks": [{"command": "ecc-shell-hook post-tool-use lint"}]
        });
        let source = serde_json::json!({});
        assert!(is_ecc_managed_hook(&entry, &source));
    }

    #[test]
    fn is_ecc_managed_hook_package_identifier() {
        let entry = serde_json::json!({
            "hooks": [{"command": "/usr/lib/node_modules/@lebocqtitouan/ecc/dist/hooks/run.js"}]
        });
        let source = serde_json::json!({});
        assert!(is_ecc_managed_hook(&entry, &source));
    }

    #[test]
    fn is_ecc_managed_hook_source_match() {
        let entry = serde_json::json!({
            "hooks": [{"command": "custom-cmd"}]
        });
        let source = serde_json::json!({
            "PreToolUse": [{"hooks": [{"command": "custom-cmd"}]}]
        });
        assert!(is_ecc_managed_hook(&entry, &source));
    }

    #[test]
    fn is_ecc_managed_hook_not_managed() {
        let entry = serde_json::json!({
            "hooks": [{"command": "my-custom-hook"}]
        });
        let source = serde_json::json!({});
        assert!(!is_ecc_managed_hook(&entry, &source));
    }

    #[test]
    fn is_ecc_managed_hook_no_hooks_array() {
        let entry = serde_json::json!({"description": "test"});
        let source = serde_json::json!({});
        assert!(!is_ecc_managed_hook(&entry, &source));
    }

    #[test]
    fn is_ecc_managed_hook_empty_hooks() {
        let entry = serde_json::json!({"hooks": []});
        let source = serde_json::json!({});
        assert!(!is_ecc_managed_hook(&entry, &source));
    }

    // --- compute_audit_score ---

    #[test]
    fn compute_audit_score_all_pass() {
        let checks = vec![AuditCheckResult {
            name: "Test".into(),
            passed: true,
            findings: vec![],
        }];
        let (score, grade) = compute_audit_score(&checks);
        assert_eq!(score, 100);
        assert_eq!(grade, "A");
    }

    #[test]
    fn compute_audit_score_with_critical() {
        let checks = vec![AuditCheckResult {
            name: "Test".into(),
            passed: false,
            findings: vec![AuditFinding {
                id: "T-1".into(),
                severity: Severity::Critical,
                title: "Crit".into(),
                detail: "d".into(),
                fix: "f".into(),
            }],
        }];
        let (score, grade) = compute_audit_score(&checks);
        assert_eq!(score, 80);
        assert_eq!(grade, "B");
    }

    #[test]
    fn compute_audit_score_with_high() {
        let checks = vec![AuditCheckResult {
            name: "Test".into(),
            passed: false,
            findings: vec![AuditFinding {
                id: "T-1".into(),
                severity: Severity::High,
                title: "High".into(),
                detail: "d".into(),
                fix: "f".into(),
            }],
        }];
        let (score, grade) = compute_audit_score(&checks);
        assert_eq!(score, 90);
        assert_eq!(grade, "A");
    }

    #[test]
    fn compute_audit_score_with_medium() {
        let checks = vec![AuditCheckResult {
            name: "Test".into(),
            passed: false,
            findings: vec![AuditFinding {
                id: "T-1".into(),
                severity: Severity::Medium,
                title: "Med".into(),
                detail: "d".into(),
                fix: "f".into(),
            }],
        }];
        let (score, _) = compute_audit_score(&checks);
        assert_eq!(score, 95);
    }

    #[test]
    fn compute_audit_score_with_low() {
        let checks = vec![AuditCheckResult {
            name: "Test".into(),
            passed: false,
            findings: vec![AuditFinding {
                id: "T-1".into(),
                severity: Severity::Low,
                title: "Low".into(),
                detail: "d".into(),
                fix: "f".into(),
            }],
        }];
        let (score, _) = compute_audit_score(&checks);
        assert_eq!(score, 98);
    }

    #[test]
    fn compute_audit_score_floor_at_zero() {
        let findings: Vec<AuditFinding> = (0..10)
            .map(|i| AuditFinding {
                id: format!("T-{i}"),
                severity: Severity::Critical,
                title: "Crit".into(),
                detail: "d".into(),
                fix: "f".into(),
            })
            .collect();
        let checks = vec![AuditCheckResult {
            name: "Test".into(),
            passed: false,
            findings,
        }];
        let (score, grade) = compute_audit_score(&checks);
        assert_eq!(score, 0);
        assert_eq!(grade, "F");
    }

    #[test]
    fn compute_audit_score_grade_c() {
        // 100 - 20 - 10 = 70 -> C
        let checks = vec![AuditCheckResult {
            name: "Test".into(),
            passed: false,
            findings: vec![
                AuditFinding {
                    id: "T-1".into(),
                    severity: Severity::Critical,
                    title: "c".into(),
                    detail: "d".into(),
                    fix: "f".into(),
                },
                AuditFinding {
                    id: "T-2".into(),
                    severity: Severity::High,
                    title: "h".into(),
                    detail: "d".into(),
                    fix: "f".into(),
                },
            ],
        }];
        let (score, grade) = compute_audit_score(&checks);
        assert_eq!(score, 70);
        assert_eq!(grade, "C");
    }

    #[test]
    fn compute_audit_score_grade_d() {
        // 100 - 20 - 20 = 60 -> D
        let checks = vec![AuditCheckResult {
            name: "Test".into(),
            passed: false,
            findings: vec![
                AuditFinding {
                    id: "T-1".into(),
                    severity: Severity::Critical,
                    title: "c1".into(),
                    detail: "d".into(),
                    fix: "f".into(),
                },
                AuditFinding {
                    id: "T-2".into(),
                    severity: Severity::Critical,
                    title: "c2".into(),
                    detail: "d".into(),
                    fix: "f".into(),
                },
            ],
        }];
        let (score, grade) = compute_audit_score(&checks);
        assert_eq!(score, 60);
        assert_eq!(grade, "D");
    }

    // --- parse_frontmatter ---

    #[test]
    fn parse_frontmatter_basic() {
        let content = "---\nname: test\ndescription: hello\n---\n# Body";
        let fm = parse_frontmatter(content);
        assert_eq!(fm.get("name").unwrap(), "test");
        assert_eq!(fm.get("description").unwrap(), "hello");
    }

    #[test]
    fn parse_frontmatter_no_frontmatter() {
        let fm = parse_frontmatter("# Just markdown");
        assert!(fm.is_empty());
    }

    #[test]
    fn parse_frontmatter_unclosed() {
        let fm = parse_frontmatter("---\nname: broken\nno closing");
        assert!(fm.is_empty());
    }

    #[test]
    fn parse_frontmatter_empty_value() {
        let content = "---\nname:\n---\n";
        let fm = parse_frontmatter(content);
        assert_eq!(fm.get("name").unwrap(), "");
    }
}
