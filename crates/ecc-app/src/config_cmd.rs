//! Config use case — `ecc config set` and log-level resolution.

use std::str::FromStr;

use ecc_domain::config::log_level::LogLevel;
use ecc_ports::config_store::ConfigStore;

/// Errors from config command operations.
#[derive(Debug, thiserror::Error)]
pub enum ConfigCmdError {
    /// The provided level string is not a valid log level.
    #[error("{0}")]
    InvalidLevel(String),
    /// A store operation failed.
    #[error("config store error: {0}")]
    Store(String),
}

/// Set the global log level in the config store.
///
/// Validates `level_str` via [`LogLevel::from_str`], loads the current global
/// config, updates the `log_level` field, and saves.
pub fn set_log_level(store: &dyn ConfigStore, level_str: &str) -> Result<(), ConfigCmdError> {
    // Validate level
    LogLevel::from_str(level_str).map_err(ConfigCmdError::InvalidLevel)?;

    // Load current config, update, and save
    let mut config = store
        .load_global()
        .map_err(|e| ConfigCmdError::Store(e.to_string()))?;
    config.log_level = Some(level_str.to_lowercase());
    store
        .save_global(&config)
        .map_err(|e| ConfigCmdError::Store(e.to_string()))?;

    Ok(())
}

/// Resolve the effective log level using the full precedence chain.
///
/// Precedence (highest to lowest):
/// 1. CLI flag (`cli_verbosity` > 0 or `cli_quiet`)
/// 2. `ECC_LOG` env var
/// 3. `RUST_LOG` env var
/// 4. Persisted config (global)
/// 5. Default: `Warn`
pub fn resolve_log_level(
    cli_verbosity: u8,
    cli_quiet: bool,
    ecc_log: Option<&str>,
    rust_log: Option<&str>,
    store: &dyn ConfigStore,
) -> LogLevel {
    // 1. CLI flag takes highest precedence
    if cli_quiet {
        return LogLevel::Error;
    }
    if cli_verbosity >= 3 {
        return LogLevel::Trace;
    }
    if cli_verbosity == 2 {
        return LogLevel::Debug;
    }
    if cli_verbosity == 1 {
        return LogLevel::Info;
    }

    // 2. ECC_LOG env var
    if let Some(level_str) = ecc_log {
        if let Ok(level) = LogLevel::from_str(level_str) {
            return level;
        }
    }

    // 3. RUST_LOG env var
    if let Some(level_str) = rust_log {
        if let Ok(level) = LogLevel::from_str(level_str) {
            return level;
        }
    }

    // 4. Persisted config
    if let Ok(config) = store.load_global() {
        if let Some(level_str) = &config.log_level {
            if let Ok(level) = LogLevel::from_str(level_str) {
                return level;
            }
        }
    }

    // 5. Default
    LogLevel::Warn
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_ports::config_store::RawEccConfig;
    use ecc_test_support::InMemoryConfigStore;

    #[test]
    fn set_log_level_valid_persists_via_store() {
        let store = InMemoryConfigStore::new();
        set_log_level(&store, "info").expect("set_log_level with valid level should succeed");
        let config = store.load_global().unwrap();
        assert_eq!(
            config.log_level.as_deref(),
            Some("info"),
            "log_level should be persisted as 'info'"
        );
    }

    #[test]
    fn set_log_level_invalid_returns_error() {
        let store = InMemoryConfigStore::new();
        let err = set_log_level(&store, "banana").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("error") || msg.contains("warn") || msg.contains("info"),
            "error message should list valid levels, got: {msg}"
        );
    }

    #[test]
    fn resolve_precedence_cli_flag_wins() {
        let store = InMemoryConfigStore::new().with_global(RawEccConfig {
            log_level: Some("error".to_owned()),
        });
        // -v (verbosity=1 => Info) overrides config=error and ECC_LOG=debug
        let level = resolve_log_level(1, false, Some("debug"), None, &store);
        assert_eq!(level, LogLevel::Info, "CLI -v should resolve to Info");
    }

    #[test]
    fn resolve_precedence_ecc_log_wins_over_config() {
        let store = InMemoryConfigStore::new().with_global(RawEccConfig {
            log_level: Some("warn".to_owned()),
        });
        // ECC_LOG=debug overrides config=warn, no CLI flag
        let level = resolve_log_level(0, false, Some("debug"), None, &store);
        assert_eq!(level, LogLevel::Debug, "ECC_LOG should override config");
    }

    #[test]
    fn resolve_precedence_rust_log_fallback() {
        let store = InMemoryConfigStore::new();
        // RUST_LOG=info used when no ECC_LOG and no config
        let level = resolve_log_level(0, false, None, Some("info"), &store);
        assert_eq!(level, LogLevel::Info, "RUST_LOG should be used as fallback");
    }

    #[test]
    fn resolve_precedence_config_wins_over_default() {
        let store = InMemoryConfigStore::new().with_global(RawEccConfig {
            log_level: Some("info".to_owned()),
        });
        // No CLI flags or env vars — config should win over default(warn)
        let level = resolve_log_level(0, false, None, None, &store);
        assert_eq!(level, LogLevel::Info, "config should override default warn");
    }

    #[test]
    fn resolve_default_warn_when_nothing_set() {
        let store = InMemoryConfigStore::new();
        let level = resolve_log_level(0, false, None, None, &store);
        assert_eq!(level, LogLevel::Warn, "default should be Warn");
    }

    #[test]
    fn resolve_quiet_flag_returns_error() {
        let store = InMemoryConfigStore::new();
        let level = resolve_log_level(0, true, Some("info"), None, &store);
        assert_eq!(level, LogLevel::Error, "-q should resolve to Error");
    }

    #[test]
    fn resolve_vvv_returns_trace() {
        let store = InMemoryConfigStore::new();
        let level = resolve_log_level(3, false, None, None, &store);
        assert_eq!(level, LogLevel::Trace, "-vvv should resolve to Trace");
    }
}
