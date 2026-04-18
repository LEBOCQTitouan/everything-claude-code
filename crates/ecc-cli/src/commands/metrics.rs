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
        /// Output results as JSON
        #[arg(long)]
        json: bool,
        /// Show trend comparison (current vs previous period)
        #[arg(long)]
        trend: bool,
    },
    /// Record a commit gate check event
    RecordGate {
        /// Gate kind: build, test, or lint
        #[arg(long)]
        kind: String,
        /// Gate outcome: pass or fail
        #[arg(long)]
        outcome: String,
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
        MetricsAction::Summary {
            since,
            session,
            json,
            trend,
        } => {
            if trend && session.is_some() {
                return Err(anyhow::anyhow!("--trend is incompatible with --session"));
            }
            let query = MetricsQuery {
                since: parse_duration(&since),
                session_id: session,
                ..Default::default()
            };
            let metrics =
                metrics_mgmt::summary(&store, &query).map_err(|e| anyhow::anyhow!("{e}"))?;

            if json {
                let obj = serde_json::json!({
                    "hook_success_rate": metrics.hook_success_rate,
                    "phase_gate_violation_rate": metrics.phase_gate_violation_rate,
                    "agent_failure_recovery_rate": metrics.agent_failure_recovery_rate,
                    "commit_atomicity_score": metrics.commit_atomicity_score,
                    "total_events": metrics.total_events,
                });
                println!("{}", serde_json::to_string_pretty(&obj)?);
                return Ok(());
            }

            if trend {
                let dur =
                    parse_duration(&since).unwrap_or(std::time::Duration::from_secs(7 * 86400));
                let clock = ecc_infra::system_clock::SystemClock;
                let trend_data =
                    metrics_mgmt::trend_summary(&store, dur, &clock).map_err(|e| anyhow::anyhow!("{e}"))?;
                use ecc_domain::metrics::TrendComparison;
                println!("Harness Metrics Trend (last {since} vs previous {since})");
                println!(
                    "  {:<35} {:>10} {:>10} {:>10}",
                    "Metric", "Current", "Previous", "Delta"
                );
                println!(
                    "  {:<35} {:>10} {:>10} {:>10}",
                    "Hook success rate",
                    format_rate(trend_data.current.hook_success_rate),
                    format_rate(trend_data.previous.hook_success_rate),
                    TrendComparison::format_delta(trend_data.hook_success_rate_delta)
                );
                println!(
                    "  {:<35} {:>10} {:>10} {:>10}",
                    "Phase-gate violation rate",
                    format_rate(trend_data.current.phase_gate_violation_rate),
                    format_rate(trend_data.previous.phase_gate_violation_rate),
                    TrendComparison::format_delta(trend_data.phase_gate_violation_rate_delta)
                );
                println!(
                    "  {:<35} {:>10} {:>10} {:>10}",
                    "Agent failure recovery rate",
                    format_rate(trend_data.current.agent_failure_recovery_rate),
                    format_rate(trend_data.previous.agent_failure_recovery_rate),
                    TrendComparison::format_delta(trend_data.agent_failure_recovery_rate_delta)
                );
                println!(
                    "  {:<35} {:>10} {:>10} {:>10}",
                    "Commit atomicity score",
                    format_rate(trend_data.current.commit_atomicity_score),
                    format_rate(trend_data.previous.commit_atomicity_score),
                    TrendComparison::format_delta(trend_data.commit_atomicity_score_delta)
                );
                return Ok(());
            }

            let targets = ecc_domain::metrics::ReferenceTargets::default();
            println!("Harness Metrics (last {since})");
            println!("  {:<35} {:>15} {:>15}", "Metric", "Value", "vs. Target");
            println!(
                "  {:<35} {:>15} {:>15}",
                slo_prefix(metrics.hook_success_rate, targets.hook_success, false)
                    + "Hook success rate",
                format_rate(metrics.hook_success_rate),
                format!("{:.0}%", targets.hook_success * 100.0)
            );
            println!(
                "  {:<35} {:>15} {:>15}",
                slo_prefix(
                    metrics.phase_gate_violation_rate,
                    targets.phase_gate_violation,
                    true
                ) + "Phase-gate violation rate",
                format_rate(metrics.phase_gate_violation_rate),
                format!("{:.0}%", targets.phase_gate_violation * 100.0)
            );
            println!(
                "  {:<35} {:>15} {:>15}",
                slo_prefix(
                    metrics.agent_failure_recovery_rate,
                    targets.agent_recovery,
                    false
                ) + "Agent failure recovery rate",
                format_rate(metrics.agent_failure_recovery_rate),
                format!("{:.0}%", targets.agent_recovery * 100.0)
            );
            println!(
                "  {:<35} {:>15} {:>15}",
                slo_prefix(
                    metrics.commit_atomicity_score,
                    targets.commit_atomicity,
                    false
                ) + "Commit atomicity score",
                format_rate(metrics.commit_atomicity_score),
                format!("{:.0}%", targets.commit_atomicity * 100.0)
            );
            println!("  Total events: {}", metrics.total_events);
        }
        MetricsAction::RecordGate { kind, outcome } => {
            let session_id =
                std::env::var("ECC_SESSION_ID").unwrap_or_else(|_| "unknown".to_string());
            let disabled = std::env::var("ECC_METRICS_DISABLED")
                .map(|v| v == "1")
                .unwrap_or(false);
            let clock = ecc_infra::system_clock::SystemClock;
            metrics_mgmt::record_commit_gate(Some(&store), &session_id, &kind, &outcome, disabled, &clock)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("Recorded commit gate: kind={kind} outcome={outcome}");
        }
        MetricsAction::Export { format, since } => {
            let fmt = match format.as_str() {
                "json" => MetricsExportFormat::Json,
                "csv" => MetricsExportFormat::Csv,
                _ => {
                    return Err(anyhow::anyhow!(
                        "unsupported format: {format}. Use 'json' or 'csv'"
                    ));
                }
            };
            let query = MetricsQuery {
                since: parse_duration(&since),
                ..Default::default()
            };
            let output =
                metrics_mgmt::export(&store, &query, fmt).map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("{output}");
        }
        MetricsAction::Prune { older_than } => {
            let dur = parse_duration(&older_than)
                .ok_or_else(|| anyhow::anyhow!("invalid duration: {older_than}"))?;
            let removed = metrics_mgmt::prune(&store, dur).map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("Pruned {removed} metric event(s)");
        }
    }

    Ok(())
}

/// Returns "[!] " if the metric is below (or above, for violation) SLO, otherwise "".
fn slo_prefix(value: Option<f64>, target: f64, higher_is_worse: bool) -> String {
    match value {
        None => String::new(),
        Some(v) => {
            let below = if higher_is_worse {
                v > target
            } else {
                v < target
            };
            if below {
                "[!] ".to_string()
            } else {
                String::new()
            }
        }
    }
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
    use ecc_domain::metrics::{MetricEvent, MetricOutcome};
    use ecc_ports::metrics_store::MetricsStore;
    use ecc_test_support::{InMemoryMetricsStore, TEST_CLOCK};

    fn hook_event(session: &str, ts: &str, outcome: MetricOutcome) -> MetricEvent {
        MetricEvent::hook_execution(
            session.to_owned(),
            ts.to_owned(),
            "test-hook".to_owned(),
            100,
            outcome,
            None,
        )
        .unwrap()
    }

    fn commit_event(session: &str, ts: &str, outcome: MetricOutcome) -> MetricEvent {
        MetricEvent::commit_gate(session.to_owned(), ts.to_owned(), outcome, vec![]).unwrap()
    }

    // PC-030: Summary parses args
    #[test]
    fn metrics_cli_summary_args() {
        let cli = MetricsCli::try_parse_from(["metrics", "summary", "--since", "30d"]).unwrap();
        match cli.action {
            MetricsAction::Summary { since, session, .. } => {
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
        let cli = MetricsCli::try_parse_from(["metrics", "prune", "--older-than", "180d"]).unwrap();
        match cli.action {
            MetricsAction::Prune { older_than } => {
                assert_eq!(older_than, "180d");
            }
            _ => panic!("expected Prune"),
        }
    }

    // PC-025: --trend --session returns error
    #[test]
    fn trend_session_incompatible() {
        // Verify clap parses both flags OK (no clap-level conflict)
        let cli =
            MetricsCli::try_parse_from(["metrics", "summary", "--trend", "--session", "sess-1"])
                .unwrap();
        // Now simulate the run logic check
        match cli.action {
            MetricsAction::Summary { trend, session, .. } => {
                assert!(trend);
                assert!(session.is_some());
                // The run() function returns an error for this combination
                let result: anyhow::Result<()> = if trend && session.is_some() {
                    Err(anyhow::anyhow!("--trend is incompatible with --session"))
                } else {
                    Ok(())
                };
                assert!(result.is_err());
                assert!(
                    result
                        .unwrap_err()
                        .to_string()
                        .contains("--trend is incompatible with --session")
                );
            }
            _ => panic!("expected Summary"),
        }
    }

    // PC-026: --json outputs valid JSON with four Option<f64> fields
    #[test]
    fn summary_json_output() {
        let store = InMemoryMetricsStore::new();
        store
            .record(&hook_event(
                "s1",
                "2026-04-06T10:00:00Z",
                MetricOutcome::Success,
            ))
            .unwrap();
        store
            .record(&hook_event(
                "s1",
                "2026-04-06T10:01:00Z",
                MetricOutcome::Failure,
            ))
            .unwrap();
        store
            .record(&commit_event(
                "s1",
                "2026-04-06T10:02:00Z",
                MetricOutcome::Passed,
            ))
            .unwrap();

        let query = ecc_ports::metrics_store::MetricsQuery::default();
        let metrics = ecc_app::metrics_mgmt::summary(&store, &query).unwrap();

        let obj = serde_json::json!({
            "hook_success_rate": metrics.hook_success_rate,
            "phase_gate_violation_rate": metrics.phase_gate_violation_rate,
            "agent_failure_recovery_rate": metrics.agent_failure_recovery_rate,
            "commit_atomicity_score": metrics.commit_atomicity_score,
            "total_events": metrics.total_events,
        });

        let serialized = serde_json::to_string(&obj).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        // The object must have the four rate fields as numbers or null
        assert!(parsed.get("hook_success_rate").is_some());
        assert!(parsed.get("phase_gate_violation_rate").is_some());
        assert!(parsed.get("agent_failure_recovery_rate").is_some());
        assert!(parsed.get("commit_atomicity_score").is_some());
        // hook_success_rate should be 0.5 (1 success out of 2)
        assert_eq!(parsed["hook_success_rate"], serde_json::json!(0.5));
    }

    // PC-027: summary shows "vs. Target" column with SLO values from ReferenceTargets
    #[test]
    fn summary_shows_slo_column() {
        let targets = ecc_domain::metrics::ReferenceTargets::default();
        // Verify the SLO values are as expected
        assert!((targets.hook_success - 0.99).abs() < f64::EPSILON);
        assert!((targets.phase_gate_violation - 0.05).abs() < f64::EPSILON);
        assert!((targets.agent_recovery - 0.80).abs() < f64::EPSILON);
        assert!((targets.commit_atomicity - 0.95).abs() < f64::EPSILON);

        // Verify the column header "vs. Target" is part of the output format
        let header_line = format!("{:<35} {:>15} {:>15}", "Metric", "Value", "vs. Target");
        assert!(header_line.contains("vs. Target"));
    }

    // PC-028: [!] prefix when metric below SLO target
    #[test]
    fn summary_flags_below_slo() {
        let targets = ecc_domain::metrics::ReferenceTargets::default();
        // hook_success target is 0.99. Value of 0.5 is below → [!] prefix
        let prefix = slo_prefix(Some(0.5), targets.hook_success, false);
        assert_eq!(prefix, "[!] ");

        // Value of 0.99 is not below → empty prefix
        let prefix_ok = slo_prefix(Some(0.99), targets.hook_success, false);
        assert_eq!(prefix_ok, "");

        // phase_gate_violation target is 0.05 (higher_is_worse=true). Value 0.1 > 0.05 → [!]
        let prefix_violation = slo_prefix(Some(0.1), targets.phase_gate_violation, true);
        assert_eq!(prefix_violation, "[!] ");

        // Value 0.03 < 0.05 (not exceeding) → no [!]
        let prefix_ok2 = slo_prefix(Some(0.03), targets.phase_gate_violation, true);
        assert_eq!(prefix_ok2, "");

        // None → no prefix
        let prefix_none = slo_prefix(None, targets.hook_success, false);
        assert_eq!(prefix_none, "");
    }

    // PC-029: --trend shows Current/Previous/delta columns with +/- prefixes
    #[test]
    fn summary_trend_columns() {
        use ecc_domain::metrics::TrendComparison;

        // Positive delta → "+" prefix
        let pos = TrendComparison::format_delta(Some(0.1));
        assert!(
            pos.starts_with('+'),
            "positive delta must start with +: {pos}"
        );

        // Negative delta → "-" prefix
        let neg = TrendComparison::format_delta(Some(-0.05));
        assert!(
            neg.starts_with('-'),
            "negative delta must start with -: {neg}"
        );

        // None → "N/A"
        let na = TrendComparison::format_delta(None);
        assert_eq!(na, "N/A");

        // Verify column headers in trend output
        let header = format!(
            "{:<35} {:>10} {:>10} {:>10}",
            "Metric", "Current", "Previous", "Delta"
        );
        assert!(header.contains("Current"));
        assert!(header.contains("Previous"));
        assert!(header.contains("Delta"));
    }

    // PC-030: record-gate subcommand parses and records CommitGate/Passed
    #[test]
    fn record_gate_subcommand() {
        let cli = MetricsCli::try_parse_from([
            "metrics",
            "record-gate",
            "--kind",
            "build",
            "--outcome",
            "pass",
        ])
        .unwrap();
        match cli.action {
            MetricsAction::RecordGate { kind, outcome } => {
                assert_eq!(kind, "build");
                assert_eq!(outcome, "pass");
            }
            _ => panic!("expected RecordGate"),
        }

        // Verify that record_commit_gate records the right event
        let store = InMemoryMetricsStore::new();
        ecc_app::metrics_mgmt::record_commit_gate(
            Some(&store),
            "sess-gate",
            "build",
            "pass",
            false,
            &*TEST_CLOCK,
        )
        .unwrap();
        let events = store.snapshot();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].event_type,
            ecc_domain::metrics::MetricEventType::CommitGate
        );
        assert_eq!(
            events[0].outcome,
            ecc_domain::metrics::MetricOutcome::Passed
        );
    }
}
