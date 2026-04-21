//! CLI wiring for `ecc worktree` subcommands.

use clap::{Args, Subcommand};
use ecc_app::worktree;
use ecc_infra::os_worktree::OsWorktreeManager;
use ecc_infra::process_executor::ProcessExecutor;
use std::io::BufRead as _;
use std::io::IsTerminal as _;
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

        /// Delete live worktrees. Requires --force. Asks for confirmation in TTY;
        /// requires --yes in non-TTY contexts.
        #[arg(long, requires = "force")]
        kill_live: bool,

        /// Bypass confirmation prompt for --kill-live. Required in non-TTY contexts.
        #[arg(long)]
        yes: bool,

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
        WorktreeAction::Gc {
            force,
            kill_live,
            yes,
            dir,
        } => {
            let project_dir = resolve_dir(dir)?;

            // TTY-aware confirmation for --kill-live.
            if kill_live {
                if std::io::stdin().is_terminal() {
                    if !yes {
                        eprintln!("Delete live session worktrees? [y/N]");
                        let mut input = String::new();
                        std::io::BufReader::new(std::io::stdin())
                            .read_line(&mut input)
                            .ok();
                        if !input.trim().eq_ignore_ascii_case("y") {
                            eprintln!("Aborted");
                            std::process::exit(1);
                        }
                    }
                } else {
                    // Non-TTY: --yes is required.
                    if !yes {
                        eprintln!("--kill-live in non-interactive context requires --yes");
                        std::process::exit(1);
                    }
                }
            }

            let clock = ecc_infra::system_clock::SystemClock;
            let fs = ecc_infra::os_fs::OsFileSystem;
            let result = worktree::gc(
                &worktree_mgr,
                &executor,
                &fs,
                &project_dir,
                worktree::GcOptions {
                    force,
                    kill_live,
                    ..worktree::GcOptions::default()
                },
                &clock,
            )?;

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
            let clock = ecc_infra::system_clock::SystemClock;
            let fs = ecc_infra::os_fs::OsFileSystem;
            let entries = worktree::status(&worktree_mgr, &executor, &fs, &project_dir, &clock)?;
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
    #[test]
    fn status_exit_zero() {
        // Verify that the status function returns Ok (exit code 0 on success).
        // Uses in-memory mock rather than real OsWorktreeManager to avoid I/O.
        use ecc_test_support::{InMemoryFileSystem, MockExecutor, MockWorktreeManager, TEST_CLOCK};
        use std::path::Path;

        let mgr = MockWorktreeManager::new();
        let executor = MockExecutor::new();
        let fs = InMemoryFileSystem::new();
        let result =
            ecc_app::worktree::status(&mgr, &executor, &fs, Path::new("/repo"), &*TEST_CLOCK);
        assert!(
            result.is_ok(),
            "status must return Ok (exit code 0 on success)"
        );
    }
}
