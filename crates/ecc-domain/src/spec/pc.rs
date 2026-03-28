//! `PcId` value object and `parse_pcs` — extraction of Pass Conditions from design files.
//!
//! The PC table is identified by a header containing all 6 expected column names.

use crate::spec::ac::AcId;
use crate::spec::error::SpecError;
use regex::Regex;
use serde::Serialize;
use std::fmt;

/// A parsed Pass Condition identifier like `PC-003`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, PartialOrd, Ord)]
pub struct PcId(pub u16);

impl PcId {
    /// Parse a PC ID string like "PC-003" into a `PcId`.
    pub fn parse(s: &str) -> Result<PcId, SpecError> {
        let re = Regex::new(r"^PC-(\d+)$").expect("valid regex");
        let caps = re
            .captures(s.trim())
            .ok_or_else(|| SpecError::InvalidPcId(s.to_owned()))?;
        let n: u16 = caps[1]
            .parse()
            .map_err(|_| SpecError::InvalidPcId(s.to_owned()))?;
        Ok(PcId(n))
    }

    /// Return the numeric value.
    pub fn number(&self) -> u16 {
        self.0
    }
}

impl fmt::Display for PcId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PC-{:03}", self.0)
    }
}

/// A parsed Pass Condition row from the design file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PassCondition {
    pub id: PcId,
    pub pc_type: String,
    pub description: String,
    pub verifies_acs: Vec<AcId>,
    pub command: String,
    pub expected: String,
}

/// Result of parsing PCs from a design file.
#[derive(Debug)]
pub struct PcReport {
    pub pcs: Vec<PassCondition>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Parse the Pass Conditions table from design file content.
///
/// Detects the table by header row containing all 6 column names.
/// Skips separator rows. Reports malformed rows, gaps, and duplicates.
pub fn parse_pcs(content: &str) -> Result<PcReport, SpecError> {
    let separator_re = Regex::new(r"^\s*\|?\s*[-:]+\s*\|").expect("valid regex");

    let mut pcs: Vec<PassCondition> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let warnings: Vec<String> = Vec::new();

    // Find the PC table header
    let lines: Vec<&str> = content.lines().collect();
    let header_idx = find_pc_table_header(&lines);
    if header_idx.is_none() {
        return Err(SpecError::NoPassConditions);
    }
    let start = header_idx.unwrap();

    // Process rows after header
    let mut row_num = 0usize;
    for line in &lines[start + 1..] {
        if !line.trim_start().starts_with('|') {
            // End of table
            break;
        }
        row_num += 1;
        let trimmed = line.trim();
        // Skip separator rows
        if separator_re.is_match(trimmed) {
            continue;
        }
        let cols: Vec<&str> = trimmed
            .trim_start_matches('|')
            .trim_end_matches('|')
            .split('|')
            .collect();

        if cols.len() < 6 {
            errors.push(format!(
                "row {row_num}: expected 6 columns, got {} in: {trimmed}",
                cols.len()
            ));
            continue;
        }

        let id_str = cols[0].trim();
        let pc_type = cols[1].trim().to_owned();
        let description = cols[2].trim().to_owned();
        let verifies_str = cols[3].trim();
        let command = cols[4].trim().to_owned();
        let expected = cols[5].trim().to_owned();

        // Validate required fields
        if id_str.is_empty() {
            errors.push(format!("row {row_num}: ID field is empty"));
            continue;
        }
        if pc_type.is_empty() {
            errors.push(format!("row {row_num}: Type field is empty for {id_str}"));
        }
        if description.is_empty() {
            errors.push(format!(
                "row {row_num}: Description field is empty for {id_str}"
            ));
        }

        let id = match PcId::parse(id_str) {
            Ok(id) => id,
            Err(_) => {
                errors.push(format!("row {row_num}: invalid PC ID: {id_str}"));
                continue;
            }
        };

        let verifies_acs = parse_ac_refs(verifies_str);

        pcs.push(PassCondition {
            id,
            pc_type,
            description,
            verifies_acs,
            command,
            expected,
        });
    }

    if pcs.is_empty() && errors.is_empty() {
        return Err(SpecError::NoPassConditions);
    }

    // Validate sequential IDs
    validate_pc_sequence(&pcs, &mut errors);

    Ok(PcReport {
        pcs,
        errors,
        warnings,
    })
}

/// Find the index of the PC table header line.
fn find_pc_table_header(lines: &[&str]) -> Option<usize> {
    for (i, line) in lines.iter().enumerate() {
        let lower = line.to_lowercase();
        if (lower.contains("| id") || lower.contains("|id") || lower.contains("| id |"))
            && lower.contains("type")
            && lower.contains("description")
            && lower.contains("verifies")
            && lower.contains("command")
            && lower.contains("expected")
        {
            return Some(i);
        }
    }
    None
}

/// Parse AC references from a comma/space-separated string.
fn parse_ac_refs(s: &str) -> Vec<AcId> {
    s.split([',', ' '])
        .filter_map(|part| AcId::parse(part.trim()).ok())
        .collect()
}

/// Validate that PC IDs are sequentially numbered.
fn validate_pc_sequence(pcs: &[PassCondition], errors: &mut Vec<String>) {
    let mut seen: std::collections::HashMap<u16, usize> = std::collections::HashMap::new();
    let mut ids: Vec<u16> = Vec::new();

    for (i, pc) in pcs.iter().enumerate() {
        let n = pc.id.number();
        if let Some(prev) = seen.get(&n) {
            errors.push(format!(
                "duplicate PC ID {} at positions {} and {}",
                pc.id,
                prev + 1,
                i + 1
            ));
        } else {
            seen.insert(n, i);
            ids.push(n);
        }
    }

    let mut sorted_ids = ids.clone();
    sorted_ids.sort_unstable();

    if let (Some(&first), Some(&last)) = (sorted_ids.first(), sorted_ids.last()) {
        for expected in first..=last {
            if !sorted_ids.contains(&expected) {
                errors.push(format!("gap in PC numbering: PC-{expected:03} is missing"));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pc_id_valid() {
        let id = PcId::parse("PC-003").unwrap();
        assert_eq!(id.number(), 3);
    }

    #[test]
    fn parse_pcs_valid_table() {
        let content = r#"## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | Test something | AC-001.1 | `cargo test` | PASS |
| PC-002 | unit | Test another | AC-001.2 | `cargo test` | PASS |
"#;
        let report = parse_pcs(content).unwrap();
        assert_eq!(report.pcs.len(), 2);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn parse_pcs_malformed_row() {
        let content = r#"## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | Short row |
"#;
        let report = parse_pcs(content).unwrap();
        assert!(!report.errors.is_empty());
        assert!(report.errors.iter().any(|e| e.contains("columns")));
    }

    #[test]
    fn parse_pcs_gap_detected() {
        let content = r#"## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | First | AC-001.1 | `cmd` | PASS |
| PC-003 | unit | Third (gap) | AC-001.2 | `cmd` | PASS |
"#;
        let report = parse_pcs(content).unwrap();
        assert!(!report.errors.is_empty());
        assert!(report.errors.iter().any(|e| e.contains("PC-002")));
    }

    #[test]
    fn parse_pcs_duplicate_detected() {
        let content = r#"## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | First | AC-001.1 | `cmd` | PASS |
| PC-001 | unit | Duplicate | AC-001.2 | `cmd` | PASS |
"#;
        let report = parse_pcs(content).unwrap();
        assert!(!report.errors.is_empty());
        assert!(report.errors.iter().any(|e| e.contains("duplicate")));
    }

    #[test]
    fn parse_pcs_empty_fields() {
        let content = r#"## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 |  |  | AC-001.1 | `cmd` | PASS |
"#;
        let report = parse_pcs(content).unwrap();
        assert!(!report.errors.is_empty());
    }

    #[test]
    fn parse_pcs_no_table_found() {
        let content = "# Design\n\nJust some prose.\n";
        let result = parse_pcs(content);
        assert!(matches!(result, Err(SpecError::NoPassConditions)));
    }

    #[test]
    fn parse_pcs_skips_separator() {
        let content = r#"## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | A test | AC-001.1 | `cmd` | PASS |
"#;
        let report = parse_pcs(content).unwrap();
        // Separator row is not counted as a PC
        assert_eq!(report.pcs.len(), 1);
    }
}
