//! NanoClaw REPL domain types.
//!
//! Models for the session-aware REPL built on `claude -p`, including
//! commands, prompt construction, session management, metrics tracking,
//! model selection, search, and turn history.

/// REPL command types and parsing.
pub mod command;
/// Session compaction and summary.
pub mod compact;
/// Session export formats.
pub mod export;
/// Session metrics (turns, tokens, etc.).
pub mod metrics;
/// Claude model selection.
pub mod model;
/// Prompt construction and templates.
pub mod prompt;
/// Session search and filtering.
pub mod search;
/// Session name parsing and generation.
pub mod session_name;
/// Conversation turn types and formatting.
pub mod turn;
