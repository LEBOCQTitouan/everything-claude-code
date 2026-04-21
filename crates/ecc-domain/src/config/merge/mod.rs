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
    /// Files that were newly added during the merge.
    pub added: Vec<String>,
    /// Files that were updated during the merge.
    pub updated: Vec<String>,
    /// Files that remained unchanged.
    pub unchanged: Vec<String>,
    /// Files that were skipped during the merge.
    pub skipped: Vec<String>,
    /// Files that were smart-merged (3-way merge applied).
    pub smart_merged: Vec<String>,
    /// Errors encountered during merge operations.
    pub errors: Vec<String>,
}

/// Result of a `merge_hooks_pure` operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeHooksPureResult {
    /// The merged hooks JSON value.
    pub merged: serde_json::Value,
    /// Number of hooks newly added.
    pub added: usize,
    /// Number of existing hooks that were updated.
    pub existing: usize,
    /// Number of legacy hooks removed during merge.
    pub legacy_removed: usize,
}

/// Result of a `merge_hooks_typed` operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeHooksTypedResult {
    /// The merged hooks typed map.
    pub merged: super::hook_types::HooksMap,
    /// Number of hooks newly added.
    pub added: usize,
    /// Number of existing hooks that were updated.
    pub existing: usize,
    /// Number of legacy hooks removed during merge.
    pub legacy_removed: usize,
}

/// A file that differs between source and destination, pending user review.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileToReview {
    /// The filename being reviewed.
    pub filename: String,
    /// Source file path.
    pub src_path: PathBuf,
    /// Destination file path.
    pub dest_path: PathBuf,
    /// Whether this is a newly created file.
    pub is_new: bool,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

// Re-export from shared location for backwards compatibility.
pub use super::ECC_PACKAGE_IDENTIFIERS;
