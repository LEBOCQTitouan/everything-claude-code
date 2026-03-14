use clap::Args;

#[derive(Args)]
pub struct InitArgs {
    /// Force overwrite without prompts
    #[arg(short, long)]
    pub force: bool,
}

pub fn run(_args: InitArgs) -> anyhow::Result<()> {
    // TODO: Phase 2
    println!("ecc init: not yet implemented in Rust build");
    Ok(())
}
