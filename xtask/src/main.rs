use clap::{Parser, Subcommand};

mod deploy;
mod rc_block;
mod shell;

#[derive(Parser)]
#[command(name = "xtask", about = "ECC developer tooling")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Deploy ECC to the local machine
    Deploy {
        /// Preview actions without performing them
        #[arg(long)]
        dry_run: bool,
        /// Build in debug mode (faster, no --release)
        #[arg(long)]
        debug: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Deploy { dry_run, debug } => deploy::run(dry_run, debug),
    }
}
