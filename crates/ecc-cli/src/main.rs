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
    /// Manage the three-tier memory system
    Memory(commands::memory::MemoryArgs),
    /// Show ECC diagnostic status
    Status(commands::status::StatusArgs),
    /// Manage ECC configuration
    Config(commands::config::ConfigArgs),
}

fn init_tracing(verbose: u8, quiet: bool) {
    let config_store = ecc_infra::file_config_store::FileConfigStore::new(
        dirs::home_dir()
            .unwrap_or_default()
            .join(".ecc"),
        std::env::current_dir().ok().map(|d| d.join(".ecc")),
    );

    let level = ecc_app::config_cmd::resolve_log_level(
        verbose,
        quiet,
        std::env::var("ECC_LOG").ok().as_deref(),
        std::env::var("RUST_LOG").ok().as_deref(),
        &config_store,
    );

    let filter = tracing_subscriber::EnvFilter::new(level.to_string());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    init_tracing(cli.verbose, cli.quiet);

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
        Command::Memory(args) => commands::memory::run(args),
        Command::Status(args) => commands::status::run(args),
        Command::Config(args) => commands::config::run(args),
    }
}
