pub mod audit;
pub mod clean;
pub mod deny_rules;
pub mod detect;
pub mod gitignore;
pub mod manifest;
pub mod merge;
pub mod validate;

/// Known ECC package identifiers in npm paths.
pub const ECC_PACKAGE_IDENTIFIERS: &[&str] =
    &["@lebocqtitouan/ecc/", "everything-claude-code/"];
