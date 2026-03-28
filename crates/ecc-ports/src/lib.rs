//! Port trait definitions for Everything Claude Code.
//!
//! Defines the abstract boundaries ([`fs::FileSystem`], [`shell::ShellExecutor`],
//! [`env::Environment`], [`terminal::TerminalIO`]) that decouple business logic
//! from infrastructure. Production adapters live in [`ecc_infra`]; test doubles
//! live in [`ecc_test_support`].

#![warn(missing_docs)]

/// Environment variable and platform access port.
pub mod env;
/// Filesystem operations port.
pub mod fs;
/// File-based locking port.
pub mod lock;
/// REPL line-input port.
pub mod repl;
/// Shell command execution port.
pub mod shell;
/// Terminal I/O port.
pub mod terminal;
