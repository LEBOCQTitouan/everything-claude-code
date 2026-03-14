use ecc_ports::shell::{CommandOutput, ShellError, ShellExecutor};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

/// Scripted mock shell executor for testing.
pub struct MockExecutor {
    responses: Mutex<HashMap<String, CommandOutput>>,
    known_commands: Mutex<Vec<String>>,
}

impl MockExecutor {
    pub fn new() -> Self {
        Self {
            responses: Mutex::new(HashMap::new()),
            known_commands: Mutex::new(Vec::new()),
        }
    }

    /// Register a scripted response for a command.
    pub fn on(self, command: &str, output: CommandOutput) -> Self {
        self.responses
            .lock()
            .unwrap()
            .insert(command.to_string(), output);
        self
    }

    /// Register a command as "existing" (for `command_exists`).
    pub fn with_command(self, name: &str) -> Self {
        self.known_commands
            .lock()
            .unwrap()
            .push(name.to_string());
        self
    }

    fn lookup(&self, command: &str) -> Result<CommandOutput, ShellError> {
        self.responses
            .lock()
            .unwrap()
            .get(command)
            .cloned()
            .ok_or_else(|| ShellError::NotFound(command.to_string()))
    }
}

impl Default for MockExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl ShellExecutor for MockExecutor {
    fn run_command(&self, command: &str, _args: &[&str]) -> Result<CommandOutput, ShellError> {
        self.lookup(command)
    }

    fn run_command_in_dir(
        &self,
        command: &str,
        _args: &[&str],
        _dir: &Path,
    ) -> Result<CommandOutput, ShellError> {
        self.lookup(command)
    }

    fn command_exists(&self, command: &str) -> bool {
        self.known_commands
            .lock()
            .unwrap()
            .contains(&command.to_string())
    }

    fn spawn_with_stdin(
        &self,
        command: &str,
        _args: &[&str],
        _stdin: &str,
    ) -> Result<CommandOutput, ShellError> {
        self.lookup(command)
    }
}
