//! Tier 1 Hooks — Simple passthrough/warn hooks with no external tool spawning.

mod clean_craft_hooks;
mod dev_hooks;
mod doc_hooks;
mod git_hooks;
mod helpers;
mod meta_hooks;

pub use clean_craft_hooks::*;
pub use dev_hooks::*;
pub use doc_hooks::*;
pub use git_hooks::*;
pub use meta_hooks::*;
