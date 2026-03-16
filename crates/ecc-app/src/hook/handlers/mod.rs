//! Hook handler implementations.
//!
//! Each handler is a pure-ish function: `fn handle(stdin, ports) -> HookResult`

mod tier1_simple;
mod tier2_notify;
mod tier2_tools;
mod tier3_session;

pub use tier1_simple::*;
pub use tier2_notify::*;
pub use tier2_tools::*;
pub use tier3_session::*;
