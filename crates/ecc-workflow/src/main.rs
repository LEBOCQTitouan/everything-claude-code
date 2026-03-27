use clap::{Parser, Subcommand};

mod commands;
mod io;
mod output;

#[derive(Parser)]
#[command(name = "ecc-workflow", about = "ECC workflow state machine")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init { concern: String, feature: String },
    Transition { target: String },
}

fn main() {
    let _cli = Cli::parse();
    // dispatch will be implemented in later PCs
}
