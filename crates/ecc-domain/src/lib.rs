//! Pure business logic for Everything Claude Code.
//!
//! This crate contains domain types, value objects, and business rules with
//! zero I/O dependencies. All side effects are pushed to the [`ecc_ports`] layer.

pub mod ansi;
pub mod claw;
pub mod config;
pub mod detection;
pub mod diff;
pub mod hook_runtime;
pub mod paths;
pub mod session;
pub mod time;
