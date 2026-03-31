use std::fmt;

/// Summary of a completed update operation.
#[derive(Debug, Clone)]
pub struct UpdateSummary {
    pub old_version: String,
    pub new_version: String,
    pub release_notes: String,
    pub files_synced: usize,
}

impl fmt::Display for UpdateSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Updated: v{} -> v{}", self.old_version, self.new_version)?;
        if !self.release_notes.is_empty() {
            writeln!(f, "\nChangelog:\n{}", self.release_notes)?;
        }
        write!(f, "Files synced: {}", self.files_synced)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_format() {
        let summary = UpdateSummary {
            old_version: "4.2.0".to_string(),
            new_version: "4.3.0".to_string(),
            release_notes: "- Added ecc update command".to_string(),
            files_synced: 42,
        };
        let output = summary.to_string();
        assert!(output.contains("v4.2.0 -> v4.3.0"));
        assert!(output.contains("Added ecc update command"));
        assert!(output.contains("Files synced: 42"));
    }
}
