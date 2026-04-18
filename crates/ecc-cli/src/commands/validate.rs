//! CLI command: `ecc validate <target>`
//!
//! Thin wiring layer — delegates all logic to `ecc_app::validate`.

use clap::{Args, Subcommand};
use ecc_infra::os_fs::OsFileSystem;
use ecc_infra::process_executor::ProcessExecutor;
use ecc_infra::std_terminal::StdTerminal;
use ecc_ports::terminal::TerminalIO;
use std::path::PathBuf;

#[derive(Args)]
pub struct ValidateArgs {
    #[command(subcommand)]
    pub target: CliValidateTarget,

    /// ECC root directory (defaults to current directory)
    #[arg(long, default_value = ".")]
    pub ecc_root: PathBuf,
}

#[derive(Subcommand)]
pub enum CliValidateTarget {
    /// Validate agent markdown files
    Agents,
    /// Validate command markdown files
    Commands,
    /// Validate convention consistency (naming, tools, placement)
    Conventions,
    /// Validate hooks.json schema
    Hooks,
    /// Validate skill directories
    Skills,
    /// Validate rule markdown files
    Rules,
    /// Check for personal paths in shipped files
    Paths,
    /// Validate statusline installation
    Statusline,
    /// Validate team manifest files
    Teams,
    /// Validate pattern library files
    Patterns {
        /// Auto-regenerate patterns/index.md from frontmatter
        #[arg(long)]
        fix: bool,
    },
    /// Validate spec AC format and numbering
    Spec {
        /// Path to spec.md
        path: PathBuf,
    },
    /// Validate design PC table and optionally cross-reference spec
    Design {
        /// Path to design.md
        path: PathBuf,
        /// Optional path to spec.md for coverage analysis
        #[arg(long)]
        spec: Option<PathBuf>,
    },
    /// Validate CLAUDE.md numeric claims and TEMPORARY marker staleness
    ClaudeMd {
        #[command(subcommand)]
        cmd: Option<ClaudeMdSubcommand>,
        /// DEPRECATED: use `ecc validate claude-md counts` instead
        #[arg(long)]
        counts: bool,
    },
    /// Validate cartography journey and flow files, check staleness and optionally show coverage
    Cartography {
        /// Show coverage dashboard (total files, referenced, percentage, priority gaps)
        #[arg(long)]
        coverage: bool,
    },
}

/// Subcommands for `ecc validate claude-md`.
#[derive(Subcommand)]
pub enum ClaudeMdSubcommand {
    /// Cross-check CLAUDE.md numeric claims against actual counts
    Counts,
    /// Flag stale TEMPORARY (BL-NNN) markers whose backlog file is missing
    Markers {
        /// Escalate warnings to errors (exit 1 on missing markers)
        #[arg(long)]
        strict: bool,
        /// Emit markdown table of all markers + resolution status to stdout
        #[arg(long)]
        audit_report: bool,
    },
    /// Run both counts and markers validation
    All {
        #[arg(long)]
        strict: bool,
        #[arg(long)]
        audit_report: bool,
    },
}

pub fn run(args: ValidateArgs) -> anyhow::Result<()> {
    let fs = OsFileSystem;
    let terminal = StdTerminal;

    match &args.target {
        CliValidateTarget::Spec { path } => {
            let path_str = path.to_string_lossy();
            match ecc_app::validate_spec::run_validate_spec(&fs, &terminal, &path_str) {
                Ok(true) => Ok(()),
                Ok(false) => std::process::exit(1),
                Err(e) => Err(anyhow::anyhow!("{e}")),
            }
        }
        CliValidateTarget::Design { path, spec } => {
            let path_str = path.to_string_lossy();
            let spec_str = spec.as_ref().map(|p| p.to_string_lossy().into_owned());
            let spec_ref = spec_str.as_deref();
            match ecc_app::validate_design::run_validate_design(&fs, &terminal, &path_str, spec_ref)
            {
                Ok(true) => Ok(()),
                Ok(false) => std::process::exit(1),
                Err(e) => Err(anyhow::anyhow!("{e}")),
            }
        }
        CliValidateTarget::ClaudeMd { cmd, counts } => {
            match (cmd, counts) {
                (Some(ClaudeMdSubcommand::Counts), _) | (None, true) => {
                    if *counts {
                        terminal.stderr_write("DEPRECATED: use 'ecc validate claude-md counts' (subcommand form); --counts will be removed in the next minor release.\n");
                    }
                    let shell = ecc_infra::process_executor::ProcessExecutor;
                    if ecc_app::validate_claude_md::run_validate_claude_md(
                        &fs,
                        &shell,
                        &terminal,
                        &args.ecc_root,
                        false,
                    ) {
                        Ok(())
                    } else {
                        std::process::exit(1);
                    }
                }
                (Some(ClaudeMdSubcommand::Markers { strict, audit_report }), _) => {
                    let disabled = std::env::var("ECC_CLAUDE_MD_MARKERS_DISABLED")
                        .is_ok_and(|v| v == "1");
                    if ecc_app::validate_claude_md::run_validate_temporary_markers(
                        &fs,
                        &terminal,
                        &args.ecc_root,
                        disabled,
                        *strict,
                        *audit_report,
                    ) {
                        Ok(())
                    } else {
                        std::process::exit(1);
                    }
                }
                (Some(ClaudeMdSubcommand::All { strict, audit_report }), _) => {
                    let disabled = std::env::var("ECC_CLAUDE_MD_MARKERS_DISABLED")
                        .is_ok_and(|v| v == "1");
                    let shell = ecc_infra::process_executor::ProcessExecutor;
                    let markers_ok = ecc_app::validate_claude_md::run_validate_temporary_markers(
                        &fs,
                        &terminal,
                        &args.ecc_root,
                        disabled,
                        *strict,
                        *audit_report,
                    );
                    let counts_ok = ecc_app::validate_claude_md::run_validate_claude_md(
                        &fs,
                        &shell,
                        &terminal,
                        &args.ecc_root,
                        false,
                    );
                    if markers_ok && counts_ok {
                        Ok(())
                    } else {
                        std::process::exit(1);
                    }
                }
                (None, false) => {
                    terminal.stderr_write("usage: ecc validate claude-md [counts|markers|all] [--strict] [--audit-report]\n");
                    std::process::exit(2);
                }
            }
        }
        CliValidateTarget::Cartography { coverage } => {
            let shell = ProcessExecutor;
            let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            if ecc_app::validate_cartography::run_validate_cartography(
                &fs,
                &shell,
                &terminal,
                &project_root,
                *coverage,
            ) {
                Ok(())
            } else {
                std::process::exit(1);
            }
        }
        CliValidateTarget::Patterns { fix } => {
            let env = ecc_infra::os_env::OsEnvironment;
            if ecc_app::validate::run_validate_patterns(&fs, &terminal, &env, &args.ecc_root, *fix)
            {
                Ok(())
            } else {
                std::process::exit(1);
            }
        }
        other => {
            let target = map_target(other);
            let env = ecc_infra::os_env::OsEnvironment;
            if ecc_app::validate::run_validate(&fs, &terminal, &env, &target, &args.ecc_root) {
                Ok(())
            } else {
                std::process::exit(1);
            }
        }
    }
}

fn map_target(cli: &CliValidateTarget) -> ecc_app::validate::ValidateTarget {
    match cli {
        CliValidateTarget::Agents => ecc_app::validate::ValidateTarget::Agents,
        CliValidateTarget::Commands => ecc_app::validate::ValidateTarget::Commands,
        CliValidateTarget::Conventions => ecc_app::validate::ValidateTarget::Conventions,
        CliValidateTarget::Hooks => ecc_app::validate::ValidateTarget::Hooks,
        CliValidateTarget::Skills => ecc_app::validate::ValidateTarget::Skills,
        CliValidateTarget::Rules => ecc_app::validate::ValidateTarget::Rules,
        CliValidateTarget::Paths => ecc_app::validate::ValidateTarget::Paths,
        CliValidateTarget::Statusline => ecc_app::validate::ValidateTarget::Statusline,
        CliValidateTarget::Teams => ecc_app::validate::ValidateTarget::Teams,

        // Spec, Design, and Cartography are handled directly in run() — unreachable here
        CliValidateTarget::ClaudeMd { .. }
        | CliValidateTarget::Patterns { .. }
        | CliValidateTarget::Spec { .. }
        | CliValidateTarget::Design { .. }
        | CliValidateTarget::Cartography { .. } => {
            unreachable!(
                "Patterns, Spec, Design, and Cartography are handled before map_target is called"
            )
        }
    }
}
