use ecc_app::version::version;
use ecc_infra::std_terminal::StdTerminal;
use ecc_ports::terminal::TerminalIO;

pub fn run() -> anyhow::Result<()> {
    log::debug!("version command: reporting ecc v{}", version());
    let terminal = StdTerminal;
    terminal.stdout_write(&format!("ecc v{}\n", version()));
    Ok(())
}
