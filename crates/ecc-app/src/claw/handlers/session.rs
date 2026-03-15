//! State-mutating command handlers for the Claw REPL.

use super::super::{ClawPorts, ClawState};
use ecc_domain::claw::compact::{compact_turns, compaction_summary};
use ecc_domain::claw::model::ClawModel;
use ecc_domain::claw::prompt::{assemble_prompt, build_system_context};
use ecc_domain::claw::session_name::is_valid_session_name;
use ecc_domain::claw::turn::{Role, Turn};
use ecc_domain::time::{datetime_from_epoch, format_datetime};
use std::time::{SystemTime, UNIX_EPOCH};

fn now_datetime_string() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format_datetime(&datetime_from_epoch(secs))
}

/// Clear the current session history.
pub fn handle_clear(state: &mut ClawState, ports: &ClawPorts<'_>) {
    let count = state.clear_turns();

    if let Some(home) = ports.env.home_dir() {
        let _ = super::super::storage::clear_session(&home, state.session_name(), ports.fs);
    }

    ports.terminal.stderr_write(&format!("Cleared {count} turns.\n"));
}

/// List or switch sessions.
pub fn handle_sessions(
    target: &Option<String>,
    state: &mut ClawState,
    ports: &ClawPorts<'_>,
) {
    let home = match ports.env.home_dir() {
        Some(h) => h,
        None => {
            ports.terminal.stderr_write("Cannot determine home directory.\n");
            return;
        }
    };

    match target {
        None => {
            // List sessions
            let sessions = super::super::storage::list_sessions(&home, ports.fs);
            if sessions.is_empty() {
                ports.terminal.stdout_write("No sessions found.\n");
            } else {
                let mut output = String::from("Sessions:\n");
                for name in &sessions {
                    let marker = if *name == state.session_name() {
                        " (active)"
                    } else {
                        ""
                    };
                    output.push_str(&format!("  {name}{marker}\n"));
                }
                ports.terminal.stdout_write(&output);
            }
        }
        Some(name) => {
            if !is_valid_session_name(name) {
                ports
                    .terminal
                    .stderr_write(&format!("Invalid session name: '{name}'\n"));
                return;
            }
            // Save current session first
            if !state.turns().is_empty() {
                let _ = super::super::storage::save_session(
                    &home,
                    state.session_name(),
                    state.turns(),
                    ports.fs,
                );
            }
            // Switch to new session
            state.set_turns(super::super::storage::load_session(&home, name, ports.fs));
            state.set_session_name(name.clone());
            ports
                .terminal
                .stderr_write(&format!("Switched to session: {name}\n"));
        }
    }
}

/// Show or change model.
pub fn handle_model(target: &Option<String>, state: &mut ClawState, ports: &ClawPorts<'_>) {
    match target {
        None => {
            ports.terminal.stdout_write(&format!(
                "Current model: {}\n",
                state.model().display_name()
            ));
        }
        Some(name) => match ClawModel::parse(name) {
            Some(model) => {
                state.set_model(model);
                ports.terminal.stderr_write(&format!(
                    "Model changed to: {}\n",
                    model.display_name()
                ));
            }
            None => {
                ports.terminal.stderr_write(&format!(
                    "Unknown model: '{name}'. Use sonnet, opus, or haiku.\n"
                ));
            }
        },
    }
}

/// Load a skill.
pub fn handle_load(skill_name: &str, state: &mut ClawState, ports: &ClawPorts<'_>) {
    match super::super::skill_loader::load_skill(skill_name, ports) {
        Ok(content) => {
            state.load_skill(content);
            ports
                .terminal
                .stderr_write(&format!("Loaded skill: {skill_name}\n"));
        }
        Err(e) => {
            ports.terminal.stderr_write(&format!("Error: {e}\n"));
        }
    }
}

/// Branch current session.
pub fn handle_branch(target_name: &str, state: &mut ClawState, ports: &ClawPorts<'_>) {
    if !is_valid_session_name(target_name) {
        ports
            .terminal
            .stderr_write(&format!("Invalid session name: '{target_name}'\n"));
        return;
    }

    let home = match ports.env.home_dir() {
        Some(h) => h,
        None => {
            ports.terminal.stderr_write("Cannot determine home directory.\n");
            return;
        }
    };

    match super::super::storage::branch_session(
        &home,
        state.session_name(),
        target_name,
        state.turns(),
        ports.fs,
    ) {
        Ok(()) => {
            let old_name = state.session_name().to_string();
            state.set_session_name(target_name.to_string());
            ports.terminal.stderr_write(&format!(
                "Branched '{old_name}' → '{target_name}'\n"
            ));
        }
        Err(e) => {
            ports.terminal.stderr_write(&format!("Error: {e}\n"));
        }
    }
}

/// Compact history.
pub fn handle_compact(keep: &Option<usize>, state: &mut ClawState, ports: &ClawPorts<'_>) {
    let original_count = state.turns().len();
    let compacted = compact_turns(state.turns(), *keep);
    let kept_count = compacted.len();
    state.set_turns(compacted);

    let msg = compaction_summary(original_count, kept_count);
    ports.terminal.stderr_write(&format!("{msg}\n"));
}

/// Handle a user message by running it through Claude.
pub fn handle_user_message(
    message: &str,
    state: &mut ClawState,
    ports: &ClawPorts<'_>,
) {
    if message.is_empty() {
        return;
    }

    // Build prompt with existing history
    let system_ctx = build_system_context(state.loaded_skills());
    let prompt = assemble_prompt(system_ctx.as_deref(), state.turns(), message);

    // Run claude — only add turns after success to avoid orphan user turns
    match super::super::claude_runner::run_claude(&prompt, state.model(), ports) {
        Ok(response) => {
            let user_turn = Turn {
                timestamp: now_datetime_string(),
                role: Role::User,
                content: message.to_string(),
            };
            let assistant_turn = Turn {
                timestamp: now_datetime_string(),
                role: Role::Assistant,
                content: response.clone(),
            };
            state.add_turn(user_turn);
            state.add_turn(assistant_turn);
            ports.terminal.stdout_write(&response);
            ports.terminal.stdout_write("\n");
        }
        Err(e) => {
            ports
                .terminal
                .stderr_write(&format!("Error from Claude: {e}\n"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_ports::shell::CommandOutput;
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
        ClawState::new(&super::super::super::ClawConfig {
            session_name: "test".to_string(),
            model: ClawModel::Sonnet,
            initial_skills: Vec::new(),
        })
    }

    fn state_with_turns() -> ClawState {
        let mut state = default_state();
        state.add_turn(Turn {
            timestamp: "ts1".to_string(),
            role: Role::User,
            content: "hello".to_string(),
        });
        state.add_turn(Turn {
            timestamp: "ts2".to_string(),
            role: Role::Assistant,
            content: "hi there".to_string(),
        });
        state
    }

    // --- handle_clear ---

    #[test]
    fn clear_empties_turns() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = state_with_turns();

        handle_clear(&mut state, &ports);
        assert!(state.turns().is_empty());
    }

    #[test]
    fn clear_reports_count() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = state_with_turns();

        handle_clear(&mut state, &ports);
        let err = term.stderr_output();
        assert!(err.iter().any(|s| s.contains("2 turns")));
    }

    // --- handle_sessions ---

    #[test]
    fn sessions_list_empty() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = default_state();

        handle_sessions(&None, &mut state, &ports);
        let out = term.stdout_output();
        assert!(out.iter().any(|s| s.contains("No sessions")));
    }

    #[test]
    fn sessions_switch_valid() {
        let fs = InMemoryFileSystem::new().with_file(
            "/home/test/.claude/claw/sessions/other.md",
            "### [ts1] User\nhi\n---",
        );
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = default_state();

        handle_sessions(&Some("other".to_string()), &mut state, &ports);
        assert_eq!(state.session_name(),"other");
        assert_eq!(state.turns().len(),1);
    }

    #[test]
    fn sessions_switch_invalid_name() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = default_state();

        handle_sessions(&Some("bad name!".to_string()), &mut state, &ports);
        let err = term.stderr_output();
        assert!(err.iter().any(|s| s.contains("Invalid")));
    }

    // --- handle_model ---

    #[test]
    fn model_show_current() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = default_state();

        handle_model(&None, &mut state, &ports);
        let out = term.stdout_output();
        assert!(out.iter().any(|s| s.contains("Sonnet")));
    }

    #[test]
    fn model_switch_valid() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = default_state();

        handle_model(&Some("opus".to_string()), &mut state, &ports);
        assert_eq!(state.model(), ClawModel::Opus);
    }

    #[test]
    fn model_switch_invalid() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = default_state();

        handle_model(&Some("gpt4".to_string()), &mut state, &ports);
        let err = term.stderr_output();
        assert!(err.iter().any(|s| s.contains("Unknown model")));
        assert_eq!(state.model(), ClawModel::Sonnet); // unchanged
    }

    // --- handle_load ---

    #[test]
    fn load_skill_success() {
        let fs = InMemoryFileSystem::new()
            .with_file("/home/test/.claude/skills/tdd/SKILL.md", "# TDD");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = default_state();

        handle_load("tdd", &mut state, &ports);
        assert_eq!(state.loaded_skills().len(),1);
    }

    #[test]
    fn load_skill_not_found() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = default_state();

        handle_load("nonexistent", &mut state, &ports);
        assert!(state.loaded_skills().is_empty());
    }

    // --- handle_branch ---

    #[test]
    fn branch_success() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = state_with_turns();

        handle_branch("new-branch", &mut state, &ports);
        assert_eq!(state.session_name(),"new-branch");
    }

    #[test]
    fn branch_invalid_name() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = state_with_turns();

        handle_branch("bad name!", &mut state, &ports);
        assert_eq!(state.session_name(),"test"); // unchanged
    }

    // --- handle_compact ---

    #[test]
    fn compact_reduces_turns() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = default_state();
        for i in 0..20 {
            state.add_turn(Turn {
                timestamp: format!("ts{i}"),
                role: Role::User,
                content: format!("msg {i}"),
            });
        }

        handle_compact(&Some(5), &mut state, &ports);
        assert_eq!(state.turns().len(),6); // header + 5
    }

    // --- handle_user_message ---

    #[test]
    fn user_message_adds_turns() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new().on(
            "claude",
            CommandOutput {
                stdout: "I'm Claude".to_string(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = default_state();

        handle_user_message("hello", &mut state, &ports);
        assert_eq!(state.turns().len(),2); // user + assistant
        assert_eq!(state.turns()[0].role, Role::User);
        assert_eq!(state.turns()[1].role, Role::Assistant);
    }

    #[test]
    fn user_message_empty_ignored() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = default_state();

        handle_user_message("", &mut state, &ports);
        assert!(state.turns().is_empty());
    }

    #[test]
    fn user_message_claude_error() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new().on(
            "claude",
            CommandOutput {
                stdout: String::new(),
                stderr: "error".to_string(),
                exit_code: 1,
            },
        );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);
        let mut state = default_state();

        handle_user_message("hello", &mut state, &ports);
        assert!(state.turns().is_empty()); // no orphan turns on error
        let err = term.stderr_output();
        assert!(err.iter().any(|s| s.contains("Error from Claude")));
    }
}
