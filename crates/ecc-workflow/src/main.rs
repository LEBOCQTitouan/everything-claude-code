use clap::{Parser, Subcommand};

use output::WorkflowOutput;

mod commands;
mod io;
mod output;
mod slug;
mod time;

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
    /// Gate Write/Edit/MultiEdit and destructive Bash commands during plan/solution phases.
    /// Reads hook protocol JSON from stdin.
    PhaseGate,
    /// Warn on stderr when the workflow is in an incomplete phase.
    /// Called by the Stop hook at the end of a Claude session.
    /// Always exits 0 — informational only.
    StopGate,
    /// Warn when the spec/campaign file lacks a grill-me interview section.
    /// Always exits 0 — informational only.
    GrillMeGate,
    /// Track TDD RED/GREEN/REFACTOR state during implement phase.
    /// Reads hook protocol JSON from stdin.
    /// Always exits 0 — informational only.
    TddEnforcement,
    /// Show current workflow phase, feature, and artifact paths.
    Status,
    /// Resolve and validate an artifact path (spec, design, tasks, campaign).
    Artifact {
        /// Artifact type: spec, design, tasks, campaign
        artifact_type: String,
    },
    /// Reset workflow to idle state. Requires --force.
    Reset {
        /// Confirm reset (required)
        #[arg(long)]
        force: bool,
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
        Commands::PhaseGate => commands::phase_gate::run(&project_dir()),
        Commands::StopGate => commands::stop_gate::run(&project_dir()),
        Commands::GrillMeGate => commands::grill_me_gate::run(&project_dir()),
        Commands::TddEnforcement => commands::tdd_enforcement::run(&project_dir()),
        Commands::Status => commands::status::run(&project_dir()),
        Commands::Artifact { artifact_type } => {
            commands::artifact::run(&artifact_type, &project_dir())
        }
        Commands::Reset { force } => commands::reset::run(force, &project_dir()),
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
            // Only print if message is non-empty (stop-gate uses empty pass for silence)
            if !output.message.is_empty() {
                println!("{output}");
            }
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
