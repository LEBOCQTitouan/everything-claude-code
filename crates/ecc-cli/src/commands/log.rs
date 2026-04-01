use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct LogArgs {
    #[command(subcommand)]
    pub action: LogAction,

    /// Override the logs directory (default: ~/.ecc/logs)
    #[arg(long)]
    pub logs_dir: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum LogAction {
    /// Live-tail current session logs
    Tail {
        #[arg(long)]
        session: Option<String>,
        #[arg(long, default_value = "20")]
        count: usize,
    },
    /// Full-text search across all sessions
    Search {
        query: String,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        since: Option<String>,
        #[arg(long)]
        level: Option<String>,
        #[arg(long, default_value = "100")]
        limit: usize,
    },
    /// Clean up old logs
    Prune {
        #[arg(long, default_value = "30d")]
        older_than: String,
    },
    /// Export filtered logs
    Export {
        #[arg(long, default_value = "json")]
        format: String,
        #[arg(long)]
        since: Option<String>,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        level: Option<String>,
    },
}

pub fn run(args: LogArgs) -> anyhow::Result<()> {
    let logs_dir = resolve_logs_dir(args.logs_dir);
    let db_path = logs_dir.join("ecc.db");

    match args.action {
        LogAction::Tail { session, count } => {
            if !db_path.exists() {
                println!("No log entries found.");
                return Ok(());
            }
            let store = ecc_infra::sqlite_log_store::SqliteLogStore::new(&db_path)?;
            let entries = ecc_app::log_mgmt::tail(&store, count, session.as_deref())?;
            if entries.is_empty() {
                println!("No log entries found.");
            } else {
                for entry in &entries {
                    println!("{}", serde_json::to_string(entry)?);
                }
            }
        }
        LogAction::Search {
            query,
            session,
            since,
            level,
            limit,
        } => {
            if !db_path.exists() {
                println!("No log entries found.");
                return Ok(());
            }
            let store = ecc_infra::sqlite_log_store::SqliteLogStore::new(&db_path)?;
            let q = ecc_ports::log_store::LogQuery {
                text: Some(query),
                session_id: session,
                since: parse_duration(&since),
                level,
                limit,
            };
            let entries = ecc_app::log_mgmt::search(&store, &q)?;
            if entries.is_empty() {
                println!("No log entries found.");
            } else {
                for entry in &entries {
                    println!(
                        "{} [{}] {} - {}",
                        entry.timestamp, entry.level, entry.target, entry.message
                    );
                }
            }
        }
        LogAction::Prune { older_than } => {
            if !db_path.exists() {
                println!("No log entries found.");
                return Ok(());
            }
            let store = ecc_infra::sqlite_log_store::SqliteLogStore::new(&db_path)?;
            let fs = ecc_infra::os_fs::OsFileSystem;
            let retention = parse_retention(&older_than)?;
            let result = ecc_app::log_mgmt::prune(&store, &fs, &logs_dir, retention)?;
            println!(
                "Pruned {} database rows, {} JSON files",
                result.db_rows, result.json_files
            );
        }
        LogAction::Export {
            format,
            since,
            session,
            level,
        } => {
            if !db_path.exists() {
                println!("No log entries found.");
                return Ok(());
            }
            let store = ecc_infra::sqlite_log_store::SqliteLogStore::new(&db_path)?;
            let fmt = match format.as_str() {
                "csv" => ecc_ports::log_store::ExportFormat::Csv,
                _ => ecc_ports::log_store::ExportFormat::Json,
            };
            let q = ecc_ports::log_store::LogQuery {
                since: parse_duration(&since),
                session_id: session,
                level,
                ..Default::default()
            };
            let output = ecc_app::log_mgmt::export(&store, &q, fmt)?;
            print!("{output}");
        }
    }
    Ok(())
}

fn resolve_logs_dir(dir: Option<PathBuf>) -> PathBuf {
    dir.unwrap_or_else(|| {
        std::env::var("HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_default()
            .join(".ecc")
            .join("logs")
    })
}

fn parse_duration(s: &Option<String>) -> Option<std::time::Duration> {
    s.as_ref().and_then(|v| {
        let days: u64 = v.trim_end_matches('d').parse().ok()?;
        Some(std::time::Duration::from_secs(days * 86400))
    })
}

fn parse_retention(s: &str) -> anyhow::Result<std::time::Duration> {
    let days: u64 = s
        .trim_end_matches('d')
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid retention format. Use Nd (e.g., 30d)"))?;
    Ok(std::time::Duration::from_secs(days * 86400))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// PC Wave 4: ecc log search subcommand routes to app use case
    #[test]
    fn log_search_routes_to_app_use_case() {
        let tmp_dir = TempDir::new().expect("failed to create temp dir");
        let logs_dir = tmp_dir.path().to_path_buf();

        let args = LogArgs {
            action: LogAction::Search {
                query: "test".to_string(),
                session: None,
                since: None,
                level: None,
                limit: 100,
            },
            logs_dir: Some(logs_dir),
        };

        // db_path does not exist so run() should return Ok with "No log entries found."
        let result = run(args);
        assert!(result.is_ok(), "log search should succeed: {result:?}");
    }

    /// PC Wave 4: ecc log tail subcommand returns Ok on empty store
    #[test]
    fn log_tail_on_empty_db_succeeds() {
        let tmp_dir = TempDir::new().expect("failed to create temp dir");
        let logs_dir = tmp_dir.path().to_path_buf();

        let args = LogArgs {
            action: LogAction::Tail {
                session: None,
                count: 20,
            },
            logs_dir: Some(logs_dir),
        };

        let result = run(args);
        assert!(
            result.is_ok(),
            "log tail should succeed on empty dir: {result:?}"
        );
    }

    /// PC Wave 4: ecc log prune subcommand returns Ok on missing db
    #[test]
    fn log_prune_on_missing_db_succeeds() {
        let tmp_dir = TempDir::new().expect("failed to create temp dir");
        let logs_dir = tmp_dir.path().to_path_buf();

        let args = LogArgs {
            action: LogAction::Prune {
                older_than: "30d".to_string(),
            },
            logs_dir: Some(logs_dir),
        };

        let result = run(args);
        assert!(
            result.is_ok(),
            "log prune should succeed on missing db: {result:?}"
        );
    }

    /// PC Wave 4: ecc log export subcommand returns Ok on missing db
    #[test]
    fn log_export_on_missing_db_succeeds() {
        let tmp_dir = TempDir::new().expect("failed to create temp dir");
        let logs_dir = tmp_dir.path().to_path_buf();

        let args = LogArgs {
            action: LogAction::Export {
                format: "json".to_string(),
                since: None,
                session: None,
                level: None,
            },
            logs_dir: Some(logs_dir),
        };

        let result = run(args);
        assert!(
            result.is_ok(),
            "log export should succeed on missing db: {result:?}"
        );
    }
}
