//! Memory domain errors.

use thiserror::Error;

/// Errors that can occur in the memory domain.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum MemoryError {
    /// The requested memory entry was not found.
    #[error("Memory not found: {0}")]
    NotFound(i64),

    /// Attempted to promote an entry that is already semantic tier.
    #[error("Already semantic")]
    AlreadySemantic,

    /// The SQLite database is corrupted.
    #[error("Database corrupted: {0}")]
    DatabaseCorrupted(String),

    /// Migration of legacy memory files failed.
    #[error("Migration failed: {0}")]
    MigrationFailed(String),

    /// The tier string could not be parsed.
    #[error("Invalid tier: '{0}' (expected working, episodic, or semantic)")]
    InvalidTier(String),

    /// The memory ID is invalid (e.g., non-positive).
    #[error("Invalid memory ID: {0}")]
    InvalidId(String),

    /// Export to file failed.
    #[error("Export failed: {0}")]
    ExportFailed(String),

    /// Content contains likely secrets.
    #[error("Content contains likely secrets: {0}")]
    SecretDetected(String),

    /// Duplicate content detected.
    #[error("Duplicate content detected")]
    DuplicateContent,
}

#[cfg(test)]
mod tests {
    use super::*;

    // PC-005: MemoryError variants all implement Display with meaningful messages
    #[test]
    fn test_error_not_found_display() {
        let e = MemoryError::NotFound(42);
        assert!(e.to_string().contains("42"));
        assert!(e.to_string().contains("not found") || e.to_string().contains("Not found"));
    }

    #[test]
    fn test_error_already_semantic_display() {
        let e = MemoryError::AlreadySemantic;
        assert_eq!(e.to_string(), "Already semantic");
    }

    #[test]
    fn test_error_database_corrupted_display() {
        let e = MemoryError::DatabaseCorrupted("bad magic number".to_owned());
        assert!(e.to_string().contains("corrupted") || e.to_string().contains("Corrupted"));
        assert!(e.to_string().contains("bad magic number"));
    }

    #[test]
    fn test_error_migration_failed_display() {
        let e = MemoryError::MigrationFailed("parse error".to_owned());
        assert!(e.to_string().contains("Migration") || e.to_string().contains("migration"));
        assert!(e.to_string().contains("parse error"));
    }

    #[test]
    fn test_error_invalid_tier_display() {
        let e = MemoryError::InvalidTier("garbage".to_owned());
        assert!(e.to_string().contains("garbage"));
    }

    #[test]
    fn test_error_invalid_id_display() {
        let e = MemoryError::InvalidId("not-a-number".to_owned());
        assert!(e.to_string().contains("not-a-number"));
    }

    #[test]
    fn test_error_export_failed_display() {
        let e = MemoryError::ExportFailed("permission denied".to_owned());
        assert!(e.to_string().contains("permission denied"));
    }

    #[test]
    fn test_error_secret_detected_display() {
        let e = MemoryError::SecretDetected("API key".to_owned());
        assert!(e.to_string().contains("API key"));
    }

    #[test]
    fn test_error_duplicate_content_display() {
        let e = MemoryError::DuplicateContent;
        assert!(!e.to_string().is_empty());
    }

    #[test]
    fn test_error_clone_and_eq() {
        let e1 = MemoryError::NotFound(1);
        let e2 = e1.clone();
        assert_eq!(e1, e2);
    }

    #[test]
    fn test_error_debug() {
        let e = MemoryError::InvalidTier("bad".to_owned());
        let s = format!("{:?}", e);
        assert!(s.contains("InvalidTier"));
    }
}
