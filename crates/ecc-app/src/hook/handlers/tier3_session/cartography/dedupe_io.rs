//! I/O-level deduplication for cartography session deltas.
//!
//! Scans pending and processed delta files to detect whether an incoming
//! `SessionDelta` is a duplicate of a recently written one.

use std::path::{Path, PathBuf};

use ecc_domain::cartography::{ChangedFile, SessionDelta};
use ecc_ports::fs::FileSystem;
use sha2::{Digest, Sha256};

/// Outcome of a deduplication check.
#[derive(Debug, PartialEq, Eq)]
pub enum DedupeOutcome {
    /// A previously written delta with the same content hash exists.
    /// Contains the session_id of the matching delta.
    Duplicate(String),
    /// No matching delta found; it is safe to write.
    Unique,
    /// flock acquisition timed out — fail-open, proceed to write.
    LockBusy,
    /// Deduplication window is disabled (`ECC_CARTOGRAPHY_DEDUPE_WINDOW=0`).
    WindowDisabled,
}

/// Compute a content-only hash of a `SessionDelta`, excluding `session_id` and `timestamp`.
///
/// This allows detecting duplicate *payloads* across sessions — i.e. two deltas from
/// different sessions that modified the same files.
pub fn content_hash(delta: &SessionDelta) -> String {
    let mut files: Vec<&ChangedFile> = delta.changed_files.iter().collect();
    files.sort_by(|a, b| a.path.cmp(&b.path));

    let mut hasher = Sha256::new();
    // Hash project type
    let project_type = serde_json::to_string(&delta.project_type).unwrap_or_default();
    hasher.update(project_type.as_bytes());
    // Hash each file entry in sorted order
    for f in &files {
        hasher.update(f.path.as_bytes());
        hasher.update(b"\x00");
        hasher.update(f.classification.as_bytes());
        hasher.update(b"\x01");
    }
    format!("{:x}", hasher.finalize())
}

/// Check whether `delta` duplicates any of the last `window` deltas on disk.
///
/// # Algorithm
///
/// 1. If `window == 0` → [`DedupeOutcome::WindowDisabled`].
/// 2. Collect `pending-delta-*.json` from `cartography_dir/` and
///    `cartography_dir/processed/`.
/// 3. Sort file names lexicographically descending, take first `window`.
/// 4. For each candidate, deserialise and compare `content_hash`.
/// 5. On hash match → [`DedupeOutcome::Duplicate`]; otherwise → [`DedupeOutcome::Unique`].
pub fn should_dedupe(
    fs: &dyn FileSystem,
    cartography_dir: &Path,
    delta: &SessionDelta,
    window: usize,
) -> DedupeOutcome {
    if window == 0 {
        return DedupeOutcome::WindowDisabled;
    }

    let incoming_hash = content_hash(delta);

    let mut candidates: Vec<PathBuf> = collect_delta_files(fs, cartography_dir);
    candidates.sort_by(|a, b| b.cmp(a)); // lexicographic descending
    candidates.truncate(window);

    for candidate in candidates {
        let content = match fs.read_to_string(&candidate) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let existing: SessionDelta = match serde_json::from_str(&content) {
            Ok(d) => d,
            Err(_) => continue,
        };
        if content_hash(&existing) == incoming_hash {
            return DedupeOutcome::Duplicate(existing.session_id);
        }
    }

    DedupeOutcome::Unique
}

/// Collect `pending-delta-*.json` from the pending dir and the processed sub-dir.
fn collect_delta_files(fs: &dyn FileSystem, cartography_dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();

    for dir in [
        cartography_dir.to_path_buf(),
        cartography_dir.join("processed"),
    ] {
        if let Ok(entries) = fs.read_dir(&dir) {
            for entry in entries {
                if is_delta_filename(&entry) {
                    files.push(entry);
                }
            }
        }
    }

    files
}

fn is_delta_filename(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with("pending-delta-") && n.ends_with(".json"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_domain::cartography::{ChangedFile, ProjectType};
    use ecc_test_support::InMemoryFileSystem;

    fn make_delta(session_id: &str, files: Vec<(&str, &str)>) -> SessionDelta {
        SessionDelta {
            session_id: session_id.to_owned(),
            timestamp: 1_000_000,
            changed_files: files
                .into_iter()
                .map(|(p, c)| ChangedFile {
                    path: p.to_owned(),
                    classification: c.to_owned(),
                })
                .collect(),
            project_type: ProjectType::Rust,
        }
    }

    #[test]
    fn duplicate_payload_skips_write() {
        // Arrange: build an existing delta and write it to the in-memory fs.
        let existing = make_delta("session-aaa", vec![("src/main.rs", "ecc-cli")]);
        let existing_json = serde_json::to_string(&existing).unwrap();

        let fs = InMemoryFileSystem::new().with_file(
            "/cartography/pending-delta-session-aaa.json",
            &existing_json,
        );

        // New delta has the same content (different session_id, same files).
        let incoming = make_delta("session-bbb", vec![("src/main.rs", "ecc-cli")]);

        // Act
        let outcome = should_dedupe(&fs, Path::new("/cartography"), &incoming, 10);

        // Assert: recognised as duplicate of the existing delta.
        assert_eq!(outcome, DedupeOutcome::Duplicate("session-aaa".to_owned()));
    }
}
