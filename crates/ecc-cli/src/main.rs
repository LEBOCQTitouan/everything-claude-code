mod commands;

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "ecc",
    about = "Everything Claude Code — CLI for setting up Claude Code configuration",
    version
)]
struct Cli {
    /// Enable verbose output (RUST_LOG=debug)
    #[arg(short, long, global = true)]
    verbose: bool,

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
    /// Knowledge sources registry
    Sources(commands::sources::SourcesArgs),
    /// Manage git worktrees for ECC sessions
    Worktree(commands::worktree::WorktreeArgs),
    /// Manage knowledge sources registry
    Sources(commands::sources::SourcesArgs),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging:
    // - Default filter is "warn" (RUST_LOG overrides this).
    // - --verbose flag elevates to "debug" when RUST_LOG is not set.
    let env = env_logger::Env::default().default_filter_or("warn");
    let mut log_builder = env_logger::Builder::from_env(env);
    if cli.verbose && std::env::var_os("RUST_LOG").is_none() {
        log_builder.filter_level(log::LevelFilter::Debug);
    }
    log_builder.init();

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
        Command::Sources(args) => commands::sources::run(args),
        Command::Worktree(args) => commands::worktree::run(args),
        Command::Sources(args) => commands::sources::run(args),
    }
}
