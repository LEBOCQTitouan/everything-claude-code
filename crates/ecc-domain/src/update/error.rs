use crate::update::platform::{Architecture, Platform};
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during the ECC update process.
#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("Unsupported platform: {platform}/{arch}")]
    UnsupportedPlatform {
        platform: Platform,
        arch: Architecture,
    },

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

    #[error("Permission denied: cannot write to {path}. Reason: {reason}")]
    PermissionDenied { path: String, reason: String },

    #[error("Update already in progress: {reason}")]
    UpdateLocked { reason: String },

    #[error("Security verification failed: {reason}")]
    SecurityVerificationFailed { reason: String },

    #[error("Rollback failed after update error. Original error: {original}. Rollback error: {rollback}. Manual cleanup required for: {backup_paths:?}")]
    RollbackFailed {
        original: String,
        rollback: String,
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
        assert!(msg.contains("/usr/local/bin/ecc") || msg.contains("read-only filesystem") || msg.contains("Permission"));
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
