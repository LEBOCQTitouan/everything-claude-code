use clap::Args;

#[derive(Args)]
pub struct InstallArgs {
    /// Force overwrite without prompts
    #[arg(short, long)]
    pub force: bool,

    /// Dry run — show changes without writing
    #[arg(long)]
    pub dry_run: bool,
}

pub fn run(_args: InstallArgs) -> anyhow::Result<()> {
    // TODO: Phase 2 — wire to InstallUseCase
    println!("ecc install: not yet implemented in Rust build");
    Ok(())
}
