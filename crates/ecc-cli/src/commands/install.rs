use clap::Args;
use ecc_app::install::{self, InstallContext, InstallOptions};
use ecc_app::version;
use ecc_infra::os_env::OsEnvironment;
use ecc_infra::os_fs::OsFileSystem;
use ecc_infra::process_executor::ProcessExecutor;
use ecc_infra::std_terminal::StdTerminal;
use ecc_ports::env::Environment;

#[derive(Args)]
pub struct InstallArgs {
    /// Force overwrite without prompts
    #[arg(short, long)]
    pub force: bool,

    /// Dry run — show changes without writing
    #[arg(long)]
    pub dry_run: bool,

    /// Skip gitignore updates
    #[arg(long)]
    pub no_gitignore: bool,

    /// Non-interactive mode (accept all changes)
    #[arg(long)]
    pub no_interactive: bool,

    /// Clean ECC-managed files before install (uses manifest)
    #[arg(long)]
    pub clean: bool,

    /// Nuclear clean — remove all ECC directories before install
    #[arg(long)]
    pub clean_all: bool,

    /// Language-specific rule groups to install (e.g., typescript, python)
    #[arg(long)]
    pub languages: Vec<String>,

    /// Path to ECC assets directory (defaults to bundled location)
    #[arg(long, env = "ECC_ROOT")]
    pub ecc_root: Option<std::path::PathBuf>,
}

pub fn run(args: InstallArgs) -> anyhow::Result<()> {
    let fs = OsFileSystem;
    let env = OsEnvironment;
    let terminal = StdTerminal;
    let shell = ProcessExecutor;

    let home = env
        .home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;
    let claude_dir = home.join(".claude");

    // Resolve ECC assets root: explicit flag > env var > npm global install path
    let ecc_root = match args.ecc_root {
        Some(root) => root,
        None => install::resolve_ecc_root(&fs, &env)
            .map_err(|e| anyhow::anyhow!(e))?,
    };

    let ctx = InstallContext {
        fs: &fs,
        shell: &shell,
        env: &env,
        terminal: &terminal,
    };

    let now = {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let dt = ecc_domain::time::datetime_from_epoch(secs);
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second
        )
    };
    let options = InstallOptions {
        dry_run: args.dry_run,
        force: args.force,
        no_gitignore: args.no_gitignore,
        interactive: !args.no_interactive,
        clean: args.clean,
        clean_all: args.clean_all,
        languages: args.languages,
    };

    let summary = install::install_global(
        &ctx,
        &ecc_root,
        &claude_dir,
        version::version(),
        &now,
        &options,
    );

    if !summary.success {
        std::process::exit(1);
    }

    Ok(())
}


