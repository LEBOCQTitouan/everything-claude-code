//! CLI wiring for `ecc worktree` subcommands.

use clap::{Args, Subcommand};
use ecc_app::worktree;
use ecc_infra::process_executor::ProcessExecutor;
use std::path::PathBuf;

#[derive(Args)]
pub struct WorktreeArgs {
    #[command(subcommand)]
    pub action: WorktreeAction,
}

#[derive(Subcommand)]
pub enum WorktreeAction {
    /// Garbage-collect stale ecc-session-* git worktrees
    Gc {
        /// Remove worktrees without confirming liveness check
        #[arg(long)]
        force: bool,

        /// Project directory (defaults to current directory)
        #[arg(long)]
        dir: Option<PathBuf>,
    },
}

pub fn run(args: WorktreeArgs) -> anyhow::Result<()> {
    let executor = ProcessExecutor;

    match args.action {
        WorktreeAction::Gc { force, dir } => {
            let project_dir = match dir {
                Some(d) => d,
                None => std::env::current_dir()
                    .map_err(|e| anyhow::anyhow!("cannot determine current directory: {e}"))?,
            };

            let result = worktree::gc(&executor, &project_dir, force)?;

            for name in &result.removed {
                println!("Removed: {name}");
            }
            for name in &result.skipped {
                println!("Skipped (active): {name}");
            }
            for err in &result.errors {
                eprintln!("Error: {err}");
            }

            if result.removed.is_empty() && result.skipped.is_empty() {
                println!("No ECC session worktrees found.");
            }
        }
    }

    Ok(())
}
