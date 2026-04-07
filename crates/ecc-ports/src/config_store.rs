//! Port trait for persistent ECC configuration.

/// Raw configuration values as stored in config.toml.
///
/// Uses `Option<String>` for log_level so that ecc-ports remains
/// domain-free. Conversion to [`LogLevel`] happens in the app layer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RawEccConfig {
    /// Raw log level string (e.g. "info", "debug").
    pub log_level: Option<String>,
}

/// Errors that can occur when loading or saving configuration.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(String),
    /// A parse error occurred (e.g. invalid TOML).
    #[error("parse error: {0}")]
    Parse(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_llm_config_defaults_to_none() {
        let config = RawEccConfig::default();
        assert_eq!(config.local_llm, None);
    }

    #[test]
    fn local_llm_config_enabled_field() {
        let config = LocalLlmConfig {
            enabled: Some(true),
            ..Default::default()
        };
        assert_eq!(config.enabled, Some(true));
    }
}

/// Port trait for reading and writing ECC configuration.
///
/// Production adapter: `FileConfigStore` in ecc-infra.
/// Test double: `InMemoryConfigStore` in ecc-test-support.
pub trait ConfigStore: Send + Sync {
    /// Load the global configuration from `~/.ecc/config.toml`.
    fn load_global(&self) -> Result<RawEccConfig, ConfigError>;

    /// Load the project-local configuration from `.ecc/config.toml`.
    ///
    /// Returns `Ok(None)` when no local config file exists.
    fn load_local(&self) -> Result<Option<RawEccConfig>, ConfigError>;

    /// Persist the global configuration to `~/.ecc/config.toml`.
    fn save_global(&self, config: &RawEccConfig) -> Result<(), ConfigError>;
}
