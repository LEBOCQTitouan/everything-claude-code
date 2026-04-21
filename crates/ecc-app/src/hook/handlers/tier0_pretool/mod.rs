//! Tier 0 hooks — path-level allow/block decisions evaluated before any tool
//! executes. These hooks are purely functional (no I/O, no shell calls) and
//! operate on the raw tool-input payload.

pub mod write_guard;

pub use write_guard::is_memory_root_path;
