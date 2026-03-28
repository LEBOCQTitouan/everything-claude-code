//! Claude subprocess runner — spawn `claude -p` via ShellExecutor port.

use super::ClawPorts;
use super::error::ClawError;
use ecc_domain::claw::model::ClawModel;
use ecc_ports::shell::{CommandOutput, ShellExecutor};

/// Run a prompt through `claude -p` and return the response.
pub fn run_claude(
    prompt: &str,
    model: ClawModel,
    ports: &ClawPorts<'_>,
) -> Result<String, ClawError> {
    let model_flag = model.to_flag();
    let args: &[&str] = &["-p", "--model", model_flag];

    let output: CommandOutput = ports
        .shell
        .spawn_with_stdin("claude", args, prompt)
        .map_err(|e| ClawError::ClaudeSubprocess {
            message: format!("Failed to run claude: {e}"),
        })?;

    if output.exit_code != 0 {
        let message = if output.stderr.is_empty() {
            format!("claude exited with code {}", output.exit_code)
        } else {
            format!(
                "claude exited with code {}: {}",
                output.exit_code,
                output.stderr.trim()
            )
        };
        return Err(ClawError::ClaudeSubprocess { message });
    }

    Ok(output.stdout)
}

/// Check if the `claude` CLI is available on PATH.
pub fn is_claude_available(shell: &dyn ShellExecutor) -> bool {
    shell.command_exists("claude")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_ports::shell::CommandOutput;
    use ecc_test_support::{
        BufferedTerminal, InMemoryFileSystem, MockEnvironment, MockExecutor, ScriptedInput,
    };

    fn make_ports<'a>(
        fs: &'a InMemoryFileSystem,
        shell: &'a MockExecutor,
        env: &'a MockEnvironment,
        term: &'a BufferedTerminal,
        input: &'a ScriptedInput,
    ) -> ClawPorts<'a> {
        ClawPorts {
            fs,
            shell,
            env,
            terminal: term,
            repl_input: input,
        }
    }

    #[test]
    fn run_claude_success() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new().on(
            "claude",
            CommandOutput {
                stdout: "Hello! I'm Claude.".to_string(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);

        let result = run_claude("hi", ClawModel::Sonnet, &ports);
        assert_eq!(result.unwrap(), "Hello! I'm Claude.");
    }

    #[test]
    fn run_claude_nonzero_exit() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new().on(
            "claude",
            CommandOutput {
                stdout: String::new(),
                stderr: "rate limited".to_string(),
                exit_code: 1,
            },
        );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);

        let result = run_claude("hi", ClawModel::Sonnet, &ports);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("rate limited"));
    }

    #[test]
    fn run_claude_nonzero_no_stderr() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new().on(
            "claude",
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 1,
            },
        );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);

        let result = run_claude("hi", ClawModel::Sonnet, &ports);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("exited with code 1")
        );
    }

    #[test]
    fn run_claude_command_not_found() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new(); // no response registered
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);

        let result = run_claude("hi", ClawModel::Sonnet, &ports);
        assert!(result.is_err());
    }

    #[test]
    fn is_claude_available_true() {
        let shell = MockExecutor::new().with_command("claude");
        assert!(is_claude_available(&shell));
    }

    #[test]
    fn is_claude_available_false() {
        let shell = MockExecutor::new();
        assert!(!is_claude_available(&shell));
    }
}
