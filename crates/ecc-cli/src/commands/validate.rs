//! CLI command: `ecc validate <target>`
//!
//! Thin wiring layer — delegates all logic to `ecc_app::validate`.

use clap::{Args, Subcommand};
use ecc_infra::os_fs::OsFileSystem;
use ecc_infra::std_terminal::StdTerminal;
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
        other => {
            let target = map_target(other);
            if ecc_app::validate::run_validate(&fs, &terminal, &target, &args.ecc_root) {
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
        CliValidateTarget::Hooks => ecc_app::validate::ValidateTarget::Hooks,
        CliValidateTarget::Skills => ecc_app::validate::ValidateTarget::Skills,
        CliValidateTarget::Rules => ecc_app::validate::ValidateTarget::Rules,
        CliValidateTarget::Paths => ecc_app::validate::ValidateTarget::Paths,
        CliValidateTarget::Statusline => ecc_app::validate::ValidateTarget::Statusline,
        // Spec and Design are handled directly in run() — unreachable here
        CliValidateTarget::Spec { .. } | CliValidateTarget::Design { .. } => {
            unreachable!("Spec and Design are handled before map_target is called")
        }
        // Conventions is handled via map_target but mapping is intentionally missing here (RED)
    }
}
