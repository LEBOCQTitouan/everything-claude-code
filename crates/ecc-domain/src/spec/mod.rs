//! Spec and design artifact validation — pure domain functions, zero I/O.
//!
//! This module provides deterministic validation for ECC spec files and design files,
//! replacing token-expensive LLM structural checks with compiled Rust.

pub mod ac;
pub mod coverage;
pub mod error;
pub mod ordering;
pub mod output;
pub mod pc;

pub use ac::{AcId, AcReport, AcceptanceCriterion, parse_acs};
pub use coverage::{CoverageReport, check_coverage};
pub use error::SpecError;
pub use ordering::{
    FileChange, OrderingResult, OrderingViolation, check_ordering, parse_file_changes,
};
pub use output::{DesignValidationOutput, SpecValidationOutput};
pub use pc::{PassCondition, PcId, PcReport, parse_pcs};
