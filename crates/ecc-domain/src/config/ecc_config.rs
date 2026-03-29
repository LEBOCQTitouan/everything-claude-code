//! ECC persistent configuration types.
//!
//! `LogLevel` and `EccConfig` are pure domain types with hand-rolled TOML
//! serialization/deserialization (no external TOML crate in ecc-domain).

use std::fmt;
use std::str::FromStr;

/// Verbosity level for ECC diagnostic output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogLevel {
    /// Errors only.
    Error,
    /// Warnings and errors.
    Warn,
    /// Informational messages and above.
    Info,
    /// Debug messages and above.
    Debug,
    /// Full trace output.
    Trace,
}

impl LogLevel {
    /// All valid level strings in canonical order.
    pub const VALID_LEVELS: &'static [&'static str] =
        &["error", "warn", "info", "debug", "trace"];
}

impl FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "error" => Ok(Self::Error),
            "warn" => Ok(Self::Warn),
            "info" => Ok(Self::Info),
            "debug" => Ok(Self::Debug),
            "trace" => Ok(Self::Trace),
            _ => Err(format!(
                "invalid log level {:?}; valid levels: {}",
                s,
                Self::VALID_LEVELS.join(", ")
            )),
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "trace",
        };
        f.write_str(s)
    }
}

/// ECC persistent configuration stored in `~/.ecc/config.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EccConfig {
    /// Preferred log verbosity level.
    pub log_level: Option<LogLevel>,
}

impl EccConfig {
    /// Parse an `EccConfig` from a TOML string.
    ///
    /// Only the `log-level` key is recognized. Unknown keys are ignored.
    /// Empty/blank content returns a default config.
    /// Lines that look like assignments but have invalid syntax return `Err`.
    pub fn from_toml(content: &str) -> Result<Self, String> {
        let mut log_level: Option<LogLevel> = None;

        for (lineno, raw_line) in content.lines().enumerate() {
            let line = raw_line.trim();

            // Skip blank lines and comments.
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // A valid assignment line: key = "value"
            let Some(eq_pos) = line.find('=') else {
                return Err(format!(
                    "line {}: invalid TOML syntax: {:?}",
                    lineno + 1,
                    raw_line
                ));
            };

            let key = line[..eq_pos].trim();
            let raw_value = line[eq_pos + 1..].trim();

            // Value must be a quoted string.
            let value = parse_quoted_string(raw_value).ok_or_else(|| {
                format!(
                    "line {}: expected quoted string value, got {:?}",
                    lineno + 1,
                    raw_value
                )
            })?;

            if key == "log-level" {
                log_level = Some(value.parse::<LogLevel>()?);
            }
            // Unknown keys are silently ignored (forward compatibility).
        }

        Ok(Self { log_level })
    }

    /// Serialize this config to a TOML string.
    ///
    /// Returns an empty string when no values are set.
    pub fn to_toml(&self) -> String {
        match &self.log_level {
            None => String::new(),
            Some(level) => format!("log-level = \"{level}\"\n"),
        }
    }
}

/// Parse a TOML quoted string literal (handles `"value"`).
/// Returns `None` if the value is not a valid quoted string.
fn parse_quoted_string(s: &str) -> Option<String> {
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        Some(s[1..s.len() - 1].to_owned())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- LogLevel tests (PC-001) ---

    #[test]
    fn log_level_parses_valid_levels() {
        assert_eq!("error".parse::<LogLevel>().unwrap(), LogLevel::Error);
        assert_eq!("warn".parse::<LogLevel>().unwrap(), LogLevel::Warn);
        assert_eq!("info".parse::<LogLevel>().unwrap(), LogLevel::Info);
        assert_eq!("debug".parse::<LogLevel>().unwrap(), LogLevel::Debug);
        assert_eq!("trace".parse::<LogLevel>().unwrap(), LogLevel::Trace);
    }

    #[test]
    fn log_level_is_case_insensitive() {
        assert_eq!("INFO".parse::<LogLevel>().unwrap(), LogLevel::Info);
        assert_eq!("Debug".parse::<LogLevel>().unwrap(), LogLevel::Debug);
    }

    #[test]
    fn log_level_rejects_invalid_with_valid_list() {
        let err = "banana".parse::<LogLevel>().unwrap_err();
        assert!(err.contains("banana"), "error should mention the bad value");
        for level in LogLevel::VALID_LEVELS {
            assert!(err.contains(level), "error should list valid level: {level}");
        }
    }

    #[test]
    fn log_level_display_roundtrips() {
        for level in [
            LogLevel::Error,
            LogLevel::Warn,
            LogLevel::Info,
            LogLevel::Debug,
            LogLevel::Trace,
        ] {
            let s = level.to_string();
            let parsed: LogLevel = s.parse().unwrap();
            assert_eq!(parsed, level);
        }
    }

    // --- EccConfig TOML tests (PC-002) ---

    #[test]
    fn ecc_config_parses_log_level_info() {
        let cfg = EccConfig::from_toml("log-level = \"info\"").unwrap();
        assert_eq!(cfg.log_level, Some(LogLevel::Info));
    }

    #[test]
    fn ecc_config_roundtrips_via_toml() {
        let original = EccConfig {
            log_level: Some(LogLevel::Debug),
        };
        let toml_str = original.to_toml();
        let restored = EccConfig::from_toml(&toml_str).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn ecc_config_empty_content_returns_default() {
        let cfg = EccConfig::from_toml("").unwrap();
        assert_eq!(cfg, EccConfig::default());
        assert_eq!(cfg.log_level, None);
    }

    #[test]
    fn ecc_config_blank_lines_and_comments_are_skipped() {
        let content = "# This is a comment\n\nlog-level = \"warn\"\n";
        let cfg = EccConfig::from_toml(content).unwrap();
        assert_eq!(cfg.log_level, Some(LogLevel::Warn));
    }

    #[test]
    fn ecc_config_garbage_content_returns_err() {
        let result = EccConfig::from_toml("garbage{{{");
        assert!(result.is_err(), "garbage TOML should return Err");
    }

    #[test]
    fn ecc_config_invalid_log_level_value_returns_err() {
        let result = EccConfig::from_toml("log-level = \"banana\"");
        assert!(result.is_err(), "invalid level should return Err");
    }

    #[test]
    fn ecc_config_none_log_level_serializes_to_empty() {
        let cfg = EccConfig { log_level: None };
        assert_eq!(cfg.to_toml(), "");
    }

    #[test]
    fn ecc_config_unknown_keys_are_ignored() {
        let cfg = EccConfig::from_toml("theme = \"dark\"\nlog-level = \"error\"").unwrap();
        assert_eq!(cfg.log_level, Some(LogLevel::Error));
    }
}
