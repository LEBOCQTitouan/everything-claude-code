//! I/O-level deduplication for cartography session deltas.
//!
//! Scans pending and processed delta files to detect whether an incoming
//! `SessionDelta` is a duplicate of a recently written one.

use std::path::{Path, PathBuf};
use std::time::Duration;

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

    let timeout = Duration::from_millis(500);
    let _lock = match ecc_flock::acquire_for_with_timeout(cartography_dir, "dedupe", timeout) {
        Ok(lock) => lock,
        Err(_) => return DedupeOutcome::LockBusy,
    };

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
    use tempfile::TempDir;

    /// Create a temp dir and return a pre-seeded `InMemoryFileSystem` where all
    /// paths use `tmp.path()` as the cartography root.  The temp dir is returned
    /// so its lifetime extends through the test.
    fn make_fs_with_pending(tmp: &TempDir, files: &[(&str, &str)]) -> InMemoryFileSystem {
        let mut fs = InMemoryFileSystem::new();
        for (rel, content) in files {
            let full = tmp.path().join(rel);
            fs = fs.with_file(&full.to_string_lossy(), content);
        }
        fs
    }

    #[test]
    fn reads_last_n_through_filesystem_port() {
        // Verify dedupe_io uses only pre-existing FileSystem port methods by
        // inspecting the non-test portion of this module's source. We check that
        // only `read_dir` and `read_to_string` — both methods present in the port
        // trait before this feature — are called, and no new port methods appear.
        const SOURCE: &str = include_str!("dedupe_io.rs");

        // Isolate the non-test portion: everything before the `#[cfg(test)]` block.
        // This prevents the test body itself from polluting the search space.
        let production_source = SOURCE
            .split("\n#[cfg(test)]")
            .next()
            .expect("source must have a #[cfg(test)] block");

        // Positive assertion: confirm the two established port methods are used in
        // the production code.
        assert!(
            production_source.contains("read_dir"),
            "dedupe_io must use FileSystem::read_dir to list candidates"
        );
        assert!(
            production_source.contains("read_to_string"),
            "dedupe_io must use FileSystem::read_to_string to load candidates"
        );

        // Negative assertion: no method calls other than the two above on `fs`.
        // We look for `fs.` call sites — only `read_dir` and `read_to_string`
        // should appear.
        let fs_call_sites: Vec<&str> = production_source
            .lines()
            .filter(|l| l.contains("fs."))
            .collect();
        for line in &fs_call_sites {
            let trimmed = line.trim();
            assert!(
                trimmed.contains("fs.read_dir") || trimmed.contains("fs.read_to_string"),
                "dedupe_io calls an unexpected FileSystem method: {trimmed}"
            );
        }

        // Behavioural: seed 25 delta files and verify should_dedupe detects a
        // duplicate within the window without needing to read all 25 files.
        let base_delta = make_delta("session-base", vec![("src/lib.rs", "ecc-app")]);
        let base_json = serde_json::to_string(&base_delta).unwrap();

        let tmp = TempDir::new().unwrap();

        // Build in-memory FS with 25 delta files (names session-000..session-024).
        // Lexicographic descending sort means the last N = window are checked first.
        let mut fs = InMemoryFileSystem::new();
        for i in 0..25_usize {
            let name = tmp
                .path()
                .join(format!("pending-delta-session-{i:03}.json"))
                .to_string_lossy()
                .into_owned();
            fs = fs.with_file(&name, &base_json);
        }

        // Use window=5: only the 5 newest files are checked.
        let incoming = make_delta("session-new", vec![("src/lib.rs", "ecc-app")]);
        let outcome = should_dedupe(&fs, tmp.path(), &incoming, 5);

        // The incoming delta matches base_delta, so Duplicate must be returned,
        // proving that the window files were read through FileSystem::read_to_string.
        assert!(
            matches!(outcome, DedupeOutcome::Duplicate(_)),
            "expected Duplicate when window covers matching file, got {outcome:?}"
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri cannot interpret libc::flock FFI calls
    fn flock_timeout_fail_open() {
        use std::time::{Duration, Instant};
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();

        // Hold the lock via ecc-flock before calling should_dedupe
        let _lock = ecc_flock::acquire_for(dir, "dedupe").unwrap();

        // Build a minimal delta and in-memory FS
        let delta = make_delta("session-test", vec![("src/lib.rs", "ecc-app")]);
        let fs = InMemoryFileSystem::new();

        let start = Instant::now();
        let outcome = should_dedupe(&fs, dir, &delta, 20);
        let elapsed = start.elapsed();

        assert!(
            matches!(outcome, DedupeOutcome::LockBusy),
            "expected LockBusy when lock is held, got {outcome:?}"
        );
        assert!(
            elapsed < Duration::from_millis(1500),
            "timeout took too long: {elapsed:?}"
        );
    }

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
    fn scans_pending_and_processed_desc() {
        // Arrange: 1 file in pending dir, 1 in processed/ sub-dir
        let delta_a = make_delta("session-aaa", vec![("src/lib.rs", "ecc-app")]);
        let delta_b = make_delta("session-bbb", vec![("src/main.rs", "ecc-cli")]);
        let json_a = serde_json::to_string(&delta_a).unwrap();
        let json_b = serde_json::to_string(&delta_b).unwrap();

        let tmp = TempDir::new().unwrap();
        let fs = make_fs_with_pending(
            &tmp,
            &[
                ("pending-delta-aaa.json", &json_a),
                ("processed/pending-delta-bbb.json", &json_b),
            ],
        );

        // Input matches delta_a (in pending/) → Duplicate
        let incoming_a = make_delta("session-new-a", vec![("src/lib.rs", "ecc-app")]);
        let outcome_a = should_dedupe(&fs, tmp.path(), &incoming_a, 20);
        assert!(
            matches!(outcome_a, DedupeOutcome::Duplicate(_)),
            "expected Duplicate for pending/ match, got {outcome_a:?}"
        );

        // Input matches delta_b (in processed/) → Duplicate (proves processed/ is scanned)
        let incoming_b = make_delta("session-new-b", vec![("src/main.rs", "ecc-cli")]);
        let outcome_b = should_dedupe(&fs, tmp.path(), &incoming_b, 20);
        assert!(
            matches!(outcome_b, DedupeOutcome::Duplicate(_)),
            "expected Duplicate for processed/ match, got {outcome_b:?}"
        );
    }

    #[test]
    fn window_0_1_100_boundaries() {
        let tmp0 = TempDir::new().unwrap();
        let fs0 = InMemoryFileSystem::new();
        let fixture = make_delta("session-fixture", vec![("src/lib.rs", "ecc-app")]);
        assert!(
            matches!(should_dedupe(&fs0, tmp0.path(), &fixture, 0), DedupeOutcome::WindowDisabled),
            "window=0 must return WindowDisabled"
        );

        // Window=1: two files — z (newer lex) and a (older lex).
        // Input matches delta_a. With window=1 only z is checked → Unique.
        let tmp1 = TempDir::new().unwrap();
        let delta_a = make_delta("session-aaa", vec![("src/lib.rs", "ecc-app")]);
        let delta_z = make_delta("session-zzz", vec![("src/main.rs", "ecc-cli")]);
        let json_a = serde_json::to_string(&delta_a).unwrap();
        let json_z = serde_json::to_string(&delta_z).unwrap();
        let fs1 = make_fs_with_pending(
            &tmp1,
            &[
                ("pending-delta-a.json", &json_a),
                ("pending-delta-z.json", &json_z),
            ],
        );
        // Incoming matches delta_a but delta_a is at lex position "a" < "z".
        // window=1 only picks "z" → no match → Unique.
        let incoming_a = make_delta("session-new", vec![("src/lib.rs", "ecc-app")]);
        assert!(
            matches!(should_dedupe(&fs1, tmp1.path(), &incoming_a, 1), DedupeOutcome::Unique),
            "window=1 must exclude older-lex delta_a and return Unique"
        );

        // Window=100: 5 seeded deltas, match is at position 0 (lex "0").
        // window=100 covers all 5 → Duplicate detected.
        let tmp100 = TempDir::new().unwrap();
        let deltas: Vec<_> = (0..5_usize)
            .map(|i| make_delta(&format!("session-{i}"), vec![(&format!("src/file{i}.rs"), "ecc-app")]))
            .collect();
        let jsons: Vec<_> = deltas.iter().map(|d| serde_json::to_string(d).unwrap()).collect();
        let file_pairs: Vec<(String, String)> = (0..5)
            .map(|i| (format!("pending-delta-{i}.json"), jsons[i].clone()))
            .collect();
        let fs100 = {
            let mut fs = InMemoryFileSystem::new();
            for (name, content) in &file_pairs {
                let full = tmp100.path().join(name);
                fs = fs.with_file(&full.to_string_lossy(), content);
            }
            fs
        };
        // Incoming matches deltas[0] — which is at lex "pending-delta-0.json"
        // Lex descending order: 4, 3, 2, 1, 0 — window=100 covers all 5.
        let incoming100 = make_delta("session-new100", vec![("src/file0.rs", "ecc-app")]);
        assert!(
            matches!(should_dedupe(&fs100, tmp100.path(), &incoming100, 100), DedupeOutcome::Duplicate(_)),
            "window=100 must detect duplicate at bottom of lex-sorted list"
        );
    }

    /// POSIX flock releases the lock when the fd is closed (i.e. when the process dies).
    /// A stale `.dedupe.lock` file that no process holds must NOT cause `LockBusy`.
    #[test]
    #[cfg_attr(miri, ignore)] // Miri cannot interpret libc::flock FFI calls
    fn stale_lock_recoverable() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();

        // Create a stale .dedupe.lock file — existence alone does not hold flock.
        let lock_path = dir.join(".dedupe.lock");
        std::fs::write(&lock_path, b"").unwrap();

        let delta = make_delta("session-stale", vec![("src/lib.rs", "ecc-app")]);
        let fs = InMemoryFileSystem::new();

        let outcome = should_dedupe(&fs, dir, &delta, 20);

        assert!(
            !matches!(outcome, DedupeOutcome::LockBusy),
            "stale lock file must not cause LockBusy; got {outcome:?}"
        );
    }

    #[test]
    fn duplicate_payload_skips_write() {
        // Arrange: build an existing delta and write it to the in-memory fs.
        let existing = make_delta("session-aaa", vec![("src/main.rs", "ecc-cli")]);
        let existing_json = serde_json::to_string(&existing).unwrap();

        let tmp = TempDir::new().unwrap();
        let fs = make_fs_with_pending(&tmp, &[("pending-delta-session-aaa.json", &existing_json)]);

        // New delta has the same content (different session_id, same files).
        let incoming = make_delta("session-bbb", vec![("src/main.rs", "ecc-cli")]);

        // Act
        let outcome = should_dedupe(&fs, tmp.path(), &incoming, 10);

        // Assert: recognised as duplicate of the existing delta.
        assert_eq!(outcome, DedupeOutcome::Duplicate("session-aaa".to_owned()));
    }
}
