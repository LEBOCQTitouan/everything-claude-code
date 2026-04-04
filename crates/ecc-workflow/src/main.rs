use clap::{ArgAction, Parser, Subcommand};

use output::WorkflowOutput;

mod commands;
mod io;
mod output;
mod slug;
mod time;

#[derive(Parser)]
#[command(name = "ecc-workflow", about = "ECC workflow state machine")]
struct Cli {
    /// Increase verbosity (-v info, -vv debug, -vvv trace)
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count, global = true, conflicts_with = "quiet")]
    verbose: u8,

    /// Quiet mode (errors only)
    #[arg(short = 'q', long = "quiet", global = true, conflicts_with = "verbose")]
    quiet: bool,

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
    /// Compute wave plan from design file's PC and File Changes tables.
    WavePlan {
        /// Path to the design.md file
        design_path: String,
    },
    /// Merge a session worktree branch into main after verify + rebase.
    Merge,
    /// Atomically add an entry to docs/backlog/ with flock-based locking.
    Backlog {
        #[command(subcommand)]
        subcmd: BacklogCmd,
    },
    /// Deterministic task synchronization subcommands.
    Tasks {
        #[command(subcommand)]
        subcmd: TasksCmd,
    },
    /// Campaign manifest management for grill-me decision persistence.
    Campaign {
        #[command(subcommand)]
        subcmd: CampaignCmd,
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

#[derive(Subcommand)]
enum CampaignCmd {
    /// Create campaign.md in the spec directory.
    Init {
        /// Path to the spec directory (e.g., docs/specs/2026-04-04-feature/)
        spec_dir: String,
    },
    /// Append a grill-me decision to campaign.md.
    AppendDecision {
        /// The question asked
        #[arg(long)]
        question: String,
        /// The answer given
        #[arg(long)]
        answer: String,
        /// Source: "recommended" or "user"
        #[arg(long)]
        source: String,
    },
    /// Show campaign.md content as JSON.
    Show,
}

#[derive(Subcommand)]
enum TasksCmd {
    /// Parse tasks.md and output JSON summary.
    Sync {
        /// Path to tasks.md file
        path: String,
    },
    /// Atomically update a PC's status in tasks.md.
    Update {
        /// Path to tasks.md file
        path: String,
        /// Entry ID (e.g., "PC-001" or "E2E tests")
        id: String,
        /// New status (pending, red, green, done, failed)
        status: String,
    },
    /// Generate tasks.md from a design file's PC table.
    Init {
        /// Path to design.md file
        design_path: String,
        /// Output path for tasks.md
        #[arg(long)]
        output: String,
        /// Overwrite existing output file
        #[arg(long)]
        force: bool,
    },
}

fn init_tracing(verbose: u8, quiet: bool) {
    let config_store = ecc_infra::file_config_store::FileConfigStore::new(
        dirs::home_dir().unwrap_or_default().join(".ecc"),
        std::env::current_dir().ok().map(|d| d.join(".ecc")),
    );

    let level = ecc_app::config_cmd::resolve_log_level(
        verbose,
        quiet,
        std::env::var("ECC_LOG").ok().as_deref(),
        std::env::var("RUST_LOG").ok().as_deref(),
        &config_store,
    );

    let filter = tracing_subscriber::EnvFilter::new(level.to_string());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();
}

fn main() {
    if std::env::var("ECC_WORKFLOW_BYPASS").as_deref() == Ok("1") {
        std::process::exit(0);
    }

    let cli = Cli::parse();

    init_tracing(cli.verbose, cli.quiet);

    let result = dispatch(cli);
    emit_and_exit(result);
}

/// Resolve the project root from `CLAUDE_PROJECT_DIR` env var, falling back to `.`.
fn project_dir() -> std::path::PathBuf {
    std::env::var("CLAUDE_PROJECT_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
}

/// Resolve the worktree-scoped state directory via env, git, and filesystem.
fn resolve_state_dir() -> std::path::PathBuf {
    let env = ecc_infra::os_env::OsEnvironment;
    let git = ecc_infra::os_git::OsGitInfo;
    let fs = ecc_infra::os_fs::OsFileSystem;
    let (state_dir, warnings) =
        ecc_app::workflow::state_resolver::resolve_state_dir(&env, &git, &fs);
    for w in &warnings {
        tracing::debug!("state_resolver warning: {}", w.message);
    }
    state_dir
}

/// Migrate state from `.claude/workflow/` to the worktree-scoped location if needed.
/// Only acquires the flock if migration appears necessary (TOCTOU-safe: re-checks under lock).
fn migrate_state_if_needed(project_dir: &std::path::Path, state_dir: &std::path::Path) {
    let old_dir = project_dir.join(".claude/workflow");
    if old_dir == *state_dir {
        return;
    }
    // Quick pre-check without lock — skip if new state already exists or old doesn't
    let new_state = state_dir.join("state.json");
    let old_state = old_dir.join("state.json");
    if new_state.exists() || !old_state.exists() {
        return;
    }
    // Lock only when migration looks needed
    let fs = ecc_infra::os_fs::OsFileSystem;
    let _guard = match ecc_flock::acquire_for(state_dir, "state") {
        Ok(g) => g,
        Err(e) => {
            eprintln!("[ecc-workflow] Cannot acquire lock for migration: {e}");
            return;
        }
    };
    // Re-check under lock (TOCTOU-safe)
    match ecc_app::workflow::state_resolver::migrate_if_needed(&old_dir, state_dir, &fs) {
        Ok(true) => eprintln!("[ecc-workflow] State migration complete"),
        Ok(false) => {}
        Err(e) => eprintln!("[ecc-workflow] State migration failed: {e}"),
    }
}

fn dispatch(cli: Cli) -> WorkflowOutput {
    tracing::debug!(
        "dispatching command: {:?}",
        std::mem::discriminant(&cli.command)
    );
    let proj = project_dir();
    let sd = resolve_state_dir();
    migrate_state_if_needed(&proj, &sd);
    match cli.command {
        Commands::Init { concern, feature } => commands::init::run(&concern, &feature, &proj, &sd),
        Commands::Transition {
            target,
            artifact,
            path,
        } => commands::transition::run(&target, artifact.as_deref(), path.as_deref(), &proj, &sd),
        Commands::ToolchainPersist {
            test_cmd,
            lint_cmd,
            build_cmd,
        } => commands::toolchain_persist::run(&test_cmd, &lint_cmd, &build_cmd, &sd),
        Commands::MemoryWrite { kind, args } => commands::memory_write::run(&kind, &args, &proj),
        Commands::PhaseGate => commands::phase_gate::run(&proj, &sd),
        Commands::StopGate => commands::stop_gate::run(&sd),
        Commands::GrillMeGate => commands::grill_me_gate::run(&sd),
        Commands::TddEnforcement => commands::tdd_enforcement::run(&sd),
        Commands::Status => commands::status::run(&sd),
        Commands::Artifact { artifact_type } => commands::artifact::run(&artifact_type, &proj, &sd),
        Commands::Reset { force } => commands::reset::run(force, &sd),
        Commands::ScopeCheck => commands::scope_check::run(&proj, &sd),
        Commands::DocEnforcement => commands::doc_enforcement::run(&sd),
        Commands::DocLevelCheck => commands::doc_level_check::run(&proj, &sd),
        Commands::PassConditionCheck => commands::pass_condition_check::run(&sd),
        Commands::E2eBoundaryCheck => commands::e2e_boundary_check::run(&sd),
        Commands::WorktreeName { concern, feature } => {
            commands::worktree_name::run(&concern, &feature)
        }
        Commands::WavePlan { design_path } => commands::wave_plan::run(&design_path, &proj),
        Commands::Merge => commands::merge::run(&proj, &sd),
        Commands::Backlog { subcmd } => match subcmd {
            BacklogCmd::AddEntry {
                title,
                scope,
                target,
                tags,
            } => commands::backlog::run(&title, &scope, &target, &tags, &proj),
        },
        Commands::Campaign { subcmd } => match subcmd {
            CampaignCmd::Init { spec_dir } => {
                commands::campaign::run_init(&spec_dir, &project_dir())
            }
            CampaignCmd::AppendDecision {
                question,
                answer,
                source,
            } => commands::campaign::run_append_decision(&question, &answer, &source, &project_dir()),
            CampaignCmd::Show => commands::campaign::run_show(&project_dir()),
        },
        Commands::Tasks { subcmd } => match subcmd {
            TasksCmd::Sync { path } => commands::tasks::run_sync(&path, &proj),
            TasksCmd::Update { path, id, status } => {
                commands::tasks::run_update(&path, &id, &status, &proj)
            }
            TasksCmd::Init {
                design_path,
                output,
                force,
            } => commands::tasks::run_init(&design_path, &output, force, &proj),
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
