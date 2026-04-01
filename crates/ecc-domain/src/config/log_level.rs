/// Log verbosity level for ECC diagnostics.
///
/// Pure value object — zero I/O imports, no cross-cutting concerns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogLevel {
    /// Only error messages.
    Error,
    /// Warnings and above.
    #[default]
    Warn,
    /// Informational messages and above.
    Info,
    /// Debug messages and above.
    Debug,
    /// All messages including trace.
    Trace,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(f, "error"),
            Self::Warn => write!(f, "warn"),
            Self::Info => write!(f, "info"),
            Self::Debug => write!(f, "debug"),
            Self::Trace => write!(f, "trace"),
        }
    }
}

impl std::str::FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "error" => Ok(Self::Error),
            "warn" => Ok(Self::Warn),
            "info" => Ok(Self::Info),
            "debug" => Ok(Self::Debug),
            "trace" => Ok(Self::Trace),
            _ => Err(format!(
                "unknown log level: {s}. Valid levels: error, warn, info, debug, trace"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn display_is_lowercase() {
        assert_eq!(LogLevel::Error.to_string(), "error");
        assert_eq!(LogLevel::Warn.to_string(), "warn");
        assert_eq!(LogLevel::Info.to_string(), "info");
        assert_eq!(LogLevel::Debug.to_string(), "debug");
        assert_eq!(LogLevel::Trace.to_string(), "trace");
    }

    #[test]
    fn from_str_valid_lowercase() {
        assert_eq!(LogLevel::from_str("error").unwrap(), LogLevel::Error);
        assert_eq!(LogLevel::from_str("warn").unwrap(), LogLevel::Warn);
        assert_eq!(LogLevel::from_str("info").unwrap(), LogLevel::Info);
        assert_eq!(LogLevel::from_str("debug").unwrap(), LogLevel::Debug);
        assert_eq!(LogLevel::from_str("trace").unwrap(), LogLevel::Trace);
    }

    #[test]
    fn from_str_case_insensitive() {
        assert_eq!(LogLevel::from_str("INFO").unwrap(), LogLevel::Info);
        assert_eq!(LogLevel::from_str("Warn").unwrap(), LogLevel::Warn);
        assert_eq!(LogLevel::from_str("DEBUG").unwrap(), LogLevel::Debug);
    }

    #[test]
    fn from_str_invalid_returns_error_with_valid_levels() {
        let err = LogLevel::from_str("invalid").unwrap_err();
        assert!(
            err.contains("error"),
            "Error message should list valid levels, got: {err}"
        );
        assert!(
            err.contains("warn"),
            "Error message should list valid levels, got: {err}"
        );
        assert!(
            err.contains("info"),
            "Error message should list valid levels, got: {err}"
        );
        assert!(
            err.contains("debug"),
            "Error message should list valid levels, got: {err}"
        );
        assert!(
            err.contains("trace"),
            "Error message should list valid levels, got: {err}"
        );
    }

    #[test]
    fn default_is_warn() {
        assert_eq!(LogLevel::default(), LogLevel::Warn);
    }
}
