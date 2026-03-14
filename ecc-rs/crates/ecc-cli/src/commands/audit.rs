use clap::Args;

#[derive(Args)]
pub struct AuditArgs {
    /// Output format
    #[arg(short, long, default_value = "text")]
    pub format: String,
}

pub fn run(_args: AuditArgs) -> anyhow::Result<()> {
    // TODO: Phase 2
    println!("ecc audit: not yet implemented in Rust build");
    Ok(())
}
