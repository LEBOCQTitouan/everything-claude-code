//! JSON output types for `ecc validate spec` and `ecc validate design`.

use crate::spec::ac::{AcId, AcceptanceCriterion};
use crate::spec::ordering::OrderingViolation;
use crate::spec::pc::PassCondition;
use serde::Serialize;

/// Output for `ecc validate spec`.
#[derive(Clone, Debug, Serialize)]
pub struct SpecValidationOutput {
    /// Whether the spec is valid.
    pub valid: bool,
    /// Number of acceptance criteria.
    pub ac_count: usize,
    /// List of parsed acceptance criteria.
    pub acs: Vec<AcceptanceCriterion>,
    /// Validation error messages.
    pub errors: Vec<String>,
    /// Validation warning messages.
    pub warnings: Vec<String>,
}

/// Output for `ecc validate design`.
#[derive(Clone, Debug, Serialize)]
pub struct DesignValidationOutput {
    /// Whether the design is valid.
    pub valid: bool,
    /// Number of pass conditions.
    pub pc_count: usize,
    /// List of parsed pass conditions.
    pub pcs: Vec<PassCondition>,
    /// Acceptance criteria not covered by any pass condition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uncovered_acs: Option<Vec<AcId>>,
    /// Acceptance criteria mentioned in the design but not in the spec.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phantom_acs: Option<Vec<AcId>>,
    /// Ordering violations between pass conditions.
    pub ordering_violations: Vec<OrderingViolation>,
    /// Validation error messages.
    pub errors: Vec<String>,
    /// Validation warning messages.
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::ac::AcId;
    use crate::spec::pc::PcId;

    #[test]
    fn spec_output_serializes_to_json() {
        let output = SpecValidationOutput {
            valid: true,
            ac_count: 1,
            acs: vec![AcceptanceCriterion {
                id: AcId {
                    us_number: 1,
                    sub_number: 1,
                },
                description: "test".into(),
            }],
            errors: Vec::new(),
            warnings: Vec::new(),
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"valid\":true"));
        assert!(json.contains("\"ac_count\":1"));
    }

    #[test]
    fn design_output_serializes_to_json() {
        let output = DesignValidationOutput {
            valid: true,
            pc_count: 1,
            pcs: vec![crate::spec::pc::PassCondition {
                id: PcId(1),
                pc_type: "unit".into(),
                description: "test".into(),
                verifies_acs: Vec::new(),
                command: "cmd".into(),
                expected: "PASS".into(),
            }],
            uncovered_acs: Some(Vec::new()),
            phantom_acs: Some(Vec::new()),
            ordering_violations: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"valid\":true"));
        assert!(json.contains("\"pc_count\":1"));
    }

    #[test]
    fn null_coverage_when_no_spec() {
        let output = DesignValidationOutput {
            valid: true,
            pc_count: 0,
            pcs: Vec::new(),
            uncovered_acs: None,
            phantom_acs: None,
            ordering_violations: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
        };
        let json = serde_json::to_string(&output).unwrap();
        // None fields are skipped (not serialized as null)
        assert!(!json.contains("uncovered_acs"));
    }
}
