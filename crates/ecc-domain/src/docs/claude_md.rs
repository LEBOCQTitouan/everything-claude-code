//! CLAUDE.md numeric claim extraction and validation.

use serde::Serialize;
use std::sync::LazyLock;

static TEMP_MARKER_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?i)TEMPORARY\s*\(BL-0*(\d{1,6})\)")
        .expect("BUG: invalid TEMP_MARKER_RE regex")
});

/// A `TEMPORARY (BL-NNN)` marker extracted from CLAUDE.md.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TemporaryMarker {
    pub backlog_id: u32,
    pub line_number: usize,
    pub raw_text: String,
}

/// Extract `TEMPORARY (BL-NNN)` markers from CLAUDE.md content.
///
/// Skips fenced code blocks. Returns markers in line-number order.
pub fn extract_temporary_markers(content: &str) -> Vec<TemporaryMarker> {
    let mut markers = Vec::new();
    let mut in_code_block = false;
    for (idx, line) in content.lines().enumerate() {
        let line_number = idx + 1;
        if line.trim().starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }
        for cap in TEMP_MARKER_RE.captures_iter(line) {
            if let Ok(id) = cap[1].parse::<u32>() {
                markers.push(TemporaryMarker {
                    backlog_id: id,
                    line_number,
                    raw_text: cap[0].to_string(),
                });
            }
        }
    }
    markers
}

static CLAIM_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"\b(\d+)\s+(tests?|crates?)\b").expect("BUG: invalid CLAIM_RE regex")
});

/// A numeric claim extracted from CLAUDE.md.
#[derive(Debug, Clone, Serialize)]
pub struct CountClaim {
    /// The extracted claim text (e.g., "997 tests").
    pub text: String,
    /// The claimed numeric value.
    pub claimed: u64,
    /// The actual measured value (if validated).
    pub actual: Option<u64>,
    /// True if claimed matches actual.
    pub matches: bool,
}

/// Extract numeric claims like "997 tests" or "9 crates" from CLAUDE.md content.
pub fn extract_claims(content: &str) -> Vec<CountClaim> {
    let mut claims = Vec::new();

    // Skip code blocks
    let mut in_code_block = false;
    for line in content.lines() {
        if line.trim().starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }

        for cap in CLAIM_RE.captures_iter(line) {
            let num: u64 = cap[1].parse().unwrap_or(0);
            let _label = &cap[2];
            let text = cap[0].to_string();
            // Avoid duplicates
            if !claims.iter().any(|c: &CountClaim| c.text == text) {
                claims.push(CountClaim {
                    text,
                    claimed: num,
                    actual: None,
                    matches: false,
                });
            }
        }
    }
    claims
}

/// Compare claimed counts against actual counts.
pub fn compare_claims(claims: &mut [CountClaim], actuals: &[(String, u64)]) {
    for claim in claims.iter_mut() {
        for (label, actual) in actuals {
            if claim.text.contains(label) {
                claim.actual = Some(*actual);
                claim.matches = claim.claimed == *actual;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_test_count() {
        let content = "cargo test  # Run all Rust tests (997 tests)";
        let claims = extract_claims(content);
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].claimed, 997);
    }

    #[test]
    fn extract_crate_count() {
        let content = "Hexagonal architecture: 9 crates";
        let claims = extract_claims(content);
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].claimed, 9);
    }

    #[test]
    fn skip_code_blocks() {
        let content = "Real: 10 tests\n```\nFake: 999 tests\n```";
        let claims = extract_claims(content);
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].claimed, 10);
    }

    #[test]
    fn compare_mismatch() {
        let mut claims = vec![CountClaim {
            text: "997 tests".to_string(),
            claimed: 997,
            actual: None,
            matches: false,
        }];
        compare_claims(&mut claims, &[("tests".to_string(), 1050)]);
        assert_eq!(claims[0].actual, Some(1050));
        assert!(!claims[0].matches);
    }

    #[test]
    fn compare_match() {
        let mut claims = vec![CountClaim {
            text: "9 crates".to_string(),
            claimed: 9,
            actual: None,
            matches: false,
        }];
        compare_claims(&mut claims, &[("crates".to_string(), 9)]);
        assert!(claims[0].matches);
    }

    // ── TemporaryMarker tests ────────────────────────────────────────────

    #[test]
    fn extract_temporary_marker_canonical() {
        let content = "TEMPORARY (BL-150)";
        let markers = extract_temporary_markers(content);
        assert_eq!(markers.len(), 1);
        assert_eq!(
            markers[0],
            TemporaryMarker {
                backlog_id: 150,
                line_number: 1,
                raw_text: "TEMPORARY (BL-150)".to_string(),
            }
        );
    }

    #[test]
    fn extract_temporary_marker_variants() {
        // lowercase
        let m = extract_temporary_markers("temporary (bl-150)");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].backlog_id, 150);

        // mixed case
        let m = extract_temporary_markers("Temporary (Bl-150)");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].backlog_id, 150);

        // extra space + leading zero
        let m = extract_temporary_markers("TEMPORARY  (BL-0150)");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].backlog_id, 150);

        // 6-digit id
        let m = extract_temporary_markers("TEMPORARY (BL-100000)");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].backlog_id, 100_000);

        // no space before paren
        let m = extract_temporary_markers("TEMPORARY(BL-150)");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].backlog_id, 150);

        // inside fenced code block → skipped
        let fenced = "```\nTEMPORARY (BL-999)\n```";
        let m = extract_temporary_markers(fenced);
        assert_eq!(m.len(), 0);
    }

    #[test]
    fn extract_temporary_marker_negative() {
        for input in &[
            "this is a temporary issue",
            "TEMPORARY: note",
            "TEMPORARY (BL-)",
            "TEMPORARY (BL-ABC)",
            "TEMPORARY (FOO-150)",
        ] {
            let m = extract_temporary_markers(input);
            assert_eq!(m.len(), 0, "expected 0 markers for {:?}", input);
        }
    }

    #[test]
    fn extract_temporary_marker_duplicates() {
        let content = "line1: TEMPORARY (BL-150)\nline2: TEMPORARY (BL-150)";
        let markers = extract_temporary_markers(content);
        assert_eq!(markers.len(), 2);
        assert_eq!(markers[0].backlog_id, 150);
        assert_eq!(markers[0].line_number, 1);
        assert_eq!(markers[1].backlog_id, 150);
        assert_eq!(markers[1].line_number, 2);
    }

    #[test]
    fn extract_temporary_marker_order() {
        let content =
            "prose\nTEMPORARY (BL-1)\nmore prose\nTEMPORARY (BL-3)\neven more\nTEMPORARY (BL-5)";
        let markers = extract_temporary_markers(content);
        assert_eq!(markers.len(), 3);
        let line_numbers: Vec<usize> = markers.iter().map(|m| m.line_number).collect();
        assert_eq!(line_numbers, vec![2, 4, 6]);
    }
}
