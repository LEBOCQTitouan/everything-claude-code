//! CLI: `ecc commit <action>` (compiled binary, not slash command)
use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct CommitCmdArgs {
    #[command(subcommand)]
    pub action: CommitCmdAction,
}

#[derive(Debug, Subcommand)]
pub enum CommitCmdAction {
    /// Lint staged files for multi-concern changes
    Lint {
        /// Check staged files
        #[arg(long)]
        staged: bool,
        /// Output JSON
        #[arg(long)]
        json: bool,
    },
}

pub fn run(args: CommitCmdArgs) -> anyhow::Result<()> {
    let shell = ecc_infra::process_executor::ProcessExecutor;
    let terminal = ecc_infra::std_terminal::StdTerminal;
    match args.action {
        CommitCmdAction::Lint { json, .. } => {
            let code = ecc_app::commit_lint::run_commit_lint(&shell, &terminal, json);
            if code != 0 {
                std::process::exit(code);
            }
            Ok(())
        }
    }
}
