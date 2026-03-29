//! Task entry types: `TaskEntry`, `EntryKind`, `StatusSegment`, and `TaskReport`.

use serde::Serialize;

use crate::spec::pc::PcId;
use crate::task::status::TaskStatus;

/// Distinguishes a Pass Condition entry from a Post-TDD entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind")]
pub enum EntryKind {
    /// A numbered pass condition (e.g. `PC-003`).
    Pc(PcId),
    /// A post-TDD activity label (e.g. `"E2E tests"`, `"Code review"`).
    PostTdd(String),
}

/// A single segment of the status trail (e.g. `pending@2026-03-29T13:59:20Z`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StatusSegment {
    pub status: TaskStatus,
    pub timestamp: String,
    /// Non-empty only for `Failed` segments that carry an error detail.
    pub error_detail: Option<String>,
}

/// A parsed task entry from tasks.md.
#[derive(Debug, Clone, Serialize)]
pub struct TaskEntry {
    pub kind: EntryKind,
    pub description: String,
    /// Present only for `Pc` entries; `None` for `PostTdd`.
    pub command: Option<String>,
    /// `true` when the checkbox is `[x]`.
    pub completed: bool,
    /// Ordered list of status trail segments.
    pub trail: Vec<StatusSegment>,
}

/// Summary report produced by parsing a tasks.md file.
#[derive(Debug, Serialize)]
pub struct TaskReport {
    pub entries: Vec<TaskEntry>,
    pub total: usize,
    pub completed: usize,
    pub pending: usize,
    pub in_progress: usize,
    pub failed: usize,
    pub progress_pct: f64,
}
