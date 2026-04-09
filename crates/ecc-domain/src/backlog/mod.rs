//! Backlog management domain — pure types and logic for BL-*.md entries.
//!
//! No I/O: all functions operate on `&str` or typed structs.

pub mod entry;
pub mod index;
pub mod lock;
pub mod similarity;
