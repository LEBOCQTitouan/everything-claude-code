//! SourcesRegistry aggregate — pure domain logic for managing knowledge source entries.
//!
//! No I/O: all operations return new values (immutable pattern).

use super::entry::{Quadrant, SourceEntry, SourceError};

/// Maps a module path to a list of subjects relevant to that module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleMapping {
    pub module_path: String,
    pub subjects: Vec<String>,
}

/// Aggregate root for the knowledge sources registry.
///
/// Holds inbox entries (unclassified), classified entries, and module mappings.
/// All mutation methods return a new registry — the original is unchanged.
#[derive(Debug, Clone, Default)]
pub struct SourcesRegistry {
    pub inbox: Vec<SourceEntry>,
    pub entries: Vec<SourceEntry>,
    pub module_mappings: Vec<ModuleMapping>,
}

impl SourcesRegistry {
    /// Add a source entry to the registry.
    ///
    /// Checks for duplicate URL across entries and inbox. Returns a new registry
    /// with the entry appended to `entries`. Returns `Err(DuplicateUrl)` on conflict.
    pub fn add(&self, entry: SourceEntry) -> Result<SourcesRegistry, SourceError> {
        todo!()
    }

    /// Filter entries by optional quadrant and subject.
    ///
    /// Only searches `entries` (not inbox).
    pub fn list<'a>(
        &'a self,
        quadrant: Option<&Quadrant>,
        subject: Option<&str>,
    ) -> Vec<&'a SourceEntry> {
        todo!()
    }

    /// Move all inbox entries into `entries`, sorted by quadrant then subject.
    ///
    /// Returns a new registry with an empty inbox.
    pub fn reindex(&self) -> SourcesRegistry {
        todo!()
    }

    /// Return all entries for a given quadrant.
    pub fn entries_by_quadrant<'a>(&'a self, q: &Quadrant) -> Vec<&'a SourceEntry> {
        todo!()
    }

    /// Return unique subjects across all entries, sorted alphabetically.
    pub fn subjects(&self) -> Vec<&str> {
        todo!()
    }

    /// Find an entry by URL across both entries and inbox.
    pub fn find_by_url(&self, url: &str) -> Option<&SourceEntry> {
        todo!()
    }

    /// Find entries whose subject matches any subject mapped to the given module path.
    pub fn find_by_module<'a>(&'a self, module_path: &str) -> Vec<&'a SourceEntry> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::entry::{SourceType};

    fn make_entry(url: &str, quadrant: Quadrant, subject: &str) -> SourceEntry {
        SourceEntry {
            url: url.to_owned(),
            title: format!("Title for {url}"),
            source_type: SourceType::Doc,
            quadrant,
            subject: subject.to_owned(),
            added_by: "human".to_owned(),
            added_date: "2026-03-29".to_owned(),
            last_checked: None,
            deprecation_reason: None,
            stale: false,
        }
    }

    #[test]
    fn add_duplicate_rejected() {
        let registry = SourcesRegistry::default();
        let entry = make_entry("https://example.com/doc", Quadrant::Adopt, "testing");
        let registry = registry.add(entry.clone()).expect("first add should succeed");

        let duplicate = make_entry("https://example.com/doc", Quadrant::Trial, "other");
        let result = registry.add(duplicate);

        assert!(
            matches!(result, Err(SourceError::DuplicateUrl(url)) if url == "https://example.com/doc"),
            "expected DuplicateUrl error for same URL"
        );
    }

    #[test]
    fn add_returns_new() {
        let original = SourcesRegistry::default();
        let entry = make_entry("https://example.com/doc", Quadrant::Adopt, "testing");

        let new_registry = original.add(entry.clone()).expect("add should succeed");

        // Original is unchanged (immutable)
        assert!(original.entries.is_empty(), "original entries should be unmodified");

        // New registry contains the entry
        assert_eq!(new_registry.entries.len(), 1);
        assert_eq!(new_registry.entries[0].url, "https://example.com/doc");
    }

    #[test]
    fn list_filters() {
        let registry = SourcesRegistry {
            entries: vec![
                make_entry("https://example.com/adopt-testing", Quadrant::Adopt, "testing"),
                make_entry("https://example.com/adopt-rust", Quadrant::Adopt, "rust"),
                make_entry("https://example.com/trial-testing", Quadrant::Trial, "testing"),
            ],
            inbox: vec![],
            module_mappings: vec![],
        };

        // Filter by quadrant only
        let adopt_entries = registry.list(Some(&Quadrant::Adopt), None);
        assert_eq!(adopt_entries.len(), 2);

        // Filter by subject only
        let testing_entries = registry.list(None, Some("testing"));
        assert_eq!(testing_entries.len(), 2);

        // Filter by both quadrant and subject
        let adopt_testing = registry.list(Some(&Quadrant::Adopt), Some("testing"));
        assert_eq!(adopt_testing.len(), 1);
        assert_eq!(adopt_testing[0].url, "https://example.com/adopt-testing");

        // No filter returns all entries
        let all = registry.list(None, None);
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn reindex_moves_inbox() {
        let registry = SourcesRegistry {
            inbox: vec![
                make_entry("https://example.com/inbox1", Quadrant::Adopt, "testing"),
                make_entry("https://example.com/inbox2", Quadrant::Trial, "rust"),
            ],
            entries: vec![
                make_entry("https://example.com/existing", Quadrant::Assess, "cli"),
            ],
            module_mappings: vec![],
        };

        let reindexed = registry.reindex();

        // Inbox is empty after reindex
        assert!(reindexed.inbox.is_empty(), "inbox should be empty after reindex");

        // All entries present (inbox moved + existing preserved)
        assert_eq!(reindexed.entries.len(), 3);

        // Entries are sorted by quadrant then subject
        // Quadrant order: Adopt < Trial < Assess < Hold (alphabetical by display)
        // Subject alphabetical within quadrant
        let urls: Vec<&str> = reindexed.entries.iter().map(|e| e.url.as_str()).collect();
        assert!(
            urls.contains(&"https://example.com/inbox1"),
            "inbox1 should be in entries"
        );
        assert!(
            urls.contains(&"https://example.com/inbox2"),
            "inbox2 should be in entries"
        );
        assert!(
            urls.contains(&"https://example.com/existing"),
            "existing should remain in entries"
        );
    }

    #[test]
    fn find_by_module() {
        let registry = SourcesRegistry {
            entries: vec![
                make_entry("https://example.com/domain", Quadrant::Adopt, "domain-modeling"),
                make_entry("https://example.com/rust", Quadrant::Adopt, "rust-patterns"),
                make_entry("https://example.com/cli", Quadrant::Trial, "cli"),
            ],
            inbox: vec![],
            module_mappings: vec![
                ModuleMapping {
                    module_path: "crates/ecc-domain/".to_owned(),
                    subjects: vec!["domain-modeling".to_owned(), "rust-patterns".to_owned()],
                },
            ],
        };

        let results = registry.find_by_module("crates/ecc-domain/");
        assert_eq!(results.len(), 2);

        let urls: Vec<&str> = results.iter().map(|e| e.url.as_str()).collect();
        assert!(urls.contains(&"https://example.com/domain"));
        assert!(urls.contains(&"https://example.com/rust"));

        // Module not in mappings returns empty
        let none = registry.find_by_module("crates/ecc-cli/");
        assert!(none.is_empty());
    }
}
