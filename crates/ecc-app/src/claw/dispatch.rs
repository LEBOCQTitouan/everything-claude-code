//! Command dispatch — routes ClawCommand to the appropriate handler.

use super::handlers;
use super::{ClawPorts, ClawState};
use ecc_domain::claw::command::ClawCommand;

/// Dispatch a command to the appropriate handler.
/// Exit is handled by the caller (REPL loop), not here.
pub fn dispatch_command(
    cmd: &ClawCommand,
    state: &mut ClawState,
    ports: &ClawPorts<'_>,
) -> anyhow::Result<()> {
    match cmd {
        ClawCommand::Help => handlers::handle_help(state, ports),
        ClawCommand::Clear => handlers::handle_clear(state, ports),
        ClawCommand::History => handlers::handle_history(state, ports),
        ClawCommand::Sessions(target) => handlers::handle_sessions(target, state, ports),
        ClawCommand::Model(target) => handlers::handle_model(target, state, ports),
        ClawCommand::Load(name) => handlers::handle_load(name, state, ports),
        ClawCommand::Branch(name) => handlers::handle_branch(name, state, ports),
        ClawCommand::Search(keyword) => handlers::handle_search(keyword, state, ports),
        ClawCommand::Compact(keep) => handlers::handle_compact(keep, state, ports),
        ClawCommand::Export(format) => handlers::handle_export(format, state, ports),
        ClawCommand::Metrics => handlers::handle_metrics(state, ports),
        ClawCommand::UserMessage(msg) => handlers::handle_user_message(msg, state, ports),
        ClawCommand::Exit => {} // Handled by caller
    }
    Ok(())
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

    #[test]
    fn dispatch_help() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = default_state();

        let result = dispatch_command(&ClawCommand::Help, &mut state, &ports);
        assert!(result.is_ok());
    }

    #[test]
    fn dispatch_clear() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = ClawState {
            session_name: "test".to_string(),
            model: ClawModel::Sonnet,
            turns: vec![Turn {
                timestamp: "ts".to_string(),
                role: Role::User,
                content: "hi".to_string(),
            }],
            loaded_skills: Vec::new(),
        };

        dispatch_command(&ClawCommand::Clear, &mut state, &ports).unwrap();
        assert!(state.turns.is_empty());
    }

    #[test]
    fn dispatch_metrics() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = default_state();

        let result = dispatch_command(&ClawCommand::Metrics, &mut state, &ports);
        assert!(result.is_ok());
    }

    #[test]
    fn dispatch_exit_is_noop() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = default_state();

        let result = dispatch_command(&ClawCommand::Exit, &mut state, &ports);
        assert!(result.is_ok());
    }

    #[test]
    fn dispatch_model_change() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = default_state();

        dispatch_command(
            &ClawCommand::Model(Some("opus".to_string())),
            &mut state,
            &ports,
        )
        .unwrap();
        assert_eq!(state.model, ClawModel::Opus);
    }

    #[test]
    fn dispatch_history() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = default_state();

        let result = dispatch_command(&ClawCommand::History, &mut state, &ports);
        assert!(result.is_ok());
    }

    #[test]
    fn dispatch_export() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = default_state();

        let result = dispatch_command(
            &ClawCommand::Export(Some("json".to_string())),
            &mut state,
            &ports,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn dispatch_search() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = default_state();

        let result = dispatch_command(
            &ClawCommand::Search("keyword".to_string()),
            &mut state,
            &ports,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn dispatch_compact() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = default_state();

        let result = dispatch_command(&ClawCommand::Compact(Some(5)), &mut state, &ports);
        assert!(result.is_ok());
    }

    #[test]
    fn dispatch_sessions_list() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = default_state();

        let result = dispatch_command(&ClawCommand::Sessions(None), &mut state, &ports);
        assert!(result.is_ok());
    }

    #[test]
    fn dispatch_load() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = default_state();

        let result = dispatch_command(
            &ClawCommand::Load("tdd".to_string()),
            &mut state,
            &ports,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn dispatch_branch() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = default_state();

        let result = dispatch_command(
            &ClawCommand::Branch("new-branch".to_string()),
            &mut state,
            &ports,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn dispatch_user_message_empty() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = default_state();

        let result = dispatch_command(
            &ClawCommand::UserMessage(String::new()),
            &mut state,
            &ports,
        );
        assert!(result.is_ok());
        assert!(state.turns.is_empty());
    }
}
