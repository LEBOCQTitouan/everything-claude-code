mod commands;

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "ecc",
    about = "Everything Claude Code — CLI for setting up Claude Code configuration",
    version
)]
struct Cli {
    /// Increase verbosity (-v info, -vv debug, -vvv trace)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    /// Quiet mode (errors only)
    #[arg(short, long, global = true)]
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
    /// Show ECC status (workflow, versions, components)
    Status,
    /// Manage ECC configuration preferences
    Config(commands::config::ConfigArgs),
}

/// Resolve the effective log level filter string.
///
/// Precedence: CLI flag > ECC_LOG > RUST_LOG (deprecated) > config file > default (warn)
fn resolve_log_filter(verbose: u8, quiet: bool) -> String {
    // 1. CLI flags (highest priority)
    if quiet {
        return "error".to_string();
    }
    if verbose >= 3 {
        return "trace".to_string();
    }
    if verbose >= 2 {
        return "debug".to_string();
    }
    if verbose >= 1 {
        return "info".to_string();
    }

    // 2. ECC_LOG env var
    if let Ok(val) = std::env::var("ECC_LOG") {
        return val;
    }

    // 3. RUST_LOG env var (deprecated, with warning)
    if let Ok(val) = std::env::var("RUST_LOG") {
        eprintln!("warning: RUST_LOG is deprecated for ECC, use ECC_LOG instead");
        return val;
    }

    // 4. Config file (~/.ecc/config.toml)
    if let Some(level) = read_config_log_level() {
        return level;
    }

    // 5. Default
    "warn".to_string()
}

/// Read log-level from ~/.ecc/config.toml if it exists.
fn read_config_log_level() -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let config_path = std::path::Path::new(&home).join(".ecc/config.toml");
    let content = std::fs::read_to_string(config_path).ok()?;
    let config = ecc_domain::config::ecc_config::EccConfig::from_toml(&content).ok()?;
    config.log_level.map(|l| l.to_string())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize tracing with resolved log level
    let filter = resolve_log_filter(cli.verbose, cli.quiet);
    let env_filter = tracing_subscriber::EnvFilter::try_new(&filter)
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .init();

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
        Command::Status => commands::status::run(),
        Command::Config(args) => commands::config::run(args),
    }
}
