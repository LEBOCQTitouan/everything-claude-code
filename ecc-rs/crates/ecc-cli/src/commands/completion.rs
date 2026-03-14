use clap::Args;
use clap_complete::Shell;

#[derive(Args)]
pub struct CompletionArgs {
    /// Shell to generate completions for
    #[arg(value_enum)]
    pub shell: Shell,
}

pub fn run(args: CompletionArgs) -> anyhow::Result<()> {
    use clap::CommandFactory;
    let mut cmd = crate::Cli::command();
    clap_complete::generate(args.shell, &mut cmd, "ecc", &mut std::io::stdout());
    Ok(())
}
