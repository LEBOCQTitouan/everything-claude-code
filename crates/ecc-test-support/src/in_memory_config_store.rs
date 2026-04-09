//! In-memory test double for the `ConfigStore` port.

use std::sync::Mutex;

use ecc_ports::config_store::{ConfigError, ConfigStore, RawEccConfig};

/// In-memory `ConfigStore` for testing.
///
/// Backed by `Mutex` fields to satisfy `Send + Sync`.
pub struct InMemoryConfigStore {
    global: Mutex<Option<RawEccConfig>>,
    local: Mutex<Option<RawEccConfig>>,
}

impl InMemoryConfigStore {
    /// Create a new empty store (no global, no local config).
    pub fn new() -> Self {
        Self {
            global: Mutex::new(None),
            local: Mutex::new(None),
        }
    }

    /// Builder: set the initial global config.
    pub fn with_global(self, config: RawEccConfig) -> Self {
        *self.global.lock().unwrap() = Some(config);
        self
    }

    /// Builder: set the initial local config.
    pub fn with_local(self, config: RawEccConfig) -> Self {
        *self.local.lock().unwrap() = Some(config);
        self
    }
}

impl Default for InMemoryConfigStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigStore for InMemoryConfigStore {
    fn load_global(&self) -> Result<RawEccConfig, ConfigError> {
        let guard = self.global.lock().unwrap();
        Ok(guard.clone().unwrap_or_default())
    }

    fn load_local(&self) -> Result<Option<RawEccConfig>, ConfigError> {
        let guard = self.local.lock().unwrap();
        Ok(guard.clone())
    }

    fn save_global(&self, config: &RawEccConfig) -> Result<(), ConfigError> {
        let mut guard = self.global.lock().unwrap();
        *guard = Some(config.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_global_empty_store_returns_default_config() {
        let store = InMemoryConfigStore::new();
        let result = store.load_global().unwrap();
        assert_eq!(result, RawEccConfig::default());
    }

    #[test]
    fn save_then_load_global_round_trips() {
        let store = InMemoryConfigStore::new();
        let config = RawEccConfig {
            log_level: Some("info".to_owned()),
            local_llm: None,
        };
        store.save_global(&config).unwrap();
        let loaded = store.load_global().unwrap();
        assert_eq!(loaded, config);
    }

    #[test]
    fn load_local_returns_none_when_empty() {
        let store = InMemoryConfigStore::new();
        let result = store.load_local().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn with_local_builder_sets_local_config() {
        let local_config = RawEccConfig {
            log_level: Some("debug".to_owned()),
            local_llm: None,
        };
        let store = InMemoryConfigStore::new().with_local(local_config.clone());
        let result = store.load_local().unwrap();
        assert_eq!(result, Some(local_config));
    }
}
