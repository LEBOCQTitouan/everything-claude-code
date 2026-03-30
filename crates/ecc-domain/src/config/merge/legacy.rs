use super::super::hook_types;
use super::ECC_PACKAGE_IDENTIFIERS;

/// Check if a command string matches any known legacy ECC hook pattern.
///
/// This is a shared helper used by both `is_legacy_ecc_hook` (untyped) and
/// `is_legacy_ecc_hook_typed` to avoid duplicating pattern-matching logic.
pub fn is_legacy_command(cmd: &str) -> bool {
    // Current wrapper commands are NOT legacy — unless they contain a
    // stale dist/hooks/ JS path from the Node.js era (3-arg format).
    if cmd.starts_with("ecc-hook ") || cmd.starts_with("ecc-shell-hook ") {
        if cmd.contains("dist/hooks/") {
            return true;
        }
        // Deprecated worktree delegation hook IDs (BL-085)
        if cmd.contains("worktree:create:init")
            || cmd.contains("stop:worktree-cleanup-reminder")
        {
            return true;
        }
        return false;
    }

    // Absolute path containing ECC package identifier
    for identifier in ECC_PACKAGE_IDENTIFIERS {
        if cmd.contains(identifier) {
            return true;
        }
    }

    // Old-style scripts/hooks/ direct paths
    if cmd.contains("scripts/hooks/") && !cmd.contains("run-with-flags-shell.sh") {
        return true;
    }

    // Unresolved placeholder commands
    if cmd.contains("${ECC_ROOT}") || cmd.contains("${CLAUDE_PLUGIN_ROOT}") {
        return true;
    }

    // Resolved absolute-path run-with-flags.js
    if cmd.contains("/dist/hooks/run-with-flags.js") {
        return true;
    }

    // Resolved absolute-path shell hook commands
    if cmd.contains("/scripts/hooks/run-with-flags-shell.sh") {
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

/// Check if a hook entry is a legacy ECC hook that should be removed.
///
/// Detects legacy hooks via:
/// 1. Absolute paths containing the ECC package identifier
/// 2. Old-style `scripts/hooks/` paths
/// 3. Unresolved placeholder commands
/// 4. Absolute-path `run-with-flags.js` / `run-with-flags-shell.sh`
/// 5. Inline `node -e` one-liners
///
/// Current `ecc-hook` / `ecc-shell-hook` wrapper commands are NOT flagged.
pub fn is_legacy_ecc_hook(entry: &serde_json::Value) -> bool {
    let hooks = match entry.get("hooks").and_then(|h| h.as_array()) {
        Some(h) => h,
        None => return false,
    };

    for hook in hooks {
        let cmd = match hook.get("command").and_then(|c| c.as_str()) {
            Some(c) => c,
            None => continue,
        };

        if is_legacy_command(cmd) {
            return true;
        }
    }

    false
}

/// Remove legacy hooks from a hooks object.
/// Returns a new hooks value with legacy hooks removed, and the count of removed hooks.
pub fn remove_legacy_hooks(hooks: &serde_json::Value) -> (serde_json::Value, usize) {
    let obj = match hooks.as_object() {
        Some(o) => o,
        None => return (hooks.clone(), 0),
    };

    let mut result = serde_json::Map::new();
    let mut removed = 0usize;

    for (event, entries) in obj {
        let arr = match entries.as_array() {
            Some(a) => a,
            None => {
                result.insert(event.clone(), entries.clone());
                continue;
            }
        };

        let original_len = arr.len();
        let filtered: Vec<serde_json::Value> = arr
            .iter()
            .filter(|entry| !is_legacy_ecc_hook(entry))
            .cloned()
            .collect();
        removed += original_len - filtered.len();
        result.insert(event.clone(), serde_json::Value::Array(filtered));
    }

    (serde_json::Value::Object(result), removed)
}

/// Check if a typed hook entry is a legacy ECC hook.
pub fn is_legacy_ecc_hook_typed(entry: &hook_types::HookEntry) -> bool {
    let hooks = match &entry.hooks {
        Some(h) => h,
        None => return false,
    };

    for hook in hooks {
        let cmd = match &hook.command {
            Some(hook_types::HookCommandValue::Single(c)) => c.as_str(),
            _ => continue,
        };

        if is_legacy_command(cmd) {
            return true;
        }
    }

    false
}

/// Remove legacy hooks from a typed hooks map.
/// Returns a new map with legacy hooks removed, and the count of removed hooks.
pub fn remove_legacy_hooks_typed(
    hooks: &hook_types::HooksMap,
) -> (hook_types::HooksMap, usize) {
    let mut result = hook_types::HooksMap::new();
    let mut removed = 0usize;

    for (event, entries) in hooks {
        let original_len = entries.len();
        let filtered: Vec<hook_types::HookEntry> = entries
            .iter()
            .filter(|entry| !is_legacy_ecc_hook_typed(entry))
            .cloned()
            .collect();
        removed += original_len - filtered.len();
        result.insert(event.clone(), filtered);
    }

    (result, removed)
}
