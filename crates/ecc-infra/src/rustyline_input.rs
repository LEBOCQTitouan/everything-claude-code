//! ReplInput adapter using rustyline for interactive line editing.

use ecc_ports::repl::ReplInput;
use ecc_ports::terminal::TerminalError;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::path::PathBuf;
use std::sync::Mutex;

/// Production ReplInput backed by rustyline with persistent history.
pub struct RustylineInput {
    editor: Mutex<DefaultEditor>,
    history_path: Option<PathBuf>,
}

impl RustylineInput {
    /// Create a new RustylineInput with optional history file path.
    pub fn new(history_path: Option<PathBuf>) -> Result<Self, String> {
        let mut editor =
            DefaultEditor::new().map_err(|e| format!("Failed to create editor: {e}"))?;

        if let Some(ref path) = history_path {
            // Ensure parent dir exists
            if let Some(parent) = path.parent() {
                if let Err(err) = std::fs::create_dir_all(parent) {
                    log::warn!("create history dir failed: {err}");
                }
            }
            if let Err(err) = editor.load_history(path) {
                log::warn!("load history failed: {err}");
            }
        }

        Ok(Self {
            editor: Mutex::new(editor),
            history_path,
        })
    }
}

impl Drop for RustylineInput {
    fn drop(&mut self) {
        if let Some(ref path) = self.history_path
            && let Ok(mut editor) = self.editor.lock()
        {
            if let Err(err) = editor.save_history(path) {
                log::warn!("save history failed: {err}");
            }
        }
    }
}

impl ReplInput for RustylineInput {
    fn read_line(&self, prompt: &str) -> Result<Option<String>, TerminalError> {
        let mut editor = self
            .editor
            .lock()
            .map_err(|e| TerminalError::Io(format!("Lock poisoned: {e}")))?;

        match editor.readline(prompt) {
            Ok(line) => {
                if let Err(err) = editor.add_history_entry(&line) {
                    log::warn!("add history entry failed: {err}");
                }
                Ok(Some(line))
            }
            Err(ReadlineError::Interrupted) => Ok(None),
            Err(ReadlineError::Eof) => Ok(None),
            Err(e) => Err(TerminalError::Io(format!("Readline error: {e}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_without_history() {
        let input = RustylineInput::new(None);
        assert!(input.is_ok());
    }

    #[test]
    fn create_with_temp_history() {
        let path = std::env::temp_dir().join("ecc-test-history");
        let input = RustylineInput::new(Some(path.clone()));
        assert!(input.is_ok());
        let _ = std::fs::remove_file(&path);
    }
}
