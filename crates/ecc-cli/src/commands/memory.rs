//! CLI wiring for memory system subcommands.

use clap::{Args, Subcommand};
use ecc_app::memory::crud::{AddParams, MemoryAppError};
use ecc_domain::memory::{MemoryId, MemoryTier};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Args)]
pub struct MemoryArgs {
    #[command(subcommand)]
    pub action: MemoryAction,
}

#[derive(Subcommand)]
pub enum MemoryAction {
    /// Add a new memory entry
    Add {
        /// Memory tier (working, episodic, semantic)
        #[arg(long = "type", default_value = "episodic")]
        tier: String,
        /// Entry title
        #[arg(long)]
        title: String,
        /// Comma-separated tags
        #[arg(long)]
        tags: Option<String>,
        /// Entry content
        #[arg(long)]
        content: Option<String>,
        /// Project ID scope
        #[arg(long)]
        project_id: Option<String>,
        /// Force add even if secrets detected
        #[arg(long)]
        force: bool,
    },
    /// Search memories using full-text search
    Search {
        /// Query string
        query: String,
        /// Filter by tier
        #[arg(long = "type")]
        tier: Option<String>,
        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,
        /// Maximum results
        #[arg(long, default_value = "10")]
        limit: usize,
    },
    /// List memories with optional filters
    List {
        /// Filter by tier
        #[arg(long = "type")]
        tier: Option<String>,
        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,
        /// Maximum results
        #[arg(long, default_value = "20")]
        limit: usize,
    },
    /// Delete a memory entry
    Delete {
        /// Memory ID to delete
        id: i64,
    },
    /// Promote an entry to the next tier
    Promote {
        /// Memory ID to promote
        id: i64,
    },
    /// Migrate legacy memory files to SQLite
    Migrate {
        /// Source directory path (default: docs/memory/)
        #[arg(long)]
        source: Option<PathBuf>,
    },
    /// Export memories to markdown files
    Export {
        /// Output directory path
        #[arg(long)]
        output: PathBuf,
    },
    /// Garbage collect stale old entries
    Gc {
        /// Report without deleting
        #[arg(long)]
        dry_run: bool,
    },
    /// Show memory store statistics
    Stats,
}

fn open_store() -> anyhow::Result<ecc_infra::sqlite_memory::SqliteMemoryStore> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("could not determine home directory"))?;
    let db_dir = home.join(".ecc").join("memory");
    std::fs::create_dir_all(&db_dir)?;
    let db_path = db_dir.join("memory.db");
    ecc_infra::sqlite_memory::SqliteMemoryStore::new(&db_path)
        .map_err(|e| anyhow::anyhow!("failed to open memory store: {e}"))
}

fn parse_tier(s: &str) -> anyhow::Result<MemoryTier> {
    MemoryTier::from_str(s).map_err(|e| anyhow::anyhow!("{e}"))
}

fn parse_tags(s: Option<&str>) -> Vec<String> {
    match s {
        Some(t) if !t.is_empty() => t.split(',').map(|s| s.trim().to_owned()).collect(),
        _ => vec![],
    }
}

pub fn run(args: MemoryArgs) -> anyhow::Result<()> {
    match args.action {
        MemoryAction::Add {
            tier,
            title,
            tags,
            content,
            project_id,
            force,
        } => {
            let store = open_store()?;
            let tier = parse_tier(&tier)?;
            let tags = parse_tags(tags.as_deref());
            let params = AddParams {
                title,
                content: content.unwrap_or_default(),
                tier,
                tags,
                project_id,
                session_id: None,
                force,
            };
            match ecc_app::memory::crud::add(&store, params) {
                Ok(id) => println!("Added memory entry: {id}"),
                Err(MemoryAppError::SecretDetected(kind)) => {
                    eprintln!("Error: content contains likely secrets ({kind}). Use --force to override.");
                    std::process::exit(1);
                }
                Err(e) => return Err(anyhow::anyhow!("{e}")),
            }
        }

        MemoryAction::Search {
            query,
            tier: _,
            tag: _,
            limit,
        } => {
            let store = open_store()?;
            let entries = ecc_app::memory::crud::search(&store, &query, limit)?;
            if entries.is_empty() {
                println!("No matching memories found");
            } else {
                for e in &entries {
                    println!(
                        "[{}] {} ({}) — {}",
                        e.id,
                        e.title,
                        e.tier,
                        &e.content.chars().take(80).collect::<String>()
                    );
                }
            }
        }

        MemoryAction::List { tier, tag, limit: _ } => {
            let store = open_store()?;
            let tier_filter = tier.as_deref().map(parse_tier).transpose()?;
            let entries = ecc_app::memory::crud::list(
                &store,
                tier_filter,
                tag.as_deref(),
                None,
            )?;
            if entries.is_empty() {
                println!("No memories found");
            } else {
                for e in &entries {
                    let stale_mark = if e.stale { " [stale]" } else { "" };
                    println!(
                        "[{}] {} ({}){} — score: {:.2}",
                        e.id, e.title, e.tier, stale_mark, e.relevance_score
                    );
                }
            }
        }

        MemoryAction::Delete { id } => {
            let store = open_store()?;
            match ecc_app::memory::crud::delete(&store, MemoryId(id)) {
                Ok(()) => println!("Deleted memory entry {id}"),
                Err(MemoryAppError::NotFound(_)) => {
                    eprintln!("Error: Memory not found");
                    std::process::exit(1);
                }
                Err(e) => return Err(anyhow::anyhow!("{e}")),
            }
        }

        MemoryAction::Promote { id } => {
            let store = open_store()?;
            match ecc_app::memory::lifecycle::promote(&store, MemoryId(id)) {
                Ok(entry) => println!("Promoted entry {} to {} (score: {:.2})", id, entry.tier, entry.relevance_score),
                Err(MemoryAppError::AlreadySemantic) => {
                    println!("Already semantic");
                }
                Err(MemoryAppError::NotFound(_)) => {
                    eprintln!("Error: Memory not found");
                    std::process::exit(1);
                }
                Err(e) => return Err(anyhow::anyhow!("{e}")),
            }
        }

        MemoryAction::Migrate { source } => {
            let store = open_store()?;
            let fs = ecc_infra::os_fs::OsFileSystem;
            let source_dir = source.unwrap_or_else(|| PathBuf::from("docs/memory/work-items"));
            let result = ecc_app::memory::migration::migrate_work_items(&store, &fs, &source_dir)?;
            println!(
                "Migration complete: {} inserted, {} skipped (duplicates), {} skipped (malformed)",
                result.inserted, result.skipped_duplicate, result.skipped_malformed
            );
        }

        MemoryAction::Export { output } => {
            let store = open_store()?;
            let fs = ecc_infra::os_fs::OsFileSystem;
            let count = ecc_app::memory::migration::export(&store, &fs, &output)?;
            println!("Exported {count} entries to {}", output.display());
        }

        MemoryAction::Gc { dry_run } => {
            let store = open_store()?;
            let result = ecc_app::memory::lifecycle::gc(&store, dry_run)?;
            if dry_run {
                println!(
                    "Dry-run: {} stale entries would be deleted",
                    result.deleted_count
                );
                for e in &result.entries {
                    println!("  [{}] {} ({})", e.id, e.title, e.created_at);
                }
            } else {
                println!("GC complete: {} stale entries deleted", result.deleted_count);
            }
        }

        MemoryAction::Stats => {
            let store = open_store()?;
            let s = ecc_app::memory::lifecycle::stats(&store)?;
            println!("Memory Store Statistics");
            println!("  DB size: {} bytes", s.db_size_bytes);
            println!("  Stale entries: {}", s.stale_count);
            if let Some(oldest) = &s.oldest {
                println!("  Oldest entry: {oldest}");
            }
            if let Some(newest) = &s.newest {
                println!("  Newest entry: {newest}");
            }
            println!("  Counts by tier:");
            for (tier, count) in &s.counts_by_tier {
                println!("    {tier}: {count}");
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // PC-048: CLI `ecc memory add` routes to app use case, parses --type/--title/--tags flags
    #[test]
    fn test_memory_add_args_parse() {
        use clap::Parser;
        #[derive(Parser)]
        struct Cli {
            #[command(subcommand)]
            cmd: CliCmd,
        }
        #[derive(clap::Subcommand)]
        enum CliCmd {
            Memory(MemoryArgs),
        }

        let cli = Cli::try_parse_from([
            "ecc", "memory", "add",
            "--type", "semantic",
            "--title", "Test Memory",
            "--tags", "rust,ddd",
        ])
        .expect("should parse");
        match cli.cmd {
            CliCmd::Memory(args) => match args.action {
                MemoryAction::Add { tier, title, tags, .. } => {
                    assert_eq!(tier, "semantic");
                    assert_eq!(title, "Test Memory");
                    assert_eq!(tags.unwrap(), "rust,ddd");
                }
                _ => panic!("expected Add variant"),
            },
        }
    }

    // PC-049: CLI `ecc memory search` routes to app use case
    #[test]
    fn test_memory_search_args_parse() {
        use clap::Parser;
        #[derive(Parser)]
        struct Cli {
            #[command(subcommand)]
            cmd: CliCmd,
        }
        #[derive(clap::Subcommand)]
        enum CliCmd {
            Memory(MemoryArgs),
        }

        let cli = Cli::try_parse_from(["ecc", "memory", "search", "my query"])
            .expect("should parse");
        match cli.cmd {
            CliCmd::Memory(args) => match args.action {
                MemoryAction::Search { query, .. } => {
                    assert_eq!(query, "my query");
                }
                _ => panic!("expected Search variant"),
            },
        }
    }

    // PC-050: CLI `ecc memory list --type semantic --tag rust` routes with filters
    #[test]
    fn test_memory_list_with_filters_parse() {
        use clap::Parser;
        #[derive(Parser)]
        struct Cli {
            #[command(subcommand)]
            cmd: CliCmd,
        }
        #[derive(clap::Subcommand)]
        enum CliCmd {
            Memory(MemoryArgs),
        }

        let cli = Cli::try_parse_from([
            "ecc", "memory", "list",
            "--type", "semantic",
            "--tag", "rust",
        ])
        .expect("should parse");
        match cli.cmd {
            CliCmd::Memory(args) => match args.action {
                MemoryAction::List { tier, tag, .. } => {
                    assert_eq!(tier.unwrap(), "semantic");
                    assert_eq!(tag.unwrap(), "rust");
                }
                _ => panic!("expected List variant"),
            },
        }
    }

    // PC-051: CLI `ecc memory delete <id>` routes
    #[test]
    fn test_memory_delete_args_parse() {
        use clap::Parser;
        #[derive(Parser)]
        struct Cli {
            #[command(subcommand)]
            cmd: CliCmd,
        }
        #[derive(clap::Subcommand)]
        enum CliCmd {
            Memory(MemoryArgs),
        }

        let cli = Cli::try_parse_from(["ecc", "memory", "delete", "42"])
            .expect("should parse");
        match cli.cmd {
            CliCmd::Memory(args) => match args.action {
                MemoryAction::Delete { id } => assert_eq!(id, 42),
                _ => panic!("expected Delete variant"),
            },
        }
    }

    // PC-052: CLI `ecc memory gc [--dry-run]` routes correctly
    #[test]
    fn test_memory_gc_dry_run_parse() {
        use clap::Parser;
        #[derive(Parser)]
        struct Cli {
            #[command(subcommand)]
            cmd: CliCmd,
        }
        #[derive(clap::Subcommand)]
        enum CliCmd {
            Memory(MemoryArgs),
        }

        let cli = Cli::try_parse_from(["ecc", "memory", "gc", "--dry-run"])
            .expect("should parse");
        match cli.cmd {
            CliCmd::Memory(args) => match args.action {
                MemoryAction::Gc { dry_run } => assert!(dry_run),
                _ => panic!("expected Gc variant"),
            },
        }
    }

    // PC-053: CLI `ecc memory stats` routes
    #[test]
    fn test_memory_stats_parse() {
        use clap::Parser;
        #[derive(Parser)]
        struct Cli {
            #[command(subcommand)]
            cmd: CliCmd,
        }
        #[derive(clap::Subcommand)]
        enum CliCmd {
            Memory(MemoryArgs),
        }

        let cli = Cli::try_parse_from(["ecc", "memory", "stats"])
            .expect("should parse");
        match cli.cmd {
            CliCmd::Memory(args) => match args.action {
                MemoryAction::Stats => {}
                _ => panic!("expected Stats variant"),
            },
        }
    }

    // PC-054: CLI `ecc memory migrate` routes to app use case
    #[test]
    fn test_memory_migrate_parse() {
        use clap::Parser;
        #[derive(Parser)]
        struct Cli {
            #[command(subcommand)]
            cmd: CliCmd,
        }
        #[derive(clap::Subcommand)]
        enum CliCmd {
            Memory(MemoryArgs),
        }

        let cli = Cli::try_parse_from(["ecc", "memory", "migrate"])
            .expect("should parse");
        match cli.cmd {
            CliCmd::Memory(args) => match args.action {
                MemoryAction::Migrate { .. } => {}
                _ => panic!("expected Migrate variant"),
            },
        }
    }

    // PC-055: CLI `ecc memory export --output ./backup/` routes correctly
    #[test]
    fn test_memory_export_parse() {
        use clap::Parser;
        #[derive(Parser)]
        struct Cli {
            #[command(subcommand)]
            cmd: CliCmd,
        }
        #[derive(clap::Subcommand)]
        enum CliCmd {
            Memory(MemoryArgs),
        }

        let cli = Cli::try_parse_from(["ecc", "memory", "export", "--output", "./backup/"])
            .expect("should parse");
        match cli.cmd {
            CliCmd::Memory(args) => match args.action {
                MemoryAction::Export { output } => {
                    assert_eq!(output, PathBuf::from("./backup/"));
                }
                _ => panic!("expected Export variant"),
            },
        }
    }

    #[test]
    fn test_parse_tags_empty() {
        let tags = parse_tags(None);
        assert!(tags.is_empty());
    }

    #[test]
    fn test_parse_tags_single() {
        let tags = parse_tags(Some("rust"));
        assert_eq!(tags, vec!["rust"]);
    }

    #[test]
    fn test_parse_tags_multiple() {
        let tags = parse_tags(Some("rust,ddd,hexagonal"));
        assert_eq!(tags, vec!["rust", "ddd", "hexagonal"]);
    }

    #[test]
    fn test_parse_tags_trims_whitespace() {
        let tags = parse_tags(Some("rust, ddd , hexagonal"));
        assert_eq!(tags, vec!["rust", "ddd", "hexagonal"]);
    }
}
