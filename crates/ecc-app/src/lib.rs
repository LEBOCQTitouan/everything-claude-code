//! Application use cases for Everything Claude Code.
//!
//! Orchestrates [`ecc_domain`] business logic through [`ecc_ports`] trait
//! boundaries. Each module corresponds to a CLI command or workflow
//! (install, audit, merge, validate, etc.).

pub mod act_ci;
pub mod analyze;
pub mod audit;
pub mod audit_web;
pub mod backlog;
pub mod bypass_mgmt;
pub mod claw;
pub mod commit_lint;
pub mod config;
pub mod config_cmd;
pub mod cost_mgmt;
pub mod detect;
pub mod detection;
pub mod dev;
pub mod diagnostics;
pub mod diagram_triggers;
pub mod docs_coverage;
pub mod docs_update_summary;
pub mod drift_check;
pub mod hook;
pub mod install;
pub mod log_mgmt;
pub mod memory;
pub mod merge;
pub mod metrics_mgmt;
pub mod metrics_session;
pub mod session;
pub mod smart_merge;
pub mod sources;
pub mod update;
pub mod validate;
pub mod validate_cartography;
pub mod validate_claude_md;
pub mod validate_design;
pub mod validate_spec;
pub mod version;
pub mod workflow;
pub mod worktree;
