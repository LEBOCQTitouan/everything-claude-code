//! Task domain types: status FSM, entry model, parser, updater, and renderer.

pub mod entry;
pub mod error;
pub mod parser;
pub mod status;
pub mod updater;

pub use entry::{EntryKind, StatusSegment, TaskEntry, TaskReport};
pub use error::TaskError;
pub use status::TaskStatus;
