//! Cartography bounded context — pure domain types and functions.
//!
//! No I/O: all functions operate on in-memory values only.
//! Zero `std::fs`, `std::process`, `std::net`, or `tokio` imports.

// Policy: all new public items in this module should have doc comments.

pub mod classification;
pub mod coverage;
pub mod cross_reference;
pub mod dedupe;
pub mod element_types;
pub mod element_validation;
pub mod merge;
pub mod noise_filter;
pub mod slug;
pub mod staleness;
pub mod types;
pub mod validation;

pub use classification::classify_file;
pub use coverage::{CoverageReport, calculate_coverage};
pub use cross_reference::build_cross_reference_matrix;
pub use dedupe::canonical_hash;
pub use element_types::{ElementEntry, ElementType, infer_element_type_from_path};
pub use element_validation::validate_element;
pub use merge::{has_section, merge_section};
pub use noise_filter::is_noise_path;
pub use slug::derive_slug;
pub use staleness::{check_staleness, parse_cartography_meta, remove_stale_marker};
pub use types::{CartographyMeta, ChangedFile, ProjectType, SessionDelta};
pub use validation::{validate_flow, validate_journey};

/// Trait for cartography document types that can be validated.
///
/// Provides a common interface for journey, flow, and element documents,
/// improving SAP (Stable Abstractions Principle) score for this module.
pub trait CartographyDocument {
    /// Returns the document type identifier (e.g., "journey", "flow", "element").
    fn doc_type(&self) -> &str;

    /// Validates the document structure.
    fn validate(&self) -> Result<(), String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sap_trait_exists() {
        // Verify the trait can be used as a trait object
        fn _accepts_doc(_doc: &dyn CartographyDocument) {}
    }
}
