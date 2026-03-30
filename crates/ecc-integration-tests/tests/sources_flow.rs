mod common;

use common::EccTestEnv;
use std::fs;

const SAMPLE_SOURCES: &str = "# Knowledge Sources\n\n\
## Inbox\n\n\
- [Inbox Entry](https://example.com/inbox) \u{2014} type: repo | quadrant: assess | subject: testing | added: 2026-03-29 | by: human\n\n\
## Adopt\n\n\
### testing\n\n\
- [Adopt Testing](https://example.com/adopt-testing) \u{2014} type: doc | subject: testing | added: 2026-03-01 | by: human\n\n\
## Trial\n\n\
## Assess\n\n\
## Hold\n\n\
## Module Mapping\n\n\
| Module | Subjects |\n\
|--------|----------|\n\
| crates/ecc-domain/ | domain-modeling, rust-patterns |\n";

// --- PC-016: list outputs entries ---

#[test]
fn list_outputs_entries() {
    let env = EccTestEnv::new();
    let sources_path = env.project.path().join("docs/sources.md");
    fs::create_dir_all(sources_path.parent().unwrap()).unwrap();
    fs::write(&sources_path, SAMPLE_SOURCES).unwrap();

    let output = env
        .cmd()
        .arg("sources")
        .arg("--file")
        .arg(&sources_path)
        .arg("list")
        .output()
        .expect("failed to run ecc sources list");
    assert!(output.status.success(), "ecc sources list failed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Adopt Testing"), "output must contain 'Adopt Testing', got: {stdout}");
}

// --- PC-017: add creates entry ---

#[test]
fn add_creates_entry() {
    let env = EccTestEnv::new();
    let sources_path = env.project.path().join("docs/sources.md");
    fs::create_dir_all(sources_path.parent().unwrap()).unwrap();
    fs::write(&sources_path, SAMPLE_SOURCES).unwrap();

    let output = env
        .cmd()
        .arg("sources")
        .arg("--file")
        .arg(&sources_path)
        .arg("add")
        .arg("https://example.com/new-entry")
        .arg("--title")
        .arg("New Entry")
        .arg("--source-type")
        .arg("doc")
        .arg("--quadrant")
        .arg("trial")
        .arg("--subject")
        .arg("testing")
        .output()
        .expect("failed to run ecc sources add");
    assert!(output.status.success(), "ecc sources add failed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Added: New Entry"), "output must contain 'Added: New Entry', got: {stdout}");

    let content = fs::read_to_string(&sources_path).unwrap();
    assert!(
        content.contains("https://example.com/new-entry"),
        "new entry must appear in file"
    );
    assert!(
        content.contains("New Entry"),
        "entry title must appear in file"
    );
}

// --- PC-018: reindex moves inbox ---

#[test]
fn reindex_moves_inbox() {
    let env = EccTestEnv::new();
    let sources_path = env.project.path().join("docs/sources.md");
    fs::create_dir_all(sources_path.parent().unwrap()).unwrap();
    fs::write(&sources_path, SAMPLE_SOURCES).unwrap();

    let output = env
        .cmd()
        .arg("sources")
        .arg("--file")
        .arg(&sources_path)
        .arg("reindex")
        .output()
        .expect("failed to run ecc sources reindex");
    assert!(output.status.success(), "ecc sources reindex failed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Reindexed"), "output must contain 'Reindexed', got: {stdout}");

    let content = fs::read_to_string(&sources_path).unwrap();
    // Inbox section should be empty after reindex
    let inbox_section = content
        .split("## Inbox")
        .nth(1)
        .and_then(|s| s.split("## ").next())
        .unwrap_or("");
    assert!(
        !inbox_section.contains("- ["),
        "inbox should be empty after reindex, but found entries: {inbox_section}"
    );
    // The inbox entry should now be in the Assess section
    assert!(
        content.contains("Inbox Entry"),
        "inbox entry must still exist in the file (moved to quadrant)"
    );
}

// --- PC-019: reindex --dry-run no write ---

#[test]
fn reindex_dry_run_no_write() {
    let env = EccTestEnv::new();
    let sources_path = env.project.path().join("docs/sources.md");
    fs::create_dir_all(sources_path.parent().unwrap()).unwrap();
    fs::write(&sources_path, SAMPLE_SOURCES).unwrap();

    let original = fs::read_to_string(&sources_path).unwrap();

    let output = env
        .cmd()
        .arg("sources")
        .arg("--file")
        .arg(&sources_path)
        .arg("reindex")
        .arg("--dry-run")
        .output()
        .expect("failed to run ecc sources reindex --dry-run");
    assert!(output.status.success(), "ecc sources reindex --dry-run failed: {}", String::from_utf8_lossy(&output.stderr));

    let after = fs::read_to_string(&sources_path).unwrap();
    assert_eq!(
        original, after,
        "file must be unchanged after --dry-run"
    );
}
