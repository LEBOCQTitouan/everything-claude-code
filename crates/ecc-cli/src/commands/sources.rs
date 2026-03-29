//! CLI for knowledge sources registry management.

use clap::{Args, Subcommand};
use ecc_infra::os_fs::OsFileSystem;
use ecc_infra::process_executor::ProcessExecutor;
use std::path::PathBuf;

#[derive(Args)]
pub struct SourcesArgs {
    #[command(subcommand)]
    pub action: SourcesAction,

    /// Path to the sources registry file
    #[arg(long, default_value = "docs/sources.md")]
    pub file: PathBuf,
}

#[derive(Subcommand)]
pub enum SourcesAction {
    /// List knowledge sources, optionally filtered
    List {
        /// Filter by quadrant (Adopt, Trial, Assess, Hold, Inbox)
        #[arg(long)]
        quadrant: Option<String>,

        /// Filter by subject
        #[arg(long)]
        subject: Option<String>,
    },

    /// Add a new knowledge source entry
    Add {
        /// URL of the source
        url: String,

        /// Human-readable title
        #[arg(long)]
        title: String,

        /// Type of source (blog, repo, docs, etc.)
        #[arg(long, id = "type")]
        source_type: String,

        /// Quadrant (Adopt, Trial, Assess, Hold, Inbox)
        #[arg(long)]
        quadrant: String,

        /// Subject/topic category
        #[arg(long)]
        subject: String,

        /// Date added (ISO 8601, defaults to today if not provided)
        #[arg(long, default_value = "")]
        added_date: String,

        /// Who is adding this entry
        #[arg(long, default_value = "")]
        added_by: String,
    },

    /// Check all source URLs for reachability
    Check,

    /// Rebuild the registry in canonical quadrant order
    Reindex,
}

pub fn run(args: SourcesArgs) -> anyhow::Result<()> {
    let fs = OsFileSystem;
    let path = &args.file;

    match args.action {
        SourcesAction::List { quadrant, subject } => {
            let entries = ecc_app::sources::list(
                &fs,
                path,
                quadrant.as_deref(),
                subject.as_deref(),
            )
            .map_err(|e| anyhow::anyhow!("{e}"))?;

            for entry in &entries {
                println!(
                    "[{}] {} — {} ({})",
                    entry.quadrant, entry.title, entry.url, entry.subject
                );
            }
        }

        SourcesAction::Add {
            url,
            title,
            source_type,
            quadrant,
            subject,
            added_date,
            added_by,
        } => {
            let entry = ecc_app::sources::SourceEntry {
                url,
                title,
                source_type,
                subject,
                quadrant,
                added_date,
                added_by,
            };
            ecc_app::sources::add(&fs, path, entry)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("Source added.");
        }

        SourcesAction::Check => {
            let shell = ProcessExecutor;
            let report = ecc_app::sources::check(&fs, &shell, path)
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            println!("Reachable: {}", report.reachable.len());
            println!("Stale:     {}", report.stale.len());

            if !report.stale.is_empty() {
                println!("\nStale sources:");
                for entry in &report.stale {
                    println!("  {} — {}", entry.title, entry.url);
                }
            }
        }

        SourcesAction::Reindex => {
            ecc_app::sources::reindex(&fs, path)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("Registry reindexed.");
        }
    }

    Ok(())
}
