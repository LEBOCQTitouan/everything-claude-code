//! `ecc` binary entrypoint: parses CLI args, initializes tracing, prunes old
//! logs, and dispatches to the matching `commands::*::run` function.
//!
//! # Startup pipeline
//!
//! ```text
//! [argv]
//!   |
//!   v
//! [Cli::parse]  -- clap derive on ecc_cli::Cli
//!   |
//!   v
//! [resolve_log_filter]  -- merges -v/-q flags, ECC_LOG, RUST_LOG, config
//!   |
//!   v
//! [init_tracing]  -- stderr fmt layer + rolling JSON file under ~/.ecc/logs
//!   |
//!   v
//! [auto_prune_logs]  -- non-fatal: drop DB rows + files > 30 days
//!   |
//!   v
//! [match cli.command] --> commands::<sub>::run(args) -> anyhow::Result<()>
//! ```
//!
//! Every subcommand arm is a 1:1 mapping — no business logic lives here.

use clap::Parser;
use ecc_cli::{Cli, Command, commands};

/// Initialize tracing with layered subscriber: stderr + optional JSON file.
fn init_tracing(env_filter: tracing_subscriber::EnvFilter) {
    use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};

    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_filter(env_filter);

    // JSON rolling file layer (debug+ regardless of user filter)
    let logs_dir = std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_default()
        .join(".ecc")
        .join("logs");

    let json_layer = if logs_dir.parent().is_some_and(|p| p.exists()) {
        std::fs::create_dir_all(&logs_dir).ok();
        let file_appender = tracing_appender::rolling::daily(&logs_dir, "ecc");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        // Leak the guard so it lives for the process lifetime
        std::mem::forget(guard);
        Some(
            tracing_subscriber::fmt::layer()
                .json()
                .with_writer(non_blocking)
                .with_filter(tracing_subscriber::EnvFilter::new("debug")),
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
        Command::Analyze(args) => commands::analyze::run(args),
        Command::Version => commands::version::run(),
        Command::Install(args) => commands::install::run(args),
        Command::Init(args) => commands::init::run(args),
        Command::Audit(args) => commands::audit::run(args),
        Command::AuditCache(args) => commands::audit_cache::run(args),
        Command::Completion(args) => commands::completion::run(args),
        Command::Hook(args) => commands::hook::run(args),
        Command::Validate(args) => commands::validate::run(args),
        Command::Claw(args) => commands::claw::run(args),
        Command::Dev(args) => commands::dev::run(args),
        Command::Backlog(args) => commands::backlog::run(args),
        Command::Drift(args) => commands::drift::run(args),
        Command::Docs(args) => commands::docs::run(args),
        Command::Diagram(args) => commands::diagram::run(args),
        Command::CommitCmd(args) => commands::commit_cmd::run(args),
        Command::Bypass(args) => commands::bypass::run(args),
        Command::Worktree(args) => commands::worktree::run(args),
        Command::Sources(args) => commands::sources::run(args),
        Command::Log(args) => commands::log::run(args),
        Command::Memory(args) => commands::memory::run(args),
        Command::AuditWeb(args) => commands::audit_web::run(args),
        Command::Status(args) => commands::status::run(args),
        Command::Config(args) => commands::config::run(args),
        Command::Update(args) => commands::update::run(&args),
        Command::Workflow(args) => commands::workflow::run(args),
        Command::Cost(args) => commands::cost::run(args),
        Command::Metrics(args) => commands::metrics::run(args),
    }
}
