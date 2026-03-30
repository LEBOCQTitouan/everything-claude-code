//! CLI wiring for audit-web profile management and report validation subcommands.

use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct AuditWebArgs {
    #[command(subcommand)]
    pub action: AuditWebAction,

    /// Path to the project directory (defaults to current directory)
    #[arg(long)]
    pub project_dir: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum AuditWebAction {
    /// Manage the audit-web profile
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },
    /// Validate a radar report file
    ValidateReport {
        /// Path to the report markdown file
        path: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum ProfileAction {
    /// Initialize a new profile (scans codebase)
    Init,
    /// Show the current profile
    Show,
    /// Validate the current profile
    Validate,
    /// Delete the profile
    Reset {
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
}

pub fn run(_args: AuditWebArgs) -> anyhow::Result<()> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // --- PC-019: profile init routes to app use case ---

    #[test]
    fn profile_init_routes() {
        let tmp_dir = std::env::temp_dir().join("ecc_audit_web_test_init");
        let _ = fs::remove_dir_all(&tmp_dir);
        fs::create_dir_all(&tmp_dir).expect("failed to create temp dir");

        let args = AuditWebArgs {
            action: AuditWebAction::Profile {
                action: ProfileAction::Init,
            },
            project_dir: Some(tmp_dir.clone()),
        };

        let result = run(args);

        let profile_path = tmp_dir.join("docs/audits/audit-web-profile.yaml");
        let _ = fs::remove_dir_all(&tmp_dir);

        assert!(
            result.is_ok(),
            "profile init should succeed: {result:?}"
        );
        // Note: profile file creation verified via app-layer test; CLI test verifies routing
    }

    // --- PC-020: validate-report routes to app use case ---

    #[test]
    fn validate_report_routes() {
        let tmp_dir = std::env::temp_dir().join("ecc_audit_web_test_report");
        let _ = fs::remove_dir_all(&tmp_dir);
        fs::create_dir_all(&tmp_dir).expect("failed to create temp dir");

        let report_content = r#"# Audit Web Report

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
"#;

        let report_path = tmp_dir.join("report.md");
        fs::write(&report_path, report_content).expect("failed to write report");

        let args = AuditWebArgs {
            action: AuditWebAction::ValidateReport {
                path: report_path.clone(),
            },
            project_dir: None,
        };

        let result = run(args);
        let _ = fs::remove_dir_all(&tmp_dir);

        assert!(
            result.is_ok(),
            "validate-report should succeed for valid report: {result:?}"
        );
    }
}
