use clap::Args;
use ecc_app::audit::{self, AuditOptions};
use ecc_infra::os_env::OsEnvironment;
use ecc_infra::os_fs::OsFileSystem;
use ecc_infra::std_terminal::StdTerminal;
use ecc_ports::env::Environment;

#[derive(Args)]
pub struct AuditArgs {
    /// Output format
    #[arg(short, long, default_value = "text")]
    pub format: String,
}

pub fn run(_args: AuditArgs) -> anyhow::Result<()> {
    let fs = OsFileSystem;
    let env = OsEnvironment;
    let terminal = StdTerminal;

    let home = env.home_dir().ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;
    let claude_dir = home.join(".claude");
    let project_dir = env.current_dir().unwrap_or_else(|| std::path::PathBuf::from("."));

    // For audit, ecc_root points to the installed claude_dir (where agents/commands live)
    let options = AuditOptions {
        claude_dir: &claude_dir,
        project_dir: &project_dir,
        ecc_root: &claude_dir,
    };

    let passed = audit::run_audit(&fs, &env, &terminal, &options);

    if !passed {
        std::process::exit(1);
    }

    Ok(())
}
