use std::path::Path;

/// Port for executing shell commands.
pub trait ShellExecutor: Send + Sync {
    /// Run a command with arguments, capturing stdout, stderr, and exit code.
    fn run_command(&self, command: &str, args: &[&str]) -> Result<CommandOutput, ShellError>;

    /// Run a command in a specific working directory, capturing output.
    fn run_command_in_dir(
        &self,
        command: &str,
        args: &[&str],
        dir: &Path,
    ) -> Result<CommandOutput, ShellError>;

    /// Return `true` if the named command exists on `PATH`.
    fn command_exists(&self, command: &str) -> bool;

    /// Spawn a command with the given string piped to its stdin, capturing output.
    fn spawn_with_stdin(
        &self,
        command: &str,
        args: &[&str],
        stdin: &str,
    ) -> Result<CommandOutput, ShellError>;
}

/// Output captured from a completed shell command.
#[derive(Debug, Clone)]
pub struct CommandOutput {
    /// Captured standard output text.
    pub stdout: String,
    /// Captured standard error text.
    pub stderr: String,
    /// Process exit code (0 = success).
    pub exit_code: i32,
}

impl CommandOutput {
    /// Return `true` if the command exited with code 0.
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }
}

/// Errors that can occur when executing shell commands.
#[derive(Debug, thiserror::Error)]
pub enum ShellError {
    /// The requested command was not found on PATH.
    #[error("command not found: {0}")]
    NotFound(String),

    /// The command ran but exited with a non-zero code.
    #[error("command failed with exit code {exit_code}: {stderr}")]
    Failed {
        /// The non-zero exit code returned by the command.
        exit_code: i32,
        /// Captured stderr from the failed command.
        stderr: String,
    },

    /// An I/O error prevented the command from running.
    #[error("I/O error running command: {0}")]
    Io(String),
}
