//! Deduplication utilities for session deltas.
//!
//! Provides canonical hashing so that two `SessionDelta` values with identical
//! content but different `changed_files` insertion order produce the same hash.

use sha2::{Digest, Sha256};

use crate::cartography::types::SessionDelta;

/// Computes a canonical, order-independent SHA-256 hash of a `SessionDelta`.
///
/// Algorithm:
/// 1. Sort `changed_files` by path.
/// 2. Serialize the sorted delta via `serde_jcs` (RFC 8785 canonical JSON).
/// 3. SHA-256 the canonical JSON bytes.
/// 4. Return a lowercase 64-character hex string.
pub fn canonical_hash(delta: &SessionDelta) -> String {
    let mut sorted = delta.clone();
    sorted.changed_files.sort_by(|a, b| a.path.cmp(&b.path));
    let canonical = serde_jcs::to_string(&sorted).expect("serde_jcs serialization failed");
    let hash = Sha256::digest(canonical.as_bytes());
    format!("{hash:x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cartography::types::{ChangedFile, ProjectType, SessionDelta};

    fn make_delta(session_id: &str, files: Vec<(&str, &str)>) -> SessionDelta {
        SessionDelta {
            session_id: session_id.to_string(),
            timestamp: 1_700_000_000,
            changed_files: files
                .into_iter()
                .map(|(path, class)| ChangedFile {
                    path: path.to_string(),
                    classification: class.to_string(),
                })
                .collect(),
            project_type: ProjectType::Rust,
        }
    }

    #[test]
    fn hash_format_sha256_hex() {
        let delta = make_delta("sess-1", vec![("foo.rs", "crate")]);
        let h = canonical_hash(&delta);
        assert_eq!(h.len(), 64, "SHA-256 hex digest must be 64 chars");
        assert!(
            h.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()),
            "hash must be lowercase hex, got: {h}"
        );
    }

    #[test]
    fn hash_snapshot_stability() {
        // Fixture: one file, fixed metadata. If serde_json or serde_jcs
        // change their canonicalization in a minor bump, this test catches it.
        let delta = SessionDelta {
            session_id: "sess-snapshot".to_string(),
            timestamp: 1_700_000_000,
            changed_files: vec![ChangedFile {
                path: "crates/ecc-domain/src/foo.rs".to_string(),
                classification: "ecc-domain".to_string(),
            }],
            project_type: ProjectType::Rust,
        };
        let actual = canonical_hash(&delta);
        let expected = "ed97c4b74f0ca17abb8bbfe469ade4f9b7b1d8b5101f23cc89d921d1e42cd2bf";
        assert_eq!(
            actual, expected,
            "canonical_hash drift detected — serde_json or serde_jcs may have bumped canonicalization"
        );
    }

    #[test]
    fn hash_is_canonical_and_deterministic() {
        let a = make_delta(
            "sess-1",
            vec![
                ("crates/ecc-domain/src/foo.rs", "ecc-domain"),
                ("docs/README.md", "docs"),
            ],
        );
        let b = make_delta(
            "sess-1",
            vec![
                ("docs/README.md", "docs"),
                ("crates/ecc-domain/src/foo.rs", "ecc-domain"),
            ],
        );
        assert_eq!(
            canonical_hash(&a),
            canonical_hash(&b),
            "hash must be order-independent"
        );
    }
}
