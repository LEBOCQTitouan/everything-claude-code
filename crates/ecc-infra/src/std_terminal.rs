use ecc_ports::terminal::{TerminalError, TerminalIO};

/// Production terminal adapter using stdout/stderr and crossterm.
///
/// # Pattern
///
/// Adapter \[Hexagonal Architecture\] — implements `ecc_ports::terminal::TerminalIO`
pub struct StdTerminal;

impl TerminalIO for StdTerminal {
    fn stdout_write(&self, text: &str) {
        use std::io::Write;
        let _ = std::io::stdout().write_all(text.as_bytes());
        let _ = std::io::stdout().flush();
    }

    fn stderr_write(&self, text: &str) {
        use std::io::Write;
        let _ = std::io::stderr().write_all(text.as_bytes());
        let _ = std::io::stderr().flush();
    }

    fn prompt(&self, message: &str) -> Result<String, TerminalError> {
        self.stderr_write(message);
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| TerminalError::Io(e.to_string()))?;
        Ok(input.trim_end().to_string())
    }

    fn is_tty(&self) -> bool {
        crossterm::tty::IsTty::is_tty(&std::io::stderr())
    }

    fn terminal_width(&self) -> Option<u16> {
        crossterm::terminal::size().ok().map(|(w, _)| w)
    }
}
