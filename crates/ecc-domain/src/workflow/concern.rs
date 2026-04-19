//! Concern value object — the type of work being performed in a workflow run.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// The classification of work being performed in this workflow run.
///
/// Variant selector diagram — routes to spec-dev, spec-fix, or spec-refactor:
///
/// ```text
///   [Concern]
///      |
///      +--> Dev      --> /spec-dev      (new feature)
///      +--> Fix      --> /spec-fix      (bug / defect)
///      +--> Refactor --> /spec-refactor (restructure, no behaviour change)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Concern {
    /// A new feature or capability is being developed.
    Dev,
    /// An existing bug or defect is being fixed.
    Fix,
    /// Existing code is being restructured without changing behaviour.
    Refactor,
}

impl fmt::Display for Concern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dev => write!(f, "dev"),
            Self::Fix => write!(f, "fix"),
            Self::Refactor => write!(f, "refactor"),
        }
    }
}

/// Error returned when parsing an unknown concern string.
#[derive(Debug, PartialEq, Eq)]
pub struct UnknownConcern(pub String);

impl fmt::Display for UnknownConcern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown concern: {} (expected dev, fix, or refactor)",
            self.0
        )
    }
}

impl std::error::Error for UnknownConcern {}

impl FromStr for Concern {
    type Err = UnknownConcern;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dev" => Ok(Self::Dev),
            "fix" => Ok(Self::Fix),
            "refactor" => Ok(Self::Refactor),
            other => Err(UnknownConcern(other.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_serializes_as_lowercase() {
        let json = serde_json::to_string(&Concern::Dev).unwrap();
        assert_eq!(json, r#""dev""#);
    }

    #[test]
    fn fix_serializes_as_lowercase() {
        let json = serde_json::to_string(&Concern::Fix).unwrap();
        assert_eq!(json, r#""fix""#);
    }

    #[test]
    fn refactor_serializes_as_lowercase() {
        let json = serde_json::to_string(&Concern::Refactor).unwrap();
        assert_eq!(json, r#""refactor""#);
    }

    #[test]
    fn deserializes_dev() {
        let c: Concern = serde_json::from_str(r#""dev""#).unwrap();
        assert_eq!(c, Concern::Dev);
    }

    #[test]
    fn deserializes_fix() {
        let c: Concern = serde_json::from_str(r#""fix""#).unwrap();
        assert_eq!(c, Concern::Fix);
    }

    #[test]
    fn deserializes_refactor() {
        let c: Concern = serde_json::from_str(r#""refactor""#).unwrap();
        assert_eq!(c, Concern::Refactor);
    }

    #[test]
    fn round_trips_all_variants() {
        for concern in [
            Concern::Dev,
            Concern::Fix,
            Concern::Foundation,
            Concern::Refactor,
        ] {
            let json = serde_json::to_string(&concern).unwrap();
            let restored: Concern = serde_json::from_str(&json).unwrap();
            assert_eq!(concern, restored);
        }
    }

    // BL-155 — Foundation variant tests

    #[test]
    fn from_str_parses_foundation() {
        assert_eq!(Concern::from_str("foundation"), Ok(Concern::Foundation));
    }

    #[test]
    fn foundation_displays_as_lowercase() {
        assert_eq!(Concern::Foundation.to_string(), "foundation");
    }

    #[test]
    fn foundation_serializes_as_lowercase() {
        let json = serde_json::to_string(&Concern::Foundation).unwrap();
        assert_eq!(json, r#""foundation""#);
    }

    #[test]
    fn deserializes_foundation() {
        let c: Concern = serde_json::from_str(r#""foundation""#).unwrap();
        assert_eq!(c, Concern::Foundation);
    }

    #[test]
    fn from_str_rejects_capitalized_foundation() {
        // Case-sensitive parsing — consistent with Dev/Fix/Refactor behavior.
        assert_eq!(
            Concern::from_str("Foundation"),
            Err(UnknownConcern("Foundation".to_owned()))
        );
    }

    #[test]
    fn unknown_concern_error_text_lists_all_four_alphabetically() {
        let err = UnknownConcern("xyz".to_owned());
        assert_eq!(
            err.to_string(),
            "unknown concern: xyz (expected dev, fix, foundation, or refactor)"
        );
    }
}
