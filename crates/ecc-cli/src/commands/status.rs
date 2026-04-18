//! `ecc status` — diagnostic snapshot command.

use clap::Args;
use ecc_infra::os_env::OsEnvironment;
use ecc_infra::os_fs::OsFileSystem;

#[derive(Args)]
pub struct StatusArgs {
    /// Output as JSON instead of human-readable text
    #[arg(long)]
    pub json: bool,
    /// Run health checks (SQLite, git, writable dir, state file)
    #[arg(long)]
    pub health: bool,
}

pub fn run(args: StatusArgs) -> anyhow::Result<()> {
    let fs = OsFileSystem;
    let env = OsEnvironment;

    if args.health {
        let report = ecc_app::diagnostics::gather_health(&fs, &env);
        if args.json {
            println!("{}", ecc_app::diagnostics::format_health_json(&report));
        } else {
            println!("{}", ecc_app::diagnostics::format_health_human(&report));
        }
        if !report.all_ok {
            std::process::exit(1);
        }
        return Ok(());
    }

    let report = ecc_app::diagnostics::gather_status(&fs, &env);

    if args.json {
        println!("{}", ecc_app::diagnostics::format_json(&report));
    } else {
        println!("{}", ecc_app::diagnostics::format_human(&report));
    }

    Ok(())
}
