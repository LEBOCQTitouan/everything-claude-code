//! Memory system use cases.
//!
//! Split into submodules per SOLID/SRP:
//! - `crud`: add, search, list, delete, show
//! - `lifecycle`: gc, stats, promote
//! - `migration`: migrate, export
//! - `consolidation`: consolidate, generate_context_md, expire_working_memories
//! - `injection`: inject_context

pub mod consolidation;
pub mod crud;
pub mod file_prune;
pub mod injection;
pub mod lifecycle;
pub mod migration;
pub mod paths;
pub mod trash_gc;

pub use crud::{AddParams, MemoryAppError};
