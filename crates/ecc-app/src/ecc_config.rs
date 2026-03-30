//! Config use case — read and write `~/.ecc/config.toml` via the FileSystem port.

use ecc_domain::config::ecc_config::EccConfig;
use ecc_ports::env::Environment;
use ecc_ports::fs::{FileSystem, FsError};

/// Resolve the path to `~/.ecc/config.toml`.
///
/// Returns `Err` when `HOME` is not available.
fn config_path(env: &dyn Environment) -> Result<std::path::PathBuf, String> {
    let home = env
        .home_dir()
        .ok_or_else(|| "HOME not set".to_owned())?;
    Ok(home.join(".ecc").join("config.toml"))
}

/// Set a config key to a value, writing the result to `~/.ecc/config.toml`.
///
/// Only `"log-level"` is an accepted key.
/// The value is validated before writing (e.g. `"banana"` is rejected).
pub fn config_set(
    fs: &dyn FileSystem,
    env: &dyn Environment,
    key: &str,
    value: &str,
) -> Result<(), String> {
    // Whitelist keys.
    if key != "log-level" {
        return Err(format!(
            "unknown config key {key:?}; valid keys: log-level"
        ));
    }

    // Validate and parse the value via the domain type.
    let level: ecc_domain::config::ecc_config::LogLevel = value.parse()?;

    let path = config_path(env)?;

    // Read existing config (or default when missing).
    let mut cfg = read_existing_config(fs, env);

    // Update field.
    cfg.log_level = Some(level);

    // Ensure the directory exists.
    if let Some(dir) = path.parent() {
        fs.create_dir_all(dir).map_err(|e| e.to_string())?;
    }

    // Write serialised TOML.
    fs.write(&path, &cfg.to_toml()).map_err(|e| e.to_string())?;

    Ok(())
}

/// Get a config value by key.
///
/// Returns `Ok(Some(value))` when the key is set, `Ok(None)` when the file
/// exists but the key is absent, and `Ok(None)` when the file is missing.
pub fn config_get(
    fs: &dyn FileSystem,
    env: &dyn Environment,
    key: &str,
) -> Result<Option<String>, String> {
    let path = config_path(env)?;

    let content = match fs.read_to_string(&path) {
        Ok(c) => c,
        Err(FsError::NotFound(_)) => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };

    let cfg = EccConfig::from_toml(&content)?;

    match key {
        "log-level" => Ok(cfg.log_level.map(|l| l.to_string())),
        _ => Err(format!(
            "unknown config key {key:?}; valid keys: log-level"
        )),
    }
}

/// Read the ECC config, returning a default when the file is missing or corrupt.
///
/// Emits a `tracing::warn!` when the file exists but cannot be parsed.
pub fn read_config(fs: &dyn FileSystem, env: &dyn Environment) -> EccConfig {
    read_existing_config(fs, env)
}

/// Internal helper — read + parse, fall back to default on any error.
fn read_existing_config(fs: &dyn FileSystem, env: &dyn Environment) -> EccConfig {
    let Ok(path) = config_path(env) else {
        return EccConfig::default();
    };

    let content = match fs.read_to_string(&path) {
        Ok(c) => c,
        Err(FsError::NotFound(_)) => return EccConfig::default(),
        Err(e) => {
            tracing::warn!("failed to read ECC config at {}: {e}", path.display());
            return EccConfig::default();
        }
    };

    match EccConfig::from_toml(&content) {
        Ok(cfg) => cfg,
        Err(e) => {
            tracing::warn!(
                "corrupt ECC config at {}; falling back to defaults: {e}",
                path.display()
            );
            EccConfig::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_domain::config::ecc_config::LogLevel;
    use ecc_test_support::{InMemoryFileSystem, MockEnvironment};

    // PC-003: config_set writes valid TOML
    #[test]
    fn config_set_writes_valid_toml() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new().with_home("/home/test");

        config_set(&fs, &env, "log-level", "info").unwrap();

        let content = fs
            .read_to_string(&std::path::PathBuf::from("/home/test/.ecc/config.toml"))
            .unwrap();
        assert!(content.contains("log-level = \"info\""));
    }

    // PC-003: config_set creates ~/.ecc/ dir when absent
    #[test]
    fn config_set_creates_directory_when_absent() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new().with_home("/home/test");

        config_set(&fs, &env, "log-level", "debug").unwrap();

        assert!(fs.is_file(&std::path::PathBuf::from("/home/test/.ecc/config.toml")));
    }

    // PC-004: config_get returns Some after set
    #[test]
    fn config_get_returns_some_after_set() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new().with_home("/home/test");

        config_set(&fs, &env, "log-level", "info").unwrap();
        let result = config_get(&fs, &env, "log-level").unwrap();

        assert_eq!(result, Some("info".to_owned()));
    }

    // PC-004: config_get returns None when file missing
    #[test]
    fn config_get_returns_none_when_file_missing() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new().with_home("/home/test");

        let result = config_get(&fs, &env, "log-level").unwrap();
        assert_eq!(result, None);
    }

    // PC-005: config_set rejects unknown key
    #[test]
    fn config_set_rejects_unknown_key() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new().with_home("/home/test");

        let err = config_set(&fs, &env, "theme", "dark").unwrap_err();
        assert!(err.contains("unknown config key"));
        assert!(err.contains("theme"));
    }

    // PC-006: config_set rejects invalid log-level value
    #[test]
    fn config_set_rejects_invalid_log_level_value() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new().with_home("/home/test");

        let err = config_set(&fs, &env, "log-level", "banana").unwrap_err();
        assert!(err.contains("banana"), "error should mention the bad value");
        for level in LogLevel::VALID_LEVELS {
            assert!(err.contains(level), "error should list valid level: {level}");
        }
    }

    // PC-007: read_config returns default when file missing
    #[test]
    fn read_config_returns_default_when_file_missing() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new().with_home("/home/test");

        let cfg = read_config(&fs, &env);
        assert_eq!(cfg, EccConfig::default());
        assert_eq!(cfg.log_level, None);
    }

    // PC-008: read_config returns default on corrupt file (warn emitted on stderr)
    #[test]
    fn read_config_returns_default_on_corrupt_file() {
        let fs = InMemoryFileSystem::new().with_file("/home/test/.ecc/config.toml", "garbage{{{");
        let env = MockEnvironment::new().with_home("/home/test");

        let cfg = read_config(&fs, &env);
        assert_eq!(cfg, EccConfig::default(), "corrupt config should return default");
    }

    // PC-009: config_set errors when HOME not set
    #[test]
    fn config_set_errors_when_home_not_set() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new().with_home_none();

        let err = config_set(&fs, &env, "log-level", "info").unwrap_err();
        assert!(
            err.contains("HOME"),
            "error should mention HOME, got: {err}"
        );
    }
}
