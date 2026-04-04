//! Bus factor analysis — per-file author diversity.
//!
//! Bus factor = count of distinct authors per file.
//! Files with bus factor = 1 are flagged as single-author risk.

use std::collections::{HashMap, HashSet};

use super::error::AnalyzeError;

/// Bus factor data for a single file.
#[derive(Debug, Clone, PartialEq)]
pub struct BusFactor {
    pub path: String,
    pub unique_authors: u32,
    pub total_commits: u32,
    pub is_risk: bool,
}

impl std::fmt::Display for BusFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let risk = if self.is_risk {
            " RISK: single author"
        } else {
            ""
        };
        write!(
            f,
            "{}: {} authors, {} commits{}",
            self.path, self.unique_authors, self.total_commits, risk
        )
    }
}

/// Compute bus factor per file.
///
/// Input: list of `(file_path, author)` tuples from git log.
/// Bus factor = count of distinct authors for each file.
/// Files with bus factor = 1 are marked as risk.
/// Results sorted by unique_authors ascending (riskiest first), limited to `top_n`.
pub fn compute_bus_factor(
    file_authors: &[(String, String)],
    top_n: usize,
) -> Result<Vec<BusFactor>, AnalyzeError> {
    if top_n == 0 {
        return Err(AnalyzeError::InvalidTopN(0));
    }

    let mut authors_per_file: HashMap<&str, HashSet<&str>> = HashMap::new();
    let mut commits_per_file: HashMap<&str, u32> = HashMap::new();

    for (path, author) in file_authors {
        authors_per_file
            .entry(path.as_str())
            .or_default()
            .insert(author.as_str());
        *commits_per_file.entry(path.as_str()).or_insert(0) += 1;
    }

    let mut results: Vec<BusFactor> = authors_per_file
        .into_iter()
        .map(|(path, authors)| {
            let unique_authors = authors.len() as u32;
            let total_commits = *commits_per_file.get(path).unwrap_or(&0);
            BusFactor {
                path: path.to_string(),
                unique_authors,
                total_commits,
                is_risk: unique_authors == 1,
            }
        })
        .collect();

    // Sort by unique_authors ascending (riskiest first), then by total_commits descending
    results.sort_by(|a, b| {
        a.unique_authors
            .cmp(&b.unique_authors)
            .then(b.total_commits.cmp(&a.total_commits))
    });

    results.truncate(top_n);

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(path: &str, author: &str) -> (String, String) {
        (path.to_string(), author.to_string())
    }

    // PC-033: single author = 1
    #[test]
    fn bus_factor_single_author() {
        let data = vec![
            entry("a.rs", "alice"),
            entry("a.rs", "alice"),
            entry("a.rs", "alice"),
        ];
        let result = compute_bus_factor(&data, 10).unwrap();
        assert_eq!(result[0].unique_authors, 1);
        assert!(result[0].is_risk);
    }

    // PC-034: two equal authors = 2
    #[test]
    fn bus_factor_two_equal() {
        let data = vec![
            entry("a.rs", "alice"),
            entry("a.rs", "bob"),
            entry("a.rs", "alice"),
            entry("a.rs", "bob"),
        ];
        let result = compute_bus_factor(&data, 10).unwrap();
        assert_eq!(result[0].unique_authors, 2);
        assert!(!result[0].is_risk);
    }

    // PC-035: dominant author = still counted as 2 if second author exists
    #[test]
    fn bus_factor_dominant_author() {
        let data = vec![
            entry("a.rs", "alice"),
            entry("a.rs", "alice"),
            entry("a.rs", "alice"),
            entry("a.rs", "alice"),
            entry("a.rs", "bob"),
        ];
        let result = compute_bus_factor(&data, 10).unwrap();
        assert_eq!(result[0].unique_authors, 2);
        assert_eq!(result[0].total_commits, 5);
    }

    // PC-036: top_n limit
    #[test]
    fn bus_factor_top_n() {
        let data = vec![
            entry("a.rs", "alice"),
            entry("b.rs", "bob"),
            entry("c.rs", "carol"),
            entry("d.rs", "dave"),
            entry("e.rs", "eve"),
        ];
        let result = compute_bus_factor(&data, 3).unwrap();
        assert_eq!(result.len(), 3);
    }

    // PC-049: bus factor=1 flagged as RISK
    #[test]
    fn bus_factor_risk_flag() {
        let data = vec![entry("a.rs", "alice"), entry("a.rs", "alice")];
        let result = compute_bus_factor(&data, 10).unwrap();
        assert!(result[0].is_risk);
        let display = format!("{}", result[0]);
        assert!(display.contains("RISK"));
    }

    #[test]
    fn bus_factor_empty_input() {
        let result = compute_bus_factor(&[], 10).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn bus_factor_top_zero_error() {
        let data = vec![entry("a.rs", "alice")];
        let err = compute_bus_factor(&data, 0).unwrap_err();
        assert!(err.to_string().contains("must be > 0"));
    }

    #[test]
    fn bus_factor_sorted_riskiest_first() {
        let data = vec![
            entry("safe.rs", "alice"),
            entry("safe.rs", "bob"),
            entry("safe.rs", "carol"),
            entry("risky.rs", "alice"),
            entry("risky.rs", "alice"),
        ];
        let result = compute_bus_factor(&data, 10).unwrap();
        assert_eq!(result[0].path, "risky.rs");
        assert_eq!(result[0].unique_authors, 1);
    }
}
