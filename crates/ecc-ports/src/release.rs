use std::path::{Path, PathBuf};

/// PC-045 compile-check: verifies download_file exists on ReleaseClient trait.
#[doc(hidden)]
#[allow(dead_code)]
fn _pc045_download_file_check(client: &dyn ReleaseClient, dest: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    client.download_file("https://example.com/file", dest)
}

/// Result of a version query.
#[derive(Debug, Clone)]
pub struct ReleaseInfo {
    /// Semver version string (e.g., "4.3.0").
    pub version: String,
    /// Release notes / changelog body.
    pub release_notes: String,
}

/// Checksum verification outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecksumResult {
    /// Checksum matches.
    Match,
    /// Checksum does not match.
    Mismatch,
}

/// Cosign verification outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CosignResult {
    /// Signature verified successfully.
    Verified,
    /// Cosign is not installed on the system.
    NotInstalled,
    /// Signature verification failed.
    Failed,
}

/// Port for querying and downloading release artifacts.
///
/// Production adapter: `GithubReleaseClient` in `ecc-infra`.
/// Test double: `MockReleaseClient` in `ecc-test-support`.
pub trait ReleaseClient: Send + Sync {
    /// Query the latest stable (non-prerelease) release version.
    ///
    /// If `include_prerelease` is true, prereleases are included.
    ///
    /// Returns `Err` on network failure, rate limiting, or no releases found.
    fn latest_version(
        &self,
        include_prerelease: bool,
    ) -> Result<ReleaseInfo, Box<dyn std::error::Error + Send + Sync>>;

    /// Query a specific release version.
    ///
    /// Returns `Err` if the version does not exist.
    fn get_version(
        &self,
        version: &str,
    ) -> Result<ReleaseInfo, Box<dyn std::error::Error + Send + Sync>>;

    /// Download the release tarball for the given artifact name to `dest`.
    ///
    /// Calls `on_progress(bytes_downloaded, total_bytes)` during download.
    fn download_tarball(
        &self,
        version: &str,
        artifact_name: &str,
        dest: &Path,
        on_progress: &dyn Fn(u64, u64),
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Verify the SHA256 checksum of a downloaded file against the release checksum.
    fn verify_checksum(
        &self,
        version: &str,
        artifact_name: &str,
        file_path: &Path,
    ) -> Result<ChecksumResult, Box<dyn std::error::Error + Send + Sync>>;

    /// Verify the cosign signature of a downloaded file.
    fn verify_cosign(
        &self,
        version: &str,
        artifact_name: &str,
        file_path: &Path,
    ) -> Result<CosignResult, Box<dyn std::error::Error + Send + Sync>>;
}
