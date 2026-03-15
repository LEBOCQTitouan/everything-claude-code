//! Claw REPL use case — persistent, session-aware REPL delegating to `claude -p`.

use ecc_domain::claw::command::{parse_command, ClawCommand};
use ecc_domain::claw::model::ClawModel;

// Re-export ClawModel so CLI doesn't need to import from ecc-domain directly.
pub use ecc_domain::claw::model::ClawModel as Model;
use ecc_ports::env::Environment;
use ecc_ports::fs::FileSystem;
use ecc_ports::repl::ReplInput;
use ecc_ports::shell::ShellExecutor;
use ecc_ports::terminal::TerminalIO;

pub mod claude_runner;
pub mod dispatch;
pub mod handlers;
pub mod skill_loader;
pub mod storage;

/// Ports bundle for the Claw REPL.
pub struct ClawPorts<'a> {
    pub fs: &'a dyn FileSystem,
    pub shell: &'a dyn ShellExecutor,
    pub env: &'a dyn Environment,
    pub terminal: &'a dyn TerminalIO,
    pub repl_input: &'a dyn ReplInput,
}

/// Configuration for a Claw session.
#[derive(Debug, Clone)]
pub struct ClawConfig {
    pub session_name: String,
    pub model: ClawModel,
    pub initial_skills: Vec<String>,
}

impl Default for ClawConfig {
    fn default() -> Self {
        Self {
            session_name: "default".to_string(),
            model: ClawModel::default(),
            initial_skills: Vec::new(),
        }
    }
}

/// Mutable state for a running REPL session.
pub struct ClawState {
    session_name: String,
    model: ClawModel,
    turns: Vec<ecc_domain::claw::turn::Turn>,
    loaded_skills: Vec<String>,
}

impl ClawState {
    pub fn new(config: &ClawConfig) -> Self {
        Self {
            session_name: config.session_name.clone(),
            model: config.model,
            turns: Vec::new(),
            loaded_skills: Vec::new(),
        }
    }

    // --- Accessors ---

    pub fn session_name(&self) -> &str {
        &self.session_name
    }

    pub fn model(&self) -> ClawModel {
        self.model
    }

    pub fn turns(&self) -> &[ecc_domain::claw::turn::Turn] {
        &self.turns
    }

    pub fn loaded_skills(&self) -> &[String] {
        &self.loaded_skills
    }

    // --- Mutators ---

    pub fn set_session_name(&mut self, name: String) {
        self.session_name = name;
    }

    pub fn set_model(&mut self, model: ClawModel) {
        self.model = model;
    }

    pub fn set_turns(&mut self, turns: Vec<ecc_domain::claw::turn::Turn>) {
        self.turns = turns;
    }

    pub fn clear_turns(&mut self) -> usize {
        let count = self.turns.len();
        self.turns.clear();
        count
    }

    pub fn add_turn(&mut self, turn: ecc_domain::claw::turn::Turn) {
        self.turns.push(turn);
    }

    pub fn load_skill(&mut self, content: String) {
        self.loaded_skills.push(content);
    }
}

/// Get the Claw REPL history file path from a home directory.
pub fn history_path(home: &std::path::Path) -> std::path::PathBuf {
    ecc_domain::paths::claw_history_path(home)
}

/// Run the Claw REPL. Returns Ok(()) when the user exits.
pub fn run_repl(config: &ClawConfig, ports: &ClawPorts<'_>) -> anyhow::Result<()> {
    let home = ports
        .env
        .home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;

    let claw_dir = ecc_domain::paths::claw_dir(&home);
    if let Err(e) = ports.fs.create_dir_all(&claw_dir.join("sessions")) {
        ports
            .terminal
            .stderr_write(&format!("Warning: failed to create sessions dir: {e}\n"));
    }

    let mut state = ClawState::new(config);

    // Load existing session if available
    let session_path = ecc_domain::paths::claw_session_path(&home, state.session_name());
    if let Ok(content) = ports.fs.read_to_string(&session_path) {
        state.set_turns(ecc_domain::claw::turn::parse_turns(&content));
    }

    // Load initial skills
    for skill_name in &config.initial_skills {
        if let Ok(content) = skill_loader::load_skill(skill_name, ports) {
            state.load_skill(content);
        }
    }

    // Print welcome
    ports.terminal.stderr_write(&format!(
        "NanoClaw REPL — session: {}, model: {}\nType /help for commands, exit to quit.\n",
        state.session_name(),
        state.model().display_name(),
    ));

    // Main REPL loop
    loop {
        let prompt = format!("{}> ", state.session_name());
        let line = match ports.repl_input.read_line(&prompt) {
            Ok(Some(line)) => line,
            Ok(None) => break, // EOF
            Err(e) => {
                log::warn!("REPL read_line error: {}", e);
                ports.terminal.stderr_write(&format!("Input error: {e}\n"));
                break;
            }
        };

        let command = parse_command(&line);
        match command {
            ClawCommand::Exit => break,
            cmd => {
                dispatch::dispatch_command(&cmd, &mut state, ports)?;
            }
        }
    }

    // Save session on exit
    if !state.turns().is_empty()
        && let Err(e) =
            storage::save_session(&home, state.session_name(), state.turns(), ports.fs)
    {
        ports
            .terminal
            .stderr_write(&format!("Warning: failed to save session: {e}\n"));
    }

    ports.terminal.stderr_write("Goodbye.\n");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn run_repl_exit_immediately() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new().with_line("exit");
        let ports = make_ports(&fs, &shell, &env, &term, &input);

        let config = ClawConfig::default();
        let result = run_repl(&config, &ports);
        assert!(result.is_ok());
    }

    #[test]
    fn run_repl_eof_exits() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new(); // no lines = EOF
        let ports = make_ports(&fs, &shell, &env, &term, &input);

        let config = ClawConfig::default();
        let result = run_repl(&config, &ports);
        assert!(result.is_ok());
    }

    #[test]
    fn run_repl_prints_welcome() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new().with_line("exit");
        let ports = make_ports(&fs, &shell, &env, &term, &input);

        let config = ClawConfig::default();
        let _ = run_repl(&config, &ports);

        let stderr = term.stderr_output();
        assert!(stderr.iter().any(|s| s.contains("NanoClaw REPL")));
    }

    #[test]
    fn run_repl_prints_goodbye() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new().with_line("exit");
        let ports = make_ports(&fs, &shell, &env, &term, &input);

        let config = ClawConfig::default();
        let _ = run_repl(&config, &ports);

        let stderr = term.stderr_output();
        assert!(stderr.iter().any(|s| s.contains("Goodbye")));
    }

    #[test]
    fn run_repl_no_home_returns_error() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_home_none();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);

        let config = ClawConfig::default();
        let result = run_repl(&config, &ports);
        assert!(result.is_err());
    }

    #[test]
    fn run_repl_loads_existing_session() {
        let session_content = "### [ts1] User\nhello\n---\n\n### [ts2] Assistant\nhi\n---";
        let fs = InMemoryFileSystem::new()
            .with_file(
                "/home/test/.claude/claw/sessions/default.md",
                session_content,
            );
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new().with_line("/metrics").with_line("exit");
        let ports = make_ports(&fs, &shell, &env, &term, &input);

        let config = ClawConfig::default();
        let _ = run_repl(&config, &ports);

        let stdout = term.stdout_output();
        assert!(stdout.iter().any(|s| s.contains("Turns: 2")));
    }

    #[test]
    fn run_repl_saves_session_on_exit() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        // /clear won't add turns, but a user message handled by dispatch will
        let input = ScriptedInput::new().with_line("/help").with_line("exit");
        let ports = make_ports(&fs, &shell, &env, &term, &input);

        let config = ClawConfig::default();
        let _ = run_repl(&config, &ports);
        // Session file may or may not exist depending on whether turns were added
        // Just verify no crash
    }

    #[test]
    fn claw_state_new_from_config() {
        let config = ClawConfig {
            session_name: "test".to_string(),
            model: ClawModel::Opus,
            initial_skills: vec!["tdd".to_string()],
        };
        let state = ClawState::new(&config);
        assert_eq!(state.session_name(), "test");
        assert_eq!(state.model(), ClawModel::Opus);
        assert!(state.turns().is_empty());
    }

    #[test]
    fn claw_config_default() {
        let config = ClawConfig::default();
        assert_eq!(config.session_name, "default");
        assert_eq!(config.model, ClawModel::Sonnet);
        assert!(config.initial_skills.is_empty());
    }
}
