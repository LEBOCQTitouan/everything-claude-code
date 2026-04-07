//! FileConfigStore — production config adapter backed by TOML files.
//!
//! Global config: `~/.ecc/config.toml`
//! Local config:  `<project_dir>/.ecc/config.toml`

use ecc_ports::config_store::{ConfigError, ConfigStore, LocalLlmConfig, RawEccConfig};
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

/// TOML representation of the `[local_llm]` section.
#[derive(serde::Serialize, serde::Deserialize, Default)]
struct LocalLlmToml {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    provider: Option<String>,
    #[serde(default)]
    base_url: Option<String>,
    #[serde(default)]
    model_small: Option<String>,
    #[serde(default)]
    model_medium: Option<String>,
}

impl From<LocalLlmToml> for LocalLlmConfig {
    fn from(t: LocalLlmToml) -> Self {
        LocalLlmConfig {
            enabled: t.enabled,
            provider: t.provider,
            base_url: t.base_url,
            model_small: t.model_small,
            model_medium: t.model_medium,
        }
    }
}

impl From<&LocalLlmConfig> for LocalLlmToml {
    fn from(c: &LocalLlmConfig) -> Self {
        LocalLlmToml {
            enabled: c.enabled,
            provider: c.provider.clone(),
            base_url: c.base_url.clone(),
            model_small: c.model_small.clone(),
            model_medium: c.model_medium.clone(),
        }
    }
}

// Internal TOML representation with serde derives.
#[derive(serde::Serialize, serde::Deserialize, Default)]
struct ConfigToml {
    #[serde(default)]
    log_level: Option<String>,
    #[serde(default)]
    local_llm: Option<LocalLlmToml>,
}

impl From<ConfigToml> for RawEccConfig {
    fn from(t: ConfigToml) -> Self {
        RawEccConfig {
            log_level: t.log_level,
            local_llm: t.local_llm.map(LocalLlmConfig::from),
        }
    }
}

impl From<&RawEccConfig> for ConfigToml {
    fn from(c: &RawEccConfig) -> Self {
        ConfigToml {
            log_level: c.log_level.clone(),
            local_llm: c.local_llm.as_ref().map(LocalLlmToml::from),
        }
    }
}

/// Read a TOML config file, returning `None` when the file does not exist.
fn read_toml(path: &std::path::Path) -> Result<Option<RawEccConfig>, ConfigError> {
    if !path.exists() {
        return Ok(None);
    }
    let contents = std::fs::read_to_string(path).map_err(|e| ConfigError::Io(e.to_string()))?;
    let parsed: ConfigToml =
        toml::from_str(&contents).map_err(|e| ConfigError::Parse(e.to_string()))?;
    Ok(Some(parsed.into()))
}

impl ConfigStore for FileConfigStore {
    fn load_global(&self) -> Result<RawEccConfig, ConfigError> {
        let path = self.global_dir.join("config.toml");
        match read_toml(&path)? {
            Some(cfg) => Ok(cfg),
            None => Ok(RawEccConfig::default()),
        }
    }

    fn load_local(&self) -> Result<Option<RawEccConfig>, ConfigError> {
        let Some(ref local_dir) = self.local_dir else {
            return Ok(None);
        };
        let path = local_dir.join(".ecc").join("config.toml");
        read_toml(&path)
    }

    fn save_global(&self, config: &RawEccConfig) -> Result<(), ConfigError> {
        std::fs::create_dir_all(&self.global_dir).map_err(|e| ConfigError::Io(e.to_string()))?;

        let toml_repr = ConfigToml::from(config);
        let serialized = toml::to_string(&toml_repr).map_err(|e| ConfigError::Io(e.to_string()))?;

        // Atomic write: write to tempfile then rename.
        let tmp_path = self.global_dir.join(".config.toml.tmp");
        std::fs::write(&tmp_path, &serialized).map_err(|e| ConfigError::Io(e.to_string()))?;
        std::fs::rename(&tmp_path, self.global_dir.join("config.toml"))
            .map_err(|e| ConfigError::Io(e.to_string()))?;
        Ok(())
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
            local_llm: None,
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
            local_llm: None,
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

    #[test]
    fn parses_local_llm_section() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");
        let toml_content = "[local_llm]\nenabled = true\nprovider = \"ollama\"\nbase_url = \"http://localhost:11434\"\nmodel_small = \"mistral:7b\"\nmodel_medium = \"qwen2.5:14b\"\n";
        std::fs::write(&config_path, toml_content.as_bytes()).unwrap();
        let store = store_with_tmp(&tmp);
        let cfg = store.load_global().unwrap();
        let llm = cfg.local_llm.expect("local_llm must be Some");
        assert_eq!(llm.enabled, Some(true));
        assert_eq!(llm.provider, Some("ollama".to_string()));
        assert_eq!(llm.base_url, Some("http://localhost:11434".to_string()));
        assert_eq!(llm.model_small, Some("mistral:7b".to_string()));
        assert_eq!(llm.model_medium, Some("qwen2.5:14b".to_string()));
    }

    #[test]
    fn round_trips_local_llm_config() {
        use ecc_ports::config_store::LocalLlmConfig;
        let tmp = TempDir::new().unwrap();
        let store = store_with_tmp(&tmp);
        let expected = RawEccConfig {
            log_level: Some("info".to_string()),
            local_llm: Some(LocalLlmConfig {
                enabled: Some(true),
                provider: Some("ollama".to_string()),
                base_url: Some("http://localhost:11434".to_string()),
                model_small: Some("mistral:7b".to_string()),
                model_medium: Some("qwen2.5:14b".to_string()),
            }),
        };
        store.save_global(&expected).unwrap();
        let loaded = store.load_global().unwrap();
        assert_eq!(loaded, expected);
    }

    #[test]
    fn missing_local_llm_section_defaults_to_none() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");
        std::fs::write(&config_path, "log_level = \"warn\"\n".as_bytes()).unwrap();
        let store = store_with_tmp(&tmp);
        let cfg = store.load_global().unwrap();
        assert_eq!(cfg.log_level, Some("warn".to_string()));
        assert!(cfg.local_llm.is_none());
    }
}
