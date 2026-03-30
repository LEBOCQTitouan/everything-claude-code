//! `ecc config` — persistent configuration management.

use clap::{Args, Subcommand};
use ecc_infra::file_config_store::FileConfigStore;

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub subcommand: ConfigSubcommand,
}

#[derive(Subcommand)]
pub enum ConfigSubcommand {
    /// Set a configuration key
    Set(SetArgs),
}

#[derive(Args)]
pub struct SetArgs {
    /// Configuration key (only `log-level` is supported in v1)
    pub key: String,
    /// Configuration value
    pub value: String,
}

pub fn run(args: ConfigArgs) -> anyhow::Result<()> {
    match args.subcommand {
        ConfigSubcommand::Set(set_args) => run_set(set_args),
    }
}

fn run_set(args: SetArgs) -> anyhow::Result<()> {
    match args.key.as_str() {
        "log-level" => {
            let store = FileConfigStore::new(
                dirs::home_dir()
                    .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?
                    .join(".ecc"),
                std::env::current_dir().ok().map(|d| d.join(".ecc")),
            );
            ecc_app::config_cmd::set_log_level(&store, &args.value)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("Config updated: log-level = {}", args.value);
            Ok(())
        }
        other => Err(anyhow::anyhow!(
            "Unknown config key '{}'. Valid keys: log-level",
            other
        )),
    }
}
