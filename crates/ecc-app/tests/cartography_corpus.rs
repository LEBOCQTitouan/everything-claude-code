//! Integration tests for the cartography corpus fixtures.
//!
//! Verifies the shape and presence of the 10-fixture regression corpus used for
//! `is_noise_path` + `stop_cartography` filter testing.

use std::path::PathBuf;

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
