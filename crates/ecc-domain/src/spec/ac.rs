//! `AcId` value object and `parse_acs` — extraction and validation of AC definitions.
//!
//! Only lines matching `^- AC-\d{3}\.\d+:` are treated as AC definitions.
//! Lines inside fenced code blocks (``` or ~~~) are ignored.

use crate::spec::error::SpecError;
use regex::Regex;
use serde::Serialize;
use std::fmt;

/// A parsed Acceptance Criterion identifier like `AC-001.2`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct AcId {
    pub us_number: u16,
    pub sub_number: u16,
}

impl AcId {
    /// Parse an AC ID string like "AC-001.2" into an `AcId`.
    ///
    /// Returns `Err(SpecError::InvalidAcId)` for malformed input.
    pub fn parse(s: &str) -> Result<AcId, SpecError> {
        let re = Regex::new(r"^AC-(\d{3})\.(\d+)$").expect("valid regex");
        let caps = re
            .captures(s)
            .ok_or_else(|| SpecError::InvalidAcId(s.to_owned()))?;
        let us: u16 = caps[1]
            .parse()
            .map_err(|_| SpecError::InvalidAcId(s.to_owned()))?;
        let sub: u16 = caps[2]
            .parse()
            .map_err(|_| SpecError::InvalidAcId(s.to_owned()))?;
        if us == 0 {
            return Err(SpecError::InvalidAcId(s.to_owned()));
        }
        if sub == 0 {
            return Err(SpecError::InvalidAcId(s.to_owned()));
        }
        Ok(AcId {
            us_number: us,
            sub_number: sub,
        })
    }
}

impl fmt::Display for AcId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AC-{:03}.{}", self.us_number, self.sub_number)
    }
}

/// A parsed Acceptance Criterion with ID and description text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AcceptanceCriterion {
    pub id: AcId,
    pub description: String,
}

/// Result of parsing ACs from a spec file — includes data and accumulated diagnostics.
#[derive(Debug)]
pub struct AcReport {
    pub acs: Vec<AcceptanceCriterion>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Extract all AC definitions from markdown content.
///
/// Rules:
/// - Only lines matching `^- AC-\d{3}\.\d+:` are AC definitions.
/// - Lines inside fenced code blocks (``` or ~~~) are ignored.
/// - Sequential numbering is validated; gaps and duplicates are reported as errors.
/// - Returns `Err(SpecError::NoAcceptanceCriteria)` if no ACs are found.
pub fn parse_acs(content: &str) -> Result<AcReport, SpecError> {
    let ac_def_re = Regex::new(r"^- (AC-\d{3}\.\d+):(.*)$").expect("valid regex");
    let malformed_re = Regex::new(r"- AC-[^:]+:").expect("valid regex");
    let fence_re = Regex::new(r"^(`{3,}|~{3,})").expect("valid regex");

    let mut acs: Vec<AcceptanceCriterion> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut in_code_block = false;

    for line in content.lines() {
        let trimmed = line.trim();
        // Toggle code block state
        if fence_re.is_match(trimmed) {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }
        if let Some(caps) = ac_def_re.captures(line) {
            let id_str = &caps[1];
            match AcId::parse(id_str) {
                Ok(id) => {
                    let description = caps[2].trim().to_owned();
                    acs.push(AcceptanceCriterion { id, description });
                }
                Err(_) => {
                    warnings.push(format!("malformed AC ID ignored: {id_str}"));
                }
            }
        } else if malformed_re.is_match(line) && line.trim_start().starts_with("- AC-") {
            // Capture lines that look like AC definitions but don't match the strict format
            // e.g., "- AC-ABC.1: ..."
            warnings.push(format!("malformed AC definition ignored: {}", line.trim()));
        }
    }

    if acs.is_empty() {
        return Err(SpecError::NoAcceptanceCriteria);
    }

    // Validate sequential numbering
    validate_ac_sequence(&acs, &mut errors);

    Ok(AcReport {
        acs,
        errors,
        warnings,
    })
}

/// Validate that ACs are sequentially numbered — no gaps, no duplicates.
fn validate_ac_sequence(acs: &[AcceptanceCriterion], errors: &mut Vec<String>) {
    // Check for duplicates
    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for (i, ac) in acs.iter().enumerate() {
        let key = ac.id.to_string();
        if let Some(prev) = seen.get(&key) {
            errors.push(format!(
                "duplicate AC ID {} at positions {} and {}",
                key,
                prev + 1,
                i + 1
            ));
        } else {
            seen.insert(key, i);
        }
    }

    // Collect unique US numbers
    let mut us_numbers: Vec<u16> = acs.iter().map(|a| a.id.us_number).collect();
    us_numbers.sort_unstable();
    us_numbers.dedup();

    // Check US number gaps
    if let (Some(&first), Some(&last)) = (us_numbers.first(), us_numbers.last()) {
        for expected in first..=last {
            if !us_numbers.contains(&expected) {
                errors.push(format!("gap in US numbering: US-{expected:03} is missing"));
            }
        }
    }

    // Check sub-number gaps per US
    let mut us_to_subs: std::collections::HashMap<u16, Vec<u16>> = std::collections::HashMap::new();
    for ac in acs {
        us_to_subs
            .entry(ac.id.us_number)
            .or_default()
            .push(ac.id.sub_number);
    }
    for (us, mut subs) in us_to_subs {
        subs.sort_unstable();
        subs.dedup();
        if let (Some(&first), Some(&last)) = (subs.first(), subs.last()) {
            for expected in first..=last {
                if !subs.contains(&expected) {
                    errors.push(format!(
                        "gap in sub-number for US-{us:03}: AC-{us:03}.{expected} is missing"
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_ac_id() {
        let id = AcId::parse("AC-001.2").unwrap();
        assert_eq!(id.us_number, 1);
        assert_eq!(id.sub_number, 2);
    }

    #[test]
    fn parse_malformed_ac_id() {
        assert!(AcId::parse("AC-ABC.1").is_err());
    }

    #[test]
    fn ac_id_zero_us() {
        assert!(AcId::parse("AC-000.1").is_err());
    }

    #[test]
    fn ac_id_zero_sub() {
        assert!(AcId::parse("AC-001.0").is_err());
    }

    #[test]
    fn ac_id_non_numeric() {
        assert!(AcId::parse("AC-ABC.1").is_err());
    }

    #[test]
    fn parse_acs_valid_sequential() {
        let content = "- AC-001.1: First criterion\n- AC-001.2: Second criterion\n- AC-002.1: Third criterion\n";
        let report = parse_acs(content).unwrap();
        assert_eq!(report.acs.len(), 3);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn parse_acs_gap_detected() {
        let content = "- AC-001.1: First\n- AC-001.3: Third (gap at 1.2)\n";
        let report = parse_acs(content).unwrap();
        assert!(!report.errors.is_empty());
        assert!(report.errors.iter().any(|e| e.contains("missing")));
    }

    #[test]
    fn parse_acs_duplicate_detected() {
        let content = "- AC-001.1: First\n- AC-001.1: Duplicate\n";
        let report = parse_acs(content).unwrap();
        assert!(!report.errors.is_empty());
        assert!(report.errors.iter().any(|e| e.contains("duplicate")));
    }

    #[test]
    fn parse_acs_empty() {
        let content = "# No ACs here\n\nJust prose.\n";
        let result = parse_acs(content);
        assert!(matches!(result, Err(SpecError::NoAcceptanceCriteria)));
    }

    #[test]
    fn parse_acs_ignores_code_blocks() {
        let content = "- AC-001.1: Real AC\n```\n- AC-002.1: Fake AC inside code block\n```\n- AC-001.2: Another real AC\n";
        let report = parse_acs(content).unwrap();
        assert_eq!(report.acs.len(), 2);
        assert!(report.acs.iter().all(|a| a.id.us_number == 1));
    }

    #[test]
    fn ignores_tilde_fenced_blocks() {
        let content = "- AC-001.1: Real AC\n~~~\n- AC-002.1: Fake AC inside tilde block\n~~~\n- AC-001.2: Another real AC\n";
        let report = parse_acs(content).unwrap();
        assert_eq!(report.acs.len(), 2);
        assert!(report.acs.iter().all(|a| a.id.us_number == 1));
    }

    #[test]
    fn parse_acs_validates_us_gaps() {
        let content = "- AC-001.1: First US\n- AC-003.1: Third US (gap at US-002)\n";
        let report = parse_acs(content).unwrap();
        assert!(!report.errors.is_empty());
        assert!(report.errors.iter().any(|e| e.contains("US-002")));
    }

    #[test]
    fn parse_acs_ignores_prose_references() {
        let content = "See AC-001.1 for details.\n- AC-001.1: Real definition\n";
        let report = parse_acs(content).unwrap();
        assert_eq!(report.acs.len(), 1);
    }
}
