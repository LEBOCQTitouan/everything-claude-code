//! Configuration management domain types.
//!
//! Covers manifest parsing, audit checks, deny rules, merge operations,
//! cleanup, validation, and gitignore handling for ECC configuration files.

/// Agent and hook frontmatter validation.
pub mod agent_frontmatter;
/// Conditional rule loading by stack.
pub mod applies_to;
/// Audit configuration and validation.
pub mod audit;
/// ECC artifact cleanup operations.
pub mod clean;
/// Deny rules for Claude Code settings.
pub mod deny_rules;
/// Stack detection heuristics.
pub mod detect;
/// Development profile configuration.
pub mod dev_profile;
/// ECC configuration root types.
pub mod ecc_config;
/// .gitignore entry management.
pub mod gitignore;
/// Hook JSON types and structures.
pub mod hook_types;
/// Log level configuration.
pub mod log_level;
/// Agent, hook, and skill manifest parsing.
pub mod manifest;
/// Configuration merge operations.
pub mod merge;
/// Status line formatting.
pub mod statusline;
/// Team manifest parsing and validation.
pub mod team;
/// Tool manifest resolution.
pub mod tool_manifest;
/// Tool set preset resolver.
pub mod tool_manifest_resolver;
/// Pure validation helpers.
pub mod validate;

/// Known ECC package identifiers in npm paths.
pub const ECC_PACKAGE_IDENTIFIERS: &[&str] = &["@lebocqtitouan/ecc/", "everything-claude-code/"];
