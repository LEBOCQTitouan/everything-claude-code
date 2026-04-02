//! CLI for deterministic git analytics.

use clap::{Args, Subcommand};
use ecc_infra::git_log_adapter::GitLogAdapter;
use ecc_infra::process_executor::ProcessExecutor;
use std::path::PathBuf;

/// Deterministic git history analysis
#[derive(Args)]
pub struct AnalyzeArgs {
    /// Which analysis to run
    #[command(subcommand)]
    pub action: AnalyzeAction,
}

/// Available analysis subcommands
#[derive(Subcommand)]
pub enum AnalyzeAction {
    /// Generate a changelog from conventional commits
    Changelog {
        /// Show commits since this tag or date (default: last 90 days)
        #[arg(long)]
        since: Option<String>,
    },
    /// Show most frequently changed files
    Hotspots {
        /// Number of files to show
        #[arg(long, default_value = "10")]
        top: usize,
        /// Show commits since this tag or date (default: last 90 days)
        #[arg(long)]
        since: Option<String>,
        /// Exclude commits touching more than N files
        #[arg(long, default_value = "20")]
        max_files_per_commit: usize,
    },
    /// Show files that frequently change together
    Coupling {
        /// Minimum coupling ratio to display (0.0 to 1.0)
        #[arg(long, default_value = "0.7")]
        threshold: f64,
        /// Minimum individual commits for a file to be considered
        #[arg(long, default_value = "3")]
        min_commits: u32,
        /// Show commits since this tag or date (default: last 90 days)
        #[arg(long)]
        since: Option<String>,
        /// Exclude commits touching more than N files
        #[arg(long, default_value = "20")]
        max_files_per_commit: usize,
    },
    /// Show files with low contributor diversity (bus factor risk)
    BusFactor {
        /// Number of files to show
        #[arg(long, default_value = "10")]
        top: usize,
        /// Show commits since this tag or date (default: last 90 days)
        #[arg(long)]
        since: Option<String>,
    },
}

/// Run the analyze subcommand.
pub fn run(args: AnalyzeArgs) -> anyhow::Result<()> {
    let executor = ProcessExecutor;
    let adapter = GitLogAdapter::new(&executor);
    let repo = PathBuf::from(".");

    // Default --since to 90 days when not specified (except changelog)
    let default_since = "90.days.ago";

    match args.action {
        AnalyzeAction::Changelog { since } => {
            let result = ecc_app::analyze::generate_changelog(
                &adapter,
                &repo,
                since.as_deref(),
            )
            .map_err(|e| anyhow::anyhow!("{e}"))?;
            print!("{result}");
        }
        AnalyzeAction::Hotspots {
            top,
            since,
            max_files_per_commit,
        } => {
            let since_val = since.as_deref().or(Some(default_since));
            let hotspots = ecc_app::analyze::compute_hotspots(
                &adapter,
                &repo,
                since_val,
                top,
                max_files_per_commit,
            )
            .map_err(|e| anyhow::anyhow!("{e}"))?;

            if hotspots.is_empty() {
                println!("No hotspots found in the specified range.");
            } else {
                println!("Hotspots (top {top}):\n");
                for (i, h) in hotspots.iter().enumerate() {
                    println!("  {:>3}. {:<60} {} changes", i + 1, h.path, h.change_count);
                }
            }
        }
        AnalyzeAction::Coupling {
            threshold,
            min_commits,
            since,
            max_files_per_commit,
        } => {
            let since_val = since.as_deref().or(Some(default_since));
            let pairs = ecc_app::analyze::compute_coupling(
                &adapter,
                &repo,
                since_val,
                threshold,
                min_commits,
                max_files_per_commit,
            )
            .map_err(|e| anyhow::anyhow!("{e}"))?;

            if pairs.is_empty() {
                println!("No coupling pairs found above threshold {threshold:.1}.");
            } else {
                println!("Coupling (threshold >= {threshold:.1}):\n");
                for pair in &pairs {
                    println!(
                        "  {} <-> {} — {:.0}% ({} commits together)",
                        pair.file_a,
                        pair.file_b,
                        pair.coupling_ratio * 100.0,
                        pair.commits_together,
                    );
                }
            }
        }
        AnalyzeAction::BusFactor { top, since } => {
            let since_val = since.as_deref().or(Some(default_since));
            let factors = ecc_app::analyze::compute_bus_factor(
                &adapter,
                &repo,
                since_val,
                top,
            )
            .map_err(|e| anyhow::anyhow!("{e}"))?;

            if factors.is_empty() {
                println!("No files found in the specified range.");
            } else {
                println!("Bus Factor (top {top}, riskiest first):\n");
                for (i, f) in factors.iter().enumerate() {
                    let risk = if f.is_risk { " RISK: single author" } else { "" };
                    println!(
                        "  {:>3}. {:<60} {} authors, {} commits{}",
                        i + 1,
                        f.path,
                        f.unique_authors,
                        f.total_commits,
                        risk,
                    );
                }
            }
        }
    }

    Ok(())
}
