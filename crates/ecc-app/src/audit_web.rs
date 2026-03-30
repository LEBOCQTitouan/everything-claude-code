//! Audit-web profile use cases — init, show, validate, reset, validate_report.
//!
//! Orchestrates domain logic through the FileSystem port.

use ecc_domain::audit_web::dimension::{self, AuditDimension};
use ecc_domain::audit_web::profile::{
    AuditWebProfile, DimensionThreshold, ImprovementSuggestion, ProfileError,
    parse_profile, serialize_profile,
};
use ecc_domain::audit_web::report_validation::validate_report;
use ecc_ports::fs::FileSystem;
use std::path::Path;

/// App-layer error type for audit-web operations.
#[derive(Debug, thiserror::Error)]
pub enum AuditWebAppError {
    #[error("profile error: {0}")]
    Profile(#[from] ProfileError),
    #[error("report error: {0}")]
    Report(String),
    #[error("I/O error: {0}")]
    Io(String),
    #[error("profile already exists at {0}. Use `reset` first.")]
    ProfileExists(String),
}

const PROFILE_PATH: &str = "docs/audits/audit-web-profile.yaml";

/// Initialize a new audit-web profile in the project directory.
///
/// Scans codebase characteristics and writes a YAML profile at
/// `docs/audits/audit-web-profile.yaml`. Fails if profile already exists.
pub fn init(fs: &dyn FileSystem, project_dir: &Path) -> Result<(), AuditWebAppError> {
    todo!()
}

/// Show the contents of the audit-web profile.
pub fn show(fs: &dyn FileSystem, profile_path: &Path) -> Result<String, AuditWebAppError> {
    todo!()
}

/// Validate the audit-web profile against the domain rules.
pub fn validate(fs: &dyn FileSystem, profile_path: &Path) -> Result<(), AuditWebAppError> {
    todo!()
}

/// Reset (delete) the audit-web profile.
pub fn reset(fs: &dyn FileSystem, profile_path: &Path) -> Result<(), AuditWebAppError> {
    todo!()
}

/// Validate a report file against required structure, score ranges, and citation counts.
pub fn validate_report_file(
    fs: &dyn FileSystem,
    report_path: &Path,
) -> Result<(), AuditWebAppError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::InMemoryFileSystem;

    fn valid_report_content() -> &'static str {
        r#"# Audit Web Report

## Techniques

**Strategic Fit**: 4/5
[source one](https://example.com/1)
[source two](https://example.com/2)
[source three](https://example.com/3)

## Tools

**Strategic Fit**: 3/5
[tool source 1](https://example.com/4)
[tool source 2](https://example.com/5)
[tool source 3](https://example.com/6)

## Platforms

**Strategic Fit**: 5/5
[platform ref 1](https://example.com/7)
[platform ref 2](https://example.com/8)
[platform ref 3](https://example.com/9)

## Languages & Frameworks

**Strategic Fit**: 4/5
[lang ref 1](https://example.com/10)
[lang ref 2](https://example.com/11)
[lang ref 3](https://example.com/12)

## Feature Opportunities

**Strategic Fit**: 2/5
[feature ref 1](https://example.com/13)
[feature ref 2](https://example.com/14)
[feature ref 3](https://example.com/15)
"#
    }

    // --- PC-013: init creates profile ---

    #[test]
    fn init_creates_profile() {
        let fs = InMemoryFileSystem::new();
        let project_dir = Path::new("/project");

        init(&fs, project_dir).expect("init should succeed");

        let profile_path = project_dir.join(PROFILE_PATH);
        assert!(
            fs.exists(&profile_path),
            "profile file should exist at {profile_path:?}"
        );
        let content = fs.read_to_string(&profile_path).unwrap();
        assert!(
            content.contains("version: 1"),
            "profile should contain version field"
        );
        assert!(
            content.contains("dimensions:"),
            "profile should contain dimensions"
        );
    }

    // --- PC-014: init rejects existing profile ---

    #[test]
    fn init_rejects_existing() {
        let project_dir = Path::new("/project");
        let profile_path = project_dir.join(PROFILE_PATH);
        let fs = InMemoryFileSystem::new().with_file(
            profile_path.to_str().unwrap(),
            "version: 1\n",
        );

        let result = init(&fs, project_dir);
        assert!(
            result.is_err(),
            "init should fail when profile already exists"
        );
        assert!(
            matches!(result.unwrap_err(), AuditWebAppError::ProfileExists(_)),
            "expected ProfileExists error"
        );
    }

    // --- PC-015: show reads profile content ---

    #[test]
    fn show_reads_profile() {
        let profile_content = "version: 1\ndimensions: []\n";
        let profile_path = Path::new("/project/docs/audits/audit-web-profile.yaml");
        let fs = InMemoryFileSystem::new().with_file(
            profile_path.to_str().unwrap(),
            profile_content,
        );

        let result = show(&fs, profile_path).expect("show should succeed");
        assert_eq!(result, profile_content);
    }

    // --- PC-016: validate passes for valid profile ---

    #[test]
    fn validate_valid_profile() {
        let project_dir = Path::new("/project");
        let fs = InMemoryFileSystem::new();

        // First create a valid profile via init
        init(&fs, project_dir).expect("init should succeed");
        let profile_path = project_dir.join(PROFILE_PATH);

        let result = validate(&fs, &profile_path);
        assert!(
            result.is_ok(),
            "validate should pass for valid profile: {result:?}"
        );
    }

    // --- PC-017: reset deletes profile file ---

    #[test]
    fn reset_deletes_profile() {
        let profile_path = Path::new("/project/docs/audits/audit-web-profile.yaml");
        let fs = InMemoryFileSystem::new().with_file(
            profile_path.to_str().unwrap(),
            "version: 1\n",
        );

        assert!(fs.exists(profile_path), "profile should exist before reset");
        reset(&fs, profile_path).expect("reset should succeed");
        assert!(
            !fs.exists(profile_path),
            "profile should not exist after reset"
        );
    }

    // --- PC-018: validate_report_file passes for valid report ---

    #[test]
    fn validate_report_passes() {
        let report_path = Path::new("/project/report.md");
        let fs = InMemoryFileSystem::new().with_file(
            report_path.to_str().unwrap(),
            valid_report_content(),
        );

        let result = validate_report_file(&fs, report_path);
        assert!(
            result.is_ok(),
            "validate_report_file should pass for valid report: {result:?}"
        );
    }
}
