//! CRUD use cases for the memory system.

use ecc_domain::memory::{MemoryEntry, MemoryId, MemoryTier};
use ecc_ports::clock::Clock;
use ecc_ports::memory_store::{MemoryStore, MemoryStoreError};

/// Result type for memory app use cases.
pub type MemoryResult<T> = Result<T, MemoryAppError>;

/// App-layer errors for memory use cases.
#[derive(Debug, thiserror::Error)]
pub enum MemoryAppError {
    #[error("memory not found: {0}")]
    NotFound(MemoryId),
    #[error("store error: {0}")]
    Store(#[from] MemoryStoreError),
    #[error("content contains likely secrets: {0}")]
    SecretDetected(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("Already semantic")]
    AlreadySemantic,
}

/// Parameters for the `add` use case.
pub struct AddParams {
    pub title: String,
    pub content: String,
    pub tier: MemoryTier,
    pub tags: Vec<String>,
    pub project_id: Option<String>,
    pub session_id: Option<String>,
    pub force: bool,
}

/// Add a new memory entry.
///
/// Rejects content that contains likely secrets (unless `force=true`).
pub fn add(store: &dyn MemoryStore, params: AddParams, clock: &dyn Clock) -> MemoryResult<MemoryId> {
    if !params.force
        && let Some(detected) = ecc_domain::memory::secrets::contains_likely_secret(&params.content)
    {
        return Err(MemoryAppError::SecretDetected(detected));
    }

    let now = current_timestamp(clock);
    let entry = MemoryEntry::new(
        MemoryId(0),
        params.tier,
        params.title,
        params.content,
        params.tags,
        params.project_id,
        params.session_id,
        1.0,
        &now,
        &now,
        false,
        vec![],
        None,
    );

    store.insert(&entry).map_err(MemoryAppError::Store)
}

/// Search memories using full-text search.
///
/// The query is wrapped in double-quotes for FTS5 safety (sanitization).
/// Returns an empty vec on no results (not an error).
pub fn search(
    store: &dyn MemoryStore,
    query: &str,
    limit: usize,
) -> MemoryResult<Vec<MemoryEntry>> {
    let sanitized = format!("\"{}\"", query.replace('"', ""));
    store
        .search_fts(&sanitized, limit)
        .map_err(MemoryAppError::Store)
}

/// List entries with optional tier and tag filters.
pub fn list(
    store: &dyn MemoryStore,
    tier: Option<MemoryTier>,
    tag: Option<&str>,
    project_id: Option<&str>,
) -> MemoryResult<Vec<MemoryEntry>> {
    store
        .list_filtered(tier, tag, project_id)
        .map_err(MemoryAppError::Store)
}

/// Delete an entry by ID.
///
/// Returns `NotFound` error for non-existent IDs.
pub fn delete(store: &dyn MemoryStore, id: MemoryId) -> MemoryResult<()> {
    store.delete(id).map_err(|e| match e {
        MemoryStoreError::NotFound(id) => MemoryAppError::NotFound(id),
        other => MemoryAppError::Store(other),
    })
}

/// Get an entry by ID.
pub fn show(store: &dyn MemoryStore, id: MemoryId) -> MemoryResult<MemoryEntry> {
    store.get(id).map_err(|e| match e {
        MemoryStoreError::NotFound(id) => MemoryAppError::NotFound(id),
        other => MemoryAppError::Store(other),
    })
}

/// Return the current timestamp as an ISO-8601 string.
pub fn current_timestamp(clock: &dyn Clock) -> String {
    clock.now_iso8601()
}


#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::{InMemoryMemoryStore, TEST_CLOCK};

    fn make_store() -> InMemoryMemoryStore {
        InMemoryMemoryStore::new()
    }

    fn simple_add_params(title: &str, content: &str) -> AddParams {
        AddParams {
            title: title.to_owned(),
            content: content.to_owned(),
            tier: MemoryTier::Episodic,
            tags: vec![],
            project_id: None,
            session_id: None,
            force: false,
        }
    }

    // PC-034: App `add` use case inserts entry with type, content, tags, relevance_score=1.0
    #[test]
    fn test_add_inserts_entry() {
        let store = make_store();
        let params = AddParams {
            title: "Test Memory".to_owned(),
            content: "Some content here".to_owned(),
            tier: MemoryTier::Semantic,
            tags: vec!["rust".to_owned()],
            project_id: None,
            session_id: None,
            force: false,
        };
        let id = add(&store, params, &*TEST_CLOCK).unwrap();
        let entry = store.get(id).unwrap();
        assert_eq!(entry.title, "Test Memory");
        assert_eq!(entry.tier, MemoryTier::Semantic);
        assert_eq!(entry.tags, vec!["rust"]);
        assert!((entry.relevance_score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_add_with_secret_in_content_is_rejected() {
        let store = make_store();
        let params = AddParams {
            title: "Secret".to_owned(),
            content: "My key is sk-abc123def456ghi789jkl000mno111".to_owned(),
            tier: MemoryTier::Episodic,
            tags: vec![],
            project_id: None,
            session_id: None,
            force: false,
        };
        let result = add(&store, params, &*TEST_CLOCK);
        assert!(matches!(result, Err(MemoryAppError::SecretDetected(_))));
    }

    #[test]
    fn test_add_with_secret_force_bypasses_check() {
        let store = make_store();
        let params = AddParams {
            title: "Secret".to_owned(),
            content: "My key is sk-abc123def456ghi789jkl000mno111".to_owned(),
            tier: MemoryTier::Episodic,
            tags: vec![],
            project_id: None,
            session_id: None,
            force: true,
        };
        let result = add(&store, params, &*TEST_CLOCK);
        assert!(result.is_ok());
    }

    // PC-035: App `search` returns FTS results; empty result returns empty vec + no error
    #[test]
    fn test_search_empty_results_returns_empty_vec_not_error() {
        let store = make_store();
        // Empty store — no results expected
        let results = search(&store, "nonexistent_query_xyz", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_sanitizes_query_with_double_quotes_no_error() {
        let store = make_store();
        // FTS5 operators stripped via double-quoting — should not error
        let results = search(&store, "hello OR world", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_sanitizes_query_double_quote_escape() {
        // Verify the sanitization strips inner double-quotes from input
        // (preventing FTS5 injection)
        let q = "hello \"world\"";
        let sanitized = format!("\"{}\"", q.replace('"', ""));
        assert_eq!(sanitized, "\"hello world\"");
    }

    #[test]
    fn test_search_returns_ok_for_any_query() {
        let store = make_store();
        // The result type is Ok even when there are entries (tests the Ok path)
        add(&store, simple_add_params("some entry", "some content"), &*TEST_CLOCK).unwrap();
        // We don't check count here — just that it doesn't error
        let result = search(&store, "some", 10);
        assert!(result.is_ok());
    }

    // PC-036: App `list` use case filters by type and tag
    #[test]
    fn test_list_filters_by_tier() {
        let store = make_store();
        let p1 = AddParams {
            title: "S1".to_owned(),
            content: "content".to_owned(),
            tier: MemoryTier::Semantic,
            tags: vec![],
            project_id: None,
            session_id: None,
            force: false,
        };
        let p2 = AddParams {
            title: "E1".to_owned(),
            content: "content".to_owned(),
            tier: MemoryTier::Episodic,
            tags: vec![],
            project_id: None,
            session_id: None,
            force: false,
        };
        add(&store, p1, &*TEST_CLOCK).unwrap();
        add(&store, p2, &*TEST_CLOCK).unwrap();
        let results = list(&store, Some(MemoryTier::Semantic), None, None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "S1");
    }

    #[test]
    fn test_list_filters_by_tag() {
        let store = make_store();
        let p1 = AddParams {
            title: "T1".to_owned(),
            content: "content".to_owned(),
            tier: MemoryTier::Episodic,
            tags: vec!["rust".to_owned()],
            project_id: None,
            session_id: None,
            force: false,
        };
        let p2 = AddParams {
            title: "T2".to_owned(),
            content: "content".to_owned(),
            tier: MemoryTier::Episodic,
            tags: vec!["python".to_owned()],
            project_id: None,
            session_id: None,
            force: false,
        };
        add(&store, p1, &*TEST_CLOCK).unwrap();
        add(&store, p2, &*TEST_CLOCK).unwrap();
        let results = list(&store, None, Some("rust"), None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "T1");
    }

    // PC-037: App `delete` removes entry; non-existent ID returns NotFound error
    #[test]
    fn test_delete_removes_entry() {
        let store = make_store();
        let id = add(&store, simple_add_params("Entry", "content"), &*TEST_CLOCK).unwrap();
        delete(&store, id).unwrap();
        assert!(store.get(id).is_err());
    }

    #[test]
    fn test_delete_nonexistent_returns_not_found() {
        let store = make_store();
        let result = delete(&store, MemoryId(999));
        assert!(matches!(result, Err(MemoryAppError::NotFound(_))));
    }
}
