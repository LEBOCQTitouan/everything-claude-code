use std::path::PathBuf;

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

pub fn run(dry_run: bool) -> anyhow::Result<()> {
    println!("deploy not yet implemented (dry_run={dry_run})");
    Ok(())
}

/// Detect cargo bin directory
fn cargo_bin_dir() -> PathBuf {
    todo!("not yet implemented")
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
