//! Ordering validation — detect PC dependency order violations via file overlap.

use crate::spec::pc::{PassCondition, PcId, SEPARATOR_RE};
use serde::Serialize;

/// A parsed File Changes entry from the design's File Changes table.
#[derive(Clone, Debug)]
pub struct FileChange {
    /// Row number in the file changes table.
    pub number: u16,
    /// File path being changed.
    pub file: String,
    /// Type of change (Create, Modify, Delete, etc.).
    pub action: String,
    /// AC ID(s) this change is specified by (space/comma-separated).
    pub spec_ref: String,
}

/// An ordering violation: `pc` must come after `depends_on` (they share a file).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OrderingViolation {
    /// The PC that is out of order.
    pub pc: PcId,
    /// The PC that must come before `pc`.
    pub depends_on: PcId,
    /// Human-readable explanation of the violation.
    pub reason: String,
}

/// Result of an ordering check — may include warnings.
#[derive(Debug)]
pub struct OrderingResult {
    /// Detected ordering violations (empty = valid order).
    pub violations: Vec<OrderingViolation>,
    /// Non-fatal warnings (e.g., no file changes table found).
    pub warnings: Vec<String>,
}

/// Parse the File Changes table from design content.
///
/// Strategy:
/// 1. Find a section heading containing "File Changes".
/// 2. Within that section, find the first `|`-delimited table row (the header).
/// 3. If header has "File" and "Action" columns → parse rows.
/// 4. If header has `|` but NOT "File"/"Action" → unrecognized format, warn and skip.
/// 5. If no section found → warn "no File Changes table found".
pub fn parse_file_changes(content: &str) -> (Vec<FileChange>, Vec<String>) {
    let mut warnings = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    // Step 1: Find "File Changes" section heading
    let section_idx = lines.iter().position(|l| {
        let lower = l.to_lowercase();
        lower.contains("file changes") && (l.starts_with('#') || l.starts_with("## "))
    });

    let search_start = match section_idx {
        Some(i) => i + 1,
        None => {
            // No heading; try to find a table with File and Action directly
            let table_header = find_table_with_file_and_action(&lines, 0);
            match table_header {
                Some(i) => i,
                None => {
                    warnings
                        .push("no File Changes table found — ordering check skipped".to_owned());
                    return (Vec::new(), warnings);
                }
            }
        }
    };

    // Step 2: Find the first table header row after the section heading
    let table_header_idx = lines[search_start..]
        .iter()
        .position(|l| l.trim_start().starts_with('|') && !SEPARATOR_RE.is_match(l.trim()))
        .map(|rel| rel + search_start);

    let header_idx = match table_header_idx {
        Some(i) => i,
        None => {
            warnings.push("no File Changes table found — ordering check skipped".to_owned());
            return (Vec::new(), warnings);
        }
    };

    // Step 3: Check column names
    let header_line = lines[header_idx];
    let header_cols: Vec<&str> = header_line
        .trim()
        .trim_start_matches('|')
        .trim_end_matches('|')
        .split('|')
        .map(str::trim)
        .collect();

    let file_col = find_col_idx(&header_cols, "file");
    let action_col = find_col_idx(&header_cols, "action");
    let spec_ref_col = find_col_idx(&header_cols, "spec ref")
        .or_else(|| find_col_idx(&header_cols, "spec_ref"))
        .or_else(|| find_col_idx(&header_cols, "specref"));

    let (file_col, action_col) = match (file_col, action_col) {
        (Some(f), Some(a)) => (f, a),
        _ => {
            // Table found but columns are unrecognized
            warnings.push(
                "File Changes table has unrecognized format — ordering check skipped".to_owned(),
            );
            return (Vec::new(), warnings);
        }
    };

    // Step 4: Parse data rows
    let mut changes = Vec::new();
    let mut row_num: u16 = 0;

    for line in &lines[header_idx + 1..] {
        if !line.trim_start().starts_with('|') {
            break;
        }
        let trimmed = line.trim();
        if SEPARATOR_RE.is_match(trimmed) {
            continue;
        }

        let cols: Vec<&str> = trimmed
            .trim_start_matches('|')
            .trim_end_matches('|')
            .split('|')
            .map(str::trim)
            .collect();

        row_num += 1;
        let file = cols.get(file_col).copied().unwrap_or("").to_owned();
        let action = cols.get(action_col).copied().unwrap_or("").to_owned();
        let spec_ref = spec_ref_col
            .and_then(|i| cols.get(i).copied())
            .unwrap_or("")
            .to_owned();

        if !file.is_empty() {
            changes.push(FileChange {
                number: row_num,
                file,
                action,
                spec_ref,
            });
        }
    }

    (changes, warnings)
}

/// Check ordering of PCs that share files via spec_ref → verifies_acs cross-reference.
///
/// For PCs that both reference the same file, verify they appear in ascending PC-ID order
/// within the PC table.
pub fn check_ordering(pcs: &[PassCondition], file_changes: &[FileChange]) -> OrderingResult {
    if file_changes.is_empty() {
        return OrderingResult {
            violations: Vec::new(),
            warnings: vec!["no file changes — ordering check skipped".to_owned()],
        };
    }

    // Build: file → list of PcIds that reference it (via spec_ref matching verifies_acs)
    let mut file_to_pcs: std::collections::HashMap<String, Vec<PcId>> =
        std::collections::HashMap::new();

    for fc in file_changes {
        let matching_pc_ids = find_pcs_for_file_change(pcs, fc);
        if !matching_pc_ids.is_empty() {
            file_to_pcs
                .entry(fc.file.clone())
                .or_default()
                .extend(matching_pc_ids);
        }
    }

    let mut violations = Vec::new();

    for (file, mut pc_ids) in file_to_pcs {
        pc_ids.sort();
        pc_ids.dedup();

        // Get the order these PCs appear in the table
        let ordered_in_table: Vec<&PcId> = pcs
            .iter()
            .map(|p| &p.id)
            .filter(|id| pc_ids.contains(id))
            .collect();

        // Check adjacent pairs: if table order is not ascending, it's a violation
        for window in ordered_in_table.windows(2) {
            let a = window[0];
            let b = window[1];
            if a > b {
                violations.push(OrderingViolation {
                    pc: b.clone(),
                    depends_on: a.clone(),
                    reason: format!("{} must come before {} (both modify file {})", b, a, file),
                });
            }
        }
    }

    OrderingResult {
        violations,
        warnings: Vec::new(),
    }
}

fn find_table_with_file_and_action(lines: &[&str], start: usize) -> Option<usize> {
    for (i, line) in lines[start..].iter().enumerate() {
        let lower = line.to_lowercase();
        if lower.contains("file") && lower.contains("action") && line.trim_start().starts_with('|')
        {
            return Some(i + start);
        }
    }
    None
}

fn find_col_idx(headers: &[&str], name: &str) -> Option<usize> {
    headers.iter().position(|h| h.to_lowercase() == name)
}

/// Find all PC IDs that reference the same AC(s) as the given file change's spec_ref.
fn find_pcs_for_file_change(pcs: &[PassCondition], fc: &FileChange) -> Vec<PcId> {
    if fc.spec_ref.is_empty() {
        return Vec::new();
    }

    let ref_acs: Vec<crate::spec::ac::AcId> = fc
        .spec_ref
        .split([',', ' '])
        .filter_map(|part| crate::spec::ac::AcId::parse(part.trim()).ok())
        .collect();

    if ref_acs.is_empty() {
        return Vec::new();
    }

    pcs.iter()
        .filter(|pc| pc.verifies_acs.iter().any(|ac| ref_acs.contains(ac)))
        .map(|pc| pc.id.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::ac::AcId;
    use crate::spec::pc::{PassCondition, PcId};

    fn make_pc(num: u16, acs: Vec<AcId>) -> PassCondition {
        PassCondition {
            id: PcId(num),
            pc_type: "unit".into(),
            description: "test".into(),
            verifies_acs: acs,
            command: "cmd".into(),
            expected: "PASS".into(),
        }
    }

    fn make_ac(us: u16, sub: u16) -> AcId {
        AcId {
            us_number: us,
            sub_number: sub,
        }
    }

    #[test]
    fn violation_struct_fields() {
        let v = OrderingViolation {
            pc: PcId(5),
            depends_on: PcId(2),
            reason: "shares file src/lib.rs".into(),
        };
        assert_eq!(v.pc, PcId(5));
        assert_eq!(v.depends_on, PcId(2));
        assert!(!v.reason.is_empty());
    }

    #[test]
    fn correct_order_no_violations() {
        let pcs = vec![
            make_pc(2, vec![make_ac(1, 1)]),
            make_pc(5, vec![make_ac(1, 1)]),
        ];
        let file_changes = vec![FileChange {
            number: 1,
            file: "src/lib.rs".into(),
            action: "Modify".into(),
            spec_ref: "AC-001.1".into(),
        }];
        let result = check_ordering(&pcs, &file_changes);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn wrong_order_detected() {
        // PC-005 comes before PC-002 in table — violation
        let pcs = vec![
            make_pc(5, vec![make_ac(1, 1)]),
            make_pc(2, vec![make_ac(1, 1)]),
        ];
        let file_changes = vec![FileChange {
            number: 1,
            file: "src/lib.rs".into(),
            action: "Modify".into(),
            spec_ref: "AC-001.1".into(),
        }];
        let result = check_ordering(&pcs, &file_changes);
        assert!(!result.violations.is_empty());
        assert!(result.violations[0].reason.contains("src/lib.rs"));
    }

    #[test]
    fn no_file_changes_table() {
        let pcs = vec![make_pc(1, vec![make_ac(1, 1)])];
        let result = check_ordering(&pcs, &[]);
        assert!(result.violations.is_empty());
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn unrecognized_table_format_skipped() {
        let content = "## File Changes\n\n| Num | Path | Op |\n|-----|------|----|\n| 1 | src/lib.rs | create |\n";
        let (changes, warnings) = parse_file_changes(content);
        assert!(changes.is_empty());
        assert!(!warnings.is_empty());
        assert!(warnings.iter().any(|w| w.contains("unrecognized")));
    }
}
