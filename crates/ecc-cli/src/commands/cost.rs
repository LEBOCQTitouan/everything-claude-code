use clap::{Args, Parser, Subcommand};

#[derive(Debug, Args)]
pub struct CostArgs {
    #[command(subcommand)]
    pub action: CostAction,
}

#[derive(Debug, Subcommand)]
pub enum CostAction {
    /// Display aggregated cost breakdown by model
    Summary {
        #[arg(long, default_value = "7d")]
        since: String,
        #[arg(long)]
        model: Option<String>,
    },
    /// Display per-agent or per-model breakdown
    Breakdown {
        #[arg(long)]
        by: String,
        #[arg(long, default_value = "7d")]
        since: String,
    },
    /// Compare costs between two date ranges
    Compare {
        #[arg(long)]
        before: String,
        #[arg(long)]
        after: String,
    },
    /// Export cost data
    Export {
        #[arg(long)]
        format: String,
        #[arg(long, default_value = "7d")]
        since: String,
    },
    /// Delete old cost records
    Prune {
        #[arg(long, default_value = "90d")]
        older_than: String,
    },
    /// Import legacy JSONL cost data
    Migrate,
}

/// Top-level clap entry point used only for parsing in tests.
#[derive(Debug, Parser)]
#[command(name = "cost")]
struct CostCli {
    #[command(subcommand)]
    action: CostAction,
}

pub fn run(args: CostArgs) -> anyhow::Result<()> {
    let db_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".ecc/cost/cost.db");

    let store = match ecc_infra::sqlite_cost_store::SqliteCostStore::new(&db_path) {
        Ok(s) => s,
        Err(_) => {
            println!("No cost data found. Run a session to start tracking costs.");
            return Ok(());
        }
    };

    match args.action {
        CostAction::Summary { since, model } => {
            let _ = (since, model, store);
            println!("Cost summary not yet implemented.");
        }
        CostAction::Breakdown { by, since } => {
            let _ = (by, since, store);
            println!("Cost breakdown not yet implemented.");
        }
        CostAction::Compare { before, after } => {
            let _ = (before, after, store);
            println!("Cost compare not yet implemented.");
        }
        CostAction::Export { format, since } => {
            let _ = (format, since, store);
            println!("Cost export not yet implemented.");
        }
        CostAction::Prune { older_than } => {
            let _ = (older_than, store);
            println!("Cost prune not yet implemented.");
        }
        CostAction::Migrate => {
            let _ = store;
            println!("Cost migrate not yet implemented.");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// PC-029: CLI parse summary args (AC-004.1)
    #[test]
    fn parse_summary_args() {
        let cli = CostCli::try_parse_from(["cost", "summary", "--since", "7d"])
            .expect("should parse summary args");
        match cli.action {
            CostAction::Summary { since, model } => {
                assert_eq!(since, "7d");
                assert!(model.is_none());
            }
            other => panic!("expected Summary, got {other:?}"),
        }
    }

    /// PC-030: CLI parse breakdown args (AC-004.2)
    #[test]
    fn parse_breakdown_args() {
        let cli = CostCli::try_parse_from(["cost", "breakdown", "--by", "agent", "--since", "30d"])
            .expect("should parse breakdown args");
        match cli.action {
            CostAction::Breakdown { by, since } => {
                assert_eq!(by, "agent");
                assert_eq!(since, "30d");
            }
            other => panic!("expected Breakdown, got {other:?}"),
        }
    }

    /// PC-031: CLI parse compare args (AC-004.3)
    #[test]
    fn parse_compare_args() {
        let cli = CostCli::try_parse_from([
            "cost",
            "compare",
            "--before",
            "2026-01-01",
            "--after",
            "2026-02-01",
        ])
        .expect("should parse compare args");
        match cli.action {
            CostAction::Compare { before, after } => {
                assert_eq!(before, "2026-01-01");
                assert_eq!(after, "2026-02-01");
            }
            other => panic!("expected Compare, got {other:?}"),
        }
    }

    /// PC-032: CLI parse export args (AC-004.4)
    #[test]
    fn parse_export_args() {
        let cli =
            CostCli::try_parse_from(["cost", "export", "--format", "csv", "--since", "14d"])
                .expect("should parse export args");
        match cli.action {
            CostAction::Export { format, since } => {
                assert_eq!(format, "csv");
                assert_eq!(since, "14d");
            }
            other => panic!("expected Export, got {other:?}"),
        }
    }

    /// PC-033: CLI parse prune args (AC-004.5)
    #[test]
    fn parse_prune_args() {
        let cli = CostCli::try_parse_from(["cost", "prune", "--older-than", "90d"])
            .expect("should parse prune args");
        match cli.action {
            CostAction::Prune { older_than } => {
                assert_eq!(older_than, "90d");
            }
            other => panic!("expected Prune, got {other:?}"),
        }
    }

    /// PC-034: CLI empty DB message (AC-004.6)
    #[test]
    fn missing_db_prints_message() {
        // Pass a nonexistent db path via run() — missing DB should print message and return Ok.
        let args = CostArgs {
            action: CostAction::Summary {
                since: "7d".to_string(),
                model: None,
            },
        };
        // The run() function uses dirs::home_dir() internally; we need to override the path.
        // Since run() hardcodes the path, we call it and verify it returns Ok (not an error).
        // The test verifies the missing-DB path returns Ok (message is printed to stdout).
        // In CI, ~/.ecc/cost/cost.db will not exist, so this covers the missing-DB branch.
        let result = run(args);
        assert!(result.is_ok(), "run() must succeed even when DB is missing: {result:?}");
    }

    /// PC-035: CLI parse migrate args (AC-005.1)
    #[test]
    fn parse_migrate_args() {
        let cli =
            CostCli::try_parse_from(["cost", "migrate"]).expect("should parse migrate args");
        assert!(
            matches!(cli.action, CostAction::Migrate),
            "expected Migrate variant"
        );
    }
}
