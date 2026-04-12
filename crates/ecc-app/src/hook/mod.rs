//! Hook dispatch use case — routes hookId to the appropriate handler.

use ecc_ports::bypass_store::BypassStore;
use ecc_ports::cost_store::CostStore;
use ecc_ports::env::Environment;
use ecc_ports::fs::FileSystem;
use ecc_ports::metrics_store::MetricsStore;
use ecc_ports::shell::ShellExecutor;
use ecc_ports::terminal::TerminalIO;

pub mod bypass_interceptor;
pub mod handlers;
pub mod bypass_handling;
pub mod dispatch;

pub use dispatch::{build_handler_registry, dispatch, truncate_stdin};

/// Maximum stdin payload size (1 MB).
pub const MAX_STDIN: usize = 1_024 * 1_024;

/// Input context for a hook invocation.
#[derive(Debug)]
pub struct HookContext {
    pub hook_id: String,
    pub stdin_payload: String,
    pub profiles_csv: Option<String>,
}

/// Result of a hook execution.
#[derive(Debug)]
pub struct HookResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl HookResult {
    /// Passthrough result — echoes stdin to stdout, no stderr.
    pub fn passthrough(stdin: &str) -> Self {
        Self {
            stdout: stdin.to_string(),
            stderr: String::new(),
            exit_code: 0,
        }
    }

    /// Warning result — echoes stdin to stdout, writes warning to stderr.
    pub fn warn(stdin: &str, message: &str) -> Self {
        Self {
            stdout: stdin.to_string(),
            stderr: message.to_string(),
            exit_code: 0,
        }
    }

    /// Block result — echoes stdin to stdout, writes error to stderr, exits with code 2.
    pub fn block(stdin: &str, message: &str) -> Self {
        Self {
            stdout: stdin.to_string(),
            stderr: message.to_string(),
            exit_code: 2,
        }
    }

    /// Silent result — echoes stdin to stdout, no output.
    pub fn silent(stdin: &str) -> Self {
        Self::passthrough(stdin)
    }
}

/// Ports bundle for hook execution.
pub struct HookPorts<'a> {
    pub fs: &'a dyn FileSystem,
    pub shell: &'a dyn ShellExecutor,
    pub env: &'a dyn Environment,
    pub terminal: &'a dyn TerminalIO,
    pub cost_store: Option<&'a dyn CostStore>,
    pub bypass_store: Option<&'a dyn BypassStore>,
    pub metrics_store: Option<&'a dyn MetricsStore>,
}

impl<'a> HookPorts<'a> {
    /// Create a test-friendly HookPorts with all optional stores set to None.
    pub fn test_default(
        fs: &'a dyn FileSystem,
        shell: &'a dyn ShellExecutor,
        env: &'a dyn Environment,
        terminal: &'a dyn TerminalIO,
    ) -> Self {
        Self {
            fs,
            shell,
            env,
            terminal,
            cost_store: None,
            bypass_store: None,
            metrics_store: None,
        }
    }
}

/// Trait for hook handlers that can be registered in a dispatch table.
pub trait Handler: Send + Sync {
    /// The hook ID this handler responds to (e.g., `"stop:cartography"`).
    fn hook_id(&self) -> &str;

    /// Handle the hook invocation.
    fn handle(&self, stdin: &str, ports: &HookPorts<'_>) -> HookResult;
}

#[cfg(test)]
mod hook_tests;
