use thiserror::Error;

/// Errors that can occur during the ECC update process.
#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("Unsupported platform: {platform}/{arch}")]
    UnsupportedPlatform { platform: String, arch: String },

    #[error("Version {version} not found")]
    VersionNotFound { version: String },

    #[error("Invalid version: {raw}")]
    InvalidVersion { raw: String },

    #[error("Checksum verification failed")]
    ChecksumMismatch,

    #[error("Binary swap failed: {reason}")]
    SwapFailed { reason: String },

    #[error("Backup restore failed: {reason}")]
    BackupRestoreFailed { reason: String },

    #[error("Partial update: {updated} updated, {failed} failed")]
    PartialUpdate { updated: String, failed: String },

    #[error("Config sync failed: {reason}")]
    ConfigSyncFailed { reason: String },

    #[error("Network error: {reason}. Check your connection and retry.")]
    NetworkError { reason: String },

    #[error("GitHub API rate limited. Resets at {reset_time}. Set GITHUB_TOKEN for higher limits.")]
    RateLimited { reset_time: String },

    #[error("Download interrupted. The original installation is untouched.")]
    DownloadInterrupted,

    #[error("cosign not found. Install cosign for enhanced security verification.")]
    CosignUnavailable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_messages() {
        let err = UpdateError::UnsupportedPlatform {
            platform: "freebsd".to_string(),
            arch: "x86_64".to_string(),
        };
        assert_eq!(err.to_string(), "Unsupported platform: freebsd/x86_64");

        let err = UpdateError::VersionNotFound {
            version: "99.0.0".to_string(),
        };
        assert_eq!(err.to_string(), "Version 99.0.0 not found");
    }

    #[test]
    fn rate_limited_display() {
        let err = UpdateError::RateLimited {
            reset_time: "2024-01-01T00:00:00Z".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "GitHub API rate limited. Resets at 2024-01-01T00:00:00Z. Set GITHUB_TOKEN for higher limits."
        );
    }
}
