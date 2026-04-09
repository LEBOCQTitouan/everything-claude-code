//! Library target exposing CLI commands and the top-level Cli struct for integration testing.

pub mod commands;

use clap::{ArgAction, Parser};

#[derive(Parser)]
#[command(
    name = "ecc",
    about = "Everything Claude Code — CLI for setting up Claude Code configuration",
    version
)]
pub struct Cli {
    /// Increase verbosity (-v info, -vv debug, -vvv trace)
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count, global = true, conflicts_with = "quiet")]
    pub verbose: u8,

    /// Quiet mode (errors only)
    #[arg(short = 'q', long = "quiet", global = true, conflicts_with = "verbose")]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand)]
pub enum Command {
    /// Deterministic git history analysis
    Analyze(commands::analyze::AnalyzeArgs),
    /// Show the current ECC version
    Version,
    /// Install ECC configuration to ~/.claude/
    Install(commands::install::InstallArgs),
    /// Initialize ECC in the current project
    Init(commands::init::InitArgs),
    /// Audit ECC configuration health
    Audit(commands::audit::AuditArgs),
    /// Manage audit cache (check/clear)
    AuditCache(commands::audit_cache::AuditCacheArgs),
    /// Generate shell completions
    Completion(commands::completion::CompletionArgs),
    /// Run a hook by ID
    Hook(commands::hook::HookArgs),
    /// Validate content files
    Validate(commands::validate::ValidateArgs),
    /// NanoClaw interactive REPL
    Claw(commands::claw::ClawArgs),
    /// Toggle ECC config on/off
    Dev(commands::dev::DevArgs),
    /// Deterministic backlog management
    Backlog(commands::backlog::BacklogArgs),
    /// Drift commands
    Drift(commands::drift::DriftArgs),
    /// Docs commands
    Docs(commands::docs::DocsArgs),
    /// Diagram commands
    Diagram(commands::diagram::DiagramArgs),
    /// CommitCmd commands
    CommitCmd(commands::commit_cmd::CommitCmdArgs),
    /// Manage workflow bypass tokens and audit trail
    Bypass(commands::bypass::BypassArgs),
    /// Manage git worktrees for ECC sessions
    Worktree(commands::worktree::WorktreeArgs),
    /// Manage knowledge sources registry
    Sources(commands::sources::SourcesArgs),
    /// Query and manage structured logs
    Log(commands::log::LogArgs),
    /// Manage the structured memory store
    Memory(commands::memory::MemoryArgs),
    /// Manage audit-web profile and validate reports
    AuditWeb(commands::audit_web::AuditWebArgs),
    /// Show ECC status (workflow, versions, components)
    Status(commands::status::StatusArgs),
    /// Manage ECC configuration preferences
    Config(commands::config::ConfigArgs),
    /// Update ECC to the latest version
    Update(commands::update::UpdateArgs),
    /// Workflow state machine (mirrors ecc-workflow)
    Workflow(commands::workflow::WorkflowArgs),
    /// Manage session cost tracking
    Cost(commands::cost::CostArgs),
    /// Harness reliability metrics
    Metrics(commands::metrics::MetricsArgs),
}
