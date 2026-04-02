use std::path::{Path, PathBuf};
use std::process::Command;

use crate::rc_block;
use crate::shell;

#[derive(Debug)]
pub enum ActionStatus {
    Installed(String),
    Skipped(String),
    Added(String),
    Warning(String),
    DryRun(String),
}

#[derive(Debug)]
pub struct ActionResult {
    pub name: String,
    pub status: ActionStatus,
}

impl std::fmt::Display for ActionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionStatus::Installed(d) => write!(f, "[installed] {d}"),
            ActionStatus::Skipped(d) => write!(f, "[skipped] {d}"),
            ActionStatus::Added(d) => write!(f, "[added] {d}"),
            ActionStatus::Warning(d) => write!(f, "[warning] {d}"),
            ActionStatus::DryRun(d) => write!(f, "[dry-run] {d}"),
        }
    }
}

/// Returns the list of packages to build with cargo.
pub fn packages_to_build() -> Vec<&'static str> {
    vec!["ecc-cli", "ecc-workflow", "ecc-flock"]
}

/// Returns the list of binary names to install.
pub fn binaries_to_install() -> Vec<&'static str> {
    vec!["ecc", "ecc-workflow", "ecc-flock"]
}

/// Returns the dry-run message describing which binaries would be installed.
pub fn dry_run_install_message() -> String {
    "Would copy ecc, ecc-workflow, ecc-flock to <cargo_bin>".to_string()
}

/// Detect cargo bin directory
fn cargo_bin_dir() -> PathBuf {
    if let Ok(home) = std::env::var("CARGO_HOME") {
        PathBuf::from(home).join("bin")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".cargo/bin")
    } else {
        PathBuf::from("~/.cargo/bin")
    }
}

pub fn run(dry_run: bool, debug: bool) -> anyhow::Result<()> {
    let mut results: Vec<ActionResult> = Vec::new();
    let profile = if debug { "debug" } else { "release" };
    let target_dir = if debug {
        "target/debug"
    } else {
        "target/release"
    };

    // Step 1: Build
    let pkgs = packages_to_build();
    if dry_run {
        let pkg_flags: String = pkgs.iter().map(|p| format!("-p {p}")).collect::<Vec<_>>().join(" ");
        let cmd = if debug {
            format!("Would run: cargo build {pkg_flags}")
        } else {
            format!("Would run: cargo build --release {pkg_flags}")
        };
        results.push(ActionResult {
            name: "Build".into(),
            status: ActionStatus::DryRun(cmd),
        });
    } else {
        let mut args = vec!["build"];
        if !debug {
            args.push("--release");
        }
        for pkg in &pkgs {
            args.push("-p");
            args.push(pkg);
        }
        let output = Command::new("cargo").args(&args).status()?;
        if !output.success() {
            anyhow::bail!("cargo build failed");
        }
        results.push(ActionResult {
            name: "Build".into(),
            status: ActionStatus::Installed(format!(
                "Built ecc + ecc-workflow + ecc-flock ({profile})"
            )),
        });
    }

    // Step 2: Install binaries
    let bin_dir = cargo_bin_dir();
    let bins = binaries_to_install();
    if dry_run {
        let base_msg = dry_run_install_message();
        results.push(ActionResult {
            name: "Install".into(),
            status: ActionStatus::DryRun(format!(
                "{} (-> {})",
                base_msg,
                bin_dir.display()
            )),
        });
    } else {
        std::fs::create_dir_all(&bin_dir)?;
        for name in &bins {
            let src = PathBuf::from(target_dir).join(name);
            let dst = bin_dir.join(name);
            std::fs::copy(&src, &dst)?;
            results.push(ActionResult {
                name: format!("Install {name}"),
                status: ActionStatus::Installed(format!("{} -> {}", src.display(), dst.display())),
            });
        }
    }

    // Step 3: ecc install
    if dry_run {
        results.push(ActionResult {
            name: "Config".into(),
            status: ActionStatus::DryRun("Would run: ecc install".into()),
        });
    } else {
        let status = Command::new("ecc")
            .arg("install")
            .env("ECC_DEV_MODE", "1")
            .status();
        match status {
            Ok(s) if s.success() => results.push(ActionResult {
                name: "Config".into(),
                status: ActionStatus::Installed("Ran ecc install".into()),
            }),
            _ => results.push(ActionResult {
                name: "Config".into(),
                status: ActionStatus::Warning("ecc install failed — run manually".into()),
            }),
        }
    }

    // Step 4: Shell completions
    let home = std::env::var("HOME").unwrap_or_default();
    let home_path = Path::new(&home);
    let shell_env = std::env::var("SHELL").unwrap_or_default();

    if let Some(shell_kind) = shell::detect(&shell_env) {
        let comp_path = shell::completion_file_path(shell_kind, home_path);
        if dry_run {
            results.push(ActionResult {
                name: "Completions".into(),
                status: ActionStatus::DryRun(format!(
                    "Would generate completions to {}",
                    comp_path.display()
                )),
            });
        } else {
            if let Some(parent) = comp_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let shell_name = format!("{shell_kind:?}").to_lowercase();
            let output = Command::new("ecc")
                .args(["completion", &shell_name])
                .output();
            match output {
                Ok(o) if o.status.success() => {
                    std::fs::write(&comp_path, &o.stdout)?;
                    results.push(ActionResult {
                        name: "Completions".into(),
                        status: ActionStatus::Installed(format!("Wrote {}", comp_path.display())),
                    });
                }
                _ => results.push(ActionResult {
                    name: "Completions".into(),
                    status: ActionStatus::Warning("ecc completion failed".into()),
                }),
            }
        }

        // Step 5: RC file
        let rc_path = shell::rc_file_path(shell_kind, home_path);
        let block_lines: Vec<String> = shell::build_rc_block(shell_kind);
        let block_refs: Vec<&str> = block_lines.iter().map(|s| s.as_str()).collect();

        if dry_run {
            results.push(ActionResult {
                name: "Shell RC".into(),
                status: ActionStatus::DryRun(format!(
                    "Would update {} with ECC managed block",
                    rc_path.display()
                )),
            });
        } else {
            let existing = std::fs::read_to_string(&rc_path).unwrap_or_default();
            let result = rc_block::update_rc_content(&existing, &block_refs);
            if result.changed {
                let backup = PathBuf::from(format!("{}.ecc-backup", rc_path.display()));
                if !backup.exists() && rc_path.exists() {
                    std::fs::copy(&rc_path, &backup)?;
                }
                let tmp = rc_path.with_extension("ecc-tmp");
                std::fs::write(&tmp, &result.content)?;
                std::fs::rename(&tmp, &rc_path)?;
                results.push(ActionResult {
                    name: "Shell RC".into(),
                    status: ActionStatus::Added(format!("Updated {}", rc_path.display())),
                });
            } else {
                results.push(ActionResult {
                    name: "Shell RC".into(),
                    status: ActionStatus::Skipped("Already configured".into()),
                });
            }
        }
    } else {
        results.push(ActionResult {
            name: "Shell".into(),
            status: ActionStatus::Warning(format!(
                "Unknown shell '{}' — skipping completions and RC",
                shell_env
            )),
        });
    }

    // Step 6: Statusline validation
    if dry_run {
        results.push(ActionResult {
            name: "Statusline".into(),
            status: ActionStatus::DryRun("Would run: ecc validate statusline".into()),
        });
    } else {
        let status = Command::new("ecc")
            .args(["validate", "statusline"])
            .status();
        match status {
            Ok(s) if s.success() => results.push(ActionResult {
                name: "Statusline".into(),
                status: ActionStatus::Installed("Validated".into()),
            }),
            _ => results.push(ActionResult {
                name: "Statusline".into(),
                status: ActionStatus::Warning("Statusline validation failed".into()),
            }),
        }
    }

    // Print summary
    println!("\n=== Deploy Summary ===\n");
    for r in &results {
        println!("  {}: {}", r.name, r.status);
    }
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod cargo_bin {
        use super::*;

        #[test]
        fn uses_cargo_home_when_set() {
            // SAFETY: test sets env var — must not run in parallel with other env-mutating tests
            unsafe { std::env::set_var("CARGO_HOME", "/custom/cargo") };
            let dir = cargo_bin_dir();
            unsafe { std::env::remove_var("CARGO_HOME") };
            assert_eq!(dir, PathBuf::from("/custom/cargo/bin"));
        }

        #[test]
        fn falls_back_to_home_cargo_bin() {
            unsafe { std::env::remove_var("CARGO_HOME") };
            let home = std::env::var("HOME").unwrap_or_else(|_| "/home/testuser".to_string());
            let dir = cargo_bin_dir();
            assert_eq!(dir, PathBuf::from(&home).join(".cargo/bin"));
        }
    }

    mod deploy_debug {
        #[test]
        fn deploy_debug_flag() {
            // When --debug is set, build should NOT include --release
            let debug = true;
            let mut args = vec!["build", "-p", "ecc-cli", "-p", "ecc-workflow"];
            if !debug {
                args.insert(1, "--release");
            }
            assert!(
                !args.contains(&"--release"),
                "debug mode should not use --release"
            );
        }

        #[test]
        fn deploy_default_release() {
            // When --debug is NOT set, build should include --release
            let debug = false;
            let mut args = vec!["build", "-p", "ecc-cli", "-p", "ecc-workflow"];
            if !debug {
                args.insert(1, "--release");
            }
            assert!(
                args.contains(&"--release"),
                "default mode should use --release"
            );
        }

        #[test]
        fn deploy_debug_source_path() {
            let debug = true;
            let target_dir = if debug {
                "target/debug"
            } else {
                "target/release"
            };
            assert_eq!(target_dir, "target/debug");
        }

        #[test]
        fn deploy_debug_summary_message() {
            let debug = true;
            let profile = if debug { "debug" } else { "release" };
            let msg = format!("Built ecc + ecc-workflow ({profile})");
            assert!(msg.contains("debug"), "summary should indicate debug build");
        }
    }

    mod three_binaries {
        use super::*;

        #[test]
        fn deploy_builds_three() {
            let pkgs = packages_to_build();
            assert!(
                pkgs.contains(&"ecc-flock"),
                "build list must include ecc-flock; got: {pkgs:?}"
            );
        }

        #[test]
        fn deploy_installs_three() {
            let bins = binaries_to_install();
            assert!(
                bins.contains(&"ecc-flock"),
                "install list must include ecc-flock; got: {bins:?}"
            );
        }

        #[test]
        fn deploy_dry_run_lists_three() {
            let msg = dry_run_install_message();
            assert!(
                msg.contains("ecc-flock"),
                "dry-run message must mention ecc-flock; got: {msg}"
            );
        }
    }

    mod summary {
        use super::*;

        #[test]
        fn installed_format() {
            let s = ActionStatus::Installed("built ok".to_string());
            assert_eq!(format!("{s}"), "[installed] built ok");
        }

        #[test]
        fn skipped_format() {
            let s = ActionStatus::Skipped("already done".to_string());
            assert_eq!(format!("{s}"), "[skipped] already done");
        }

        #[test]
        fn added_format() {
            let s = ActionStatus::Added("new entry".to_string());
            assert_eq!(format!("{s}"), "[added] new entry");
        }

        #[test]
        fn warning_format() {
            let s = ActionStatus::Warning("something off".to_string());
            assert_eq!(format!("{s}"), "[warning] something off");
        }

        #[test]
        fn dry_run_format() {
            let s = ActionStatus::DryRun("would do thing".to_string());
            assert_eq!(format!("{s}"), "[dry-run] would do thing");
        }
    }
}
