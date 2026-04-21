//! Hook profile resolution.
//!
//! Resolves which hook profiles are active based on CLI arguments and
//! environment, enabling conditional hook execution.

/// Hook bypass grant tracking.
pub mod bypass;
/// Hook execution profile selection.
pub mod profiles;
