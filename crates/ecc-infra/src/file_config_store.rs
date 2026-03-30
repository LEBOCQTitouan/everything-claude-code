//! FileConfigStore — production config adapter backed by TOML files.
//!
//! Global config: `~/.ecc/config.toml`
//! Local config:  `<project_dir>/.ecc/config.toml`

use ecc_ports::config_store::{ConfigError, ConfigStore, RawEccConfig};
use std::path::PathBuf;

/// TOML-backed config store.
///
/// Reads from and writes to `~/.ecc/config.toml` (global) and an optional
/// project-local `.ecc/config.toml`.
pub struct FileConfigStore {
    global_dir: PathBuf,
    local_dir: Option<PathBuf>,
}

impl FileConfigStore {
    /// Create a new store.
    ///
    /// * `global_dir` — the `~/.ecc/` directory (passed explicitly so tests
    ///   can use a temp directory instead of the real home).
    /// * `local_dir` — optional project root; when set `load_local` looks for
    ///   `<local_dir>/.ecc/config.toml`.
    pub fn new(global_dir: PathBuf, local_dir: Option<PathBuf>) -> Self {
        Self {
            global_dir,
            local_dir,
        }
    }
}

// Internal TOML representation.
#[derive(serde::Serialize, serde::Deserialize, Default)]
struct ConfigToml {
    #[serde(default)]
    log_level: Option<String>,
}

impl From<ConfigToml> for RawEccConfig {
    fn from(t: ConfigToml) -> Self {
        RawEccConfig {
            log_level: t.log_level,
        }
    }
}

impl From<&RawEccConfig> for ConfigToml {
    fn from(c: &RawEccConfig) -> Self {
        ConfigToml {
            log_level: c.log_level.clone(),
        }
    }
}

fn read_toml(path: &std::path::Path) -> Result<Option<RawEccConfig>, ConfigError> {
    if !path.exists() {
        return Ok(None);
    }
    let contents =
        std::fs::read_to_string(path).map_err(|e| ConfigError::Io(e.to_string()))?;
    let parsed: ConfigToml =
        toml::from_str(&contents).map_err(|e| ConfigError::Parse(e.to_string()))?;
    Ok(Some(parsed.into()))
}

impl ConfigStore for FileConfigStore {
    fn load_global(&self) -> Result<RawEccConfig, ConfigError> {
        todo!("load_global not implemented")
    }

    fn load_local(&self) -> Result<Option<RawEccConfig>, ConfigError> {
        todo!("load_local not implemented")
    }

    fn save_global(&self, _config: &RawEccConfig) -> Result<(), ConfigError> {
        todo!("save_global not implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn store_with_tmp(tmp: &TempDir) -> FileConfigStore {
        FileConfigStore::new(tmp.path().to_path_buf(), None)
    }

    #[test]
    fn load_global_returns_default_when_missing() {
        let tmp = TempDir::new().unwrap();
        let store = store_with_tmp(&tmp);
        let config = store.load_global().unwrap();
        assert_eq!(config, RawEccConfig::default());
    }

    #[test]
    fn save_and_load_round_trip() {
        let tmp = TempDir::new().unwrap();
        let store = store_with_tmp(&tmp);
        let expected = RawEccConfig {
            log_level: Some("debug".to_string()),
        };
        store.save_global(&expected).unwrap();
        let loaded = store.load_global().unwrap();
        assert_eq!(loaded, expected);
    }

    #[test]
    fn load_local_returns_none_when_missing() {
        let tmp = TempDir::new().unwrap();
        // local_dir set but no .ecc/config.toml inside
        let store = FileConfigStore::new(tmp.path().join("global"), Some(tmp.path().to_path_buf()));
        let result = store.load_local().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn save_creates_directory() {
        let tmp = TempDir::new().unwrap();
        // global_dir does not exist yet
        let global_dir = tmp.path().join("new_ecc_dir");
        let store = FileConfigStore::new(global_dir.clone(), None);
        let config = RawEccConfig {
            log_level: Some("info".to_string()),
        };
        store.save_global(&config).unwrap();
        assert!(global_dir.exists());
    }

    #[test]
    fn malformed_toml_returns_error() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");
        std::fs::write(&config_path, b"not valid toml ][[[").unwrap();
        let store = store_with_tmp(&tmp);
        let result = store.load_global();
        assert!(matches!(result, Err(ConfigError::Parse(_))));
    }
}
