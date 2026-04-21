use crate::update::platform::{Architecture, Platform};
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during the ECC update process.
#[derive(Debug, Error)]
pub enum UpdateError {
    /// Platform or architecture combination not supported.
    #[error("Unsupported platform: {platform}/{arch}")]
    UnsupportedPlatform {
        /// The detected platform.
        platform: Platform,
        /// The detected architecture.
        arch: Architecture,
    },

    /// The requested version was not found in the release repository.
    #[error("Version {version} not found")]
    VersionNotFound {
        /// The version that was not found.
        version: String,
    },

    /// The version string is invalid or unparseable.
    #[error("Invalid version: {raw}")]
    InvalidVersion {
        /// The raw version string.
        raw: String,
    },

    /// Checksum verification of the downloaded binary failed.
    #[error("Checksum verification failed")]
    ChecksumMismatch,

    /// Failed to swap the old binary with the new one.
    #[error("Binary swap failed: {reason}")]
    SwapFailed {
        /// Details about why the swap failed.
        reason: String,
    },

    /// Failed to restore the backup after an error.
    #[error("Backup restore failed: {reason}")]
    BackupRestoreFailed {
        /// Details about why the restore failed.
        reason: String,
    },

    /// Update completed partially, with some components failing.
    #[error("Partial update: {updated} updated, {failed} failed")]
    PartialUpdate {
        /// Description of what was successfully updated.
        updated: String,
        /// Description of what failed.
        failed: String,
    },

    /// Synchronizing configuration failed.
    #[error("Config sync failed: {reason}")]
    ConfigSyncFailed {
        /// Details about why sync failed.
        reason: String,
    },

    /// A network error occurred during the update.
    #[error("Network error: {reason}. Check your connection and retry.")]
    NetworkError {
        /// Details about the network error.
        reason: String,
    },

    /// GitHub API rate limit was exceeded.
    #[error("GitHub API rate limited. Resets at {reset_time}. Set GITHUB_TOKEN for higher limits.")]
    RateLimited {
        /// ISO 8601 timestamp when the rate limit resets.
        reset_time: String,
    },

    /// The download was interrupted before completion.
    #[error("Download interrupted. The original installation is untouched.")]
    DownloadInterrupted,

    /// cosign binary is not available for signature verification.
    #[error("cosign not found. Install cosign for enhanced security verification.")]
    CosignUnavailable,

    /// Permission denied when writing to a file or directory.
    #[error("Permission denied: cannot write to {path}. Reason: {reason}")]
    PermissionDenied {
        /// The path where write was denied.
        path: String,
        /// Reason for the permission denial.
        reason: String,
    },

    /// An update is already in progress, preventing concurrent updates.
    #[error("Update already in progress: {reason}")]
    UpdateLocked {
        /// Details about the existing update.
        reason: String,
    },

    /// Security verification of the update package failed.
    #[error("Security verification failed: {reason}")]
    SecurityVerificationFailed {
        /// Details about the verification failure.
        reason: String,
    },

    /// Rollback failed after an update error, leaving the system in an inconsistent state.
    #[error(
        "Rollback failed after update error. Original error: {original}. Rollback error: {rollback}. Manual cleanup required for: {backup_paths:?}"
    )]
    RollbackFailed {
        /// The original error that triggered the rollback.
        original: String,
        /// The error encountered during rollback.
        rollback: String,
        /// Backup paths that may need manual cleanup.
        backup_paths: Vec<PathBuf>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::update::platform::{Architecture, Platform};
    use std::path::PathBuf;

    #[test]
    fn error_display_messages() {
        let err = UpdateError::UnsupportedPlatform {
            platform: Platform::Unknown,
            arch: Architecture::Amd64,
        };
        let msg = err.to_string();
        assert!(msg.contains("Unknown") || msg.contains("Amd64") || msg.contains("Unsupported"));

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

    #[test]
    fn error_display_unsupported_platform_holds_enums() {
        // PC-006: UnsupportedPlatform holds Platform + Architecture enums
        let err = UpdateError::UnsupportedPlatform {
            platform: Platform::MacOS,
            arch: Architecture::Unknown,
        };
        let msg = err.to_string();
        assert!(!msg.is_empty());
        // Verify enum fields are accessible
        if let UpdateError::UnsupportedPlatform { platform, arch } = err {
            assert_eq!(platform, Platform::MacOS);
            assert_eq!(arch, Architecture::Unknown);
        }
    }

    #[test]
    fn error_display_permission_denied() {
        // PC-006: PermissionDenied variant exists
        let err = UpdateError::PermissionDenied {
            path: "/usr/local/bin/ecc".to_string(),
            reason: "read-only filesystem".to_string(),
        };
        let msg = err.to_string();
        assert!(!msg.is_empty());
        assert!(
            msg.contains("/usr/local/bin/ecc")
                || msg.contains("read-only filesystem")
                || msg.contains("Permission")
        );
    }

    #[test]
    fn error_display_update_locked() {
        // PC-006: UpdateLocked variant exists
        let err = UpdateError::UpdateLocked {
            reason: "another instance is running".to_string(),
        };
        let msg = err.to_string();
        assert!(!msg.is_empty());
    }

    #[test]
    fn error_display_security_verification_failed() {
        // PC-006: SecurityVerificationFailed variant exists
        let err = UpdateError::SecurityVerificationFailed {
            reason: "cosign signature mismatch".to_string(),
        };
        let msg = err.to_string();
        assert!(!msg.is_empty());
    }

    #[test]
    fn error_display_rollback_failed() {
        // PC-006: RollbackFailed variant exists
        let err = UpdateError::RollbackFailed {
            original: "swap failed".to_string(),
            rollback: "rename failed".to_string(),
            backup_paths: vec![PathBuf::from("/tmp/ecc.bak")],
        };
        let msg = err.to_string();
        assert!(!msg.is_empty());
    }
}
