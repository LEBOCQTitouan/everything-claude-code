//! CLI command: `ecc workflow <subcommand>`
//!
//! Mirrors all ecc-workflow subcommands. Delegates to the ecc-workflow binary
//! for behavioral parity during migration. After the cleanup PR, these will
//! be implemented directly using port traits.

use clap::{Args, Subcommand};
use std::process::Command;

#[derive(Args)]
pub struct WorkflowArgs {
    #[command(subcommand)]
    pub command: WorkflowCommand,

    /// Increase verbosity
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Quiet mode
    #[arg(short = 'q', long = "quiet", global = true)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum WorkflowCommand {
    /// Initialize workflow state for a new session
    Init { concern: String, feature: String },
    /// Advance the workflow to the target phase
    Transition {
        target: String,
        #[arg(long)]
        artifact: Option<String>,
        #[arg(long)]
        path: Option<String>,
        #[arg(long)]
        justify: Option<String>,
    },
    /// Display workflow transition history
    History {
        /// Output as JSON array
        #[arg(long)]
        json: bool,
    },
    /// Persist detected toolchain commands
    ToolchainPersist {
        test_cmd: String,
        lint_cmd: String,
        build_cmd: String,
    },
    /// Write memory entries
    MemoryWrite { kind: String, args: Vec<String> },
    /// Gate Write/Edit during plan/solution phases
    PhaseGate,
    /// Warn on incomplete workflow at session end
    StopGate,
    /// Warn when spec lacks grill-me section
    GrillMeGate,
    /// Track TDD state during implement phase
    TddEnforcement,
    /// Show current workflow state
    Status,
    /// Resolve an artifact path
    Artifact { artifact_type: String },
    /// Reset workflow to idle (requires --force)
    Reset {
        #[arg(long)]
        force: bool,
    },
    /// Compare git diff against expected file changes
    ScopeCheck,
    /// Check for required documentation sections
    DocEnforcement,
    /// Warn when docs exceed size limits
    DocLevelCheck,
    /// Check pass condition results
    PassConditionCheck,
    /// Check E2E test section
    E2eBoundaryCheck,
    /// Generate a worktree name for session isolation
    WorktreeName { concern: String, feature: String },
    /// Compute wave plan from design file
    WavePlan { design_path: String },
    /// Merge session worktree into main
    Merge,
    /// Backlog management
    Backlog {
        #[command(subcommand)]
        subcmd: BacklogSubcommand,
    },
    /// Task synchronization
    Tasks {
        #[command(subcommand)]
        subcmd: TasksSubcommand,
    },
    /// Archive state and reset to idle
    Recover,
}

#[derive(Subcommand)]
pub enum BacklogSubcommand {
    /// Add a new backlog entry
    AddEntry {
        title: String,
        #[arg(long, default_value = "MEDIUM")]
        scope: String,
        #[arg(long, default_value = "/spec-dev")]
        target: String,
        #[arg(long, default_value = "")]
        tags: String,
    },
}

#[derive(Subcommand)]
pub enum TasksSubcommand {
    /// Parse tasks.md and output JSON
    Sync { path: String },
    /// Update a PC status in tasks.md
    Update {
        path: String,
        id: String,
        status: String,
    },
    /// Generate tasks.md from design
    Init {
        design_path: String,
        #[arg(long)]
        output: String,
        #[arg(long)]
        force: bool,
    },
}

pub fn run(args: WorkflowArgs) -> anyhow::Result<()> {
    let mut cmd_args = build_args(&args.command);

    // Forward verbosity flags
    for _ in 0..args.verbose {
        cmd_args.insert(0, "-v".to_owned());
    }
    if args.quiet {
        cmd_args.insert(0, "-q".to_owned());
    }

    delegate_to_ecc_workflow(&cmd_args)
}

/// Build the argument list for the ecc-workflow binary.
fn build_args(command: &WorkflowCommand) -> Vec<String> {
    match command {
        WorkflowCommand::Init { concern, feature } => {
            vec!["init".into(), concern.clone(), feature.clone()]
        }
        WorkflowCommand::Transition {
            target,
            artifact,
            path,
            justify,
        } => {
            let mut args = vec!["transition".into(), target.clone()];
            if let Some(a) = artifact {
                args.push("--artifact".into());
                args.push(a.clone());
            }
            if let Some(p) = path {
                args.push("--path".into());
                args.push(p.clone());
            }
            if let Some(j) = justify {
                args.push("--justify".into());
                args.push(j.clone());
            }
            args
        }
        WorkflowCommand::History { json } => {
            let mut args = vec!["history".into()];
            if *json {
                args.push("--json".into());
            }
            args
        }
        WorkflowCommand::ToolchainPersist {
            test_cmd,
            lint_cmd,
            build_cmd,
        } => vec![
            "toolchain-persist".into(),
            test_cmd.clone(),
            lint_cmd.clone(),
            build_cmd.clone(),
        ],
        WorkflowCommand::MemoryWrite { kind, args } => {
            let mut v = vec!["memory-write".into(), kind.clone()];
            v.extend(args.iter().cloned());
            v
        }
        WorkflowCommand::PhaseGate => vec!["phase-gate".into()],
        WorkflowCommand::StopGate => vec!["stop-gate".into()],
        WorkflowCommand::GrillMeGate => vec!["grill-me-gate".into()],
        WorkflowCommand::TddEnforcement => vec!["tdd-enforcement".into()],
        WorkflowCommand::Status => vec!["status".into()],
        WorkflowCommand::Artifact { artifact_type } => {
            vec!["artifact".into(), artifact_type.clone()]
        }
        WorkflowCommand::Reset { force } => {
            let mut args = vec!["reset".into()];
            if *force {
                args.push("--force".into());
            }
            args
        }
        WorkflowCommand::ScopeCheck => vec!["scope-check".into()],
        WorkflowCommand::DocEnforcement => vec!["doc-enforcement".into()],
        WorkflowCommand::DocLevelCheck => vec!["doc-level-check".into()],
        WorkflowCommand::PassConditionCheck => vec!["pass-condition-check".into()],
        WorkflowCommand::E2eBoundaryCheck => vec!["e2e-boundary-check".into()],
        WorkflowCommand::WorktreeName { concern, feature } => {
            vec!["worktree-name".into(), concern.clone(), feature.clone()]
        }
        WorkflowCommand::WavePlan { design_path } => {
            vec!["wave-plan".into(), design_path.clone()]
        }
        WorkflowCommand::Merge => vec!["merge".into()],
        WorkflowCommand::Backlog { subcmd } => match subcmd {
            BacklogSubcommand::AddEntry {
                title,
                scope,
                target,
                tags,
            } => vec![
                "backlog".into(),
                "add-entry".into(),
                title.clone(),
                "--scope".into(),
                scope.clone(),
                "--target".into(),
                target.clone(),
                "--tags".into(),
                tags.clone(),
            ],
        },
        WorkflowCommand::Tasks { subcmd } => match subcmd {
            TasksSubcommand::Sync { path } => vec!["tasks".into(), "sync".into(), path.clone()],
            TasksSubcommand::Update { path, id, status } => vec![
                "tasks".into(),
                "update".into(),
                path.clone(),
                id.clone(),
                status.clone(),
            ],
            TasksSubcommand::Init {
                design_path,
                output,
                force,
            } => {
                let mut args = vec![
                    "tasks".into(),
                    "init".into(),
                    design_path.clone(),
                    "--output".into(),
                    output.clone(),
                ];
                if *force {
                    args.push("--force".into());
                }
                args
            }
        },
        WorkflowCommand::Recover => vec!["recover".into()],
    }
}

/// Delegate to the ecc-workflow binary, forwarding stdin/stdout/stderr and exit code.
fn delegate_to_ecc_workflow(args: &[String]) -> anyhow::Result<()> {
    let status = Command::new("ecc-workflow")
        .args(args)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}
