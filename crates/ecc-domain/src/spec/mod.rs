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

pub use ac::{parse_acs, AcceptanceCriterion, AcId, AcReport};
pub use coverage::{check_coverage, CoverageReport};
pub use error::SpecError;
pub use ordering::{check_ordering, parse_file_changes, FileChange, OrderingResult, OrderingViolation};
pub use output::{DesignValidationOutput, SpecValidationOutput};
pub use pc::{parse_pcs, PassCondition, PcId, PcReport};
