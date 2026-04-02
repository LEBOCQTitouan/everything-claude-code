//! Production infrastructure adapters for Everything Claude Code.
//!
//! Implements [`ecc_ports`] traits against real operating system primitives:
//! filesystem I/O, process execution, environment variables, and terminal
//! interaction.

pub mod github_release;
pub mod os_env;
pub mod os_fs;
pub mod os_git;
pub mod system_clock;
pub mod process_executor;
pub mod rustyline_input;
pub mod std_terminal;

#[cfg(unix)]
pub mod flock_lock;

pub mod file_config_store;
pub mod log_schema;
pub mod sqlite_log_store;
pub mod sqlite_memory;
