//! Co-change coupling analysis.
//!
//! Identifies file pairs that frequently change together.
//! Formula: `commits_together / max(commits_A, commits_B)`

use std::collections::{HashMap, HashSet};

/// A pair of files with their coupling score.
#[derive(Debug, Clone, PartialEq)]
pub struct CouplingPair {
    pub file_a: String,
    pub file_b: String,
    pub coupling_ratio: f64,
    pub commits_together: u32,
}

/// Compute co-change coupling from commit file groups.
///
/// - `threshold`: minimum coupling ratio to include (0.0 = all, 1.0 = perfect coupling)
/// - `min_commits`: minimum individual appearances for a file to be considered
/// - `max_files_per_commit`: commits exceeding this file count are excluded
///
/// Boundary: commits with exactly `max_files_per_commit` files ARE included;
/// only commits strictly exceeding the threshold are excluded.
pub fn compute_coupling(
    commit_files: &[Vec<String>],
    threshold: f64,
    min_commits: u32,
    max_files_per_commit: usize,
) -> Vec<CouplingPair> {
    let mut file_commits: HashMap<&str, u32> = HashMap::new();
    let mut pair_commits: HashMap<(&str, &str), u32> = HashMap::new();

    for files in commit_files {
        // Exclude commits strictly exceeding the cap
        if files.len() > max_files_per_commit {
            continue;
        }

        // Deduplicate files within a commit
        let unique: HashSet<&str> = files.iter().map(String::as_str).collect();
        let sorted: Vec<&str> = {
            let mut v: Vec<&str> = unique.into_iter().collect();
            v.sort();
            v
        };

        // Count individual file appearances
        for &file in &sorted {
            *file_commits.entry(file).or_insert(0) += 1;
        }

        // Count pair co-occurrences
        for i in 0..sorted.len() {
            for j in (i + 1)..sorted.len() {
                let pair = (sorted[i], sorted[j]);
                *pair_commits.entry(pair).or_insert(0) += 1;
            }
        }
    }

    let mut pairs: Vec<CouplingPair> = pair_commits
        .into_iter()
        .filter_map(|((a, b), together)| {
            let count_a = *file_commits.get(a)?;
            let count_b = *file_commits.get(b)?;

            // Filter by min_commits
            if count_a < min_commits || count_b < min_commits {
                return None;
            }

            let max_count = count_a.max(count_b);
            if max_count == 0 {
                return None;
            }

            let ratio = f64::from(together) / f64::from(max_count);

            if ratio >= threshold {
                Some(CouplingPair {
                    file_a: a.to_string(),
                    file_b: b.to_string(),
                    coupling_ratio: ratio,
                    commits_together: together,
                })
            } else {
                None
            }
        })
        .collect();

    pairs.sort_by(|a, b| {
        b.coupling_ratio
            .partial_cmp(&a.coupling_ratio)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    pairs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files(paths: &[&str]) -> Vec<String> {
        paths.iter().map(|s| s.to_string()).collect()
    }

    // PC-025: basic pair
    #[test]
    fn coupling_basic_pair() {
        let commits = vec![files(&["a.rs", "b.rs"]), files(&["a.rs", "b.rs"])];
        let result = compute_coupling(&commits, 0.5, 1, 20);
        assert_eq!(result.len(), 1);
        assert!((result[0].coupling_ratio - 1.0).abs() < f64::EPSILON);
    }

    // PC-026: formula correct — commits_together / max(A, B)
    #[test]
    fn coupling_formula() {
        // A appears in 10 commits, B in 8, together in 8
        let mut commits: Vec<Vec<String>> = Vec::new();
        for _ in 0..8 {
            commits.push(files(&["a.rs", "b.rs"]));
        }
        for _ in 0..2 {
            commits.push(files(&["a.rs"]));
        }
        let result = compute_coupling(&commits, 0.0, 1, 20);
        let pair = result
            .iter()
            .find(|p| {
                (p.file_a == "a.rs" && p.file_b == "b.rs")
                    || (p.file_a == "b.rs" && p.file_b == "a.rs")
            })
            .unwrap();
        assert!((pair.coupling_ratio - 0.8).abs() < f64::EPSILON);
        assert_eq!(pair.commits_together, 8);
    }

    // PC-027: threshold filter
    #[test]
    fn coupling_threshold_filter() {
        let commits = vec![
            files(&["a.rs", "b.rs"]),
            files(&["a.rs"]),
            files(&["a.rs"]),
        ];
        // a=3, b=1, together=1, ratio = 1/3 ≈ 0.33 — below 0.7
        let result = compute_coupling(&commits, 0.7, 1, 20);
        assert!(result.is_empty());
    }

    // PC-028: min_commits filter
    #[test]
    fn coupling_min_commits_filter() {
        let commits = vec![files(&["a.rs", "b.rs"]), files(&["a.rs", "b.rs"])];
        // Both files have 2 commits; min_commits=3 should filter them out
        let result = compute_coupling(&commits, 0.0, 3, 20);
        assert!(result.is_empty());
    }

    // PC-029: filters large commits
    #[test]
    fn coupling_filters_large_commits() {
        let large: Vec<String> = (0..25).map(|i| format!("file{i}.rs")).collect();
        let commits = vec![large, files(&["a.rs", "b.rs"])];
        let result = compute_coupling(&commits, 0.0, 1, 20);
        // Only the small commit counts
        assert_eq!(result.len(), 1);
    }

    // PC-030: sorted descending
    #[test]
    fn coupling_sorted_descending() {
        let commits = vec![
            files(&["a.rs", "b.rs"]),
            files(&["a.rs", "b.rs"]),
            files(&["c.rs", "d.rs"]),
            files(&["a.rs"]),
        ];
        let result = compute_coupling(&commits, 0.0, 1, 20);
        if result.len() >= 2 {
            assert!(result[0].coupling_ratio >= result[1].coupling_ratio);
        }
    }

    // PC-031: no self-pairs
    #[test]
    fn coupling_no_self_pairs() {
        let commits = vec![files(&["a.rs"]), files(&["a.rs"])];
        let result = compute_coupling(&commits, 0.0, 1, 20);
        for pair in &result {
            assert_ne!(pair.file_a, pair.file_b);
        }
    }

    // PC-048: exactly max_files included
    #[test]
    fn coupling_exactly_max_files() {
        let exact: Vec<String> = (0..20).map(|i| format!("file{i}.rs")).collect();
        let commits = vec![exact.clone(), exact];
        let result = compute_coupling(&commits, 0.0, 1, 20);
        assert!(!result.is_empty(), "Commits with exactly max_files should be included");
    }

    // PC-051: threshold=0.0 includes all
    #[test]
    fn coupling_threshold_zero() {
        let commits = vec![
            files(&["a.rs", "b.rs"]),
            files(&["a.rs"]),
            files(&["a.rs"]),
            files(&["a.rs"]),
        ];
        // ratio = 1/4 = 0.25, threshold=0.0 should still include
        let result = compute_coupling(&commits, 0.0, 1, 20);
        assert!(!result.is_empty());
    }

    // PC-052: exactly min_commits boundary
    #[test]
    fn coupling_exactly_min_commits() {
        let commits = vec![
            files(&["a.rs", "b.rs"]),
            files(&["a.rs", "b.rs"]),
            files(&["a.rs", "b.rs"]),
        ];
        // Both files have 3 commits, min_commits=3 — should be included
        let result = compute_coupling(&commits, 0.0, 3, 20);
        assert!(!result.is_empty(), "Files with exactly min_commits should be included");
    }

    #[test]
    fn coupling_empty_input() {
        let result = compute_coupling(&[], 0.7, 3, 20);
        assert!(result.is_empty());
    }
}
