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
    Init {
        concern: String,
        feature: String,
    },
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
    /// Compare git diff against expected files in the design's File Changes table.
    /// Warns about unexpected files but always exits 0.
    ScopeCheck,
    /// Check implement-done.md for required documentation sections at "done" phase.
    /// Warns on stderr if sections are missing, always exits 0.
    DocEnforcement,
    /// Warn when CLAUDE.md, README.md, or ARCHITECTURE.md exceed recommended size limits.
    /// Only runs at "done" phase. Always exits 0 — informational only.
    DocLevelCheck,
    /// Check implement-done.md for pass condition results at "done" phase.
    /// Warns on stderr if the section is missing or failures are found, always exits 0.
    PassConditionCheck,
    /// Check implement-done.md for an "## E2E Tests" section at "done" phase.
    /// Warns on stderr if the section is missing, always exits 0.
    E2eBoundaryCheck,
    /// Generate a new git worktree name for session isolation.
    /// Generate a session-isolated git worktree name from concern and feature.
    WorktreeName {
        concern: String,
        feature: String,
    },
    /// Merge a session worktree branch into main after verify + rebase.
    Merge,
    /// Atomically add an entry to docs/backlog/ with flock-based locking.
    Backlog {
        #[command(subcommand)]
        subcmd: BacklogCmd,
    },
}

#[derive(Subcommand)]
enum BacklogCmd {
    /// Add a new backlog entry atomically with flock locking.
    AddEntry {
        /// Entry title
        title: String,
        #[arg(long, default_value = "MEDIUM")]
        scope: String,
        #[arg(long, default_value = "/spec-dev")]
        target: String,
        #[arg(long, default_value = "")]
        tags: String,
    },
}

fn main() {
    if std::env::var("ECC_WORKFLOW_BYPASS").as_deref() == Ok("1") {
        std::process::exit(0);
    }

    // Initialize logging. RUST_LOG controls the level; default is off to avoid
    // polluting hook JSON output in normal operation.
    env_logger::init();

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
    log::debug!(
        "dispatching command: {:?}",
        std::mem::discriminant(&cli.command)
    );
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
        Commands::ScopeCheck => commands::scope_check::run(&project_dir()),
        Commands::DocEnforcement => commands::doc_enforcement::run(&project_dir()),
        Commands::DocLevelCheck => commands::doc_level_check::run(&project_dir()),
        Commands::PassConditionCheck => commands::pass_condition_check::run(&project_dir()),
        Commands::E2eBoundaryCheck => commands::e2e_boundary_check::run(&project_dir()),
        Commands::WorktreeName { concern, feature } => {
            commands::worktree_name::run(&concern, &feature)
        }
        Commands::Merge => commands::merge::run(&project_dir()),
        Commands::Backlog { subcmd } => match subcmd {
            BacklogCmd::AddEntry {
                title,
                scope,
                target,
                tags,
            } => commands::backlog::run(&title, &scope, &target, &tags, &project_dir()),
        },
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
