//! CLI: `ecc drift <action>`
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct DriftArgs {
    #[command(subcommand)]
    pub action: DriftAction,
}

#[derive(Debug, Subcommand)]
pub enum DriftAction {
    /// Check spec-vs-implementation drift
    Check {
        /// Spec directory (defaults to docs/specs/)
        #[arg(long, default_value = ".")]
        spec_dir: PathBuf,
        /// Output JSON
        #[arg(long)]
        json: bool,
    },
}

pub fn run(args: DriftArgs) -> anyhow::Result<()> {
    let fs = ecc_infra::os_fs::OsFileSystem;
    let terminal = ecc_infra::std_terminal::StdTerminal;
    match args.action {
        DriftAction::Check { spec_dir, json } => {
            if !ecc_app::drift_check::run_drift_check(&fs, &terminal, &spec_dir, json) {
                std::process::exit(1);
            }
            Ok(())
        }
    }
}
