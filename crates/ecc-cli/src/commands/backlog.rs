//! CLI for deterministic backlog management.

use clap::{Args, Subcommand};
use ecc_infra::fs_backlog::FsBacklogRepository;
use ecc_infra::os_fs::OsFileSystem;
use ecc_infra::os_worktree::OsWorktreeManager;
use ecc_infra::system_clock::SystemClock;
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
    let repo = FsBacklogRepository::new(&fs);
    let worktree_mgr = OsWorktreeManager;
    let clock = SystemClock;
    let dir = &args.dir;

    // Determine project root (parent of backlog dir, fallback to cwd)
    let project_dir = dir
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    match args.action {
        BacklogAction::NextId => {
            let id = ecc_app::backlog::next_id(&repo, dir).map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("{id}");
        }
        BacklogAction::CheckDuplicates { query, tags } => {
            let tag_list: Vec<String> = tags
                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();
            let candidates =
                ecc_app::backlog::check_duplicates(&repo, dir, &query, &tag_list)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
            let json = serde_json::to_string_pretty(&candidates)?;
            println!("{json}");
        }
        BacklogAction::Reindex { dry_run } => {
            let output = ecc_app::backlog::reindex(
                &repo,
                &repo,
                &repo,
                &worktree_mgr,
                &clock,
                dir,
                &project_dir,
                dry_run,
            )
            .map_err(|e| anyhow::anyhow!("{e}"))?;
            if let Some(content) = output {
                print!("{content}");
            }
        }
    }

    Ok(())
}
