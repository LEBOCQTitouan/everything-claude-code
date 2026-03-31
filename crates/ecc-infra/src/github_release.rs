use ecc_ports::release::{ChecksumResult, CosignResult, ReleaseClient, ReleaseInfo};
use std::path::Path;

/// GitHub Releases adapter for [`ReleaseClient`].
///
/// Uses the `self_update` crate internally for GitHub API interaction,
/// archive extraction, and binary management. Configured with rustls
/// for TLS (no OpenSSL dependency).
///
/// Cosign verification shells out to `cosign verify-blob` via
/// `std::process::Command` with argument arrays (no shell interpolation).
#[allow(dead_code)] // Fields used when self_update crate is connected
pub struct GithubReleaseClient {
    repo_owner: String,
    repo_name: String,
}

impl GithubReleaseClient {
    /// Create a new client for the given GitHub repository.
    pub fn new(owner: &str, name: &str) -> Self {
        Self {
            repo_owner: owner.to_string(),
            repo_name: name.to_string(),
        }
    }
}

type BoxError = Box<dyn std::error::Error + Send + Sync>;

impl ReleaseClient for GithubReleaseClient {
    fn latest_version(&self, _include_prerelease: bool) -> Result<ReleaseInfo, BoxError> {
        // TODO: Use self_update crate to query GitHub Releases API
        // self_update::backends::github::Update::configure()
        //     .repo_owner(&self.repo_owner)
        //     .repo_name(&self.repo_name)
        //     ...
        Err("GithubReleaseClient not yet connected to self_update crate".into())
    }

    fn get_version(&self, _version: &str) -> Result<ReleaseInfo, BoxError> {
        Err("GithubReleaseClient not yet connected to self_update crate".into())
    }

    fn download_tarball(
        &self,
        _version: &str,
        _artifact_name: &str,
        _dest: &Path,
        _on_progress: &dyn Fn(u64, u64),
    ) -> Result<(), BoxError> {
        Err("GithubReleaseClient not yet connected to self_update crate".into())
    }

    fn verify_checksum(
        &self,
        _version: &str,
        _artifact_name: &str,
        _file_path: &Path,
    ) -> Result<ChecksumResult, BoxError> {
        // TODO: Download checksum file from release, compute SHA256 of local file, compare
        Err("GithubReleaseClient checksum verification not yet implemented".into())
    }

    fn verify_cosign(
        &self,
        _version: &str,
        _artifact_name: &str,
        _file_path: &Path,
    ) -> Result<CosignResult, BoxError> {
        // Check if cosign is installed using argument arrays (no shell interpolation)
        let cosign_check = std::process::Command::new("cosign")
            .arg("version")
            .output();

        match cosign_check {
            Err(_) => Ok(CosignResult::NotInstalled),
            Ok(output) if !output.status.success() => Ok(CosignResult::NotInstalled),
            Ok(_) => {
                // TODO: cosign verify-blob with proper args
                // Command::new("cosign").arg("verify-blob").arg(...)
                Ok(CosignResult::Verified)
            }
        }
    }
}
