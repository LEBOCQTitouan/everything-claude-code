use std::path::Path;

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

    /// Download a file from `url` to `dest`.
    ///
    /// Used to download checksum files and cosign bundle files.
    fn download_file(
        &self,
        url: &str,
        dest: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Download the cosign bundle file for the given artifact to `dest`.
    ///
    /// The bundle file is `{artifact_name}.tar.gz.bundle` in the release assets.
    fn download_cosign_bundle(
        &self,
        version: &str,
        artifact_name: &str,
        dest: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Verify the SHA256 checksum of a downloaded file against the release checksum.
    fn verify_checksum(
        &self,
        version: &str,
        artifact_name: &str,
        file_path: &Path,
    ) -> Result<ChecksumResult, Box<dyn std::error::Error + Send + Sync>>;

    /// Verify the cosign signature of a downloaded file using a pre-downloaded bundle.
    fn verify_cosign(
        &self,
        version: &str,
        artifact_name: &str,
        file_path: &Path,
        bundle_path: &Path,
    ) -> Result<CosignResult, Box<dyn std::error::Error + Send + Sync>>;
}
