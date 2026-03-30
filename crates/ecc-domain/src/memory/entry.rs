//! Core memory entry types: MemoryEntry, MemoryId.

use crate::memory::tier::MemoryTier;
use std::fmt;

/// Newtype wrapper for a memory entry's primary key (i64).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemoryId(pub i64);

impl fmt::Display for MemoryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i64> for MemoryId {
    fn from(id: i64) -> Self {
        MemoryId(id)
    }
}

/// A single memory entry in the three-tier memory system.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryEntry {
    /// Primary key (0 for unsaved entries).
    pub id: MemoryId,
    /// Tier classification: working, episodic, or semantic.
    pub tier: MemoryTier,
    /// Short title for display.
    pub title: String,
    /// Full content of the memory.
    pub content: String,
    /// Categorical tags.
    pub tags: Vec<String>,
    /// Optional project scope.
    pub project_id: Option<String>,
    /// Optional session identifier.
    pub session_id: Option<String>,
    /// Relevance score (higher = more important).
    pub relevance_score: f64,
    /// ISO-8601 creation timestamp.
    pub created_at: String,
    /// ISO-8601 last-update timestamp.
    pub updated_at: String,
    /// Whether this entry has been marked stale.
    pub stale: bool,
    /// Related BL-NNN work item references.
    pub related_work_items: Vec<String>,
    /// Original source path (for migration idempotency).
    pub source_path: Option<String>,
}

impl MemoryEntry {
    /// Create a new MemoryEntry with required fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: MemoryId,
        tier: MemoryTier,
        title: impl Into<String>,
        content: impl Into<String>,
        tags: Vec<String>,
        project_id: Option<String>,
        session_id: Option<String>,
        relevance_score: f64,
        created_at: impl Into<String>,
        updated_at: impl Into<String>,
        stale: bool,
        related_work_items: Vec<String>,
        source_path: Option<String>,
    ) -> Self {
        MemoryEntry {
            id,
            tier,
            title: title.into(),
            content: content.into(),
            tags,
            project_id,
            session_id,
            relevance_score,
            created_at: created_at.into(),
            updated_at: updated_at.into(),
            stale,
            related_work_items,
            source_path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::tier::MemoryTier;
    use std::collections::HashSet;

    fn make_entry() -> MemoryEntry {
        MemoryEntry::new(
            MemoryId(1),
            MemoryTier::Episodic,
            "Test Entry",
            "Some content here",
            vec!["rust".to_owned(), "ddd".to_owned()],
            Some("proj-123".to_owned()),
            Some("sess-456".to_owned()),
            1.0,
            "2026-01-01T00:00:00Z",
            "2026-01-02T00:00:00Z",
            false,
            vec!["BL-001".to_owned()],
            Some("/path/to/source.md".to_owned()),
        )
    }

    // PC-003: MemoryEntry construction with all fields; immutable struct, Clone + Debug
    #[test]
    fn test_entry_construction_all_fields() {
        let entry = make_entry();
        assert_eq!(entry.id, MemoryId(1));
        assert_eq!(entry.tier, MemoryTier::Episodic);
        assert_eq!(entry.title, "Test Entry");
        assert_eq!(entry.content, "Some content here");
        assert_eq!(entry.tags, vec!["rust", "ddd"]);
        assert_eq!(entry.project_id, Some("proj-123".to_owned()));
        assert_eq!(entry.session_id, Some("sess-456".to_owned()));
        assert!((entry.relevance_score - 1.0).abs() < f64::EPSILON);
        assert_eq!(entry.created_at, "2026-01-01T00:00:00Z");
        assert_eq!(entry.updated_at, "2026-01-02T00:00:00Z");
        assert!(!entry.stale);
        assert_eq!(entry.related_work_items, vec!["BL-001"]);
        assert_eq!(entry.source_path, Some("/path/to/source.md".to_owned()));
    }

    #[test]
    fn test_entry_clone() {
        let entry = make_entry();
        let cloned = entry.clone();
        assert_eq!(entry, cloned);
    }

    #[test]
    fn test_entry_debug() {
        let entry = make_entry();
        let s = format!("{:?}", entry);
        assert!(s.contains("MemoryEntry"));
        assert!(s.contains("Test Entry"));
    }

    #[test]
    fn test_entry_partial_eq() {
        let e1 = make_entry();
        let e2 = make_entry();
        assert_eq!(e1, e2);
    }

    #[test]
    fn test_entry_with_no_optional_fields() {
        let entry = MemoryEntry::new(
            MemoryId(0),
            MemoryTier::Working,
            "Minimal",
            "content",
            vec![],
            None,
            None,
            1.0,
            "2026-01-01T00:00:00Z",
            "2026-01-01T00:00:00Z",
            false,
            vec![],
            None,
        );
        assert!(entry.project_id.is_none());
        assert!(entry.session_id.is_none());
        assert!(entry.source_path.is_none());
    }

    // PC-004: MemoryId newtype wraps i64, Display, Eq, Hash
    #[test]
    fn test_memory_id_display() {
        let id = MemoryId(42);
        assert_eq!(id.to_string(), "42");
    }

    #[test]
    fn test_memory_id_eq() {
        let id1 = MemoryId(10);
        let id2 = MemoryId(10);
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_memory_id_not_eq() {
        let id1 = MemoryId(1);
        let id2 = MemoryId(2);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_memory_id_hash() {
        let mut set = HashSet::new();
        set.insert(MemoryId(1));
        set.insert(MemoryId(2));
        set.insert(MemoryId(1)); // duplicate
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_memory_id_copy() {
        let id1 = MemoryId(5);
        let id2 = id1; // Copy trait
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_memory_id_from_i64() {
        let id: MemoryId = 99i64.into();
        assert_eq!(id, MemoryId(99));
    }

    #[test]
    fn test_memory_id_wraps_negative() {
        let id = MemoryId(-1);
        assert_eq!(id.0, -1);
        assert_eq!(id.to_string(), "-1");
    }

    #[test]
    fn test_memory_id_debug() {
        let id = MemoryId(7);
        let s = format!("{:?}", id);
        assert!(s.contains("7"));
    }
}
