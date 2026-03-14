//! Validation of the local `act` CI testing setup.
//!
//! Pure validation functions that take port traits (`FileSystem`, `ShellExecutor`,
//! `Environment`) so they can be fully tested with in-memory doubles.

use std::path::Path;

use ecc_ports::env::Environment;
use ecc_ports::fs::FileSystem;
use ecc_ports::shell::ShellExecutor;

// ── Types ──────────────────────────────────────────────────────────────

/// Status of a single validation check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

/// A single validation check result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    pub name: String,
    pub status: CheckStatus,
    pub detail: String,
}

/// Aggregated report of all validation checks.
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub checks: Vec<Check>,
}

// ── Expected job names per workflow ────────────────────────────────────

const CI_JOBS: &[&str] = &["validate", "security", "lint"];
const RELEASE_JOBS: &[&str] = &["validate", "build", "publish-npm", "github-release"];
const MAINTENANCE_JOBS: &[&str] = &["dependency-check", "security-audit", "stale"];

// ── Validation functions ──────────────────────────────────────────────

/// Validate that `.actrc` exists and contains required configuration lines.
pub fn validate_actrc(fs: &dyn FileSystem, project_dir: &Path) -> Check {
    let path = project_dir.join(".actrc");
    let content = match fs.read_to_string(&path) {
        Ok(c) => c,
        Err(_) => {
            return Check {
                name: "actrc-exists".to_string(),
                status: CheckStatus::Fail,
                detail: ".actrc not found in project root".to_string(),
            };
        }
    };

    if content.trim().is_empty() {
        return Check {
            name: "actrc-content".to_string(),
            status: CheckStatus::Fail,
            detail: ".actrc is empty".to_string(),
        };
    }

    let has_platform = content.contains("-P ");
    let has_secret_file = content.contains("--secret-file");

    if !has_platform || !has_secret_file {
        let mut missing = Vec::new();
        if !has_platform {
            missing.push("-P (platform mapping)");
        }
        if !has_secret_file {
            missing.push("--secret-file");
        }
        return Check {
            name: "actrc-content".to_string(),
            status: CheckStatus::Fail,
            detail: format!(".actrc missing required config: {}", missing.join(", ")),
        };
    }

    Check {
        name: "actrc".to_string(),
        status: CheckStatus::Pass,
        detail: ".actrc is valid".to_string(),
    }
}

/// Validate that `.secrets.example` exists and has documented variables.
pub fn validate_secrets_example(fs: &dyn FileSystem, project_dir: &Path) -> Check {
    let path = project_dir.join(".secrets.example");
    let content = match fs.read_to_string(&path) {
        Ok(c) => c,
        Err(_) => {
            return Check {
                name: "secrets-example".to_string(),
                status: CheckStatus::Fail,
                detail: ".secrets.example not found in project root".to_string(),
            };
        }
    };

    let has_vars = content.lines().any(|line| {
        let trimmed = line.trim();
        !trimmed.is_empty() && !trimmed.starts_with('#') && trimmed.contains('=')
    });

    if !has_vars {
        return Check {
            name: "secrets-example".to_string(),
            status: CheckStatus::Fail,
            detail: ".secrets.example has no documented variables".to_string(),
        };
    }

    Check {
        name: "secrets-example".to_string(),
        status: CheckStatus::Pass,
        detail: ".secrets.example is valid".to_string(),
    }
}

/// Check whether `act` is available on the system.
/// Returns `Warn` (not `Fail`) when missing — act is optional for development.
pub fn is_act_available(shell: &dyn ShellExecutor) -> Check {
    if shell.command_exists("act") {
        Check {
            name: "act-available".to_string(),
            status: CheckStatus::Pass,
            detail: "act is installed".to_string(),
        }
    } else {
        Check {
            name: "act-available".to_string(),
            status: CheckStatus::Warn,
            detail: "act is not installed — install via: brew install act".to_string(),
        }
    }
}

/// Validate that `act -l` output contains the expected workflow jobs.
///
/// Returns `Warn` when running on CI or when act is unavailable.
pub fn validate_act_jobs(shell: &dyn ShellExecutor, env: &dyn Environment) -> Check {
    if env.var("GITHUB_ACTIONS").as_deref() == Some("true") {
        return Check {
            name: "act-jobs".to_string(),
            status: CheckStatus::Warn,
            detail: "skipped on GitHub CI".to_string(),
        };
    }

    if !shell.command_exists("act") {
        return Check {
            name: "act-jobs".to_string(),
            status: CheckStatus::Warn,
            detail: "act not installed — cannot validate jobs".to_string(),
        };
    }

    let output = match shell.run_command("act", &["-l"]) {
        Ok(o) if o.success() => o.stdout,
        _ => {
            return Check {
                name: "act-jobs".to_string(),
                status: CheckStatus::Warn,
                detail: "act -l failed".to_string(),
            };
        }
    };

    let all_expected: Vec<&str> = CI_JOBS
        .iter()
        .chain(RELEASE_JOBS.iter())
        .chain(MAINTENANCE_JOBS.iter())
        .copied()
        .collect();

    let missing: Vec<&str> = all_expected
        .iter()
        .filter(|job| !output.contains(**job))
        .copied()
        .collect();

    if missing.is_empty() {
        Check {
            name: "act-jobs".to_string(),
            status: CheckStatus::Pass,
            detail: "all expected jobs found".to_string(),
        }
    } else {
        Check {
            name: "act-jobs".to_string(),
            status: CheckStatus::Fail,
            detail: format!("missing jobs: {}", missing.join(", ")),
        }
    }
}

/// Run all validation checks and return an aggregated report.
///
/// File-based checks always run. Shell-dependent checks are skipped (Warn) on CI.
pub fn validate_all(
    fs: &dyn FileSystem,
    shell: &dyn ShellExecutor,
    env: &dyn Environment,
    project_dir: &Path,
) -> ValidationReport {
    let checks = vec![
        validate_actrc(fs, project_dir),
        validate_secrets_example(fs, project_dir),
        is_act_available(shell),
        validate_act_jobs(shell, env),
    ];
    ValidationReport { checks }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_ports::shell::CommandOutput;
    use ecc_test_support::{InMemoryFileSystem, MockEnvironment, MockExecutor};

    // ── Phase 1: config validation ─────────────────────────────────

    #[test]
    fn validate_actrc_pass() {
        let fs = InMemoryFileSystem::new().with_file(
            "/project/.actrc",
            "-P ubuntu-latest=catthehacker/ubuntu:act-latest\n--secret-file .secrets\n",
        );
        let check = validate_actrc(&fs, Path::new("/project"));
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn validate_actrc_missing() {
        let fs = InMemoryFileSystem::new();
        let check = validate_actrc(&fs, Path::new("/project"));
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.detail.contains("not found"));
    }

    #[test]
    fn validate_actrc_empty() {
        let fs = InMemoryFileSystem::new().with_file("/project/.actrc", "  \n");
        let check = validate_actrc(&fs, Path::new("/project"));
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.detail.contains("empty"));
    }

    #[test]
    fn validate_actrc_missing_platform() {
        let fs =
            InMemoryFileSystem::new().with_file("/project/.actrc", "--secret-file .secrets\n");
        let check = validate_actrc(&fs, Path::new("/project"));
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.detail.contains("-P"));
    }

    #[test]
    fn validate_secrets_example_pass() {
        let fs = InMemoryFileSystem::new().with_file(
            "/project/.secrets.example",
            "# Secrets\nNPM_TOKEN=your_token_here\n",
        );
        let check = validate_secrets_example(&fs, Path::new("/project"));
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn validate_secrets_example_missing() {
        let fs = InMemoryFileSystem::new();
        let check = validate_secrets_example(&fs, Path::new("/project"));
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.detail.contains("not found"));
    }

    #[test]
    fn act_available_pass() {
        let shell = MockExecutor::new().with_command("act");
        let check = is_act_available(&shell);
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn act_not_available_warn() {
        let shell = MockExecutor::new();
        let check = is_act_available(&shell);
        assert_eq!(check.status, CheckStatus::Warn);
    }

    // ── Phase 2: job listing validation ────────────────────────────

    fn mock_act_list_output() -> String {
        // Simulates `act -l` output containing all expected jobs
        [
            "Stage  Job ID            Workflow         Workflow File  Events",
            "0      validate          CI               ci.yml         push",
            "0      security          CI               ci.yml         push",
            "0      lint              CI               ci.yml         push",
            "0      validate          Release          release.yml    release",
            "1      build             Release          release.yml    release",
            "2      publish-npm       Release          release.yml    release",
            "2      github-release    Release          release.yml    release",
            "0      dependency-check  Maintenance      maintenance.yml schedule",
            "0      security-audit    Maintenance      maintenance.yml schedule",
            "0      stale             Maintenance      maintenance.yml schedule",
        ]
        .join("\n")
    }

    #[test]
    fn validate_jobs_all_present() {
        let shell = MockExecutor::new().with_command("act").on(
            "act",
            CommandOutput {
                stdout: mock_act_list_output(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let env = MockEnvironment::new();
        let check = validate_act_jobs(&shell, &env);
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn validate_jobs_missing_some() {
        let partial = "Stage  Job ID    Workflow  Workflow File  Events\n\
                       0      validate  CI        ci.yml         push\n\
                       0      lint      CI        ci.yml         push\n";
        let shell = MockExecutor::new().with_command("act").on(
            "act",
            CommandOutput {
                stdout: partial.to_string(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let env = MockEnvironment::new();
        let check = validate_act_jobs(&shell, &env);
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.detail.contains("missing jobs"));
        assert!(check.detail.contains("security"));
    }

    #[test]
    fn validate_jobs_act_unavailable() {
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let check = validate_act_jobs(&shell, &env);
        assert_eq!(check.status, CheckStatus::Warn);
    }

    #[test]
    fn validate_jobs_on_ci_skipped() {
        let shell = MockExecutor::new().with_command("act");
        let env = MockEnvironment::new().with_var("GITHUB_ACTIONS", "true");
        let check = validate_act_jobs(&shell, &env);
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.detail.contains("skipped"));
    }

    // ── Phase 3: validate_all ──────────────────────────────────────

    #[test]
    fn validate_all_happy_path() {
        let fs = InMemoryFileSystem::new()
            .with_file(
                "/project/.actrc",
                "-P ubuntu-latest=catthehacker/ubuntu:act-latest\n--secret-file .secrets\n",
            )
            .with_file(
                "/project/.secrets.example",
                "# Secrets\nNPM_TOKEN=your_token_here\n",
            );
        let shell = MockExecutor::new().with_command("act").on(
            "act",
            CommandOutput {
                stdout: mock_act_list_output(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let env = MockEnvironment::new();

        let report = validate_all(&fs, &shell, &env, Path::new("/project"));
        assert!(report
            .checks
            .iter()
            .all(|c| c.status == CheckStatus::Pass));
    }

    #[test]
    fn validate_all_on_ci() {
        let fs = InMemoryFileSystem::new()
            .with_file(
                "/project/.actrc",
                "-P ubuntu-latest=catthehacker/ubuntu:act-latest\n--secret-file .secrets\n",
            )
            .with_file(
                "/project/.secrets.example",
                "# Secrets\nNPM_TOKEN=your_token_here\n",
            );
        let shell = MockExecutor::new().with_command("act");
        let env = MockEnvironment::new().with_var("GITHUB_ACTIONS", "true");

        let report = validate_all(&fs, &shell, &env, Path::new("/project"));

        // File checks still pass
        let actrc = report.checks.iter().find(|c| c.name == "actrc").unwrap();
        assert_eq!(actrc.status, CheckStatus::Pass);

        let secrets = report
            .checks
            .iter()
            .find(|c| c.name == "secrets-example")
            .unwrap();
        assert_eq!(secrets.status, CheckStatus::Pass);

        // Shell-dependent check is warned/skipped
        let jobs = report
            .checks
            .iter()
            .find(|c| c.name == "act-jobs")
            .unwrap();
        assert_eq!(jobs.status, CheckStatus::Warn);
    }
}
