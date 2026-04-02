//! Deterministic git analytics — pure domain logic.
//!
//! All functions in this module are pure: they take domain types as input
//! and return domain types as output. Zero I/O.

pub mod bus_factor;
pub mod changelog;
pub mod commit;
pub mod coupling;
pub mod error;
pub mod hotspot;
