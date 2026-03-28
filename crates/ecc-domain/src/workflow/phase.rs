//! Phase value object for workflow state machine.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Legal workflow phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Plan,
    Solution,
    Implement,
    Done,
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plan => write!(f, "plan"),
            Self::Solution => write!(f, "solution"),
            Self::Implement => write!(f, "implement"),
            Self::Done => write!(f, "done"),
        }
    }
}

/// Parse error for unknown phase strings.
#[derive(Debug, PartialEq, Eq)]
pub struct UnknownPhase(pub String);

impl fmt::Display for UnknownPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown phase: {}", self.0)
    }
}

impl std::error::Error for UnknownPhase {}

impl FromStr for Phase {
    type Err = UnknownPhase;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "plan" | "spec" => Ok(Self::Plan),
            "solution" | "design" => Ok(Self::Solution),
            "implement" => Ok(Self::Implement),
            "done" => Ok(Self::Done),
            other => Err(UnknownPhase(other.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Phase;
    use std::str::FromStr;

    // --- Display ---

    #[test]
    fn plan_displays_as_plan() {
        assert_eq!(Phase::Plan.to_string(), "plan");
    }

    #[test]
    fn solution_displays_as_solution() {
        assert_eq!(Phase::Solution.to_string(), "solution");
    }

    #[test]
    fn implement_displays_as_implement() {
        assert_eq!(Phase::Implement.to_string(), "implement");
    }

    #[test]
    fn done_displays_as_done() {
        assert_eq!(Phase::Done.to_string(), "done");
    }

    // --- FromStr canonical ---

    #[test]
    fn from_str_plan() {
        assert_eq!(Phase::from_str("plan").unwrap(), Phase::Plan);
    }

    #[test]
    fn from_str_solution() {
        assert_eq!(Phase::from_str("solution").unwrap(), Phase::Solution);
    }

    #[test]
    fn from_str_implement() {
        assert_eq!(Phase::from_str("implement").unwrap(), Phase::Implement);
    }

    #[test]
    fn from_str_done() {
        assert_eq!(Phase::from_str("done").unwrap(), Phase::Done);
    }

    // --- FromStr aliases ---

    #[test]
    fn from_str_spec_alias_maps_to_plan() {
        assert_eq!(Phase::from_str("spec").unwrap(), Phase::Plan);
    }

    #[test]
    fn from_str_design_alias_maps_to_solution() {
        assert_eq!(Phase::from_str("design").unwrap(), Phase::Solution);
    }

    // --- FromStr unknown ---

    #[test]
    fn from_str_unknown_returns_err() {
        assert!(Phase::from_str("unknown").is_err());
    }

    // --- Serde ---

    #[test]
    fn serializes_plan_as_lowercase_string() {
        let json = serde_json::to_string(&Phase::Plan).unwrap();
        assert_eq!(json, r#""plan""#);
    }

    #[test]
    fn serializes_solution_as_lowercase_string() {
        let json = serde_json::to_string(&Phase::Solution).unwrap();
        assert_eq!(json, r#""solution""#);
    }

    #[test]
    fn serializes_implement_as_lowercase_string() {
        let json = serde_json::to_string(&Phase::Implement).unwrap();
        assert_eq!(json, r#""implement""#);
    }

    #[test]
    fn serializes_done_as_lowercase_string() {
        let json = serde_json::to_string(&Phase::Done).unwrap();
        assert_eq!(json, r#""done""#);
    }

    #[test]
    fn deserializes_plan_from_lowercase_string() {
        let phase: Phase = serde_json::from_str(r#""plan""#).unwrap();
        assert_eq!(phase, Phase::Plan);
    }

    #[test]
    fn deserializes_solution_from_lowercase_string() {
        let phase: Phase = serde_json::from_str(r#""solution""#).unwrap();
        assert_eq!(phase, Phase::Solution);
    }

    #[test]
    fn deserializes_implement_from_lowercase_string() {
        let phase: Phase = serde_json::from_str(r#""implement""#).unwrap();
        assert_eq!(phase, Phase::Implement);
    }

    #[test]
    fn deserializes_done_from_lowercase_string() {
        let phase: Phase = serde_json::from_str(r#""done""#).unwrap();
        assert_eq!(phase, Phase::Done);
    }

    #[test]
    fn round_trips_plan() {
        let original = Phase::Plan;
        let json = serde_json::to_string(&original).unwrap();
        let restored: Phase = serde_json::from_str(&json).unwrap();
        assert_eq!(original, restored);
    }

    // --- Copy/Clone ---

    #[test]
    fn phase_is_copy() {
        let a = Phase::Implement;
        let b = a; // copy
        assert_eq!(a, b);
    }

    // --- Idle Display ---

    #[test]
    fn idle_displays_as_idle() {
        assert_eq!(Phase::Idle.to_string(), "idle");
    }

    // --- Idle FromStr ---

    #[test]
    fn from_str_idle() {
        assert_eq!(Phase::from_str("idle").unwrap(), Phase::Idle);
    }

    // --- Idle Serde ---

    #[test]
    fn round_trips_idle() {
        let original = Phase::Idle;
        let json = serde_json::to_string(&original).unwrap();
        let restored: Phase = serde_json::from_str(&json).unwrap();
        assert_eq!(original, restored);
    }

    // --- is_gated ---

    #[test]
    fn plan_is_gated() {
        assert!(Phase::Plan.is_gated());
    }

    #[test]
    fn solution_is_gated() {
        assert!(Phase::Solution.is_gated());
    }

    #[test]
    fn idle_is_not_gated() {
        assert!(!Phase::Idle.is_gated());
    }

    #[test]
    fn implement_is_not_gated() {
        assert!(!Phase::Implement.is_gated());
    }

    #[test]
    fn done_is_not_gated() {
        assert!(!Phase::Done.is_gated());
    }
}
