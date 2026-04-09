//! CLI command: `ecc bypass <action>`
//!
//! Manage bypass tokens and query the bypass audit trail.

use clap::{Args, Subcommand};
use std::sync::Arc;

#[derive(Debug, Args)]
pub struct BypassArgs {
    #[command(subcommand)]
    pub action: BypassAction,
}

#[derive(Debug, Subcommand)]
pub enum BypassAction {
    /// Grant a bypass for a specific hook in the current session
    Grant {
        /// Hook ID to bypass (e.g., "pre:write-edit:worktree-guard")
        #[arg(long)]
        hook: String,
        /// Reason for the bypass (required for audit trail)
        #[arg(long)]
        reason: String,
    },
    /// List recent bypass decisions
    List {
        /// Filter by hook ID
        #[arg(long)]
        hook: Option<String>,
    },
    /// Show per-hook bypass summary (accepted/refused counts)
    Summary,
    /// Delete old bypass records
    Prune {
        /// Delete records older than this duration (e.g., "90d")
        #[arg(long)]
        older_than: String,
    },
    /// Clean up stale bypass token files from ended sessions
    Gc,
}

#[allow(clippy::print_literal)]
pub fn run(args: BypassArgs) -> anyhow::Result<()> {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let db_path = home.join(".ecc").join("bypass.db");

    match args.action {
        BypassAction::Grant { hook, reason } => {
            let session_id =
                std::env::var("CLAUDE_SESSION_ID").unwrap_or_else(|_| "unknown".to_string());
            if session_id == "unknown" || session_id.is_empty() {
                anyhow::bail!("CLAUDE_SESSION_ID not set — cannot create bypass token");
            }
            let clock = Arc::new(ecc_infra::system_clock::SystemClock);
            let store = ecc_infra::sqlite_bypass_store::SqliteBypassStore::new(&db_path, clock)?;
            let fs = ecc_infra::os_fs::OsFileSystem;
            let token =
                ecc_app::bypass_mgmt::grant(&store, &fs, &home, &hook, &reason, &session_id)?;
            println!(
                "Bypass granted for hook '{}' in session '{}'",
                token.hook_id, token.session_id
            );
            println!(
                "Token file: {}/.ecc/bypass-tokens/{}/{}.json",
                home.display(),
                session_id,
                hook.replace(':', "__")
            );
            Ok(())
        }
        BypassAction::List { hook } => {
            let clock = Arc::new(ecc_infra::system_clock::SystemClock);
            let store = ecc_infra::sqlite_bypass_store::SqliteBypassStore::new(&db_path, clock)?;
            let decisions = ecc_app::bypass_mgmt::list(&store, hook.as_deref(), 50)?;
            if decisions.is_empty() {
                println!("No bypass decisions found.");
            } else {
                println!(
                    "{:<5} {:<40} {:<10} {:<20} {}",
                    "ID", "Hook ID", "Verdict", "Timestamp", "Reason"
                );
                let sep = "-".repeat(100);
                println!("{sep}");
                for d in &decisions {
                    println!(
                        "{:<5} {:<40} {:<10} {:<20} {}",
                        d.id.unwrap_or(0),
                        d.hook_id,
                        d.verdict,
                        d.timestamp,
                        if d.reason.len() > 30 {
                            &d.reason[..30]
                        } else {
                            &d.reason
                        }
                    );
                }
            }
            Ok(())
        }
        BypassAction::Summary => {
            let clock = Arc::new(ecc_infra::system_clock::SystemClock);
            let store = ecc_infra::sqlite_bypass_store::SqliteBypassStore::new(&db_path, clock)?;
            let summary = ecc_app::bypass_mgmt::summary(&store)?;
            println!(
                "{:<40} {:<10} {:<10} {:<10}",
                "Hook ID", "Accepted", "Refused", "Ratio"
            );
            println!("{}", "-".repeat(70));
            for h in &summary.per_hook {
                let total = h.accepted + h.refused;
                let ratio = if total > 0 {
                    h.accepted as f64 / total as f64
                } else {
                    0.0
                };
                println!(
                    "{:<40} {:<10} {:<10} {:.1}%",
                    h.hook_id,
                    h.accepted,
                    h.refused,
                    ratio * 100.0
                );
            }
            println!(
                "\nTotal: {} accepted, {} refused",
                summary.total_accepted, summary.total_refused
            );
            Ok(())
        }
        BypassAction::Prune { older_than } => {
            let days = parse_duration_days(&older_than)?;
            let clock = Arc::new(ecc_infra::system_clock::SystemClock);
            let store = ecc_infra::sqlite_bypass_store::SqliteBypassStore::new(&db_path, clock)?;
            let deleted = ecc_app::bypass_mgmt::prune(&store, days)?;
            println!("Pruned {deleted} bypass records older than {older_than}");
            Ok(())
        }
        BypassAction::Gc => {
            let session_id =
                std::env::var("CLAUDE_SESSION_ID").unwrap_or_else(|_| "current".to_string());
            let fs = ecc_infra::os_fs::OsFileSystem;
            let removed = ecc_app::bypass_mgmt::gc(&fs, &home, &session_id)?;
            println!("Cleaned up {removed} stale bypass token directories");
            Ok(())
        }
    }
}

fn parse_duration_days(s: &str) -> anyhow::Result<u64> {
    if let Some(days_str) = s.strip_suffix('d') {
        days_str
            .parse::<u64>()
            .map_err(|_| anyhow::anyhow!("Invalid duration: {s}"))
    } else {
        s.parse::<u64>()
            .map_err(|_| anyhow::anyhow!("Invalid duration: {s}. Use format: 90d"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct TestCli {
        #[command(subcommand)]
        action: BypassAction,
    }

    #[test]
    fn bypass_parse_grant_args() {
        let cli = TestCli::try_parse_from([
            "test",
            "grant",
            "--hook",
            "pre:write:guard",
            "--reason",
            "hotfix",
        ]);
        assert!(cli.is_ok());
    }

    #[test]
    fn bypass_parse_list_args() {
        let cli = TestCli::try_parse_from(["test", "list"]);
        assert!(cli.is_ok());

        let cli = TestCli::try_parse_from(["test", "list", "--hook", "my-hook"]);
        assert!(cli.is_ok());
    }

    #[test]
    fn bypass_parse_summary_args() {
        let cli = TestCli::try_parse_from(["test", "summary"]);
        assert!(cli.is_ok());
    }

    #[test]
    fn bypass_parse_prune_args() {
        let cli = TestCli::try_parse_from(["test", "prune", "--older-than", "90d"]);
        assert!(cli.is_ok());
    }

    #[test]
    fn bypass_parse_gc_args() {
        let cli = TestCli::try_parse_from(["test", "gc"]);
        assert!(cli.is_ok());
    }

    #[test]
    fn bypass_grant_requires_reason() {
        let cli = TestCli::try_parse_from(["test", "grant", "--hook", "x"]);
        assert!(cli.is_err()); // --reason is required
    }

    #[test]
    fn parse_duration_days_valid() {
        assert_eq!(parse_duration_days("90d").unwrap(), 90);
        assert_eq!(parse_duration_days("30d").unwrap(), 30);
        assert_eq!(parse_duration_days("7d").unwrap(), 7);
    }

    #[test]
    fn parse_duration_days_invalid() {
        assert!(parse_duration_days("abc").is_err());
    }
}
