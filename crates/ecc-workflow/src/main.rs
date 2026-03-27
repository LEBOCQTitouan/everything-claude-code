use clap::{Parser, Subcommand};

use output::WorkflowOutput;

mod commands;
mod io;
mod output;
mod slug;

#[derive(Parser)]
#[command(name = "ecc-workflow", about = "ECC workflow state machine")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init { concern: String, feature: String },
    Transition {
        target: String,
        #[arg(long)]
        artifact: Option<String>,
        #[arg(long)]
        path: Option<String>,
    },
    ToolchainPersist {
        test_cmd: String,
        lint_cmd: String,
        build_cmd: String,
    },
    MemoryWrite {
        /// Subcommand kind: action, work-item, daily, memory-index
        kind: String,
        /// Remaining arguments for the subcommand
        args: Vec<String>,
    },
}

fn main() {
    if std::env::var("ECC_WORKFLOW_BYPASS").as_deref() == Ok("1") {
        std::process::exit(0);
    }
    let cli = Cli::parse();

    let result = dispatch(cli);
    emit_and_exit(result);
}

/// Resolve the project root from `CLAUDE_PROJECT_DIR` env var, falling back to `.`.
fn project_dir() -> std::path::PathBuf {
    std::env::var("CLAUDE_PROJECT_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
}

fn dispatch(cli: Cli) -> WorkflowOutput {
    match cli.command {
        Commands::Init { concern, feature } => {
            commands::init::run(&concern, &feature, &project_dir())
        }
        Commands::Transition {
            target,
            artifact,
            path,
        } => commands::transition::run(
            &target,
            artifact.as_deref(),
            path.as_deref(),
            &project_dir(),
        ),
        Commands::ToolchainPersist {
            test_cmd,
            lint_cmd,
            build_cmd,
        } => commands::toolchain_persist::run(&test_cmd, &lint_cmd, &build_cmd, &project_dir()),
        Commands::MemoryWrite { kind, args } => {
            commands::memory_write::run(&kind, &args, &project_dir())
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
