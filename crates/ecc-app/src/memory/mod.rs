//! Memory system use cases.
//!
//! Split into 4 submodules per SOLID/SRP:
//! - `crud`: add, search, list, delete, show
//! - `lifecycle`: gc, stats, promote
//! - `migration`: migrate, export

pub mod crud;
pub mod lifecycle;
pub mod migration;

pub use crud::{AddParams, MemoryAppError};
