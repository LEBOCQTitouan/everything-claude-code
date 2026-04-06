//! Production infrastructure adapters for Everything Claude Code.
//!
//! Implements [`ecc_ports`] traits against real operating system primitives:
//! filesystem I/O, process execution, environment variables, and terminal
//! interaction.

pub mod git_log_adapter;
pub mod github_release;
pub mod os_env;
pub mod os_fs;
pub mod os_git;
pub mod process_executor;
pub mod rustyline_input;
pub mod std_terminal;
pub mod system_clock;

#[cfg(unix)]
pub mod flock_lock;

pub mod cost_schema;
pub mod metrics_schema;
pub mod sqlite_metrics_store;
pub mod file_config_store;
pub mod log_schema;
pub mod sqlite_cost_store;
pub mod sqlite_log_store;
pub mod sqlite_memory;
pub mod tarball_extractor;
pub mod os_worktree;
