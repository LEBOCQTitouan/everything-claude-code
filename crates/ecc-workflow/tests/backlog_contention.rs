/// Multi-process backlog add-entry contention tests.
///
/// These tests spawn two `ecc-workflow backlog add-entry` processes concurrently against
/// the same temp directory to verify that flock-based locking serializes writes
/// and prevents data corruption (duplicate IDs, lost entries).
///
/// All tests are `#[ignore]` — run with:
///   cargo test -p ecc-workflow --test backlog_contention -- --ignored
use std::path::{Path, PathBuf};
use std::process::Command;

fn binary_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // deps/
    path.pop(); // debug/
    path.push("ecc-workflow");
    path
}

/// Set up a temp directory with the minimal `docs/backlog/` structure.
fn init_backlog(project_dir: &Path) {
    let backlog_dir = project_dir.join("docs/backlog");
    std::fs::create_dir_all(&backlog_dir).expect("failed to create docs/backlog");

    // Minimal BACKLOG.md with the expected table header
    let backlog_md = backlog_dir.join("BACKLOG.md");
    std::fs::write(
        &backlog_md,
        "# Backlog\n\n| ID | Title | Body | Scope | Target | Status | Created |\n|---|---|---|---|---|---|---|\n",
    )
    .expect("failed to write BACKLOG.md");
}

/// PC-013: Two concurrent backlog add-entry calls produce unique IDs.
///
/// Spawns two processes simultaneously that each add a different entry.
/// After both complete, verifies that two distinct BL-NNN-*.md files exist
/// with different IDs.
#[test]
#[ignore]
fn two_concurrent_add_entries_have_unique_ids() {
    let bin = binary_path();
    let temp_dir = tempfile::tempdir().unwrap();
    let project_dir = temp_dir.path();

    init_backlog(project_dir);

    // Spawn both processes concurrently
    let mut child1 = Command::new(&bin)
        .args([
            "backlog",
            "add-entry",
            "Feature A",
            "--scope",
            "LOW",
            "--target",
            "/spec-dev",
        ])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .env_remove("ECC_WORKFLOW_BYPASS")
        .spawn()
        .expect("failed to spawn process 1");

    let mut child2 = Command::new(&bin)
        .args([
            "backlog",
            "add-entry",
            "Feature B",
            "--scope",
            "MEDIUM",
            "--target",
            "/spec-dev",
        ])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .env_remove("ECC_WORKFLOW_BYPASS")
        .spawn()
        .expect("failed to spawn process 2");

    let status1 = child1.wait().expect("child1 did not exit");
    let status2 = child2.wait().expect("child2 did not exit");

    assert!(
        status1.success(),
        "process 1 failed with exit code {:?}",
        status1.code()
    );
    assert!(
        status2.success(),
        "process 2 failed with exit code {:?}",
        status2.code()
    );

    // Collect BL-*.md files in docs/backlog/
    let backlog_dir = project_dir.join("docs/backlog");
    let entries: Vec<_> = std::fs::read_dir(&backlog_dir)
        .expect("failed to read docs/backlog")
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let name_str = name.to_string_lossy();
            name_str.starts_with("BL-") && name_str.ends_with(".md")
        })
        .collect();

    assert_eq!(
        entries.len(),
        2,
        "expected 2 BL-*.md files, found {}",
        entries.len()
    );

    // Extract IDs and verify they are unique
    let mut ids: Vec<u32> = entries
        .iter()
        .map(|e| {
            let name = e.file_name();
            let name_str = name.to_string_lossy();
            // BL-NNN-slug.md → extract NNN
            let after_bl = name_str.strip_prefix("BL-").expect("missing BL- prefix");
            let id_str: String = after_bl
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            id_str.parse::<u32>().expect("failed to parse ID number")
        })
        .collect();

    ids.sort_unstable();
    assert_eq!(ids[0], ids[1] - 1, "IDs must be consecutive: got {ids:?}");
    // Explicitly verify both are unique
    assert_ne!(ids[0], ids[1], "IDs must be different: both are {}", ids[0]);
}

/// PC-014: Two concurrent backlog add-entry calls — both entries appear in BACKLOG.md.
///
/// Spawns two processes simultaneously. After both complete, reads BACKLOG.md
/// and verifies it contains both "Feature A" and "Feature B".
#[test]
#[ignore]
fn two_concurrent_add_entries_both_appear_in_backlog_md() {
    let bin = binary_path();
    let temp_dir = tempfile::tempdir().unwrap();
    let project_dir = temp_dir.path();

    init_backlog(project_dir);

    // Spawn both processes concurrently
    let mut child1 = Command::new(&bin)
        .args([
            "backlog",
            "add-entry",
            "Feature A",
            "--scope",
            "LOW",
            "--target",
            "/spec-dev",
        ])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .env_remove("ECC_WORKFLOW_BYPASS")
        .spawn()
        .expect("failed to spawn process 1");

    let mut child2 = Command::new(&bin)
        .args([
            "backlog",
            "add-entry",
            "Feature B",
            "--scope",
            "MEDIUM",
            "--target",
            "/spec-dev",
        ])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .env_remove("ECC_WORKFLOW_BYPASS")
        .spawn()
        .expect("failed to spawn process 2");

    let status1 = child1.wait().expect("child1 did not exit");
    let status2 = child2.wait().expect("child2 did not exit");

    assert!(
        status1.success(),
        "process 1 failed with exit code {:?}",
        status1.code()
    );
    assert!(
        status2.success(),
        "process 2 failed with exit code {:?}",
        status2.code()
    );

    // Read BACKLOG.md and verify both entries are present
    let backlog_md = project_dir.join("docs/backlog/BACKLOG.md");
    let content = std::fs::read_to_string(&backlog_md).expect("failed to read BACKLOG.md");

    assert!(
        content.contains("Feature A"),
        "BACKLOG.md does not contain 'Feature A':\n{content}"
    );
    assert!(
        content.contains("Feature B"),
        "BACKLOG.md does not contain 'Feature B':\n{content}"
    );
}
