//! CLI wiring for knowledge sources registry subcommands.

use clap::{Args, Subcommand};
use ecc_app::sources::CheckStatus;
use ecc_infra::os_fs::OsFileSystem;
use ecc_infra::process_executor::ProcessExecutor;
use std::path::PathBuf;

#[derive(Args)]
pub struct SourcesArgs {
    #[command(subcommand)]
    pub action: SourcesAction,

    /// Path to sources file
    #[arg(long, default_value = "docs/sources.md")]
    pub file: PathBuf,
}

#[derive(Subcommand)]
pub enum SourcesAction {
    /// List sources, optionally filtered by quadrant or subject
    List {
        #[arg(long)]
        quadrant: Option<String>,
        #[arg(long)]
        subject: Option<String>,
    },
    /// Add a new source entry
    Add {
        /// Source URL
        url: String,
        #[arg(long)]
        title: String,
        #[arg(long, name = "type")]
        source_type: String,
        #[arg(long)]
        quadrant: String,
        #[arg(long)]
        subject: String,
        #[arg(long, default_value = "human")]
        added_by: String,
    },
    /// Check all sources for reachability
    Check,
    /// Rebuild sources.md, process inbox entries
    Reindex {
        #[arg(long)]
        dry_run: bool,
    },
}

fn status_label(status: &CheckStatus) -> &str {
    match status {
        CheckStatus::Ok => "OK",
        CheckStatus::Stale => "STALE",
        CheckStatus::Unreachable => "UNREACHABLE",
        CheckStatus::WarnAge(_) => "WARN",
        CheckStatus::ErrorAge(_) => "ERROR",
    }
}

/// Return today's date as YYYY-MM-DD using the system clock.
fn today_date() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Days since epoch (1970-01-01)
    let days = secs / 86400;
    // Convert to calendar date using proleptic Gregorian
    let (year, month, day) = days_to_ymd(days);
    format!("{year:04}-{month:02}-{day:02}")
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    // Offset from 1970-01-01; step through 400-year cycles
    let days_400 = 146097u64;
    let days_100 = 36524u64;
    let days_4 = 1461u64;
    let days_1 = 365u64;

    let n400 = days / days_400;
    days %= days_400;
    let n100 = (days / days_100).min(3);
    days -= n100 * days_100;
    let n4 = days / days_4;
    days %= days_4;
    let n1 = (days / days_1).min(3);
    days -= n1 * days_1;

    let year = n400 * 400 + n100 * 100 + n4 * 4 + n1 + 1970;
    let leap = (n1 == 0 && (n4 != 0 || n100 == 0)) as u64;
    let month_days = [31u64, 28 + leap, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 0u64;
    let mut remaining = days;
    for (i, &md) in month_days.iter().enumerate() {
        if remaining < md {
            month = i as u64 + 1;
            break;
        }
        remaining -= md;
    }
    (year, month, remaining + 1)
}

pub fn run(args: SourcesArgs) -> anyhow::Result<()> {
    let fs = OsFileSystem;
    let shell = ProcessExecutor;
    let path = &args.file;

    match args.action {
        SourcesAction::List { quadrant, subject } => {
            let entries =
                ecc_app::sources::list(&fs, path, quadrant.as_deref(), subject.as_deref())
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
            for entry in &entries {
                println!(
                    "{} | {} | {} | {}",
                    entry.title,
                    entry.url.as_str(),
                    entry.source_type,
                    entry.quadrant
                );
            }
            if entries.is_empty() {
                println!("No sources found matching filters.");
            }
        }
        SourcesAction::Add {
            url,
            title,
            source_type,
            quadrant,
            subject,
            added_by,
        } => {
            let date = today_date();
            ecc_app::sources::add(
                &fs,
                path,
                &url,
                &title,
                &source_type,
                &quadrant,
                &subject,
                &added_by,
                &date,
            )
            .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("Added: {title}");
        }
        SourcesAction::Check => {
            let today = today_date();
            let results = ecc_app::sources::check(&fs, &shell, path, &today)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            for r in &results {
                println!("{}: {} — {}", status_label(&r.status), r.title, r.url);
            }
        }
        SourcesAction::Reindex { dry_run } => {
            let output = ecc_app::sources::reindex(&fs, path, dry_run)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            if let Some(content) = output {
                print!("{content}");
            } else {
                println!("Reindexed sources.md");
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // --- PC-031: list routes to app use case ---

    #[test]
    fn sources_list_routes_to_app_use_case() {
        // Create a temp file with a known entry so the app use case actually runs
        let tmp_path = std::env::temp_dir().join("ecc_sources_test.md");
        let content = "# Knowledge Sources\n\n## Inbox\n\n\n## Adopt\n\n### testing\n- [Test Source](https://example.com/test) — type: doc | subject: testing | added: 2026-01-01 | by: human\n\n## Trial\n\n## Assess\n\n## Hold\n\n## Module Mapping\n\n| Module | Subjects |\n|--------|----------|\n";
        fs::write(&tmp_path, content).expect("failed to write temp sources file");

        let args = SourcesArgs {
            action: SourcesAction::List {
                quadrant: None,
                subject: None,
            },
            file: tmp_path.clone(),
        };

        // If routing is broken, run() would return an error or panic
        let result = run(args);
        let _ = fs::remove_file(&tmp_path);
        assert!(result.is_ok(), "sources list should succeed: {result:?}");
    }
}
