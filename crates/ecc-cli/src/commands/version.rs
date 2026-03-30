use ecc_app::version::version;
use ecc_infra::os_env::OsEnvironment;
use ecc_infra::std_terminal::StdTerminal;
use ecc_ports::terminal::TerminalIO;

pub fn run() -> anyhow::Result<()> {
    let env = OsEnvironment;
    tracing::debug!("version command: reporting ecc v{}", version(&env));
    let terminal = StdTerminal;
    terminal.stdout_write(&format!("ecc v{}\n", version(&env)));
    Ok(())
}
