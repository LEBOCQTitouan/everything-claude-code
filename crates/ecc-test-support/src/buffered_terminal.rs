use ecc_ports::terminal::{TerminalError, TerminalIO};
use std::sync::Mutex;

/// Buffered terminal for testing — captures all output, provides scripted input.
pub struct BufferedTerminal {
    stdout: Mutex<Vec<String>>,
    stderr: Mutex<Vec<String>>,
    inputs: Mutex<Vec<String>>,
    is_tty: bool,
    width: Option<u16>,
}

impl BufferedTerminal {
    pub fn new() -> Self {
        Self {
            stdout: Mutex::new(Vec::new()),
            stderr: Mutex::new(Vec::new()),
            inputs: Mutex::new(Vec::new()),
            is_tty: true,
            width: Some(80),
        }
    }

    pub fn with_input(self, input: &str) -> Self {
        self.inputs.lock().unwrap().push(input.to_string());
        self
    }

    pub fn with_tty(mut self, is_tty: bool) -> Self {
        self.is_tty = is_tty;
        self
    }

    pub fn with_width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    pub fn stdout_output(&self) -> Vec<String> {
        self.stdout.lock().unwrap().clone()
    }

    pub fn stderr_output(&self) -> Vec<String> {
        self.stderr.lock().unwrap().clone()
    }
}

impl Default for BufferedTerminal {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalIO for BufferedTerminal {
    fn stdout_write(&self, text: &str) {
        self.stdout.lock().unwrap().push(text.to_string());
    }

    fn stderr_write(&self, text: &str) {
        self.stderr.lock().unwrap().push(text.to_string());
    }

    fn prompt(&self, _message: &str) -> Result<String, TerminalError> {
        let mut inputs = self.inputs.lock().unwrap();
        if inputs.is_empty() {
            Err(TerminalError::Cancelled)
        } else {
            Ok(inputs.remove(0))
        }
    }

    fn is_tty(&self) -> bool {
        self.is_tty
    }

    fn terminal_width(&self) -> Option<u16> {
        self.width
    }
}
