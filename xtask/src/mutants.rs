use std::process::Command;

use anyhow::{Context, Result, bail};

/// Build cargo-mutants command arguments.
fn build_args(
    packages: &[String],
    timeout: Option<u64>,
    in_diff: bool,
    nextest: bool,
) -> Vec<String> {
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

/// Run cargo-mutants with structured flags.
///
/// Wraps the external `cargo mutants` binary. Does NOT import any ECC
/// domain/port/infra crates — this is pure developer tooling.
pub fn run(packages: &[String], timeout: Option<u64>, in_diff: bool, nextest: bool) -> Result<()> {
    which::which("cargo-mutants")
        .context("cargo-mutants not installed. Run: cargo install cargo-mutants")?;

    let args = build_args(packages, timeout, in_diff, nextest);
    let mut cmd = Command::new("cargo");
    for arg in &args {
        cmd.arg(arg);
    }

    let status = cmd.status().context("failed to execute cargo mutants")?;

    if !status.success() {
        bail!("cargo mutants exited with status {}", status);
    }

    Ok(())
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
                "--package",
                "ecc-domain",
                "--package",
                "ecc-app",
                "--timeout",
                "120",
                "--test-tool",
                "nextest",
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
        // Use an empty PATH so cargo-mutants cannot be found
        let original_path = std::env::var("PATH").unwrap_or_default();
        // SAFETY: single-threaded test; PATH manipulation isolated to this test.
        unsafe { std::env::set_var("PATH", "/nonexistent") };
        let result = run(&["ecc-domain".to_string()], None, false, false);
        // SAFETY: restoring PATH to original value; same single-threaded invariant.
        unsafe { std::env::set_var("PATH", &original_path) };
        assert!(
            result.is_err(),
            "should error when cargo-mutants not in PATH"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not installed"),
            "error should mention installation: {err_msg}"
        );
    }

    #[test]
    fn empty_packages_produces_minimal_args() {
        let args = build_args(&[], None, false, false);
        assert_eq!(args, vec!["mutants"]);
    }
}
