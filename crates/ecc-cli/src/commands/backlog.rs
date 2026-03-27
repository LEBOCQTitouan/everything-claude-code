//! CLI for deterministic backlog management.

use clap::{Args, Subcommand};
use ecc_infra::os_fs::OsFileSystem;
use std::path::PathBuf;

#[derive(Args)]
pub struct BacklogArgs {
    #[command(subcommand)]
    pub action: BacklogAction,

    /// Path to the backlog directory
    #[arg(long, default_value = "docs/backlog")]
    pub dir: PathBuf,
}

#[derive(Subcommand)]
pub enum BacklogAction {
    /// Print the next sequential BL-NNN ID
    NextId,

    /// Check for duplicate backlog entries by title similarity
    CheckDuplicates {
        /// Title to check for duplicates
        query: String,

        /// Comma-separated tags to boost matching score
        #[arg(long)]
        tags: Option<String>,
    },

    /// Regenerate BACKLOG.md index from BL-*.md files
    Reindex {
        /// Print generated content without writing to file
        #[arg(long)]
        dry_run: bool,
    },
}

pub fn run(args: BacklogArgs) -> anyhow::Result<()> {
    let fs = OsFileSystem;
    let dir = &args.dir;

    match args.action {
        BacklogAction::NextId => {
            let id = ecc_app::backlog::next_id(&fs, dir)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("{id}");
        }
        BacklogAction::CheckDuplicates { query, tags } => {
            let tag_list: Vec<String> = tags
                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();
            let candidates = ecc_app::backlog::check_duplicates(&fs, dir, &query, &tag_list)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            let json = serde_json::to_string_pretty(&candidates)?;
            println!("{json}");
        }
        BacklogAction::Reindex { dry_run } => {
            let output = ecc_app::backlog::reindex(&fs, dir, dry_run)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            if let Some(content) = output {
                print!("{content}");
            }
        }
    }

    Ok(())
}
