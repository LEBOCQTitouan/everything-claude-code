//! Port trait definitions for Everything Claude Code.
//!
//! Defines the abstract boundaries ([`fs::FileSystem`], [`shell::ShellExecutor`],
//! [`env::Environment`], [`terminal::TerminalIO`]) that decouple business logic
//! from infrastructure. Production adapters live in [`ecc_infra`]; test doubles
//! live in [`ecc_test_support`].

#![warn(missing_docs)]

/// Backlog entry, lock, and index store ports.
pub mod backlog;
pub mod bypass_store;
/// Audit result caching port.
pub mod cache_store;
/// Clock (time source) port.
pub mod clock;
/// Persistent ECC configuration port.
pub mod config_store;
/// Cost and token tracking store port.
pub mod cost_store;
/// Environment variable and platform access port.
pub mod env;
/// Tarball extraction port.
pub mod extract;
/// Filesystem operations port.
pub mod fs;
/// Git repository information port.
pub mod git;
/// Git log data access port.
pub mod git_log;
/// File-based locking port.
pub mod lock;
/// Structured log storage port.
pub mod log_store;
/// Three-tier memory store port.
pub mod memory_store;
/// Harness reliability metrics store port.
pub mod metrics_store;
/// Release artifact discovery and download port.
pub mod release;
/// REPL line-input port.
pub mod repl;
/// Shell command execution port.
pub mod shell;
/// Terminal I/O port.
pub mod terminal;
/// Worktree management port.
pub mod worktree;
