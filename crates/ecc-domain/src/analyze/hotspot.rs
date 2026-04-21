//! Hotspot analysis — file change frequency.

use std::collections::HashMap;

use super::error::AnalyzeError;

/// A file with its change frequency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hotspot {
    /// File path.
    pub path: String,
    /// Number of commits that changed this file.
    pub change_count: u32,
}

/// Compute hotspots from commit file lists.
///
/// Each entry in `commit_files` is the list of files changed in one commit.
/// Commits with more than `max_files_per_commit` files are excluded (noise filter).
/// Results are sorted by change count descending, limited to `top_n`.
pub fn compute_hotspots(
    commit_files: &[Vec<String>],
    top_n: usize,
    max_files_per_commit: usize,
) -> Result<Vec<Hotspot>, AnalyzeError> {
    if top_n == 0 {
        return Err(AnalyzeError::InvalidTopN(0));
    }

    let mut counts: HashMap<&str, u32> = HashMap::new();

    for files in commit_files {
        // Exclude commits with too many files (bulk operations)
        if files.len() > max_files_per_commit {
            continue;
        }
        for file in files {
            *counts.entry(file.as_str()).or_insert(0) += 1;
        }
    }

    let mut hotspots: Vec<Hotspot> = counts
        .into_iter()
        .map(|(path, change_count)| Hotspot {
            path: path.to_string(),
            change_count,
        })
        .collect();

    hotspots.sort_by(|a, b| b.change_count.cmp(&a.change_count));
    hotspots.truncate(top_n);

    Ok(hotspots)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files(paths: &[&str]) -> Vec<String> {
        paths.iter().map(|s| s.to_string()).collect()
    }

    // PC-020: counts correctly
    #[test]
    fn hotspot_counts() {
        let commits = vec![
            files(&["a.rs", "b.rs"]),
            files(&["a.rs", "c.rs"]),
            files(&["a.rs"]),
        ];
        let result = compute_hotspots(&commits, 10, 20).unwrap();
        assert_eq!(result[0].path, "a.rs");
        assert_eq!(result[0].change_count, 3);
    }

    // PC-021: sorted descending
    #[test]
    fn hotspots_sorted_descending() {
        let commits = vec![
            files(&["b.rs"]),
            files(&["a.rs", "b.rs"]),
            files(&["a.rs", "b.rs", "c.rs"]),
        ];
        let result = compute_hotspots(&commits, 10, 20).unwrap();
        assert!(result[0].change_count >= result[1].change_count);
    }

    // PC-022: top_n limit
    #[test]
    fn hotspots_top_n_limit() {
        let commits = vec![files(&["a.rs", "b.rs", "c.rs", "d.rs", "e.rs"])];
        let result = compute_hotspots(&commits, 3, 20).unwrap();
        assert_eq!(result.len(), 3);
    }

    // PC-023: filters large commits
    #[test]
    fn hotspots_filters_large_commits() {
        let large: Vec<String> = (0..25).map(|i| format!("file{i}.rs")).collect();
        let commits = vec![large, files(&["a.rs"])];
        let result = compute_hotspots(&commits, 10, 20).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path, "a.rs");
    }

    // PC-046: deleted files counted
    #[test]
    fn hotspots_include_deleted() {
        let commits = vec![files(&["deleted.rs", "alive.rs"]), files(&["deleted.rs"])];
        let result = compute_hotspots(&commits, 10, 20).unwrap();
        assert!(result.iter().any(|h| h.path == "deleted.rs"));
        assert_eq!(
            result
                .iter()
                .find(|h| h.path == "deleted.rs")
                .unwrap()
                .change_count,
            2
        );
    }

    // PC-043 / PC-047: top 0 returns error
    #[test]
    fn hotspots_top_zero_error() {
        let commits = vec![files(&["a.rs"])];
        let err = compute_hotspots(&commits, 0, 20).unwrap_err();
        assert!(err.to_string().contains("must be > 0"));
    }

    #[test]
    fn hotspots_empty_input() {
        let result = compute_hotspots(&[], 10, 20).unwrap();
        assert!(result.is_empty());
    }
}
