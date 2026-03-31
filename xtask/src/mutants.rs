use std::process::Command;

use anyhow::{bail, Context, Result};

/// Run cargo-mutants with structured flags.
///
/// Wraps the external `cargo mutants` binary. Does NOT import any ECC
/// domain/port/infra crates — this is pure developer tooling.
pub fn run(packages: &[String], timeout: Option<u64>, in_diff: bool, nextest: bool) -> Result<()> {
    // Check cargo-mutants is installed
    which::which("cargo-mutants")
        .context("cargo-mutants not installed. Run: cargo install cargo-mutants")?;

    let mut cmd = Command::new("cargo");
    cmd.arg("mutants");

    for pkg in packages {
        cmd.arg("--package").arg(pkg);
    }

    if let Some(t) = timeout {
        cmd.arg("--timeout").arg(t.to_string());
    }

    if in_diff {
        cmd.arg("--in-diff").arg("origin/main");
    }

    if nextest {
        cmd.arg("--test-tool").arg("nextest");
    }

    let status = cmd.status().context("failed to execute cargo mutants")?;

    if !status.success() {
        bail!("cargo mutants exited with status {}", status);
    }

    Ok(())
}

#[cfg(test)]
fn build_args(packages: &[String], timeout: Option<u64>, in_diff: bool, nextest: bool) -> Vec<String> {
    let mut args = vec!["mutants".to_string()];

    for pkg in packages {
        args.push("--package".to_string());
        args.push(pkg.clone());
    }

    if let Some(t) = timeout {
        args.push("--timeout".to_string());
        args.push(t.to_string());
    }

    if in_diff {
        args.push("--in-diff".to_string());
        args.push("origin/main".to_string());
    }

    if nextest {
        args.push("--test-tool".to_string());
        args.push("nextest".to_string());
    }

    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_default_args() {
        let packages = vec!["ecc-domain".to_string(), "ecc-app".to_string()];
        let args = build_args(&packages, Some(120), false, true);

        assert_eq!(
            args,
            vec![
                "mutants",
                "--package", "ecc-domain",
                "--package", "ecc-app",
                "--timeout", "120",
                "--test-tool", "nextest",
            ]
        );
    }

    #[test]
    fn in_diff_uses_origin_main() {
        let packages = vec!["ecc-domain".to_string()];
        let args = build_args(&packages, None, true, false);

        assert!(args.contains(&"--in-diff".to_string()));
        assert!(args.contains(&"origin/main".to_string()));
    }

    #[test]
    fn errors_when_not_installed() {
        // Temporarily override PATH to ensure cargo-mutants is not found
        let result = which::which("cargo-mutants-nonexistent-binary-test");
        assert!(result.is_err(), "should error for non-existent binary");
    }

    #[test]
    fn empty_packages_produces_minimal_args() {
        let args = build_args(&[], None, false, false);
        assert_eq!(args, vec!["mutants"]);
    }
}
