//! Drift detection domain — spec-vs-implementation comparison.
//!
//! Pure functions for computing drift between planned ACs/PCs and
//! actual implementation results. Zero I/O.

use std::collections::HashSet;
use std::sync::LazyLock;

static AC_ID_EXTRACT_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"AC-(\d{3}\.\d+)").expect("BUG: invalid AC_ID_EXTRACT_RE regex")
});

static PC_ID_EXTRACT_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"PC-(\d{3})").expect("BUG: invalid PC_ID_EXTRACT_RE regex")
});

/// Drift classification level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriftLevel {
    /// All ACs implemented, no unexpected files.
    None,
    /// 0 unimplemented ACs, <3 unexpected files.
    Low,
    /// 1-2 unimplemented ACs OR >3 unexpected files.
    Medium,
    /// 3+ unimplemented ACs.
    High,
}

impl std::fmt::Display for DriftLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DriftLevel::None => write!(f, "NONE"),
            DriftLevel::Low => write!(f, "LOW"),
            DriftLevel::Medium => write!(f, "MEDIUM"),
            DriftLevel::High => write!(f, "HIGH"),
        }
    }
}

/// Drift analysis report.
#[derive(Debug, Clone, Default)]
pub struct DriftReport {
    /// Drift severity level (None, Low, Medium, High).
    pub level: Option<DriftLevel>,
    /// ACs that have no coverage in PCs.
    pub unimplemented_acs: Vec<String>,
    /// Files in implementation that aren't in spec.
    pub unexpected_files: Vec<String>,
    /// Files in spec that aren't in implementation.
    pub missing_files: Vec<String>,
    /// Total number of ACs in spec.
    pub total_acs: usize,
    /// Number of ACs covered by PCs.
    pub covered_acs: usize,
}

/// Extract AC IDs from spec/plan content via regex.
pub fn extract_ac_ids(content: &str) -> Vec<String> {
    AC_ID_EXTRACT_RE
        .find_iter(content)
        .map(|m| m.as_str().to_string())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}

/// Extract PC IDs from design/solution content via regex.
pub fn extract_pc_ids(content: &str) -> Vec<String> {
    PC_ID_EXTRACT_RE
        .find_iter(content)
        .map(|m| m.as_str().to_string())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}

/// Extract "Verifies AC" mappings from PC table rows.
pub fn extract_pc_ac_coverage(content: &str) -> Vec<(String, Vec<String>)> {
    let mut mappings = Vec::new();
    for line in content.lines() {
        let Some(start) = line.find("PC-") else {
            continue;
        };
        let end = line[start..].find('|').map_or(line.len(), |e| start + e);
        let pc = line[start..end].trim().to_string();
        if pc.starts_with("PC-") {
            let acs: Vec<String> = extract_ac_ids(line);
            if !acs.is_empty() {
                mappings.push((pc, acs));
            }
        }
    }
    mappings
}

/// Compute drift between spec ACs and implementation coverage.
pub fn compute_drift(
    spec_acs: &[String],
    pc_ac_coverage: &[(String, Vec<String>)],
    expected_files: &[String],
    actual_files: &[String],
) -> DriftReport {
    let covered: HashSet<&str> = pc_ac_coverage
        .iter()
        .flat_map(|(_, acs)| acs.iter().map(|s| s.as_str()))
        .collect();

    let unimplemented: Vec<String> = spec_acs
        .iter()
        .filter(|ac| !covered.contains(ac.as_str()))
        .cloned()
        .collect();

    let expected_set: HashSet<&str> = expected_files.iter().map(|s| s.as_str()).collect();
    let actual_set: HashSet<&str> = actual_files.iter().map(|s| s.as_str()).collect();

    let unexpected: Vec<String> = actual_set
        .difference(&expected_set)
        .map(|s| s.to_string())
        .collect();
    let missing: Vec<String> = expected_set
        .difference(&actual_set)
        .map(|s| s.to_string())
        .collect();

    let level = classify_drift(unimplemented.len(), unexpected.len());

    DriftReport {
        level: Some(level),
        unimplemented_acs: unimplemented,
        unexpected_files: unexpected,
        missing_files: missing,
        total_acs: spec_acs.len(),
        covered_acs: covered.len(),
    }
}

/// Classify drift level from counts.
pub fn classify_drift(unimplemented_count: usize, unexpected_count: usize) -> DriftLevel {
    if unimplemented_count >= 3 {
        DriftLevel::High
    } else if unimplemented_count >= 1 || unexpected_count > 3 {
        DriftLevel::Medium
    } else if unexpected_count > 0 {
        DriftLevel::Low
    } else {
        DriftLevel::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_none() {
        assert_eq!(classify_drift(0, 0), DriftLevel::None);
    }

    #[test]
    fn classify_low() {
        assert_eq!(classify_drift(0, 2), DriftLevel::Low);
    }

    #[test]
    fn classify_medium_unimplemented() {
        assert_eq!(classify_drift(1, 0), DriftLevel::Medium);
        assert_eq!(classify_drift(2, 0), DriftLevel::Medium);
    }

    #[test]
    fn classify_medium_unexpected() {
        assert_eq!(classify_drift(0, 4), DriftLevel::Medium);
    }

    #[test]
    fn classify_high() {
        assert_eq!(classify_drift(3, 0), DriftLevel::High);
        assert_eq!(classify_drift(5, 10), DriftLevel::High);
    }

    #[test]
    fn extract_ac_ids_from_spec() {
        let content = "- AC-001.1: Given X\n- AC-001.2: When Y\n- AC-002.1: Then Z";
        let ids = extract_ac_ids(content);
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&"AC-001.1".to_string()));
    }

    #[test]
    fn extract_pc_ids_from_design() {
        let content = "| PC-001 | unit | test | AC-001.1 |\n| PC-002 | unit | test2 | AC-001.2 |";
        let ids = extract_pc_ids(content);
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn compute_drift_all_covered() {
        let acs = vec!["AC-001.1".to_string()];
        let coverage = vec![("PC-001".to_string(), vec!["AC-001.1".to_string()])];
        let report = compute_drift(&acs, &coverage, &[], &[]);
        assert_eq!(report.level, Some(DriftLevel::None));
        assert!(report.unimplemented_acs.is_empty());
    }

    #[test]
    fn compute_drift_with_unimplemented() {
        let acs = vec!["AC-001.1".to_string(), "AC-001.2".to_string()];
        let coverage = vec![("PC-001".to_string(), vec!["AC-001.1".to_string()])];
        let report = compute_drift(&acs, &coverage, &[], &[]);
        assert_eq!(report.level, Some(DriftLevel::Medium));
        assert_eq!(report.unimplemented_acs, vec!["AC-001.2"]);
    }

    #[test]
    fn drift_level_display() {
        assert_eq!(DriftLevel::None.to_string(), "NONE");
        assert_eq!(DriftLevel::High.to_string(), "HIGH");
    }
}
