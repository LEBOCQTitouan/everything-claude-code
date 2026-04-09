//! CLI: `ecc diagram <action>`
use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct DiagramArgs {
    #[command(subcommand)]
    pub action: DiagramAction,
}

#[derive(Debug, Subcommand)]
pub enum DiagramAction {
    /// Evaluate diagram generation triggers from changed files
    Triggers {
        /// Changed file paths
        #[arg(long, value_delimiter = ',')]
        changed_files: Vec<String>,
        /// Output JSON
        #[arg(long)]
        json: bool,
    },
}

pub fn run(args: DiagramArgs) -> anyhow::Result<()> {
    let fs = ecc_infra::os_fs::OsFileSystem;
    let terminal = ecc_infra::std_terminal::StdTerminal;
    match args.action {
        DiagramAction::Triggers {
            changed_files,
            json,
        } => {
            ecc_app::diagram_triggers::run_diagram_triggers(&fs, &terminal, &changed_files, json);
            Ok(())
        }
    }
}
