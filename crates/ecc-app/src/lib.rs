//! Application use cases for Everything Claude Code.
//!
//! Orchestrates [`ecc_domain`] business logic through [`ecc_ports`] trait
//! boundaries. Each module corresponds to a CLI command or workflow
//! (install, audit, merge, validate, etc.).

pub mod act_ci;
pub mod audit;
pub mod audit_web;
pub mod backlog;
pub mod claw;
pub mod config;
pub mod config_cmd;
pub mod detect;
pub mod detection;
pub mod dev;
pub mod diagnostics;
pub mod hook;
pub mod install;
pub mod log_mgmt;
pub mod memory;
pub mod merge;
pub mod session;
pub mod smart_merge;
pub mod sources;
pub mod update;
pub mod validate;
pub mod validate_design;
pub mod validate_spec;
pub mod version;
pub mod workflow;
pub mod worktree;
