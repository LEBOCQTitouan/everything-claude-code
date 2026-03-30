//! Configuration management domain types.
//!
//! Covers manifest parsing, audit checks, deny rules, merge operations,
//! cleanup, validation, and gitignore handling for ECC configuration files.

pub mod agent_frontmatter;
pub mod ecc_config;
pub mod audit;
pub mod clean;
pub mod deny_rules;
pub mod detect;
pub mod dev_profile;
pub mod gitignore;
pub mod hook_types;
pub mod manifest;
pub mod merge;
pub mod log_level;
pub mod statusline;
pub mod validate;

/// Known ECC package identifiers in npm paths.
pub const ECC_PACKAGE_IDENTIFIERS: &[&str] = &["@lebocqtitouan/ecc/", "everything-claude-code/"];
