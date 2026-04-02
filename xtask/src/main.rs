use clap::{Parser, Subcommand};

mod deploy;
mod mutants;
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
    /// Run mutation testing via cargo-mutants
    Mutants {
        /// Packages to test (default: ecc-domain, ecc-app)
        #[arg(long, short)]
        package: Vec<String>,
        /// Per-mutant timeout in seconds
        #[arg(long)]
        timeout: Option<u64>,
        /// Run mutations only on code changed vs origin/main
        #[arg(long)]
        in_diff: bool,
        /// Use cargo-nextest as test runner (default: true)
        #[arg(long, default_value_t = true)]
        nextest: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Deploy { dry_run, debug } => deploy::run(dry_run, debug),
        Commands::Mutants {
            package,
            timeout,
            in_diff,
            nextest,
        } => {
            let packages = if package.is_empty() {
                vec!["ecc-domain".to_string(), "ecc-app".to_string()]
            } else {
                package
            };
            mutants::run(&packages, timeout, in_diff, nextest)
        }
    }
}
