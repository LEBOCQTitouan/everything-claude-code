//! Pure business logic for Everything Claude Code.
//!
//! This crate contains domain types, value objects, and business rules with
//! zero I/O dependencies. All side effects are pushed to the [`ecc_ports`] layer.
#![warn(missing_docs)]

/// Code analysis and metrics (coupling, hotspots, bus factor).
pub mod analyze;
/// ANSI color codes and terminal formatting.
pub mod ansi;
/// Audit report web scraping and validation.
pub mod audit_web;
/// Backlog item management and tracking.
pub mod backlog;
/// Cartography (code mapping and architecture analysis).
pub mod cartography;
/// NanoClaw REPL session management.
pub mod claw;
/// Configuration parsing and validation.
pub mod config;
/// Cost estimation and token budgets.
pub mod cost;
/// Programming language and framework auto-detection.
pub mod detection;
/// Line-level text diffing (LCS-based).
pub mod diff;
/// Documentation generation (coverage, diagrams, etc.).
pub mod docs;
/// Spec-vs-implementation drift detection.
pub mod drift;
/// Hook profile and runtime execution.
pub mod hook_runtime;
/// Structured logging types.
pub mod log;
/// Session memory and action tracking.
pub mod memory;
/// Session metrics and cost tracking.
pub mod metrics;
/// File path utilities and manipulation.
pub mod paths;
/// User session state management.
pub mod session;
/// Knowledge sources registry and analysis.
pub mod sources;
/// Specification and design artifact parsing.
pub mod spec;
/// Task status and TDD lifecycle.
pub mod task;
/// Date and time utilities.
pub mod time;
/// Domain trait definitions.
pub mod traits;
/// Update planning and versioning.
pub mod update;
/// Workflow phase state machine.
pub mod workflow;
/// Git worktree lifecycle management.
pub mod worktree;
