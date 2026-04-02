//! Cartography bounded context — pure domain types and functions.
//!
//! No I/O: all functions operate on in-memory values only.
//! Zero `std::fs`, `std::process`, `std::net`, or `tokio` imports.

pub mod merge;
pub mod slug;
pub mod types;

pub use merge::{has_section, merge_section};
pub use slug::derive_slug;
pub use types::{CartographyMeta, ChangedFile, ProjectType, SessionDelta};
