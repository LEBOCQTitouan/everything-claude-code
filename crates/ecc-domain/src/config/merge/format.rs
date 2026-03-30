use super::MergeReport;

/// Create an empty merge report with all fields initialized to empty vectors.
pub fn empty_report() -> MergeReport {
    MergeReport {
        added: Vec::new(),
        updated: Vec::new(),
        unchanged: Vec::new(),
        skipped: Vec::new(),
        smart_merged: Vec::new(),
        errors: Vec::new(),
    }
}

/// Combine multiple merge reports into a single report by concatenating all fields.
pub fn combine_reports(reports: &[MergeReport]) -> MergeReport {
    MergeReport {
        added: reports.iter().flat_map(|r| r.added.clone()).collect(),
        updated: reports.iter().flat_map(|r| r.updated.clone()).collect(),
        unchanged: reports.iter().flat_map(|r| r.unchanged.clone()).collect(),
        skipped: reports.iter().flat_map(|r| r.skipped.clone()).collect(),
        smart_merged: reports
            .iter()
            .flat_map(|r| r.smart_merged.clone())
            .collect(),
        errors: reports.iter().flat_map(|r| r.errors.clone()).collect(),
    }
}

/// Check if two strings differ after trimming whitespace.
pub fn contents_differ(a: &str, b: &str) -> bool {
    a.trim() != b.trim()
}

/// Format a merge report as a human-readable string.
pub fn format_merge_report(label: &str, report: &MergeReport) -> String {
    let mut parts = Vec::new();

    if !report.added.is_empty() {
        parts.push(format!("{} added", report.added.len()));
    }
    if !report.updated.is_empty() {
        parts.push(format!("{} updated", report.updated.len()));
    }
    if !report.unchanged.is_empty() {
        parts.push(format!("{} unchanged", report.unchanged.len()));
    }
    if !report.skipped.is_empty() {
        parts.push(format!("{} skipped", report.skipped.len()));
    }
    if !report.smart_merged.is_empty() {
        parts.push(format!("{} smart-merged", report.smart_merged.len()));
    }
    if !report.errors.is_empty() {
        parts.push(format!("{} errors", report.errors.len()));
    }

    if parts.is_empty() {
        format!("  {label}: (no changes)")
    } else {
        format!("  {label}: {}", parts.join(", "))
    }
}
