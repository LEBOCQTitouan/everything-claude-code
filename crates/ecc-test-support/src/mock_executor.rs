use ecc_ports::shell::{CommandOutput, ShellError, ShellExecutor};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

/// Scripted mock shell executor for testing.
///
/// Supports two lookup modes:
/// 1. Command-only: `on("git", output)` — matches any invocation of "git"
/// 2. Command+args: `on_args("git", &["rev-parse", "--git-dir"], output)` — matches exact args
///
/// Command+args matches are checked first; command-only is the fallback.
pub struct MockExecutor {
    responses: Mutex<HashMap<String, CommandOutput>>,
    args_responses: Mutex<HashMap<String, CommandOutput>>,
    known_commands: Mutex<Vec<String>>,
}

impl MockExecutor {
    pub fn new() -> Self {
        Self {
            responses: Mutex::new(HashMap::new()),
            args_responses: Mutex::new(HashMap::new()),
            known_commands: Mutex::new(Vec::new()),
        }
    }

    /// Register a scripted response for a command (matches any args).
    pub fn on(self, command: &str, output: CommandOutput) -> Self {
        self.responses
            .lock()
            .unwrap()
            .insert(command.to_string(), output);
        self
    }

    /// Register a scripted response for a specific command + args combination.
    pub fn on_args(self, command: &str, args: &[&str], output: CommandOutput) -> Self {
        let key = Self::args_key(command, args);
        self.args_responses
            .lock()
            .unwrap()
            .insert(key, output);
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

    fn args_key(command: &str, args: &[&str]) -> String {
        format!("{}\0{}", command, args.join("\0"))
    }

    fn lookup(&self, command: &str, args: &[&str]) -> Result<CommandOutput, ShellError> {
        // Try command+args first
        let key = Self::args_key(command, args);
        if let Some(output) = self.args_responses.lock().unwrap().get(&key) {
            return Ok(output.clone());
        }
        // Fall back to command-only
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
    fn run_command(&self, command: &str, args: &[&str]) -> Result<CommandOutput, ShellError> {
        self.lookup(command, args)
    }

    fn run_command_in_dir(
        &self,
        command: &str,
        args: &[&str],
        _dir: &Path,
    ) -> Result<CommandOutput, ShellError> {
        self.lookup(command, args)
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
        args: &[&str],
        _stdin: &str,
    ) -> Result<CommandOutput, ShellError> {
        self.lookup(command, args)
    }
}
