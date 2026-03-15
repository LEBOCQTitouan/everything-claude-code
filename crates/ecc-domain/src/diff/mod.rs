//! Line-level text diffing.
//!
//! LCS-based diff computation and unified-diff formatting for comparing
//! configuration file contents during merge and audit operations.

pub mod formatter;
pub mod lcs;
