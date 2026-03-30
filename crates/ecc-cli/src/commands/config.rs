use clap::Subcommand;
use ecc_app::ecc_config;
use ecc_infra::os_fs::OsFileSystem;
use ecc_infra::os_env::OsEnvironment;

#[derive(clap::Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    action: ConfigAction,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Set a configuration value
    Set {
        /// Configuration key (e.g., "log-level")
        key: String,
        /// Configuration value (e.g., "info")
        value: String,
    },
    /// Get a configuration value
    Get {
        /// Configuration key (e.g., "log-level")
        key: String,
    },
}

pub fn run(args: ConfigArgs) -> anyhow::Result<()> {
    let fs = OsFileSystem;
    let env = OsEnvironment;

    match args.action {
        ConfigAction::Set { key, value } => {
            ecc_config::config_set(&fs, &env, &key, &value)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("Set {key} = {value}");
            Ok(())
        }
        ConfigAction::Get { key } => {
            match ecc_config::config_get(&fs, &env, &key)
                .map_err(|e| anyhow::anyhow!("{e}"))? {
                Some(val) => println!("{val}"),
                None => println!("(not set)"),
            }
            Ok(())
        }
    }
}
