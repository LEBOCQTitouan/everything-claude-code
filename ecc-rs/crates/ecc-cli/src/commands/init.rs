use clap::Args;
use ecc_app::install::{self, InstallContext};
use ecc_infra::os_env::OsEnvironment;
use ecc_infra::os_fs::OsFileSystem;
use ecc_infra::process_executor::ProcessExecutor;
use ecc_infra::std_terminal::StdTerminal;
use ecc_ports::env::Environment;

#[derive(Args)]
pub struct InitArgs {
    /// Force overwrite without prompts
    #[arg(short, long)]
    pub force: bool,

    /// Skip gitignore updates
    #[arg(long)]
    pub no_gitignore: bool,

    /// Non-interactive mode
    #[arg(long)]
    pub no_interactive: bool,

    /// Dry run — show changes without writing
    #[arg(long)]
    pub dry_run: bool,
}

pub fn run(args: InitArgs) -> anyhow::Result<()> {
    let fs = OsFileSystem;
    let env = OsEnvironment;
    let terminal = StdTerminal;
    let shell = ProcessExecutor;

    let project_dir = env
        .current_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    let ctx = InstallContext {
        fs: &fs,
        shell: &shell,
        env: &env,
        terminal: &terminal,
    };

    install::init_project(&ctx, &project_dir, args.no_gitignore, args.dry_run);

    Ok(())
}
