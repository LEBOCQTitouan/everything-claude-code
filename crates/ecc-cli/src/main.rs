mod commands;

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "ecc",
    about = "Everything Claude Code — CLI for setting up Claude Code configuration",
    version
)]
enum Cli {
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

    match cli {
        Cli::Version => commands::version::run(),
        Cli::Install(args) => commands::install::run(args),
        Cli::Init(args) => commands::init::run(args),
        Cli::Audit(args) => commands::audit::run(args),
        Cli::Completion(args) => commands::completion::run(args),
        Cli::Hook(args) => commands::hook::run(args),
        Cli::Validate(args) => commands::validate::run(args),
        Cli::Claw(args) => commands::claw::run(args),
    }
}
