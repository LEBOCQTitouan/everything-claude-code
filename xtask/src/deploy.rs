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

pub fn run(dry_run: bool) -> anyhow::Result<()> {
    let mut results: Vec<ActionResult> = Vec::new();

    // Step 1: Build
    if dry_run {
        results.push(ActionResult {
            name: "Build".into(),
            status: ActionStatus::DryRun(
                "Would run: cargo build --release -p ecc-cli -p ecc-workflow".into(),
            ),
        });
    } else {
        let output = Command::new("cargo")
            .args(["build", "--release", "-p", "ecc-cli", "-p", "ecc-workflow"])
            .status()?;
        if !output.success() {
            anyhow::bail!("cargo build --release failed");
        }
        results.push(ActionResult {
            name: "Build".into(),
            status: ActionStatus::Installed("Built ecc + ecc-workflow (release)".into()),
        });
    }

    // Step 2: Install binaries
    let bin_dir = cargo_bin_dir();
    if dry_run {
        results.push(ActionResult {
            name: "Install".into(),
            status: ActionStatus::DryRun(format!(
                "Would copy ecc, ecc-workflow to {}",
                bin_dir.display()
            )),
        });
    } else {
        std::fs::create_dir_all(&bin_dir)?;
        for name in ["ecc", "ecc-workflow"] {
            let src = PathBuf::from("target/release").join(name);
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
        let status = Command::new("ecc").arg("install").status();
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
