//! Typed error enum for config module operations.

/// Errors that can occur in config module operations.
#[derive(Debug, thiserror::Error)]
pub enum ConfigAppError {
    /// Could not read a config file.
    #[error(
        "read_json_safe: cannot read '{path}' — check the file exists and is readable: {reason}"
    )]
    ReadFile { path: String, reason: String },

    /// A config file contained invalid JSON.
    #[error("read_json_safe: invalid JSON in '{path}' — ensure the file is valid JSON: {reason}")]
    InvalidJson { path: String, reason: String },

    /// Could not create a parent directory during file copy.
    #[error(
        "apply_accept: failed to create directory for '{path}' — check filesystem permissions: {reason}"
    )]
    CreateDir { path: String, reason: String },

    /// Could not copy a file.
    #[error(
        "apply_accept: failed to copy '{src}' to '{dest}' — check filesystem permissions: {reason}"
    )]
    CopyFile {
        src: String,
        dest: String,
        reason: String,
    },

    /// Settings JSON parse failed.
    #[error(
        "remove_ecc_hooks: failed to parse settings.json content — ensure the file is valid JSON: {reason}"
    )]
    ParseSettings { reason: String },

    /// Settings JSON serialization failed.
    #[error(
        "remove_ecc_hooks: failed to serialize settings.json — this is a bug, please report it: {reason}"
    )]
    SerializeSettings { reason: String },

    /// Failed to write settings.json.
    #[error(
        "remove_ecc_hooks: failed to write settings.json — check filesystem permissions: {reason}"
    )]
    WriteSettings { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_ports::fs::FileSystem;
    use std::path::Path;

    // --- Display format tests ---

    #[test]
    fn read_file_display_contains_operation_name() {
        let err = ConfigAppError::ReadFile {
            path: "settings.json".to_string(),
            reason: "not found".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("read_json_safe"),
            "Display must contain operation name, got: {msg}"
        );
        assert!(
            msg.contains("readable"),
            "Display must contain remediation hint, got: {msg}"
        );
    }

    #[test]
    fn invalid_json_display_contains_operation_name() {
        let err = ConfigAppError::InvalidJson {
            path: "settings.json".to_string(),
            reason: "unexpected eof".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("read_json_safe"),
            "Display must contain operation name, got: {msg}"
        );
        assert!(
            msg.contains("valid JSON"),
            "Display must contain remediation hint, got: {msg}"
        );
    }

    #[test]
    fn create_dir_display_contains_operation_name() {
        let err = ConfigAppError::CreateDir {
            path: "/dest".to_string(),
            reason: "permission denied".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("apply_accept"),
            "Display must contain operation name, got: {msg}"
        );
        assert!(
            msg.contains("permissions"),
            "Display must contain remediation hint, got: {msg}"
        );
    }

    #[test]
    fn copy_file_display_contains_operation_name() {
        let err = ConfigAppError::CopyFile {
            src: "/src/agent.md".to_string(),
            dest: "/dest/agent.md".to_string(),
            reason: "permission denied".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("apply_accept"),
            "Display must contain operation name, got: {msg}"
        );
        assert!(
            msg.contains("permissions"),
            "Display must contain remediation hint, got: {msg}"
        );
    }

    #[test]
    fn parse_settings_display_contains_operation_name() {
        let err = ConfigAppError::ParseSettings {
            reason: "invalid syntax".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("remove_ecc_hooks"),
            "Display must contain operation name, got: {msg}"
        );
        assert!(
            msg.contains("valid JSON"),
            "Display must contain remediation hint, got: {msg}"
        );
    }

    // --- Compile-time signature test: apply_accept must return Result<(), ConfigAppError> ---
    // This test fails in RED (apply_accept still returns Result<(), String>) and passes in GREEN.
    #[test]
    fn apply_accept_returns_config_app_error() {
        fn assert_return_type(
            _: fn(&dyn FileSystem, &Path, &Path, bool) -> Result<(), ConfigAppError>,
        ) {
        }
        assert_return_type(crate::config::merge::apply_accept);
    }
}
