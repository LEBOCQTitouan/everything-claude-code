//! Detection use cases.
//!
//! Filesystem-driven detection of languages, frameworks, and package
//! managers by scanning project files through [`ecc_ports::fs::FileSystem`].

pub mod framework;
pub mod language;
pub mod package_manager;
