// BL-132 fixture: clap-derive file with diagram ON /// above derived struct — classifier MUST FAIL.
//
// Expected classifier verdict: FAIL
// Reason: the `///` block below is promoted by clap into the `--help` long description.
// An ASCII diagram here corrupts user-visible CLI output. This is the exact corruption
// scenario R-1 forbids.

use clap::Parser;

/// Arguments for the widget command.
///
/// ```text
/// [widget cmd]
///    |
///    +---> [name]
///    +---> [count]
/// ```
#[derive(Parser)]
pub struct Args {
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub count: u32,
}
