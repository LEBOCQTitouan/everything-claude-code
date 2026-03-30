use super::*;

// --- is_legacy_ecc_hook ---

#[test]
fn is_legacy_ecc_hook_package_identifier_lebocqtitouan() {
    let entry = serde_json::json!({
        "hooks": [{"command": "/usr/lib/node_modules/@lebocqtitouan/ecc/dist/hooks/run.js"}]
    });
    assert!(is_legacy_ecc_hook(&entry));
}

#[test]
fn is_legacy_ecc_hook_package_identifier_everything() {
    let entry = serde_json::json!({
        "hooks": [{"command": "/home/.npm/everything-claude-code/hooks/run.js"}]
    });
    assert!(is_legacy_ecc_hook(&entry));
}

#[test]
fn is_legacy_ecc_hook_scripts_hooks() {
    let entry = serde_json::json!({
        "hooks": [{"command": "node scripts/hooks/check.js"}]
    });
    assert!(is_legacy_ecc_hook(&entry));
}

#[test]
fn is_legacy_ecc_hook_ecc_root_placeholder() {
    let entry = serde_json::json!({
        "hooks": [{"command": "${ECC_ROOT}/hooks/run.js"}]
    });
    assert!(is_legacy_ecc_hook(&entry));
}

#[test]
fn is_legacy_ecc_hook_claude_plugin_root() {
    let entry = serde_json::json!({
        "hooks": [{"command": "${CLAUDE_PLUGIN_ROOT}/hooks/run.js"}]
    });
    assert!(is_legacy_ecc_hook(&entry));
}

#[test]
fn is_legacy_ecc_hook_run_with_flags() {
    let entry = serde_json::json!({
        "hooks": [{"command": "node /abs/path/dist/hooks/run-with-flags.js"}]
    });
    assert!(is_legacy_ecc_hook(&entry));
}

#[test]
fn is_legacy_ecc_hook_shell_hook_path() {
    let entry = serde_json::json!({
        "hooks": [{"command": "bash /abs/path/scripts/hooks/run-with-flags-shell.sh"}]
    });
    assert!(is_legacy_ecc_hook(&entry));
}

#[test]
fn is_legacy_ecc_hook_node_e_dev_server() {
    let entry = serde_json::json!({
        "hooks": [{"command": "node -e 'require(\"dev-server\")'"}]
    });
    assert!(is_legacy_ecc_hook(&entry));
}

#[test]
fn is_legacy_ecc_hook_node_e_tmux() {
    let entry = serde_json::json!({
        "hooks": [{"command": "node -e 'tmux split'"}]
    });
    assert!(is_legacy_ecc_hook(&entry));
}

#[test]
fn is_legacy_ecc_hook_node_e_build_complete() {
    let entry = serde_json::json!({
        "hooks": [{"command": "node -e 'build-complete()'"}]
    });
    assert!(is_legacy_ecc_hook(&entry));
}

#[test]
fn is_legacy_ecc_hook_not_for_ecc_hook_wrapper() {
    let entry = serde_json::json!({
        "hooks": [{"command": "ecc-hook pre-tool-use format"}]
    });
    assert!(!is_legacy_ecc_hook(&entry));
}

#[test]
fn is_legacy_ecc_hook_not_for_ecc_shell_hook_wrapper() {
    let entry = serde_json::json!({
        "hooks": [{"command": "ecc-shell-hook post-tool-use lint"}]
    });
    assert!(!is_legacy_ecc_hook(&entry));
}

#[test]
fn is_legacy_ecc_hook_stale_3arg_ecc_hook_with_dist_path() {
    let entry = serde_json::json!({
        "hooks": [{"command": "ecc-hook \"pre:bash:dev-server-block\" \"dist/hooks/pre-bash-dev-server-block.js\" \"standard,strict\""}]
    });
    assert!(is_legacy_ecc_hook(&entry));
}

#[test]
fn is_legacy_ecc_hook_stale_3arg_ecc_hook_post_tool() {
    let entry = serde_json::json!({
        "hooks": [{"command": "ecc-hook \"post:edit:format\" \"dist/hooks/post-edit-format.js\" \"standard,strict\""}]
    });
    assert!(is_legacy_ecc_hook(&entry));
}

#[test]
fn is_legacy_ecc_hook_not_for_user_hook() {
    let entry = serde_json::json!({
        "hooks": [{"command": "my-custom-hook"}]
    });
    assert!(!is_legacy_ecc_hook(&entry));
}

#[test]
fn is_legacy_ecc_hook_no_hooks_array() {
    let entry = serde_json::json!({"description": "test"});
    assert!(!is_legacy_ecc_hook(&entry));
}

// --- remove_legacy_hooks ---

#[test]
fn remove_legacy_hooks_removes_legacy() {
    let hooks = serde_json::json!({
        "PreToolUse": [
            {"hooks": [{"command": "ecc-hook format"}]},
            {"hooks": [{"command": "node scripts/hooks/old.js"}]}
        ]
    });

    let (result, removed) = remove_legacy_hooks(&hooks);
    assert_eq!(removed, 1);
    let arr = result["PreToolUse"].as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(
        arr[0]["hooks"][0]["command"].as_str().unwrap(),
        "ecc-hook format"
    );
}

#[test]
fn remove_legacy_hooks_keeps_current() {
    let hooks = serde_json::json!({
        "PreToolUse": [
            {"hooks": [{"command": "ecc-hook format"}]},
            {"hooks": [{"command": "ecc-shell-hook lint"}]}
        ]
    });

    let (result, removed) = remove_legacy_hooks(&hooks);
    assert_eq!(removed, 0);
    let arr = result["PreToolUse"].as_array().unwrap();
    assert_eq!(arr.len(), 2);
}

#[test]
fn remove_legacy_hooks_empty() {
    let hooks = serde_json::json!({});
    let (result, removed) = remove_legacy_hooks(&hooks);
    assert_eq!(removed, 0);
    assert!(result.as_object().unwrap().is_empty());
}

// --- BL-085: legacy worktree hook detection ---

#[test]
fn is_legacy_ecc_hook_worktree_create_init() {
    let entry = serde_json::json!({
        "hooks": [{"command": "ecc-hook \"worktree:create:init\" \"standard,strict\""}]
    });
    assert!(is_legacy_ecc_hook(&entry));
}

#[test]
fn is_legacy_ecc_hook_worktree_cleanup_reminder() {
    let entry = serde_json::json!({
        "hooks": [{"command": "ecc-hook \"stop:worktree-cleanup-reminder\" \"standard,strict\""}]
    });
    assert!(is_legacy_ecc_hook(&entry));
}

#[test]
fn remove_legacy_hooks_removes_worktree_delegation() {
    let hooks = serde_json::json!({
        "WorktreeCreate": [
            {"hooks": [{"command": "ecc-hook \"worktree:create:init\" \"standard,strict\""}]}
        ],
        "Stop": [
            {"hooks": [{"command": "ecc-hook \"stop:worktree-cleanup-reminder\" \"standard,strict\""}]}
        ]
    });
    let (result, removed) = remove_legacy_hooks(&hooks);
    assert_eq!(removed, 2);
    assert_eq!(result["WorktreeCreate"].as_array().unwrap().len(), 0);
    assert_eq!(result["Stop"].as_array().unwrap().len(), 0);
}

#[test]
fn remove_legacy_hooks_real_world_worktree_command() {
    let hooks = serde_json::json!({
        "WorktreeCreate": [
            {"hooks": [{"command": "ecc-hook \"worktree:create:init\" \"standard,strict\""}]}
        ]
    });
    let (result, removed) = remove_legacy_hooks(&hooks);
    assert_eq!(removed, 1);
    let arr = result["WorktreeCreate"].as_array().unwrap();
    assert_eq!(arr.len(), 0);
}

#[test]
fn is_legacy_ecc_hook_new_worktree_hooks_not_legacy() {
    let entry_enter = serde_json::json!({
        "hooks": [{"command": "ecc-hook \"post:enter-worktree:session-log\" \"standard,strict\""}]
    });
    let entry_exit = serde_json::json!({
        "hooks": [{"command": "ecc-hook \"post:exit-worktree:cleanup-reminder\" \"standard,strict\""}]
    });
    assert!(!is_legacy_ecc_hook(&entry_enter));
    assert!(!is_legacy_ecc_hook(&entry_exit));
}

// --- is_legacy_command ---

#[test]
fn is_legacy_command_dist_hooks() {
    // dist/hooks/ in ecc-hook 3-arg form is legacy
    assert!(is_legacy_command("ecc-hook \"pre:bash\" \"dist/hooks/pre-bash.js\" \"standard\""));
}

#[test]
fn is_legacy_command_worktree_create_init() {
    assert!(is_legacy_command("ecc-hook \"worktree:create:init\" \"standard,strict\""));
}

#[test]
fn is_legacy_command_ecc_package_identifier() {
    assert!(is_legacy_command("/home/.npm/everything-claude-code/hooks/run.js"));
}

#[test]
fn is_legacy_command_scripts_hooks() {
    assert!(is_legacy_command("node scripts/hooks/check.js"));
}

#[test]
fn is_legacy_command_ecc_root_placeholder() {
    assert!(is_legacy_command("${ECC_ROOT}/hooks/run.js"));
}

#[test]
fn is_legacy_command_run_with_flags_js() {
    assert!(is_legacy_command("node /abs/path/dist/hooks/run-with-flags.js"));
}

#[test]
fn is_legacy_command_run_with_flags_shell() {
    assert!(is_legacy_command("bash /abs/path/scripts/hooks/run-with-flags-shell.sh"));
}

#[test]
fn is_legacy_command_node_e_dev_server() {
    assert!(is_legacy_command("node -e 'require(\"dev-server\")'"));
}

#[test]
fn is_legacy_command_node_e_tmux() {
    assert!(is_legacy_command("node -e 'tmux split'"));
}

#[test]
fn is_legacy_command_not_for_ecc_hook_wrapper() {
    assert!(!is_legacy_command("ecc-hook pre-tool-use format"));
}

#[test]
fn is_legacy_command_not_for_ecc_shell_hook_wrapper() {
    assert!(!is_legacy_command("ecc-shell-hook post-tool-use lint"));
}

#[test]
fn is_legacy_command_not_for_user_hook() {
    assert!(!is_legacy_command("my-custom-hook"));
}

#[test]
fn is_legacy_command_stop_worktree_cleanup_reminder() {
    assert!(is_legacy_command("ecc-hook \"stop:worktree-cleanup-reminder\" \"standard,strict\""));
}
