use ecc_ports::repl::ReplInput;
use ecc_ports::terminal::TerminalError;
use std::sync::Mutex;

/// Test double for ReplInput — feeds lines from a Vec, returns None when exhausted.
pub struct ScriptedInput {
    lines: Mutex<Vec<String>>,
}

impl ScriptedInput {
    pub fn new() -> Self {
        Self {
            lines: Mutex::new(Vec::new()),
        }
    }

    /// Builder: add a line to the input queue.
    pub fn with_line(self, line: &str) -> Self {
        self.lines.lock().unwrap().push(line.to_string());
        self
    }

    /// Builder: add multiple lines.
    pub fn with_lines(self, lines: &[&str]) -> Self {
        let mut queue = self.lines.lock().unwrap();
        for line in lines {
            queue.push((*line).to_string());
        }
        drop(queue);
        self
    }

    /// How many lines remain unread.
    pub fn remaining(&self) -> usize {
        self.lines.lock().unwrap().len()
    }
}

impl Default for ScriptedInput {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplInput for ScriptedInput {
    fn read_line(&self, _prompt: &str) -> Result<Option<String>, TerminalError> {
        let mut lines = self.lines.lock().unwrap();
        if lines.is_empty() {
            Ok(None)
        } else {
            Ok(Some(lines.remove(0)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_lines_in_order() {
        let input = ScriptedInput::new().with_line("first").with_line("second");
        assert_eq!(input.read_line("> ").unwrap(), Some("first".to_string()));
        assert_eq!(input.read_line("> ").unwrap(), Some("second".to_string()));
    }

    #[test]
    fn returns_none_when_exhausted() {
        let input = ScriptedInput::new().with_line("only");
        assert_eq!(input.read_line("> ").unwrap(), Some("only".to_string()));
        assert_eq!(input.read_line("> ").unwrap(), None);
    }

    #[test]
    fn empty_input_returns_none_immediately() {
        let input = ScriptedInput::new();
        assert_eq!(input.read_line("> ").unwrap(), None);
    }

    #[test]
    fn with_lines_adds_multiple() {
        let input = ScriptedInput::new().with_lines(&["a", "b", "c"]);
        assert_eq!(input.remaining(), 3);
        assert_eq!(input.read_line("> ").unwrap(), Some("a".to_string()));
        assert_eq!(input.remaining(), 2);
    }

    #[test]
    fn remaining_decreases() {
        let input = ScriptedInput::new().with_line("x").with_line("y");
        assert_eq!(input.remaining(), 2);
        let _ = input.read_line("> ");
        assert_eq!(input.remaining(), 1);
    }

    #[test]
    fn default_is_empty() {
        let input = ScriptedInput::default();
        assert_eq!(input.remaining(), 0);
    }

    #[test]
    fn handles_empty_string_line() {
        let input = ScriptedInput::new().with_line("");
        assert_eq!(input.read_line("> ").unwrap(), Some(String::new()));
    }

    #[test]
    fn handles_multiline_content() {
        let input = ScriptedInput::new().with_line("line1\nline2");
        assert_eq!(
            input.read_line("> ").unwrap(),
            Some("line1\nline2".to_string())
        );
    }
}
