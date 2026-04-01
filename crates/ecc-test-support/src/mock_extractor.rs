use ecc_ports::extract::{ExtractError, TarballExtractor};
use std::path::{Path, PathBuf};

/// Mock tarball extractor for tests.
///
/// Simulates extraction by creating expected files in the destination directory.
pub struct MockExtractor {
    files_to_create: Vec<String>,
    should_fail: bool,
}

impl MockExtractor {
    /// Create a new mock extractor.
    pub fn new() -> Self {
        Self {
            files_to_create: vec!["bin/ecc".to_string(), "bin/ecc-workflow".to_string()],
            should_fail: false,
        }
    }

    /// Configure files to create on extraction.
    pub fn with_files(mut self, files: Vec<String>) -> Self {
        self.files_to_create = files;
        self
    }

    /// Configure the extractor to fail with CorruptArchive.
    pub fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

impl Default for MockExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl TarballExtractor for MockExtractor {
    fn extract(&self, _tarball: &Path, dest: &Path) -> Result<Vec<PathBuf>, ExtractError> {
        if self.should_fail {
            return Err(ExtractError::CorruptArchive("mock failure".to_string()));
        }
        let paths: Vec<PathBuf> = self.files_to_create.iter().map(|f| dest.join(f)).collect();
        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// PC-010: MockExtractor implements TarballExtractor and returns paths.
    #[test]
    fn mock_extractor_returns_paths() {
        let extractor = MockExtractor::new();
        let tarball = PathBuf::from("/tmp/fake.tar.gz");
        let dest = PathBuf::from("/tmp/extract");
        let paths = extractor.extract(&tarball, &dest).expect("should succeed");
        assert!(!paths.is_empty());
        assert!(paths.iter().any(|p| p.to_string_lossy().contains("ecc")));
    }

    #[test]
    fn mock_extractor_failure_returns_corrupt_archive_error() {
        let extractor = MockExtractor::new().with_failure();
        let tarball = PathBuf::from("/tmp/fake.tar.gz");
        let dest = PathBuf::from("/tmp/extract");
        let err = extractor.extract(&tarball, &dest).unwrap_err();
        assert!(matches!(err, ExtractError::CorruptArchive(_)));
    }
}
