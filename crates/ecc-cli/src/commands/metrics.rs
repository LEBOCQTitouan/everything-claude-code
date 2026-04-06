use clap::{Args, Parser, Subcommand};
use ecc_app::metrics_mgmt;
use ecc_ports::metrics_store::{MetricsExportFormat, MetricsQuery};

#[derive(Debug, Args)]
pub struct MetricsArgs {
    #[command(subcommand)]
    pub action: MetricsAction,
}

#[derive(Debug, Subcommand)]
pub enum MetricsAction {
    /// Display aggregated harness reliability rates
    Summary {
        #[arg(long, default_value = "7d")]
        since: String,
        #[arg(long)]
        session: Option<String>,
    },
    /// Export raw metric events
    Export {
        #[arg(long)]
        format: String,
        #[arg(long, default_value = "7d")]
        since: String,
    },
    /// Delete old metric events
    Prune {
        #[arg(long, default_value = "90d")]
        older_than: String,
    },
}

/// Top-level clap entry point used only for parsing in tests.
#[derive(Debug, Parser)]
#[command(name = "metrics")]
struct MetricsCli {
    #[command(subcommand)]
    action: MetricsAction,
}

pub fn run(args: MetricsArgs) -> anyhow::Result<()> {
    let db_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".ecc/metrics/metrics.db");

    let store = match ecc_infra::sqlite_metrics_store::SqliteMetricsStore::new(&db_path) {
        Ok(s) => s,
        Err(_) => {
            println!("No metrics data found. Run a session to start tracking harness metrics.");
            return Ok(());
        }
    };

    match args.action {
        MetricsAction::Summary { since, session } => {
            let query = MetricsQuery {
                since: parse_duration(&since),
                session_id: session,
                ..Default::default()
            };
            let metrics = metrics_mgmt::summary(&store, &query)
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            println!("Harness Metrics (last {since})");
            println!("  Hook success rate:           {}", format_rate(metrics.hook_success_rate));
            println!("  Phase-gate violation rate:    {}", format_rate(metrics.phase_gate_violation_rate));
            println!("  Agent failure recovery rate:  {}", format_rate(metrics.agent_failure_recovery_rate));
            println!("  Commit atomicity score:       {}", format_rate(metrics.commit_atomicity_score));
            println!("  Total events:                 {}", metrics.total_events);
        }
        MetricsAction::Export { format, since } => {
            let fmt = match format.as_str() {
                "json" => MetricsExportFormat::Json,
                "csv" => MetricsExportFormat::Csv,
                _ => return Err(anyhow::anyhow!("unsupported format: {format}. Use 'json' or 'csv'")),
            };
            let query = MetricsQuery {
                since: parse_duration(&since),
                ..Default::default()
            };
            let output = metrics_mgmt::export(&store, &query, fmt)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("{output}");
        }
        MetricsAction::Prune { older_than } => {
            let dur = parse_duration(&older_than)
                .ok_or_else(|| anyhow::anyhow!("invalid duration: {older_than}"))?;
            let removed = metrics_mgmt::prune(&store, dur)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("Pruned {removed} metric event(s)");
        }
    }

    Ok(())
}

fn format_rate(rate: Option<f64>) -> String {
    match rate {
        Some(r) => format!("{:.1}%", r * 100.0),
        None => "N/A (no events)".to_string(),
    }
}

fn parse_duration(s: &str) -> Option<std::time::Duration> {
    let s = s.trim();
    if s.ends_with('d') {
        let days: u64 = s.trim_end_matches('d').parse().ok()?;
        Some(std::time::Duration::from_secs(days * 86400))
    } else if s.ends_with('h') {
        let hours: u64 = s.trim_end_matches('h').parse().ok()?;
        Some(std::time::Duration::from_secs(hours * 3600))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // PC-030: Summary parses args
    #[test]
    fn metrics_cli_summary_args() {
        let cli = MetricsCli::try_parse_from(["metrics", "summary", "--since", "30d"]).unwrap();
        match cli.action {
            MetricsAction::Summary { since, session } => {
                assert_eq!(since, "30d");
                assert!(session.is_none());
            }
            _ => panic!("expected Summary"),
        }

        let cli = MetricsCli::try_parse_from(["metrics", "summary", "--session", "s1"]).unwrap();
        match cli.action {
            MetricsAction::Summary { session, .. } => {
                assert_eq!(session, Some("s1".into()));
            }
            _ => panic!("expected Summary"),
        }
    }

    // PC-031: Export parses args
    #[test]
    fn metrics_cli_export_args() {
        let cli =
            MetricsCli::try_parse_from(["metrics", "export", "--format", "json", "--since", "14d"])
                .unwrap();
        match cli.action {
            MetricsAction::Export { format, since } => {
                assert_eq!(format, "json");
                assert_eq!(since, "14d");
            }
            _ => panic!("expected Export"),
        }
    }

    // PC-032: Prune parses args
    #[test]
    fn metrics_cli_prune_args() {
        let cli =
            MetricsCli::try_parse_from(["metrics", "prune", "--older-than", "180d"]).unwrap();
        match cli.action {
            MetricsAction::Prune { older_than } => {
                assert_eq!(older_than, "180d");
            }
            _ => panic!("expected Prune"),
        }
    }
}
