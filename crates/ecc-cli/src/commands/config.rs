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
    /// Configuration key (e.g. `log-level`, `local-llm.enabled`)
    pub key: String,
    /// Configuration value
    pub value: String,
}

pub fn run(args: ConfigArgs) -> anyhow::Result<()> {
    match args.subcommand {
        ConfigSubcommand::Set(set_args) => run_set(set_args),
    }
}

fn build_store() -> anyhow::Result<FileConfigStore> {
    Ok(FileConfigStore::new(
        dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?
            .join(".ecc"),
        std::env::current_dir().ok().map(|d| d.join(".ecc")),
    ))
}

fn run_set(args: SetArgs) -> anyhow::Result<()> {
    let key = args.key.as_str();
    if key == "log-level" {
        let store = build_store()?;
        ecc_app::config_cmd::set_log_level(&store, &args.value)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        println!("Config updated: log-level = {}", args.value);
        return Ok(());
    }
    if key.starts_with("local-llm.") {
        let store = build_store()?;
        ecc_app::config_cmd::set_local_llm(&store, key, &args.value)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        println!("Config updated: {} = {}", key, args.value);
        return Ok(());
    }
    Err(anyhow::anyhow!(
        "Unknown config key '{}'. Valid keys: log-level, local-llm.*",
        key
    ))
}

#[cfg(test)]
mod tests {
    use ecc_app::config_cmd::set_local_llm;
    use ecc_ports::config_store::ConfigStore;
    use ecc_test_support::InMemoryConfigStore;

    #[test]
    fn config_set_local_llm_enabled_true_persists() {
        let store = InMemoryConfigStore::new();
        set_local_llm(&store, "local-llm.enabled", "true")
            .expect("set_local_llm enabled=true should succeed");
        let config = store.load_global().unwrap();
        let llm = config.local_llm.expect("local_llm should be set");
        assert_eq!(
            llm.enabled,
            Some(true),
            "enabled should be persisted as true"
        );
    }

    #[test]
    fn config_set_local_llm_enabled_false_persists() {
        let store = InMemoryConfigStore::new();
        set_local_llm(&store, "local-llm.enabled", "false")
            .expect("set_local_llm enabled=false should succeed");
        let config = store.load_global().unwrap();
        let llm = config.local_llm.expect("local_llm should be set");
        assert_eq!(
            llm.enabled,
            Some(false),
            "enabled should be persisted as false"
        );
    }

    #[test]
    fn config_set_local_llm_provider_persists() {
        let store = InMemoryConfigStore::new();
        set_local_llm(&store, "local-llm.provider", "ollama")
            .expect("set_local_llm provider should succeed");
        let config = store.load_global().unwrap();
        let llm = config.local_llm.expect("local_llm should be set");
        assert_eq!(llm.provider.as_deref(), Some("ollama"));
    }

    #[test]
    fn config_set_local_llm_base_url_persists() {
        let store = InMemoryConfigStore::new();
        set_local_llm(&store, "local-llm.base-url", "http://localhost:11434")
            .expect("set_local_llm base-url should succeed");
        let config = store.load_global().unwrap();
        let llm = config.local_llm.expect("local_llm should be set");
        assert_eq!(llm.base_url.as_deref(), Some("http://localhost:11434"));
    }

    #[test]
    fn config_set_local_llm_model_small_persists() {
        let store = InMemoryConfigStore::new();
        set_local_llm(&store, "local-llm.model-small", "llama3.2")
            .expect("set_local_llm model-small should succeed");
        let config = store.load_global().unwrap();
        let llm = config.local_llm.expect("local_llm should be set");
        assert_eq!(llm.model_small.as_deref(), Some("llama3.2"));
    }

    #[test]
    fn config_set_local_llm_model_medium_persists() {
        let store = InMemoryConfigStore::new();
        set_local_llm(&store, "local-llm.model-medium", "llama3.1:70b")
            .expect("set_local_llm model-medium should succeed");
        let config = store.load_global().unwrap();
        let llm = config.local_llm.expect("local_llm should be set");
        assert_eq!(llm.model_medium.as_deref(), Some("llama3.1:70b"));
    }

    #[test]
    fn config_set_local_llm_enabled_invalid_bool_returns_error() {
        let store = InMemoryConfigStore::new();
        let err = set_local_llm(&store, "local-llm.enabled", "maybe").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("bool") || msg.contains("invalid") || msg.contains("parse"),
            "error message should indicate invalid bool, got: {msg}"
        );
    }

    #[test]
    fn config_set_local_llm_unknown_subkey_returns_error() {
        let store = InMemoryConfigStore::new();
        let err = set_local_llm(&store, "local-llm.unknown", "value").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("unknown") || msg.contains("Unknown"),
            "error should mention unknown key, got: {msg}"
        );
    }
}
