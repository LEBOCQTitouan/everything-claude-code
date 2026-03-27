//! Production infrastructure adapters for Everything Claude Code.
//!
//! Implements [`ecc_ports`] traits against real operating system primitives:
//! filesystem I/O, process execution, environment variables, and terminal
//! interaction.

pub mod os_env;
pub mod os_fs;
pub mod process_executor;
pub mod rustyline_input;
pub mod std_terminal;

#[cfg(unix)]
pub mod flock_lock;
