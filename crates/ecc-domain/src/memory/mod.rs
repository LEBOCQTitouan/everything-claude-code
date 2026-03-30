//! Three-tier memory system domain — pure types and logic.
//!
//! No I/O: all functions operate on typed structs and pure data.
//! SQLite storage lives in `ecc-infra` behind the `MemoryStore` port.

pub mod consolidation;
pub mod context;
pub mod entry;
pub mod error;
pub mod export;
pub mod migration;
pub mod stats;
pub mod tier;

// Re-exports for convenience
pub use entry::{MemoryEntry, MemoryId};
pub use error::MemoryError;
pub use stats::MemoryStats;
pub use tier::MemoryTier;
