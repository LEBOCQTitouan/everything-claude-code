//! Cartography bounded context — pure domain types and functions.
//!
//! No I/O: all functions operate on in-memory values only.
//! Zero `std::fs`, `std::process`, `std::net`, or `tokio` imports.

pub mod coverage;
pub mod merge;
pub mod slug;
pub mod staleness;
pub mod types;
pub mod validation;

pub use coverage::{calculate_coverage, CoverageReport};
pub use merge::{has_section, merge_section};
pub use slug::derive_slug;
pub use staleness::{check_staleness, parse_cartography_meta, remove_stale_marker};
pub use types::{CartographyMeta, ChangedFile, ProjectType, SessionDelta};
