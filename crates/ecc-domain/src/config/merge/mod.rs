use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Submodules
// ---------------------------------------------------------------------------

mod format;
mod legacy;
mod merge_typed;
mod merge_value;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_legacy;
#[cfg(test)]
mod tests_typed;

// ---------------------------------------------------------------------------
// Public re-exports (submodule items)
// ---------------------------------------------------------------------------

pub use format::{combine_reports, contents_differ, empty_report, format_merge_report};
pub use legacy::{
    is_legacy_command, is_legacy_ecc_hook, is_legacy_ecc_hook_typed, remove_legacy_hooks,
    remove_legacy_hooks_typed,
};
pub use merge_typed::merge_hooks_typed;
pub use merge_value::merge_hooks_pure;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Summary of a merge operation across one artifact category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeReport {
    pub added: Vec<String>,
    pub updated: Vec<String>,
    pub unchanged: Vec<String>,
    pub skipped: Vec<String>,
    pub smart_merged: Vec<String>,
    pub errors: Vec<String>,
}

/// Result of a `merge_hooks_pure` operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeHooksPureResult {
    pub merged: serde_json::Value,
    pub added: usize,
    pub existing: usize,
    pub legacy_removed: usize,
}

/// Result of a `merge_hooks_typed` operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeHooksTypedResult {
    pub merged: super::hook_types::HooksMap,
    pub added: usize,
    pub existing: usize,
    pub legacy_removed: usize,
}

/// A file that differs between source and destination, pending user review.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileToReview {
    pub filename: String,
    pub src_path: PathBuf,
    pub dest_path: PathBuf,
    pub is_new: bool,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

// Re-export from shared location for backwards compatibility.
pub use super::ECC_PACKAGE_IDENTIFIERS;
