//! Install orchestrator — full ECC installation flow.
//!
//! Ports `install-orchestrator.ts`.

pub mod error;
mod global;
mod helpers;
mod init;
mod resolve;

pub use global::install_global;
pub use init::init_project;
pub use resolve::resolve_ecc_root;

use ecc_ports::env::Environment;
use ecc_ports::fs::FileSystem;
use ecc_ports::shell::ShellExecutor;
use ecc_ports::terminal::TerminalIO;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Options for the install command.
#[derive(Debug, Clone)]
pub struct InstallOptions {
    pub dry_run: bool,
    pub force: bool,
    pub no_gitignore: bool,
    pub interactive: bool,
    pub clean: bool,
    pub clean_all: bool,
    pub languages: Vec<String>,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            dry_run: false,
            force: false,
            no_gitignore: false,
            interactive: true,
            clean: false,
            clean_all: false,
            languages: vec![],
        }
    }
}

/// Default install options — interactive, no flags.
pub fn default_install_options() -> InstallOptions {
    InstallOptions::default()
}

/// Context bundling all ports for install operations.
pub struct InstallContext<'a> {
    pub fs: &'a dyn FileSystem,
    pub shell: &'a dyn ShellExecutor,
    pub env: &'a dyn Environment,
    pub terminal: &'a dyn TerminalIO,
}

/// Summary of an install operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallSummary {
    pub added: usize,
    pub updated: usize,
    pub unchanged: usize,
    pub skipped: usize,
    pub smart_merged: usize,
    pub errors: Vec<String>,
    pub success: bool,
}
