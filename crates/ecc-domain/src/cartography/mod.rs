//! Cartography bounded context — pure domain types and functions.
//!
//! No I/O: all functions operate on in-memory values only.
//! Zero `std::fs`, `std::process`, `std::net`, or `tokio` imports.

pub mod slug;
pub mod types;

pub use slug::derive_slug;
pub use types::{CartographyMeta, ChangedFile, ProjectType, SessionDelta};
