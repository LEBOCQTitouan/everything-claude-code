use ecc_ports::shell::{CommandOutput, ShellError, ShellExecutor};
use std::path::Path;
use std::process::Command;

/// Production shell executor backed by `std::process::Command`.
///
/// # Pattern
///
/// Adapter \[Hexagonal Architecture\] — implements `ecc_ports::shell::ShellExecutor`
pub struct ProcessExecutor;

impl ShellExecutor for ProcessExecutor {
    fn run_command(&self, command: &str, args: &[&str]) -> Result<CommandOutput, ShellError> {
        let output = Command::new(command).args(args).output().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ShellError::NotFound(command.to_string())
            } else {
                ShellError::Io(e.to_string())
            }
        })?;

        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            exit_code: output.status.code().unwrap_or(-1),
        })
    }

    fn run_command_in_dir(
        &self,
        command: &str,
        args: &[&str],
        dir: &Path,
    ) -> Result<CommandOutput, ShellError> {
        let output = Command::new(command)
            .args(args)
            .current_dir(dir)
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    ShellError::NotFound(command.to_string())
                } else {
                    ShellError::Io(e.to_string())
                }
            })?;

        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            exit_code: output.status.code().unwrap_or(-1),
        })
    }

    fn command_exists(&self, command: &str) -> bool {
        Command::new("which")
            .arg(command)
            .output()
            .is_ok_and(|o| o.status.success())
    }

    fn spawn_with_stdin(
        &self,
        command: &str,
        args: &[&str],
        stdin_data: &str,
    ) -> Result<CommandOutput, ShellError> {
        use std::io::Write;
        use std::process::Stdio;

        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    ShellError::NotFound(command.to_string())
                } else {
                    ShellError::Io(e.to_string())
                }
            })?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(stdin_data.as_bytes())
                .map_err(|e| ShellError::Io(e.to_string()))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| ShellError::Io(e.to_string()))?;

        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            exit_code: output.status.code().unwrap_or(-1),
        })
    }
}
