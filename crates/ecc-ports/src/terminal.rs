/// Port for terminal I/O (prompts, colored output, TTY detection).
///
/// # Pattern
///
/// Port \[Hexagonal Architecture\]
pub trait TerminalIO: Send + Sync {
    /// Write text to stdout (no newline appended).
    fn stdout_write(&self, text: &str);
    /// Write text to stderr (no newline appended).
    fn stderr_write(&self, text: &str);
    /// Display a prompt and read one line of user input.
    fn prompt(&self, message: &str) -> Result<String, TerminalError>;
    /// Return `true` if stdout is connected to a TTY.
    fn is_tty(&self) -> bool;
    /// Return the terminal width in columns, or `None` if unknown.
    fn terminal_width(&self) -> Option<u16>;
}

/// Errors that can occur during terminal I/O operations.
#[derive(Debug, thiserror::Error)]
pub enum TerminalError {
    /// The user cancelled input (e.g., Ctrl-D or Ctrl-C).
    #[error("input cancelled by user")]
    Cancelled,

    /// An I/O error occurred while reading from or writing to the terminal.
    #[error("I/O error: {0}")]
    Io(String),
}
