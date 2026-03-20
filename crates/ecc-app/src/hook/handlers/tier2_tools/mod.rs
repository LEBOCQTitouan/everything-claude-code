//! Tier 2 Hooks — External tool spawning (formatter, typecheck, quality gate, dev server block).

mod dev_server;
mod formatting;
mod helpers;
mod quality;

pub use dev_server::pre_bash_dev_server_block;
pub use formatting::post_edit_format;
pub use quality::{post_edit_typecheck, quality_gate};
