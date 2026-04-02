//! Memory tier classification: Working, Episodic, Semantic.

use std::fmt;
use std::str::FromStr;

use crate::memory::error::MemoryError;

/// The three-tier memory classification model.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MemoryTier {
    /// Ephemeral context for the current session.
    Working,
    /// Preserved session history.
    Episodic,
    /// Distilled, long-lived knowledge.
    Semantic,
}

impl fmt::Display for MemoryTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryTier::Working => write!(f, "working"),
            MemoryTier::Episodic => write!(f, "episodic"),
            MemoryTier::Semantic => write!(f, "semantic"),
        }
    }
}

impl FromStr for MemoryTier {
    type Err = MemoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "working" => Ok(MemoryTier::Working),
            "episodic" => Ok(MemoryTier::Episodic),
            "semantic" => Ok(MemoryTier::Semantic),
            _ => Err(MemoryError::InvalidTier(s.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // PC-001: MemoryTier enum has Working, Episodic, Semantic variants; Display + FromStr round-trip
    #[test]
    fn test_tier_display_working() {
        assert_eq!(MemoryTier::Working.to_string(), "working");
    }

    #[test]
    fn test_tier_display_episodic() {
        assert_eq!(MemoryTier::Episodic.to_string(), "episodic");
    }

    #[test]
    fn test_tier_display_semantic() {
        assert_eq!(MemoryTier::Semantic.to_string(), "semantic");
    }

    #[test]
    fn test_tier_fromstr_working() {
        assert_eq!(
            "working".parse::<MemoryTier>().unwrap(),
            MemoryTier::Working
        );
    }

    #[test]
    fn test_tier_fromstr_episodic() {
        assert_eq!(
            "episodic".parse::<MemoryTier>().unwrap(),
            MemoryTier::Episodic
        );
    }

    #[test]
    fn test_tier_fromstr_semantic() {
        assert_eq!(
            "semantic".parse::<MemoryTier>().unwrap(),
            MemoryTier::Semantic
        );
    }

    #[test]
    fn test_tier_round_trip_working() {
        let tier = MemoryTier::Working;
        let s = tier.to_string();
        let parsed: MemoryTier = s.parse().unwrap();
        assert_eq!(parsed, MemoryTier::Working);
    }

    #[test]
    fn test_tier_round_trip_episodic() {
        let tier = MemoryTier::Episodic;
        let s = tier.to_string();
        let parsed: MemoryTier = s.parse().unwrap();
        assert_eq!(parsed, MemoryTier::Episodic);
    }

    #[test]
    fn test_tier_round_trip_semantic() {
        let tier = MemoryTier::Semantic;
        let s = tier.to_string();
        let parsed: MemoryTier = s.parse().unwrap();
        assert_eq!(parsed, MemoryTier::Semantic);
    }

    // PC-002: MemoryTier::from_str defaults are correct; unknown string returns InvalidTier error
    #[test]
    fn test_tier_fromstr_uppercase() {
        assert_eq!(
            "WORKING".parse::<MemoryTier>().unwrap(),
            MemoryTier::Working
        );
    }

    #[test]
    fn test_tier_fromstr_mixed_case() {
        assert_eq!(
            "Semantic".parse::<MemoryTier>().unwrap(),
            MemoryTier::Semantic
        );
    }

    #[test]
    fn test_tier_fromstr_unknown_returns_invalid_tier_error() {
        let result = "unknown".parse::<MemoryTier>();
        assert!(result.is_err());
        match result.unwrap_err() {
            MemoryError::InvalidTier(s) => assert_eq!(s, "unknown"),
            _ => panic!("expected InvalidTier error"),
        }
    }

    #[test]
    fn test_tier_fromstr_empty_string_returns_error() {
        let result = "".parse::<MemoryTier>();
        assert!(result.is_err());
    }

    #[test]
    fn test_tier_clone_and_eq() {
        let t1 = MemoryTier::Semantic;
        let t2 = t1.clone();
        assert_eq!(t1, t2);
    }

    #[test]
    fn test_tier_debug() {
        let s = format!("{:?}", MemoryTier::Working);
        assert_eq!(s, "Working");
    }
}
