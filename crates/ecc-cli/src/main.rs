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
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging: --verbose sets RUST_LOG=debug unless already set.
    if cli.verbose && std::env::var_os("RUST_LOG").is_none() {
        // SAFETY: Called before any threads are spawned (start of main).
        unsafe { std::env::set_var("RUST_LOG", "debug") };
    }
    env_logger::init();

    match cli.command {
        Command::Version => commands::version::run(),
        Command::Install(args) => commands::install::run(args),
        Command::Init(args) => commands::init::run(args),
        Command::Audit(args) => commands::audit::run(args),
        Command::Completion(args) => commands::completion::run(args),
        Command::Hook(args) => commands::hook::run(args),
        Command::Validate(args) => commands::validate::run(args),
        Command::Claw(args) => commands::claw::run(args),
    }
}
