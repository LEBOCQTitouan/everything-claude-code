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
        None => resolve_ecc_root()?,
    };

    let ctx = InstallContext {
        fs: &fs,
        shell: &shell,
        env: &env,
        terminal: &terminal,
    };

    let now = iso_now();
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

/// Resolve the ECC root directory containing agents/, commands/, skills/, rules/, hooks.json.
fn resolve_ecc_root() -> anyhow::Result<std::path::PathBuf> {
    // Try the npm global install location
    let npm_paths = [
        // macOS/Linux global npm
        "/usr/local/lib/node_modules/@lebocqtitouan/ecc",
        "/usr/lib/node_modules/@lebocqtitouan/ecc",
    ];

    for path in &npm_paths {
        let p = std::path::PathBuf::from(path);
        if p.join("agents").exists() {
            return Ok(p);
        }
    }

    // Try relative to binary location
    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        let relative = parent.join("../share/ecc");
        if relative.join("agents").exists() {
            return Ok(relative);
        }
    }

    anyhow::bail!(
        "Cannot find ECC assets directory. Set ECC_ROOT environment variable \
         or use --ecc-root flag."
    )
}

/// Generate an ISO 8601 timestamp.
fn iso_now() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    // Simple UTC timestamp
    let secs = now.as_secs();
    let days = secs / 86400;
    let time_in_day = secs % 86400;
    let hours = time_in_day / 3600;
    let minutes = (time_in_day % 3600) / 60;
    let seconds = time_in_day % 60;

    // Approximate date from days since epoch (1970-01-01)
    // Good enough for manifest timestamps
    let (year, month, day) = days_to_ymd(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}Z"
    )
}

fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    // Simplified date calculation
    let mut remaining = days;
    let mut year = 1970u64;

    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        year += 1;
    }

    let days_in_months: [u64; 12] = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1u64;
    for dim in &days_in_months {
        if remaining < *dim {
            break;
        }
        remaining -= dim;
        month += 1;
    }

    (year, month, remaining + 1)
}

fn is_leap(year: u64) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}
