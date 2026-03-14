//! CLI command: `ecc validate <target>`
//!
//! Thin wiring layer — delegates all logic to `ecc_app::validate`.

use clap::{Args, Subcommand};
use ecc_infra::os_fs::OsFileSystem;
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
    /// Validate hooks.json schema
    Hooks,
    /// Validate skill directories
    Skills,
    /// Validate rule markdown files
    Rules,
    /// Check for personal paths in shipped files
    Paths,
}

pub fn run(args: ValidateArgs) -> anyhow::Result<()> {
    let fs = OsFileSystem;
    let target = map_target(&args.target);

    if ecc_app::validate::run_validate(&fs, &target, &args.ecc_root) {
        Ok(())
    } else {
        std::process::exit(1);
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
    }
}
