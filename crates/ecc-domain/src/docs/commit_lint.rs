//! Commit lint — detect multi-concern staged changes.

use serde::Serialize;

/// Verdict for commit lint analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ConcernVerdict {
    /// Commit is atomic (single concern).
    Pass,
    /// Commit spans multiple concerns (should be split).
    Warn,
}

/// Result of commit lint analysis.
#[derive(Debug, Clone, Serialize)]
pub struct LintResult {
    /// List of detected multi-concern issues.
    pub concerns: Vec<String>,
    /// Pass/Warn verdict based on concerns.
    pub verdict: ConcernVerdict,
}

/// Detect multi-concern changes in staged file paths.
pub fn detect_concerns(files: &[String]) -> LintResult {
    if files.is_empty() {
        return LintResult {
            concerns: vec![],
            verdict: ConcernVerdict::Pass,
        };
    }

    let mut concerns = Vec::new();

    // Check top-level directory span
    let top_dirs: std::collections::HashSet<&str> = files
        .iter()
        .filter_map(|f| f.split('/').next())
        .filter(|d| *d != "Cargo.lock") // Cargo.lock is expected co-change
        .collect();

    if top_dirs.len() > 1 {
        concerns.push(format!(
            "Files span {} top-level directories: {}",
            top_dirs.len(),
            top_dirs.into_iter().collect::<Vec<_>>().join(", ")
        ));
    }

    // Check src + docs mix
    let has_src = files
        .iter()
        .any(|f| f.starts_with("crates/") || f.starts_with("src/"));
    let has_docs = files
        .iter()
        .any(|f| f.starts_with("docs/") || f.starts_with("agents/") || f.starts_with("commands/"));
    if has_src && has_docs {
        concerns.push("Mixed source code and documentation/agent files".to_string());
    }

    let verdict = if concerns.is_empty() {
        ConcernVerdict::Pass
    } else {
        ConcernVerdict::Warn
    };

    LintResult { concerns, verdict }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_concern_passes() {
        let files = vec!["crates/ecc-domain/src/lib.rs".to_string()];
        let result = detect_concerns(&files);
        assert_eq!(result.verdict, ConcernVerdict::Pass);
    }

    #[test]
    fn multi_dir_warns() {
        let files = vec![
            "crates/ecc-domain/src/lib.rs".to_string(),
            "agents/drift-checker.md".to_string(),
        ];
        let result = detect_concerns(&files);
        assert_eq!(result.verdict, ConcernVerdict::Warn);
    }

    #[test]
    fn src_docs_mix_warns() {
        let files = vec![
            "crates/ecc-app/src/lib.rs".to_string(),
            "docs/README.md".to_string(),
        ];
        let result = detect_concerns(&files);
        assert_eq!(result.verdict, ConcernVerdict::Warn);
        assert!(result.concerns.iter().any(|c| c.contains("Mixed")));
    }

    #[test]
    fn empty_files_passes() {
        let result = detect_concerns(&[]);
        assert_eq!(result.verdict, ConcernVerdict::Pass);
    }
}
