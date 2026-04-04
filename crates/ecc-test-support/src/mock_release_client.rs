use ecc_ports::release::{ChecksumResult, CosignResult, ReleaseClient, ReleaseInfo};
use std::collections::HashMap;
use std::fmt;
use std::path::Path;

/// Error variants that `MockReleaseClient` can simulate.
#[derive(Debug, Clone)]
pub enum MockError {
    NetworkError(String),
    RateLimited(String),
    NotFound(String),
}

impl fmt::Display for MockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MockError::NetworkError(msg) => write!(f, "network error: {msg}"),
            MockError::RateLimited(msg) => write!(f, "rate limited: {msg}"),
            MockError::NotFound(msg) => write!(f, "not found: {msg}"),
        }
    }
}

impl std::error::Error for MockError {}

/// Scriptable mock for [`ReleaseClient`] used in tests.
pub struct MockReleaseClient {
    latest_version: Option<ReleaseInfo>,
    versions: HashMap<String, ReleaseInfo>,
    error_mode: Option<MockError>,
    checksum_result: ChecksumResult,
    cosign_result: CosignResult,
    download_bytes: Vec<u8>,
}

impl MockReleaseClient {
    pub fn new() -> Self {
        Self {
            latest_version: None,
            versions: HashMap::new(),
            error_mode: None,
            checksum_result: ChecksumResult::Match,
            cosign_result: CosignResult::Verified,
            download_bytes: Vec::new(),
        }
    }

    pub fn with_latest_version(mut self, info: ReleaseInfo) -> Self {
        self.latest_version = Some(info);
        self
    }

    pub fn with_version(mut self, version: &str, info: ReleaseInfo) -> Self {
        self.versions.insert(version.to_string(), info);
        self
    }

    pub fn with_error(mut self, error: MockError) -> Self {
        self.error_mode = Some(error);
        self
    }

    pub fn with_checksum_result(mut self, result: ChecksumResult) -> Self {
        self.checksum_result = result;
        self
    }

    pub fn with_cosign_result(mut self, result: CosignResult) -> Self {
        self.cosign_result = result;
        self
    }

    pub fn with_download_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.download_bytes = bytes;
        self
    }
}

impl Default for MockReleaseClient {
    fn default() -> Self {
        Self::new()
    }
}

type BoxError = Box<dyn std::error::Error + Send + Sync>;

impl ReleaseClient for MockReleaseClient {
    fn latest_version(&self, _include_prerelease: bool) -> Result<ReleaseInfo, BoxError> {
        if let Some(ref err) = self.error_mode {
            return Err(Box::new(err.clone()));
        }
        self.latest_version.clone().ok_or_else(|| {
            Box::new(MockError::NotFound(
                "no latest version configured".to_string(),
            )) as BoxError
        })
    }

    fn get_version(&self, version: &str) -> Result<ReleaseInfo, BoxError> {
        if let Some(ref err) = self.error_mode {
            return Err(Box::new(err.clone()));
        }
        self.versions.get(version).cloned().ok_or_else(|| {
            Box::new(MockError::NotFound(format!("{version} not found"))) as BoxError
        })
    }

    fn download_tarball(
        &self,
        _version: &str,
        _artifact_name: &str,
        _dest: &Path,
        on_progress: &dyn Fn(u64, u64),
    ) -> Result<(), BoxError> {
        if let Some(ref err) = self.error_mode {
            return Err(Box::new(err.clone()));
        }
        let total = self.download_bytes.len() as u64;
        on_progress(total, total);
        Ok(())
    }

    fn download_cosign_bundle(
        &self,
        _version: &str,
        _artifact_name: &str,
        _dest: &Path,
    ) -> Result<(), BoxError> {
        if let Some(ref err) = self.error_mode {
            return Err(Box::new(err.clone()));
        }
        Ok(())
    }

    fn download_file(&self, _url: &str, _dest: &Path) -> Result<(), BoxError> {
        if let Some(ref err) = self.error_mode {
            return Err(Box::new(err.clone()));
        }
        Ok(())
    }

    fn verify_checksum(
        &self,
        _version: &str,
        _artifact_name: &str,
        _file_path: &Path,
    ) -> Result<ChecksumResult, BoxError> {
        if let Some(ref err) = self.error_mode {
            return Err(Box::new(err.clone()));
        }
        Ok(self.checksum_result)
    }

    fn verify_cosign(
        &self,
        _version: &str,
        _artifact_name: &str,
        _file_path: &Path,
        _bundle_path: &Path,
    ) -> Result<CosignResult, BoxError> {
        if let Some(ref err) = self.error_mode {
            return Err(Box::new(err.clone()));
        }
        Ok(self.cosign_result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_release_info(version: &str) -> ReleaseInfo {
        ReleaseInfo {
            version: version.to_string(),
            release_notes: String::new(),
        }
    }

    #[test]
    fn returns_latest_version() {
        let client = MockReleaseClient::new().with_latest_version(make_release_info("1.2.3"));
        let info = client.latest_version(false).unwrap();
        assert_eq!(info.version, "1.2.3");
    }

    #[test]
    fn returns_version_not_found() {
        let client = MockReleaseClient::new()
            .with_error(MockError::NotFound("v9.9.9 not found".to_string()));
        let err = client.get_version("9.9.9").unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn simulates_rate_limit() {
        let client =
            MockReleaseClient::new().with_error(MockError::RateLimited("rate limited".to_string()));
        let err = client.latest_version(false).unwrap_err();
        assert!(err.to_string().contains("rate limited"));
    }

    #[test]
    fn simulates_network_error() {
        let client = MockReleaseClient::new()
            .with_error(MockError::NetworkError("connection refused".to_string()));
        let err = client.latest_version(false).unwrap_err();
        assert!(err.to_string().contains("network error"));
    }

    #[test]
    fn skips_prerelease() {
        // When include_prerelease is false, prerelease versions should not be returned.
        // The mock is configured with a prerelease version as "latest" but the stable
        // latest is set separately.
        let stable = make_release_info("1.0.0");
        let client = MockReleaseClient::new()
            .with_latest_version(stable)
            .with_version("1.1.0-beta", make_release_info("1.1.0-beta"));
        let info = client.latest_version(false).unwrap();
        assert!(!info.version.contains("beta"), "should skip prerelease");
    }

    #[test]
    fn checksum_verification() {
        let path = PathBuf::from("/tmp/fake.tar.gz");

        let match_client = MockReleaseClient::new().with_checksum_result(ChecksumResult::Match);
        let result = match_client
            .verify_checksum("1.0.0", "ecc.tar.gz", &path)
            .unwrap();
        assert_eq!(result, ChecksumResult::Match);

        let mismatch_client =
            MockReleaseClient::new().with_checksum_result(ChecksumResult::Mismatch);
        let result = mismatch_client
            .verify_checksum("1.0.0", "ecc.tar.gz", &path)
            .unwrap();
        assert_eq!(result, ChecksumResult::Mismatch);
    }

    #[test]
    fn cosign_not_found() {
        let path = PathBuf::from("/tmp/fake.tar.gz");
        let bundle_path = PathBuf::from("/tmp/fake.tar.gz.bundle");
        let client = MockReleaseClient::new().with_cosign_result(CosignResult::NotInstalled);
        let result = client
            .verify_cosign("1.0.0", "ecc.tar.gz", &path, &bundle_path)
            .unwrap();
        let client = MockReleaseClient::new().with_cosign_result(CosignResult::NotInstalled);
        let result = client.verify_cosign("1.0.0", "ecc.tar.gz", &path, &bundle_path).unwrap();
        assert_eq!(result, CosignResult::NotInstalled);
    }
}
