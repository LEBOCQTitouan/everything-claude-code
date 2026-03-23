use clap::{Args, Subcommand, ValueEnum};
use ecc_app::dev;
use ecc_app::install::{self, InstallContext};
use ecc_app::version;
use ecc_domain::config::dev_profile::DevProfile;
use ecc_infra::os_env::OsEnvironment;
use ecc_infra::os_fs::OsFileSystem;
use ecc_infra::process_executor::ProcessExecutor;
use ecc_infra::std_terminal::StdTerminal;
use ecc_ports::env::Environment;
use ecc_ports::terminal::TerminalIO;

/// CLI-level wrapper for `DevProfile` that derives `ValueEnum`.
#[derive(Debug, Clone, ValueEnum)]
pub enum CliDevProfile {
    /// Production profile: real `~/.claude/` install (no symlinks).
    Default,
    /// Development profile: symlinks from ECC repo into `~/.claude/`.
    Dev,
}

impl From<CliDevProfile> for DevProfile {
    fn from(p: CliDevProfile) -> Self {
        match p {
            CliDevProfile::Default => DevProfile::Default,
            CliDevProfile::Dev => DevProfile::Dev,
        }
    }
}

#[derive(Args)]
pub struct DevArgs {
    #[command(subcommand)]
    pub action: DevAction,
}

#[derive(Subcommand)]
pub enum DevAction {
    /// Activate ECC config (clean + force reinstall)
    On {
        /// Preview changes without writing
        #[arg(long)]
        dry_run: bool,

        /// Path to ECC assets directory
        #[arg(long, env = "ECC_ROOT")]
        ecc_root: Option<std::path::PathBuf>,
    },
    /// Deactivate ECC config (remove manifest-tracked artifacts)
    Off {
        /// Preview changes without writing
        #[arg(long)]
        dry_run: bool,
    },
    /// Show current ECC installation status
    Status,
    /// Switch to the given config profile (dev or default)
    Switch {
        /// Target profile to activate
        profile: CliDevProfile,
        /// Preview changes without writing
        #[arg(long)]
        dry_run: bool,
    },
}

pub fn run(args: DevArgs) -> anyhow::Result<()> {
    let fs = OsFileSystem;
    let env = OsEnvironment;
    let terminal = StdTerminal;
    let shell = ProcessExecutor;

    let home = env
        .home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;
    let claude_dir = home.join(".claude");
    let colored = env.var("NO_COLOR").is_none();

    match args.action {
        DevAction::On { dry_run, ecc_root } => {
            let ecc_root = match ecc_root {
                Some(root) => root,
                None => install::resolve_ecc_root(&fs, &env).map_err(|e| anyhow::anyhow!(e))?,
            };

            let ctx = InstallContext {
                fs: &fs,
                shell: &shell,
                env: &env,
                terminal: &terminal,
            };

            let now = format_now();
            let summary = dev::dev_on(
                &ctx,
                &ecc_root,
                &claude_dir,
                version::version(),
                &now,
                dry_run,
            );

            if !summary.success {
                std::process::exit(1);
            }
        }
        DevAction::Off { dry_run } => {
            let result = dev::dev_off(&fs, &terminal, &claude_dir, dry_run);
            if !result.success {
                std::process::exit(1);
            }
        }
        DevAction::Status => {
            let status = dev::dev_status(&fs, &claude_dir);
            let output = dev::format_status(&status, colored);
            terminal.stdout_write(&output);
        }
        DevAction::Switch { profile, dry_run } => {
            let ecc_root =
                install::resolve_ecc_root(&fs, &env).map_err(|e| anyhow::anyhow!(e))?;
            let domain_profile: ecc_domain::config::dev_profile::DevProfile = profile.into();
            dev::dev_switch(
                &fs, &terminal, &ecc_root, &claude_dir, domain_profile.clone(), dry_run,
            )
            .map_err(|e| anyhow::anyhow!("{e}"))?;

            // After switching to Default, reinstall copied files
            if matches!(domain_profile, ecc_domain::config::dev_profile::DevProfile::Default) {
                let ctx = InstallContext {
                    fs: &fs,
                    shell: &shell,
                    env: &env,
                    terminal: &terminal,
                };
                let now = format_now();
                let summary = dev::dev_on(
                    &ctx, &ecc_root, &claude_dir, version::version(), &now, dry_run,
                );
                if !summary.success {
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}

fn format_now() -> String {
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
}
