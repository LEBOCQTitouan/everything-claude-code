//! CLAUDE.md numeric claim extraction and validation.

use serde::Serialize;
use std::sync::LazyLock;

static CLAIM_RE: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"\b(\d+)\s+(tests?|crates?)\b").expect("BUG: invalid CLAIM_RE regex"));

/// A numeric claim extracted from CLAUDE.md.
#[derive(Debug, Clone, Serialize)]
pub struct CountClaim {
    pub text: String,
    pub claimed: u64,
    pub actual: Option<u64>,
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
}
