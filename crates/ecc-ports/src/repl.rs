use crate::terminal::TerminalError;

/// Port for REPL line input (abstracts rustyline or scripted input).
pub trait ReplInput: Send + Sync {
    /// Read a line with the given prompt.
    /// Returns `Ok(Some(line))` for input, `Ok(None)` for EOF/Ctrl-D, `Err` for errors.
    fn read_line(&self, prompt: &str) -> Result<Option<String>, TerminalError>;
}
