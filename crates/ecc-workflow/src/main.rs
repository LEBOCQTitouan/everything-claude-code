use clap::{Parser, Subcommand};

use output::WorkflowOutput;

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
    let cli = Cli::parse();

    let result = dispatch(cli);
    emit_and_exit(result);
}

fn dispatch(cli: Cli) -> WorkflowOutput {
    match cli.command {
        Commands::Init { concern, feature } => {
            commands::init::run(&concern, &feature)
        }
        Commands::Transition { target } => {
            WorkflowOutput::warn(format!("transition to '{target}' not yet implemented"))
        }
    }
}

/// Print the output JSON to the appropriate stream and exit with the correct code.
///
/// - pass  → stdout, exit 0
/// - warn  → stderr, exit 0
/// - block → stderr, exit 2
fn emit_and_exit(output: WorkflowOutput) -> ! {
    match output.status {
        output::Status::Pass => {
            println!("{output}");
            std::process::exit(0);
        }
        output::Status::Warn => {
            eprintln!("{output}");
            std::process::exit(0);
        }
        output::Status::Block => {
            eprintln!("{output}");
            std::process::exit(2);
        }
    }
}
