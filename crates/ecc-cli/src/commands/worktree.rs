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

        /// Preview mode: print what would be deleted without performing any destructive operations.
        #[arg(long)]
        dry_run: bool,

        /// Emit output as JSON (use with --dry-run for machine-readable preview).
        #[arg(long)]
        json: bool,

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
            dry_run,
            json,
            dir,
        } => {
            let project_dir = resolve_dir(dir)?;

            // TTY-aware confirmation for --kill-live.
            // Skipped entirely in dry-run mode (non-destructive, no prompt needed).
            if kill_live && !dry_run {
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

            // AC-009.1: kill switch — suppress liveness read AND write paths.
            let liveness_disabled =
                std::env::var("ECC_WORKTREE_LIVENESS_DISABLED").as_deref() == Ok("1");
            if liveness_disabled {
                // Emit once per process to stderr so integration tests can assert on it.
                eprintln!(
                    "ecc: worktree liveness check disabled (ECC_WORKTREE_LIVENESS_DISABLED=1)"
                );
                tracing::warn!(
                    "worktree liveness check disabled via ECC_WORKTREE_LIVENESS_DISABLED"
                );
            }

            // AC-009.2: validate TTL env var; fall back to default on malformed input.
            let _ttl = parse_u64_env_secs("ECC_WORKTREE_LIVENESS_TTL_SECS", 3600);
            // AC-009.3: validate self-skip fallback env var.
            let _self_skip_fallback =
                parse_u64_env_secs("ECC_WORKTREE_SELF_SKIP_FALLBACK_SECS", 3600);

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
                    dry_run,
                    liveness_disabled,
                    ..worktree::GcOptions::default()
                },
                &clock,
            )?;

            // AC-008.1: render dry-run preview output.
            if dry_run {
                if json {
                    let rows: Vec<_> = result
                        .would_delete
                        .iter()
                        .map(|w| {
                            serde_json::json!({
                                "name": w.name,
                                "action": "would_delete",
                                "reason": w.reason.as_str(),
                            })
                        })
                        .collect();
                    println!(
                        "{}",
                        serde_json::to_string(&rows)
                            .map_err(|e| anyhow::anyhow!("JSON serialization failed: {e}"))?
                    );
                } else {
                    for w in &result.would_delete {
                        println!("WOULD DELETE: {} (reason: {})", w.name, w.reason.as_str());
                    }
                    if result.would_delete.is_empty() {
                        println!("No ECC session worktrees would be deleted.");
                    }
                }
                return Ok(());
            }

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

/// Parse a `u64` environment variable representing seconds.
///
/// Returns `default` if the variable is absent, non-numeric, zero, or negative.
/// Emits a [`tracing::warn!`] when the value is present but invalid.
fn parse_u64_env_secs(key: &str, default: u64) -> u64 {
    match std::env::var(key) {
        Ok(v) => match v.parse::<u64>() {
            Ok(n) if n > 0 => n,
            _ => {
                tracing::warn!(
                    key = %key,
                    value = %v,
                    default,
                    "invalid env var value; using default"
                );
                default
            }
        },
        Err(_) => default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    // PC-061: Malformed ECC_WORKTREE_LIVENESS_TTL_SECS logs WARN + uses default (AC-009.2)
    #[test]
    #[tracing_test::traced_test]
    fn invalid_ttl_env_warns_and_defaults() {
        let key = "ECC_WORKTREE_LIVENESS_TTL_SECS";
        let default = 3600u64;

        // Non-numeric value → default
        // SAFETY: single-threaded test; no other threads read this env var.
        unsafe { std::env::set_var(key, "abc") };
        let result = parse_u64_env_secs(key, default);
        assert_eq!(
            result, default,
            "non-numeric 'abc' must return default {default}"
        );
        assert!(
            logs_contain("invalid env var value; using default"),
            "must emit WARN for 'abc'"
        );

        // Negative sentinel (parse::<u64>() rejects "-1") → default
        // SAFETY: same test, single-threaded.
        unsafe { std::env::set_var(key, "-1") };
        let result = parse_u64_env_secs(key, default);
        assert_eq!(
            result, default,
            "'-1' must return default (u64 parse failure)"
        );
        assert!(
            logs_contain("invalid env var value; using default"),
            "must emit WARN for '-1'"
        );

        // Zero → default (u64 parses but n > 0 fails)
        // SAFETY: same test, single-threaded.
        unsafe { std::env::set_var(key, "0") };
        let result = parse_u64_env_secs(key, default);
        assert_eq!(result, default, "'0' must return default (n > 0 guard)");
        assert!(
            logs_contain("invalid env var value; using default"),
            "must emit WARN for '0'"
        );

        // SAFETY: cleanup.
        unsafe { std::env::remove_var(key) };
    }

    // PC-062: Malformed ECC_WORKTREE_SELF_SKIP_FALLBACK_SECS logs WARN + uses default (AC-009.3)
    #[test]
    #[tracing_test::traced_test]
    fn invalid_fallback_env_warns_and_defaults() {
        let key = "ECC_WORKTREE_SELF_SKIP_FALLBACK_SECS";
        let default = 3600u64;

        // Non-numeric value → default
        // SAFETY: single-threaded test; no other threads read this env var.
        unsafe { std::env::set_var(key, "abc") };
        let result = parse_u64_env_secs(key, default);
        assert_eq!(
            result, default,
            "non-numeric 'abc' must return default {default}"
        );
        assert!(
            logs_contain("invalid env var value; using default"),
            "must emit WARN for 'abc'"
        );

        // "-1" → default (u64 parse failure)
        // SAFETY: same test, single-threaded.
        unsafe { std::env::set_var(key, "-1") };
        let result = parse_u64_env_secs(key, default);
        assert_eq!(result, default, "'-1' must return default");
        assert!(
            logs_contain("invalid env var value; using default"),
            "must emit WARN for '-1'"
        );

        // Zero → default (n > 0 guard)
        // SAFETY: same test, single-threaded.
        unsafe { std::env::set_var(key, "0") };
        let result = parse_u64_env_secs(key, default);
        assert_eq!(result, default, "'0' must return default");
        assert!(
            logs_contain("invalid env var value; using default"),
            "must emit WARN for '0'"
        );

        // SAFETY: cleanup.
        unsafe { std::env::remove_var(key) };
    }
}
