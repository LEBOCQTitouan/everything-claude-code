use clap::Args;

/// Update ECC to the latest version from GitHub Releases.
#[derive(Args, Debug)]
pub struct UpdateArgs {
    /// Pin to a specific version (e.g., 4.3.0)
    #[arg(long)]
    pub version: Option<String>,

    /// Preview update without making changes
    #[arg(long)]
    pub dry_run: bool,

    /// Include prerelease versions
    #[arg(long)]
    pub pre: bool,
}

pub fn run(args: &UpdateArgs) -> anyhow::Result<()> {
    use ecc_app::update::context::UpdateContext;
    use ecc_app::update::orchestrator::UpdateOutcome;
    use ecc_app::update::{UpdateOptions, run_update};
    use ecc_infra::github_release::GithubReleaseClient;
    use ecc_infra::os_env::OsEnvironment;
    use ecc_infra::os_fs::OsFileSystem;
    use ecc_infra::process_executor::ProcessExecutor;
    use ecc_infra::std_terminal::StdTerminal;

    let fs = OsFileSystem;
    let env = OsEnvironment;
    let shell = ProcessExecutor;
    let terminal = StdTerminal;
    let client = GithubReleaseClient::new("LEBOCQTitouan", "everything-claude-code");

    let ctx = UpdateContext {
        fs: &fs,
        env: &env,
        shell: &shell,
        terminal: &terminal,
        release_client: &client,
    };

    let options = UpdateOptions {
        target_version: args.version.clone(),
        dry_run: args.dry_run,
        include_prerelease: args.pre,
    };

    let current_version = env!("CARGO_PKG_VERSION");

    let result = run_update(&ctx, &options, current_version, &|downloaded, total| {
        if total > 0 {
            let pct = (downloaded as f64 / total as f64 * 100.0) as u32;
            eprint!("\rDownloading... {pct}%");
        }
    });

    match result {
        Ok(UpdateOutcome::Updated(summary)) => {
            eprintln!();
            println!("{summary}");
        }
        Ok(UpdateOutcome::AlreadyCurrent(msg)) => {
            println!("{msg}");
        }
        Ok(UpdateOutcome::DryRun(msg)) => {
            println!("{msg}");
        }
        Err(e) => {
            eprintln!("Update failed: {e}");
            std::process::exit(1);
        }
    }

    Ok(())
}
