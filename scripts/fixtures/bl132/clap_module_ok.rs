//! BL-132 fixture: clap-derive file with diagram at MODULE level — classifier MUST PASS.
//!
//! ```text
//! [cli args]
//!    |
//!    v
//! [parsed struct]
//! ```
//!
//! Expected classifier verdict: PASS
//! Reason: module-level `//!` doc-comments are never consumed by clap's `--help`
//! machinery. Diagrams here are always safe.

use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[arg(long)]
    pub name: String,
}
