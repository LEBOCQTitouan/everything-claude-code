//! Phase value object for workflow state machine.

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
}
