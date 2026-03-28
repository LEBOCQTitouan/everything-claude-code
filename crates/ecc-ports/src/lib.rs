//! Port trait definitions for Everything Claude Code.
//!
//! Defines the abstract boundaries ([`fs::FileSystem`], [`shell::ShellExecutor`],
//! [`env::Environment`], [`terminal::TerminalIO`]) that decouple business logic
//! from infrastructure. Production adapters live in [`ecc_infra`]; test doubles
//! live in [`ecc_test_support`].

#![warn(missing_docs)]

pub mod env;
pub mod fs;
pub mod lock;
pub mod repl;
pub mod shell;
pub mod terminal;
