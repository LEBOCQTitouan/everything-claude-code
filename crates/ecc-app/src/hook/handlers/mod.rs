//! Hook handler implementations.
//!
//! Each handler is a pure-ish function: `fn handle(stdin, ports) -> HookResult`

pub mod tier0_pretool;
mod tier1_simple;
mod tier2_notify;
mod tier2_post_tool_use;
mod tier2_tools;
mod tier3_session;

pub use tier1_simple::{
    check_console_log, check_hook_enabled, doc_coverage_reminder, doc_file_warning,
    instructions_loaded_validate, post_bash_build_complete, post_bash_pr_created,
    post_edit_boy_scout_delta, post_edit_console_warn, post_edit_naming_review,
    post_edit_newspaper_check, post_exit_worktree_cleanup_reminder, post_failure_error_context,
    pre_bash_git_push_reminder, pre_bash_tmux_reminder, pre_edit_boundary_crossing,
    pre_edit_stepdown_warning, pre_edit_write_workflow_branch_guard, pre_prompt_context_hydrate,
    pre_prompt_context_inject, pre_worktree_write_guard, session_end_marker,
    stop_uncommitted_reminder, subagent_start_effort, suggest_compact,
};
pub use tier2_notify::stop_notify;
pub use tier2_post_tool_use::post_tool_use_heartbeat;
pub use tier2_tools::{
    post_edit_format, post_edit_typecheck, pre_bash_dev_server_block, quality_gate,
};
pub use tier3_session::{
    config_change_log, cost_tracker, craft_velocity, daily_summary, evaluate_session,
    oath_reflection, post_compact, post_enter_worktree_session_log, pre_compact, session_end,
    session_end_merge, session_start, start_cartography, stop_cartography, subagent_start_log,
    subagent_stop_log,
};
