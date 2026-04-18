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
    /// The status at this point in the trail.
    pub status: TaskStatus,
    /// ISO 8601 timestamp when this status was recorded.
    pub timestamp: String,
    /// Error detail for `Failed` segments (None for other statuses).
    pub error_detail: Option<String>,
}

/// A parsed task entry from tasks.md.
#[derive(Debug, Clone, Serialize)]
pub struct TaskEntry {
    /// The kind of entry (Pass Condition or Post-TDD activity).
    pub kind: EntryKind,
    /// The description text of the task.
    pub description: String,
    /// The command to run (Present only for `Pc` entries; `None` for `PostTdd`).
    pub command: Option<String>,
    /// Whether the checkbox is marked `[x]`.
    pub completed: bool,
    /// Ordered list of status trail segments tracking changes over time.
    pub trail: Vec<StatusSegment>,
}

/// Summary report produced by parsing a tasks.md file.
#[derive(Debug, Serialize)]
pub struct TaskReport {
    /// All parsed task entries.
    pub entries: Vec<TaskEntry>,
    /// Total number of task entries.
    pub total: usize,
    /// Number of completed entries.
    pub completed: usize,
    /// Number of pending entries.
    pub pending: usize,
    /// Number of entries in progress.
    pub in_progress: usize,
    /// Number of failed entries.
    pub failed: usize,
    /// Progress percentage (0-100).
    pub progress_pct: f64,
}
