//! Harness reliability metric types.
//!
//! Pure domain types for recording and aggregating harness operational metrics:
//! hook executions, phase transitions, agent spawns, and commit gates.

pub mod aggregate;
pub mod error;
pub mod event;

pub use aggregate::{HarnessMetrics, MetricAggregator};
pub use error::MetricsError;
pub use event::{CommitGateKind, MetricEvent, MetricEventType, MetricOutcome};
