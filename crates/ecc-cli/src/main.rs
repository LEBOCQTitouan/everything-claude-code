mod commands;

use clap::{ArgAction, Parser};

#[derive(Parser)]
#[command(
    name = "ecc",
    about = "Everything Claude Code — CLI for setting up Claude Code configuration",
    version
)]
struct Cli {
    /// Increase verbosity (-v info, -vv debug, -vvv trace)
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count, global = true, conflicts_with = "quiet")]
    verbose: u8,

    /// Quiet mode (errors only)
    #[arg(short = 'q', long = "quiet", global = true, conflicts_with = "verbose")]
    quiet: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Show the current ECC version
    Version,
    /// Install ECC configuration to ~/.claude/
    Install(commands::install::InstallArgs),
    /// Initialize ECC in the current project
    Init(commands::init::InitArgs),
    /// Audit ECC configuration health
    Audit(commands::audit::AuditArgs),
    /// Generate shell completions
    Completion(commands::completion::CompletionArgs),
    /// Run a hook by ID
    Hook(commands::hook::HookArgs),
    /// Validate content files
    Validate(commands::validate::ValidateArgs),
    /// NanoClaw interactive REPL
    Claw(commands::claw::ClawArgs),
    /// Toggle ECC config on/off
    Dev(commands::dev::DevArgs),
    /// Deterministic backlog management
    Backlog(commands::backlog::BacklogArgs),
    /// Manage git worktrees for ECC sessions
    Worktree(commands::worktree::WorktreeArgs),
    /// Manage knowledge sources registry
    Sources(commands::sources::SourcesArgs),
    /// Query and manage structured logs
    Log(commands::log::LogArgs),
    /// Manage the structured memory store
    Memory(commands::memory::MemoryArgs),
    /// Show ECC status (workflow, versions, components)
    Status(commands::status::StatusArgs),
    /// Manage ECC configuration preferences
    Config(commands::config::ConfigArgs),
}

/// Initialize tracing with layered subscriber: stderr + optional JSON file.
fn init_tracing(env_filter: tracing_subscriber::EnvFilter) {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_filter(env_filter);

    // JSON rolling file layer (debug+ regardless of user filter)
    let logs_dir = std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_default()
        .join(".ecc")
        .join("logs");

    let json_layer = if logs_dir
        .parent()
        .is_some_and(|p| p.exists())
    {
        std::fs::create_dir_all(&logs_dir).ok();
        let file_appender = tracing_appender::rolling::daily(&logs_dir, "ecc");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        // Leak the guard so it lives for the process lifetime
        std::mem::forget(guard);
        Some(
            tracing_subscriber::fmt::layer()
                .json()
                .with_writer(non_blocking)
                .with_filter(
                    tracing_subscriber::EnvFilter::new("debug"),
                ),
        )
    } else {
        None
    };

    // Session correlation: log first event with session_id
    let session_id = std::env::var("CLAUDE_SESSION_ID").unwrap_or_else(|_| {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let pid = std::process::id();
        format!("ecc-{secs}-{pid:04x}")
    });

    tracing_subscriber::registry()
        .with(stderr_layer)
        .with(json_layer)
        .init();

    tracing::info!(session_id = %session_id, "ECC session started");
}

/// Auto-prune logs older than 30 days at startup.
fn auto_prune_logs() -> anyhow::Result<()> {
    let logs_dir = std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_default()
        .join(".ecc")
        .join("logs");
    let db_path = logs_dir.join("ecc.db");
    if !db_path.exists() {
        return Ok(());
    }
    let store = ecc_infra::sqlite_log_store::SqliteLogStore::new(&db_path)?;
    let fs = ecc_infra::os_fs::OsFileSystem;
    let retention = std::time::Duration::from_secs(30 * 86400);
    let result = ecc_app::log_mgmt::prune(&store, &fs, &logs_dir, retention)?;
    tracing::info!(
        db_rows = result.db_rows,
        json_files = result.json_files,
        "auto-prune complete"
    );
    Ok(())
}

/// Resolve the tracing filter string from verbosity flags, env vars, and config.
fn resolve_log_filter(verbose: u8, quiet: bool) -> String {
    use ecc_infra::file_config_store::FileConfigStore;
    let config_store = FileConfigStore::new(
        dirs::home_dir().unwrap_or_default().join(".ecc"),
        std::env::current_dir().ok().map(|d| d.join(".ecc")),
    );
    let level = ecc_app::config_cmd::resolve_log_level(
        verbose,
        quiet,
        std::env::var("ECC_LOG").ok().as_deref(),
        std::env::var("RUST_LOG").ok().as_deref(),
        &config_store,
    );
    level.to_string()
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize tracing with resolved log level
    let filter = resolve_log_filter(cli.verbose, cli.quiet);
    let env_filter = tracing_subscriber::EnvFilter::try_new(&filter)
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"));

    // Compose layered subscriber: stderr + optional JSON rolling file
    init_tracing(env_filter);

    // Auto-prune old logs (non-fatal)
    if let Err(e) = auto_prune_logs() {
        tracing::warn!("auto-prune failed: {e}");
    }

    match cli.command {
        Command::Version => commands::version::run(),
        Command::Install(args) => commands::install::run(args),
        Command::Init(args) => commands::init::run(args),
        Command::Audit(args) => commands::audit::run(args),
        Command::Completion(args) => commands::completion::run(args),
        Command::Hook(args) => commands::hook::run(args),
        Command::Validate(args) => commands::validate::run(args),
        Command::Claw(args) => commands::claw::run(args),
        Command::Dev(args) => commands::dev::run(args),
        Command::Backlog(args) => commands::backlog::run(args),
        Command::Worktree(args) => commands::worktree::run(args),
        Command::Sources(args) => commands::sources::run(args),
        Command::Log(args) => commands::log::run(args),
        Command::Memory(args) => commands::memory::run(args),
        Command::Status(args) => commands::status::run(args),
        Command::Config(args) => commands::config::run(args),
    }
}
