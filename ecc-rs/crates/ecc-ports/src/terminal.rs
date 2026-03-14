/// Port for terminal I/O (prompts, colored output, TTY detection).
pub trait TerminalIO: Send + Sync {
    fn stdout_write(&self, text: &str);
    fn stderr_write(&self, text: &str);
    fn prompt(&self, message: &str) -> Result<String, TerminalError>;
    fn is_tty(&self) -> bool;
    fn terminal_width(&self) -> Option<u16>;
}

#[derive(Debug, thiserror::Error)]
pub enum TerminalError {
    #[error("input cancelled by user")]
    Cancelled,

    #[error("I/O error: {0}")]
    Io(String),
}
