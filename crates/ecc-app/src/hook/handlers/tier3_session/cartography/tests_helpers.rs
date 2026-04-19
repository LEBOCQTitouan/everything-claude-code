//! Tests for cartography hook handlers — decomposed into focused submodules.
//!
//! - `start_test_helpers`: session start, scaffold creation, dirty-state handling,
//!   lock idempotency, archive/reset behavior, and safety-net edge cases.
//! - `delta_test_helpers`: agent interaction, enriched context dispatch, journey/flow
//!   schema validation, delta-merge, GAP markers, external I/O detection.
//! - `stop_test_helpers`: element wiring — elements/ scaffold, element generator dispatch,
//!   INDEX.md lifecycle, and element failure/success archiving.
use super::*;

#[path = "start_test_helpers.rs"]
mod start_test_helpers;

#[path = "delta_test_helpers.rs"]
mod delta_test_helpers;

#[path = "stop_test_helpers.rs"]
mod stop_test_helpers;

