//! CLI wiring for `ecc worktree` subcommands.

use clap::{Args, Subcommand};
use ecc_app::worktree;
use ecc_infra::os_worktree::OsWorktreeManager;
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
    /// Show status table for all active ecc-session-* worktrees
    Status {
        /// Project directory (defaults to current directory)
        #[arg(long)]
        dir: Option<PathBuf>,
    },
}

pub fn run(args: WorktreeArgs) -> anyhow::Result<()> {
    let executor = ProcessExecutor;
    let worktree_mgr = OsWorktreeManager;

    match args.action {
        WorktreeAction::Gc { force, dir } => {
            let project_dir = resolve_dir(dir)?;
            let result = worktree::gc(&worktree_mgr, &executor, &project_dir, force)?;

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
        WorktreeAction::Status { dir } => {
            let project_dir = resolve_dir(dir)?;
            let entries = worktree::status(&worktree_mgr, &executor, &project_dir)?;
            let table = worktree::format_status_table(&entries);
            println!("{table}");
        }
    }

    Ok(())
}

fn resolve_dir(dir: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    match dir {
        Some(d) => Ok(d),
        None => std::env::current_dir()
            .map_err(|e| anyhow::anyhow!("cannot determine current directory: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_exit_zero() {
        // Verify that the status function returns Ok (exit code 0 on success).
        // Uses in-memory mock rather than real OsWorktreeManager to avoid I/O.
        use ecc_test_support::{MockExecutor, MockWorktreeManager};
        use std::path::Path;

        let mgr = MockWorktreeManager::new();
        let executor = MockExecutor::new();
        let result = ecc_app::worktree::status(&mgr, &executor, Path::new("/repo"));
        assert!(result.is_ok(), "status must return Ok (exit code 0 on success)");
    }
}
