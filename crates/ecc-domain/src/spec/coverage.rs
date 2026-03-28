//! Coverage analysis — cross-reference ACs against PCs' `verifies_acs` fields.

use crate::spec::ac::{AcId, AcceptanceCriterion};
use crate::spec::pc::PassCondition;
use serde::Serialize;
use std::collections::HashSet;

/// Coverage cross-reference report.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoverageReport {
    /// ACs that appear in the spec but are not covered by any PC.
    pub uncovered_acs: Vec<AcId>,
    /// AcIds referenced in PCs that do not exist in the spec.
    pub phantom_acs: Vec<AcId>,
}

/// Cross-reference ACs and PCs to find coverage gaps and phantom references.
///
/// - `uncovered_acs`: ACs not referenced in any PC's `verifies_acs`.
/// - `phantom_acs`: AcIds in PCs not present in the AC list.
pub fn check_coverage(acs: &[AcceptanceCriterion], pcs: &[PassCondition]) -> CoverageReport {
    let ac_set: HashSet<&AcId> = acs.iter().map(|a| &a.id).collect();

    // AcIds referenced by any PC
    let mut referenced: HashSet<&AcId> = HashSet::new();
    let mut phantom_set: HashSet<AcId> = HashSet::new();

    for pc in pcs {
        for ac_ref in &pc.verifies_acs {
            referenced.insert(ac_ref);
            if !ac_set.contains(ac_ref) {
                phantom_set.insert(ac_ref.clone());
            }
        }
    }

    let mut uncovered_acs: Vec<AcId> = acs
        .iter()
        .filter(|a| !referenced.contains(&a.id))
        .map(|a| a.id.clone())
        .collect();
    uncovered_acs.sort_by_key(|a| (a.us_number, a.sub_number));

    let mut phantom_acs: Vec<AcId> = phantom_set.into_iter().collect();
    phantom_acs.sort_by_key(|a| (a.us_number, a.sub_number));

    CoverageReport {
        uncovered_acs,
        phantom_acs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::ac::AcceptanceCriterion;
    use crate::spec::pc::{PassCondition, PcId};

    fn ac(us: u16, sub: u16) -> AcceptanceCriterion {
        AcceptanceCriterion {
            id: AcId {
                us_number: us,
                sub_number: sub,
            },
            description: String::new(),
        }
    }

    fn pc(num: u16, acs: Vec<AcId>) -> PassCondition {
        PassCondition {
            id: PcId(num),
            pc_type: "unit".into(),
            description: "test".into(),
            verifies_acs: acs,
            command: "cmd".into(),
            expected: "PASS".into(),
        }
    }

    #[test]
    fn all_acs_covered() {
        let acs = vec![ac(1, 1), ac(1, 2)];
        let pcs = vec![pc(
            1,
            vec![
                AcId {
                    us_number: 1,
                    sub_number: 1,
                },
                AcId {
                    us_number: 1,
                    sub_number: 2,
                },
            ],
        )];
        let report = check_coverage(&acs, &pcs);
        assert!(report.uncovered_acs.is_empty());
    }

    #[test]
    fn uncovered_ac_reported() {
        let acs = vec![ac(1, 1), ac(1, 2)];
        let pcs = vec![pc(
            1,
            vec![AcId {
                us_number: 1,
                sub_number: 1,
            }],
        )];
        let report = check_coverage(&acs, &pcs);
        assert_eq!(report.uncovered_acs.len(), 1);
        assert_eq!(
            report.uncovered_acs[0],
            AcId {
                us_number: 1,
                sub_number: 2
            }
        );
    }

    #[test]
    fn phantom_ac_detected() {
        let acs = vec![ac(1, 1)];
        let pcs = vec![pc(
            1,
            vec![
                AcId {
                    us_number: 1,
                    sub_number: 1,
                },
                AcId {
                    us_number: 4,
                    sub_number: 1,
                },
            ],
        )];
        let report = check_coverage(&acs, &pcs);
        assert!(report.phantom_acs.iter().any(|a| a.us_number == 4));
    }

    #[test]
    fn multiple_pcs_same_ac() {
        let acs = vec![ac(1, 1)];
        let pcs = vec![
            pc(
                1,
                vec![AcId {
                    us_number: 1,
                    sub_number: 1,
                }],
            ),
            pc(
                2,
                vec![AcId {
                    us_number: 1,
                    sub_number: 1,
                }],
            ),
        ];
        let report = check_coverage(&acs, &pcs);
        assert!(report.uncovered_acs.is_empty());
        assert!(report.phantom_acs.is_empty());
    }

    #[test]
    fn coverage_report_all_covered() {
        let acs = vec![ac(1, 1)];
        let pcs = vec![pc(
            1,
            vec![AcId {
                us_number: 1,
                sub_number: 1,
            }],
        )];
        let report = check_coverage(&acs, &pcs);
        assert!(report.uncovered_acs.is_empty());
        assert!(report.phantom_acs.is_empty());
    }
}
