//! Tier 1 Hooks — Simple passthrough/warn hooks with no external tool spawning.

mod ci_hooks;
mod clean_craft_hooks;
pub mod context_hydration;
mod dev_hooks;
mod doc_hooks;
mod git_hooks;
mod helpers;
mod meta_hooks;
pub mod worktree_guard;

pub use ci_hooks::pre_edit_write_workflow_branch_guard;
pub use clean_craft_hooks::{
    post_edit_boy_scout_delta, post_edit_naming_review, post_edit_newspaper_check,
    pre_edit_boundary_crossing, pre_edit_stepdown_warning,
};
pub use context_hydration::pre_prompt_context_hydrate;
pub use dev_hooks::{
    instructions_loaded_validate, post_bash_build_complete, post_bash_pr_created,
    post_exit_worktree_cleanup_reminder, post_failure_error_context, pre_bash_tmux_reminder,
    pre_prompt_context_inject, suggest_compact,
};
pub use doc_hooks::{doc_coverage_reminder, doc_file_warning};
pub use git_hooks::{
    check_console_log, post_edit_console_warn, pre_bash_git_push_reminder,
    stop_uncommitted_reminder,
};
pub use meta_hooks::{check_hook_enabled, session_end_marker};
pub use worktree_guard::pre_worktree_write_guard;
