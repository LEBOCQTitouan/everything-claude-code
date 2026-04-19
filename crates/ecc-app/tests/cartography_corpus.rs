//! Integration tests for the cartography corpus fixtures.
//!
//! Verifies the shape and presence of the 10-fixture regression corpus used for
//! `is_noise_path` + `stop_cartography` filter testing.

use ecc_domain::cartography::{SessionDelta, is_noise_path};
use serde::Deserialize;
use std::path::PathBuf;

/// Expected outcomes for a single fixture file.
#[derive(Debug, Deserialize)]
struct FixtureExpectation {
    file: String,
    expected_write: bool,
    expected_kept_paths: Vec<String>,
}

/// Top-level structure of expected.yaml.
#[derive(Debug, Deserialize)]
struct ExpectedFile {
    fixtures: Vec<FixtureExpectation>,
}

fn fixtures_dir() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.push("tests/fixtures/cartography-corpus");
    dir
}

/// Verifies that all 10 fixture JSON files, expected.yaml, and README.md exist.
#[test]
fn fixtures_shape_present() {
    let dir = fixtures_dir();

    let fixture_files = [
        "01-all-workflow.json",
        "02-all-specs.json",
        "03-all-backlog.json",
        "04-all-cartography.json",
        "05-cargo-lock-only.json",
        "06-workflow-plus-crate.json",
        "07-spec-plus-crate.json",
        "08-backlog-plus-doc.json",
        "09-pure-crate.json",
        "10-pure-app.json",
    ];

    for file in &fixture_files {
        let path = dir.join(file);
        assert!(
            path.exists(),
            "fixture file missing: {}",
            path.display()
        );
    }

    let expected_yaml = dir.join("expected.yaml");
    assert!(expected_yaml.exists(), "expected.yaml missing");

    // Verify expected.yaml parses as valid YAML with 'fixtures' key
    let content = std::fs::read_to_string(&expected_yaml)
        .expect("failed to read expected.yaml");
    assert!(
        content.contains("fixtures:"),
        "expected.yaml should contain 'fixtures:' key"
    );
    assert_eq!(
        content.matches("- file:").count(),
        10,
        "expected.yaml should have 10 fixture entries"
    );

    let readme = dir.join("README.md");
    assert!(readme.exists(), "README.md missing");
    let readme_content = std::fs::read_to_string(&readme)
        .expect("failed to read README.md");
    assert!(!readme_content.is_empty(), "README.md should be non-empty");
}

/// Corpus runner: parses each fixture JSON, applies `is_noise_path`, and
/// asserts that the outcomes match expected.yaml (AC-008.2).
#[test]
fn outcomes_match_expected() {
    let corpus_dir = fixtures_dir();
    let expected_yaml = std::fs::read_to_string(corpus_dir.join("expected.yaml"))
        .expect("read expected.yaml");
    let expected: ExpectedFile =
        serde_saphyr::from_str(&expected_yaml).expect("parse expected.yaml");

    for fix in expected.fixtures {
        let fixture_path = corpus_dir.join(&fix.file);
        let json = std::fs::read_to_string(&fixture_path)
            .unwrap_or_else(|e| panic!("read {}: {e}", fix.file));
        let delta: SessionDelta = serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("parse {}: {e}", fix.file));

        let kept: Vec<&str> = delta
            .changed_files
            .iter()
            .map(|f| f.path.as_str())
            .filter(|p| !is_noise_path(p))
            .collect();

        let write_expected = !kept.is_empty();
        assert_eq!(
            write_expected,
            fix.expected_write,
            "{}: expected_write mismatch (kept={kept:?})",
            fix.file
        );

        let kept_paths: Vec<String> = kept.into_iter().map(String::from).collect();
        assert_eq!(
            kept_paths,
            fix.expected_kept_paths,
            "{}: kept paths mismatch",
            fix.file
        );
    }
}
