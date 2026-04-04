//! Low-level I/O primitives for campaign.md files.

use std::path::Path;

/// Atomically write `content` to `path` using tempfile + rename.
pub fn atomic_write(path: &Path, content: &str) -> Result<(), anyhow::Error> {
    let parent = path.parent().ok_or_else(|| {
        anyhow::anyhow!("Campaign path has no parent: {}", path.display())
    })?;
    let _guard = ecc_flock::acquire(parent, "campaign")
        .map_err(|e| anyhow::anyhow!("Failed to acquire campaign lock at {}: {e}", parent.display()))?;
    let tmp_path = path.with_extension("md.tmp");
    std::fs::write(&tmp_path, content)
        .map_err(|e| anyhow::anyhow!("Failed to write tempfile at {}: {e}", tmp_path.display()))?;
    if let Err(e) = std::fs::rename(&tmp_path, path) {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(anyhow::anyhow!("Failed to rename tempfile to {}: {e}", path.display()));
    }
    Ok(())
}

/// Escape a string for Markdown table cell: `|` -> `\|`, `\n` -> `<br>`.
pub fn escape_table_cell(s: &str) -> String {
    s.replace('|', r"\|").replace('\n', "<br>")
}

/// Find next row number in decisions table.
pub fn next_decision_number(content: &str) -> u32 {
    let mut max = 0u32;
    if !content.contains("## Grill-Me Decisions") {
        return 1;
    }
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('|') {
            let parts: Vec<&str> = trimmed.split('|').collect();
            if parts.len() >= 3 {
                    max = parts[1].trim().parse::<u32>().unwrap_or(0).max(max);
                }
        }
    }
    max + 1
}

/// A parsed grill-me decision entry.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Decision {
    pub n: u32,
    pub question: String,
    pub answer: String,
    pub source: String,
}

/// Parse the `## Grill-Me Decisions` table into structured entries.
pub fn parse_decisions(content: &str) -> Vec<Decision> {
    let mut decisions = Vec::new();
    let mut in_section = false;
    let mut past_separator = false;
    for line in content.lines() {
        if line.contains("## Grill-Me Decisions") {
            in_section = true;
            past_separator = false;
            continue;
        }
        if in_section && line.starts_with("## ") { break; }
        if !in_section { continue; }
        let trimmed = line.trim();
        if trimmed.is_empty() || !trimmed.starts_with('|') { continue; }
        if trimmed.contains("---") { past_separator = true; continue; }
        if !past_separator { continue; }
        let parts: Vec<&str> = trimmed.split('|').collect();
        if parts.len() >= 5 {
            let n = parts[1].trim().parse::<u32>().unwrap_or(0);
            if n > 0 {
                decisions.push(Decision {
                    n,
                    question: parts[2].trim().to_string(),
                    answer: parts[3].trim().to_string(),
                    source: parts[4].trim().to_string(),
                });
            }
        }
    }
    decisions
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn atomic_write_uses_tempfile() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("campaign.md");
        atomic_write(&path, "# Test").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "# Test");
        assert!(!tmp.path().join("campaign.md.tmp").exists());
    }

    #[test]
    fn cleanup_on_failure() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("campaign.md");
        atomic_write(&path, "content").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "content");
    }

    #[test]
    fn escape_pipes_and_newlines() {
        assert_eq!(escape_table_cell("a|b"), r"a\|b");
        assert_eq!(escape_table_cell("line1\nline2"), "line1<br>line2");
    }

    #[test]
    fn next_decision_number_empty_table() {
        let c = "## Grill-Me Decisions\n\n| # | Q | A | S |\n|---|---|---|---|\n";
        assert_eq!(next_decision_number(c), 1);
    }

    #[test]
    fn next_decision_number_with_rows() {
        let c = "## Grill-Me Decisions\n\n| # | Q | A | S |\n|---|---|---|---|\n| 1 | Q1 | A1 | r |\n| 2 | Q2 | A2 | u |\n";
        assert_eq!(next_decision_number(c), 3);
    }

    #[test]
    fn parse_decisions_basic() {
        let c = "## Grill-Me Decisions\n\n| # | Q | A | S |\n|---|---|---|---|\n| 1 | Q1 | A1 | recommended |\n| 2 | Q2 | A2 | user |\n";
        let d = parse_decisions(c);
        assert_eq!(d.len(), 2);
        assert_eq!(d[0].question, "Q1");
        assert_eq!(d[1].source, "user");
    }
}
