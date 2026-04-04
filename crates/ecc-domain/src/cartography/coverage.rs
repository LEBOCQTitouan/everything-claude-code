//! Coverage calculation for the cartography bounded context.
//!
//! Computes what percentage of source files are referenced in at least
//! one journey or flow document. Pure logic — no I/O.

/// Report produced by `calculate_coverage`.
#[derive(Debug, Clone, PartialEq)]
pub struct CoverageReport {
    /// Total number of source files in the project.
    pub total: usize,
    /// Number of source files referenced in at least one journey or flow.
    pub referenced: usize,
    /// Coverage as a percentage (0.0–100.0).
    pub percentage: f64,
    /// Top-10 undocumented files by change frequency (populated only when
    /// `percentage < 50.0`). Each entry is the file path.
    pub priority_gaps: Vec<String>,
}

/// Calculate cartography coverage.
///
/// - `all_source_files`: every source file in the project
/// - `referenced_files`: files that appear in at least one journey or flow document
/// - `file_frequencies`: `(file_path, change_count)` pairs used to rank gaps;
///   files not present default to 0
///
/// When coverage is below 50 %, `priority_gaps` is populated with the top-10
/// unreferenced files sorted by descending change frequency.
pub fn calculate_coverage(
    all_source_files: &[String],
    referenced_files: &[String],
    file_frequencies: &[(String, u32)],
) -> CoverageReport {
    let total = all_source_files.len();
    let referenced = all_source_files
        .iter()
        .filter(|f| referenced_files.contains(f))
        .count();

    let percentage = if total == 0 {
        100.0
    } else {
        (referenced as f64 / total as f64) * 100.0
    };

    let priority_gaps = if percentage < 50.0 {
        let unreferenced: Vec<&String> = all_source_files
            .iter()
            .filter(|f| !referenced_files.contains(f))
            .collect();

        let mut with_freq: Vec<(&String, u32)> = unreferenced
            .iter()
            .map(|f| {
                let freq = file_frequencies
                    .iter()
                    .find(|(path, _)| path == *f)
                    .map(|(_, count)| *count)
                    .unwrap_or(0);
                (*f, freq)
            })
            .collect();

        with_freq.sort_by(|a, b| b.1.cmp(&a.1));
        with_freq
            .into_iter()
            .take(10)
            .map(|(path, _)| path.clone())
            .collect()
    } else {
        Vec::new()
    };

    CoverageReport {
        total,
        referenced,
        percentage,
        priority_gaps,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_files(paths: &[&str]) -> Vec<String> {
        paths.iter().map(|s| s.to_string()).collect()
    }

    fn make_freqs(pairs: &[(&str, u32)]) -> Vec<(String, u32)> {
        pairs.iter().map(|(s, n)| (s.to_string(), *n)).collect()
    }

    // ── basic ratio ───────────────────────────────────────────────────────────

    #[test]
    fn coverage_60_percent_with_10_files_and_6_referenced() {
        let all = make_files(&[
            "src/a.rs", "src/b.rs", "src/c.rs", "src/d.rs", "src/e.rs", "src/f.rs", "src/g.rs",
            "src/h.rs", "src/i.rs", "src/j.rs",
        ]);
        let referenced = make_files(&[
            "src/a.rs", "src/b.rs", "src/c.rs", "src/d.rs", "src/e.rs", "src/f.rs",
        ]);
        let freqs = make_freqs(&[]);
        let report = calculate_coverage(&all, &referenced, &freqs);
        assert_eq!(report.total, 10);
        assert_eq!(report.referenced, 6);
        assert!(
            (report.percentage - 60.0).abs() < 0.01,
            "percentage should be 60.0, got {}",
            report.percentage
        );
        // 60% >= 50% so priority_gaps must be empty
        assert!(
            report.priority_gaps.is_empty(),
            "no priority gaps when coverage >= 50%"
        );
    }

    #[test]
    fn zero_referenced_gives_zero_percent() {
        let all = make_files(&["src/a.rs", "src/b.rs"]);
        let referenced = make_files(&[]);
        let freqs = make_freqs(&[]);
        let report = calculate_coverage(&all, &referenced, &freqs);
        assert_eq!(report.total, 2);
        assert_eq!(report.referenced, 0);
        assert!((report.percentage - 0.0).abs() < 0.01);
    }

    #[test]
    fn all_referenced_gives_100_percent() {
        let all = make_files(&["src/a.rs", "src/b.rs"]);
        let referenced = make_files(&["src/a.rs", "src/b.rs"]);
        let freqs = make_freqs(&[]);
        let report = calculate_coverage(&all, &referenced, &freqs);
        assert!((report.percentage - 100.0).abs() < 0.01);
        assert!(report.priority_gaps.is_empty());
    }

    #[test]
    fn zero_source_files_gives_100_percent_with_no_gaps() {
        let all = make_files(&[]);
        let referenced = make_files(&[]);
        let freqs = make_freqs(&[]);
        let report = calculate_coverage(&all, &referenced, &freqs);
        assert_eq!(report.total, 0);
        assert_eq!(report.referenced, 0);
        assert!(
            (report.percentage - 100.0).abs() < 0.01,
            "empty set should be 100%"
        );
        assert!(report.priority_gaps.is_empty());
    }

    // ── priority gaps ─────────────────────────────────────────────────────────

    #[test]
    fn below_50_percent_returns_top10_gaps_sorted_by_frequency() {
        // 20 files total, 0 referenced → 0% → gaps populated
        let mut all = Vec::new();
        let mut freqs_raw = Vec::new();
        for i in 1..=20u32 {
            let path = format!("src/file{:02}.rs", i);
            all.push(path.clone());
            freqs_raw.push((path, i)); // file01 has freq 1, file20 has freq 20
        }
        let referenced = make_files(&[]);
        let report = calculate_coverage(&all, &referenced, &freqs_raw);
        assert_eq!(
            report.priority_gaps.len(),
            10,
            "should have exactly 10 gaps"
        );
        // Highest frequency files should appear first
        assert!(
            report.priority_gaps[0].contains("file20"),
            "highest-freq first, got {:?}",
            report.priority_gaps[0]
        );
        assert!(
            report.priority_gaps[1].contains("file19"),
            "second highest, got {:?}",
            report.priority_gaps[1]
        );
    }

    #[test]
    fn below_50_percent_does_not_include_already_referenced_files_in_gaps() {
        // 4 files, 1 referenced → 25%
        let all = make_files(&["src/a.rs", "src/b.rs", "src/c.rs", "src/d.rs"]);
        let referenced = make_files(&["src/a.rs"]);
        let freqs = make_freqs(&[
            ("src/a.rs", 10), // referenced — should not appear in gaps
            ("src/b.rs", 5),
            ("src/c.rs", 8),
            ("src/d.rs", 3),
        ]);
        let report = calculate_coverage(&all, &referenced, &freqs);
        assert!((report.percentage - 25.0).abs() < 0.01);
        assert!(
            !report.priority_gaps.contains(&"src/a.rs".to_string()),
            "referenced file must not be in gaps"
        );
        // Gaps should be sorted by frequency desc: c(8) > b(5) > d(3)
        assert_eq!(report.priority_gaps[0], "src/c.rs");
        assert_eq!(report.priority_gaps[1], "src/b.rs");
        assert_eq!(report.priority_gaps[2], "src/d.rs");
    }

    #[test]
    fn gaps_capped_at_10_even_when_more_unreferenced_exist() {
        // 25 files, none referenced → many gaps but report capped at 10
        let all: Vec<String> = (1..=25).map(|i| format!("src/file{:02}.rs", i)).collect();
        let referenced = make_files(&[]);
        let freqs = make_freqs(&[]);
        let report = calculate_coverage(&all, &referenced, &freqs);
        assert!(report.percentage < 50.0);
        assert_eq!(report.priority_gaps.len(), 10);
    }

    #[test]
    fn files_without_frequency_entry_default_to_zero() {
        // 4 files, 0 referenced → 0%. No frequencies provided.
        let all = make_files(&["src/a.rs", "src/b.rs", "src/c.rs", "src/d.rs"]);
        let referenced = make_files(&[]);
        let freqs = make_freqs(&[]);
        let report = calculate_coverage(&all, &referenced, &freqs);
        assert!(report.percentage < 50.0);
        assert_eq!(report.priority_gaps.len(), 4); // fewer than 10, so all shown
    }
}
