//! CLI: `ecc docs <action>`
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct DocsArgs {
    #[command(subcommand)]
    pub action: DocsAction,
}

#[derive(Debug, Subcommand)]
pub enum DocsAction {
    /// Update MODULE-SUMMARIES.md entries for changed crates
    UpdateModuleSummary {
        /// Changed file paths
        #[arg(long, value_delimiter = ',')]
        changed_files: Vec<String>,
        /// Feature name
        #[arg(long)]
        feature: String,
        /// Output JSON
        #[arg(long)]
        json: bool,
    },
    /// Count doc comment coverage per module
    Coverage {
        /// Source directory to scan
        #[arg(long)]
        scope: PathBuf,
        /// Output JSON
        #[arg(long)]
        json: bool,
    },
}

pub fn run(args: DocsArgs) -> anyhow::Result<()> {
    let fs = ecc_infra::os_fs::OsFileSystem;
    let terminal = ecc_infra::std_terminal::StdTerminal;
    match args.action {
        DocsAction::UpdateModuleSummary {
            changed_files,
            feature,
            json,
        } => {
            if !ecc_app::docs_update_summary::run_update_summary(
                &fs,
                &terminal,
                &changed_files,
                &feature,
                json,
            ) {
                std::process::exit(1);
            }
            Ok(())
        }
        DocsAction::Coverage { scope, json } => {
            if !ecc_app::docs_coverage::run_docs_coverage(&fs, &terminal, &scope, json) {
                std::process::exit(1);
            }
            Ok(())
        }
    }
}
