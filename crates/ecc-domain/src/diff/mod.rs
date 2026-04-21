//! Line-level text diffing.
//!
//! LCS-based diff computation and unified-diff formatting for comparing
//! configuration file contents during merge and audit operations.

/// Unified diff formatter.
pub mod formatter;
/// Longest Common Subsequence diff computation.
pub mod lcs;
