//! CLI command: `ecc hook <hookId> [profilesCsv]`
//!
//! Reads stdin, dispatches to the appropriate hook handler, writes stdout/stderr.

use clap::Args;
use ecc_app::hook::{dispatch, HookContext, HookPorts, MAX_STDIN};
use ecc_infra::os_env::OsEnvironment;
use ecc_infra::os_fs::OsFileSystem;
use ecc_infra::process_executor::ProcessExecutor;
use ecc_infra::std_terminal::StdTerminal;
use std::io::{self, Read, Write};

#[derive(Args)]
pub struct HookArgs {
    /// Hook identifier (e.g., "pre:bash:dev-server-block")
    pub hook_id: String,

    /// Comma-separated profile names (e.g., "standard,strict")
    pub profiles: Option<String>,
}

pub fn run(args: HookArgs) -> anyhow::Result<()> {
    // Read stdin (bounded)
    let mut raw = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut buf = [0u8; 8192];
    loop {
        let n = handle.read(&mut buf)?;
        if n == 0 {
            break;
        }
        let remaining = MAX_STDIN.saturating_sub(raw.len());
        if remaining == 0 {
            break;
        }
        let take = n.min(remaining);
        raw.push_str(&String::from_utf8_lossy(&buf[..take]));
    }

    // Set up ports
    let fs = OsFileSystem;
    let shell = ProcessExecutor;
    let env = OsEnvironment;
    let terminal = StdTerminal;

    let ports = HookPorts {
        fs: &fs,
        shell: &shell,
        env: &env,
        terminal: &terminal,
    };

    let ctx = HookContext {
        hook_id: args.hook_id,
        stdin_payload: raw,
        profiles_csv: args.profiles,
    };

    let result = dispatch(&ctx, &ports);

    // Write outputs
    if !result.stdout.is_empty() {
        io::stdout().write_all(result.stdout.as_bytes())?;
    }
    if !result.stderr.is_empty() {
        io::stderr().write_all(result.stderr.as_bytes())?;
    }

    std::process::exit(result.exit_code);
}
