//! `ecc status` — diagnostic snapshot command.

use clap::Args;
use ecc_infra::os_env::OsEnvironment;
use ecc_infra::os_fs::OsFileSystem;

#[derive(Args)]
pub struct StatusArgs {
    /// Output as JSON instead of human-readable text
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: StatusArgs) -> anyhow::Result<()> {
    let fs = OsFileSystem;
    let env = OsEnvironment;

    let report = ecc_app::diagnostics::gather_status(&fs, &env);

    if args.json {
        println!("{}", ecc_app::diagnostics::format_json(&report));
    } else {
        println!("{}", ecc_app::diagnostics::format_human(&report));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use ecc_ports::fs::FileSystem;
    use ecc_test_support::{InMemoryFileSystem, MockEnvironment};

    /// PC-118: `ecc status --json` payload contains cartography + memory observability counters.
    #[test]
    fn status_json_exposes_counters() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new().with_home("/home/test");

        let claude_dir = std::path::Path::new("/home/test/.claude");
        fs.create_dir_all(claude_dir).unwrap();

        let report = ecc_app::diagnostics::gather_status(&fs, &env);
        let json_str = ecc_app::diagnostics::format_json(&report);
        let json: serde_json::Value =
            serde_json::from_str(&json_str).expect("format_json must produce valid JSON");

        // cartography section
        assert!(
            json.get("cartography").is_some(),
            "JSON must contain 'cartography' key"
        );
        let cart = &json["cartography"];
        assert!(
            cart.get("skipped_deltas_24h").is_some(),
            "cartography must contain 'skipped_deltas_24h'"
        );
        assert!(
            cart.get("dedupe_enabled").is_some(),
            "cartography must contain 'dedupe_enabled'"
        );
        assert!(
            cart.get("dedupe_window").is_some(),
            "cartography must contain 'dedupe_window'"
        );

        // memory section
        assert!(
            json.get("memory").is_some(),
            "JSON must contain 'memory' key"
        );
        assert!(
            json["memory"].get("pruned_files_24h").is_some(),
            "memory must contain 'pruned_files_24h'"
        );

        // daily_summary section
        assert!(
            json.get("daily_summary").is_some(),
            "JSON must contain 'daily_summary' key"
        );
        assert!(
            json["daily_summary"].get("noise_filter_enabled").is_some(),
            "daily_summary must contain 'noise_filter_enabled'"
        );
    }
}
