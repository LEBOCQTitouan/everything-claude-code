//! CLI command: `ecc claw` — NanoClaw interactive REPL.

use clap::Args;
use ecc_app::claw::{run_repl, ClawConfig, ClawPorts, Model as ClawModel};
use ecc_ports::env::Environment;
use ecc_infra::os_env::OsEnvironment;
use ecc_infra::os_fs::OsFileSystem;
use ecc_infra::process_executor::ProcessExecutor;
use ecc_infra::rustyline_input::RustylineInput;
use ecc_infra::std_terminal::StdTerminal;

#[derive(Args)]
pub struct ClawArgs {
    /// Session name (default: "default")
    #[arg(short, long, default_value = "default")]
    pub session: String,

    /// Model to use (sonnet, opus, haiku)
    #[arg(short, long, default_value = "sonnet")]
    pub model: String,

    /// Skills to preload (comma-separated)
    #[arg(short = 'k', long, value_delimiter = ',')]
    pub skills: Vec<String>,
}

pub fn run(args: ClawArgs) -> anyhow::Result<()> {
    let model = ClawModel::parse(&args.model)
        .ok_or_else(|| anyhow::anyhow!("Invalid model: '{}'. Use sonnet, opus, or haiku.", args.model))?;

    let config = ClawConfig {
        session_name: args.session,
        model,
        initial_skills: args.skills,
    };

    // Set up production adapters
    let fs = OsFileSystem;
    let shell = ProcessExecutor;
    let env = OsEnvironment;
    let terminal = StdTerminal;

    // Set up rustyline with history in claw dir
    let history_path = env.home_dir().map(|h| {
        ecc_app::claw::history_path(&h)
    });
    let repl_input = RustylineInput::new(history_path)
        .map_err(|e| anyhow::anyhow!(e))?;

    let ports = ClawPorts {
        fs: &fs,
        shell: &shell,
        env: &env,
        terminal: &terminal,
        repl_input: &repl_input,
    };

    run_repl(&config, &ports)
}
