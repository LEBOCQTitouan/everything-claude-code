//! Read-only command handlers for the Claw REPL.

use super::super::{ClawPorts, ClawState};
use ecc_domain::claw::export::{export_turns, ExportFormat};
use ecc_domain::claw::metrics::{compute_metrics, format_metrics};
use ecc_domain::claw::search::{format_search_results, search_turns};

/// Show help text.
pub fn handle_help(_state: &ClawState, ports: &ClawPorts<'_>) {
    let help = "\
Commands:
  /help, /h          Show this help
  /clear             Clear session history
  /history, /hist    Show conversation history
  /sessions [name]   List sessions or switch to one
  /model [name]      Show or change model (sonnet/opus/haiku)
  /load <skill>      Load a skill by name
  /branch <name>     Branch session into a new name
  /search <keyword>  Search history
  /compact [n]       Compact to last N turns (default: 10)
  /export [format]   Export session (md/json/text)
  /metrics, /stats   Show session metrics
  exit, quit, /q     Exit the REPL";

    ports.terminal.stdout_write(help);
    ports.terminal.stdout_write("\n");
}

/// Show conversation history.
pub fn handle_history(state: &ClawState, ports: &ClawPorts<'_>) {
    if state.turns.is_empty() {
        ports.terminal.stdout_write("No history.\n");
        return;
    }

    let md = ecc_domain::claw::turn::format_turns(&state.turns);
    ports.terminal.stdout_write(&md);
    ports.terminal.stdout_write("\n");
}

/// Search history.
pub fn handle_search(keyword: &str, state: &ClawState, ports: &ClawPorts<'_>) {
    let indices = search_turns(&state.turns, keyword);
    let output = format_search_results(&state.turns, &indices);
    ports.terminal.stdout_write(&output);
    ports.terminal.stdout_write("\n");
}

/// Export session.
pub fn handle_export(
    format_arg: &Option<String>,
    state: &ClawState,
    ports: &ClawPorts<'_>,
) {
    let format = match format_arg {
        Some(f) => match ExportFormat::parse(f) {
            Some(fmt) => fmt,
            None => {
                ports.terminal.stderr_write(&format!(
                    "Unknown format: '{f}'. Use md, json, or text.\n"
                ));
                return;
            }
        },
        None => ExportFormat::Markdown, // default
    };

    let json_serializer = |turns: &[ecc_domain::claw::turn::Turn]| {
        serde_json::to_string_pretty(turns).unwrap_or_else(|_| "[]".to_string())
    };
    let output = export_turns(&state.session_name, &state.turns, format, json_serializer);
    ports.terminal.stdout_write(&output);
    ports.terminal.stdout_write("\n");
}

/// Show session metrics.
pub fn handle_metrics(state: &ClawState, ports: &ClawPorts<'_>) {
    let metrics = compute_metrics(&state.turns);
    let output = format_metrics(&metrics);
    ports.terminal.stdout_write(&output);
    ports.terminal.stdout_write("\n");
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_domain::claw::model::ClawModel;
    use ecc_domain::claw::turn::{Role, Turn};
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

    fn default_state() -> ClawState {
        ClawState {
            session_name: "test".to_string(),
            model: ClawModel::Sonnet,
            turns: Vec::new(),
            loaded_skills: Vec::new(),
        }
    }

    fn state_with_turns() -> ClawState {
        ClawState {
            session_name: "test".to_string(),
            model: ClawModel::Sonnet,
            turns: vec![
                Turn {
                    timestamp: "ts1".to_string(),
                    role: Role::User,
                    content: "hello".to_string(),
                },
                Turn {
                    timestamp: "ts2".to_string(),
                    role: Role::Assistant,
                    content: "hi there".to_string(),
                },
            ],
            loaded_skills: Vec::new(),
        }
    }

    // --- handle_help ---

    #[test]
    fn help_shows_commands() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let state = default_state();

        handle_help(&state, &ports);
        let out = term.stdout_output();
        assert!(out.iter().any(|s| s.contains("/help")));
        assert!(out.iter().any(|s| s.contains("/export")));
    }

    // --- handle_history ---

    #[test]
    fn history_empty() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let state = default_state();

        handle_history(&state, &ports);
        let out = term.stdout_output();
        assert!(out.iter().any(|s| s.contains("No history")));
    }

    #[test]
    fn history_shows_turns() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let state = state_with_turns();

        handle_history(&state, &ports);
        let out = term.stdout_output();
        assert!(out.iter().any(|s| s.contains("hello")));
    }

    // --- handle_search ---

    #[test]
    fn search_finds_matches() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let state = state_with_turns();

        handle_search("hello", &state, &ports);
        let out = term.stdout_output();
        assert!(out.iter().any(|s| s.contains("1 match")));
    }

    #[test]
    fn search_no_matches() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let state = state_with_turns();

        handle_search("xyz", &state, &ports);
        let out = term.stdout_output();
        assert!(out.iter().any(|s| s.contains("No matches")));
    }

    // --- handle_export ---

    #[test]
    fn export_markdown_default() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let state = state_with_turns();

        handle_export(&None, &state, &ports);
        let out = term.stdout_output();
        assert!(out.iter().any(|s| s.contains("# Session:")));
    }

    #[test]
    fn export_json() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let state = state_with_turns();

        handle_export(&Some("json".to_string()), &state, &ports);
        let out = term.stdout_output();
        assert!(out.iter().any(|s| s.contains("\"role\"")));
    }

    #[test]
    fn export_unknown_format() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let state = state_with_turns();

        handle_export(&Some("xml".to_string()), &state, &ports);
        let err = term.stderr_output();
        assert!(err.iter().any(|s| s.contains("Unknown format")));
    }

    // --- handle_metrics ---

    #[test]
    fn metrics_shows_counts() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let state = state_with_turns();

        handle_metrics(&state, &ports);
        let out = term.stdout_output();
        assert!(out.iter().any(|s| s.contains("Turns: 2")));
    }
}
