use std::path::Path;

/// Port for executing shell commands.
pub trait ShellExecutor: Send + Sync {
    fn run_command(&self, command: &str, args: &[&str]) -> Result<CommandOutput, ShellError>;

    fn run_command_in_dir(
        &self,
        command: &str,
        args: &[&str],
        dir: &Path,
    ) -> Result<CommandOutput, ShellError>;

    fn command_exists(&self, command: &str) -> bool;

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
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl CommandOutput {
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }
}

/// Errors that can occur when executing shell commands.
#[derive(Debug, thiserror::Error)]
pub enum ShellError {
    #[error("command not found: {0}")]
    NotFound(String),

    #[error("command failed with exit code {exit_code}: {stderr}")]
    Failed { exit_code: i32, stderr: String },

    #[error("I/O error running command: {0}")]
    Io(String),
}
