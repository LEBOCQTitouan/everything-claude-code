//! Typed error enum for the merge module.

/// Errors that can occur in merge operations.
#[derive(Debug, thiserror::Error)]
pub enum MergeError {
    /// Could not read a file required for merging.
    #[error("merge_hooks: cannot read '{path}' — check the file exists and is readable: {reason}")]
    ReadFile { path: String, reason: String },

    /// A file contained invalid JSON.
    #[error("merge_hooks: invalid JSON in '{path}' — ensure the file is valid JSON: {reason}")]
    InvalidJson { path: String, reason: String },

    /// Could not create a destination directory.
    #[error("copy_dir: cannot create directory '{path}' — check filesystem permissions: {reason}")]
    CreateDir { path: String, reason: String },

    /// Could not read a directory for recursive copy.
    #[error("copy_dir: cannot read directory '{path}' — check the directory exists: {reason}")]
    ReadDir { path: String, reason: String },

    /// Could not copy a file.
    #[error("copy_dir: cannot copy '{path}' — check filesystem permissions: {reason}")]
    CopyFile { path: String, reason: String },

    /// User prompt was cancelled.
    #[error("prompt_file_review: user prompt cancelled — re-run the merge command to try again")]
    PromptCancelled,

    /// JSON serialization failed during hooks merge.
    #[error("merge_hooks: serialization failed — this is a bug, please report it: {reason}")]
    Serialization { reason: String },

    /// Write to settings.json failed during hooks merge.
    #[error("merge_hooks: cannot write settings.json — check filesystem permissions: {reason}")]
    WriteSettings { reason: String },

    /// settings.json is not a JSON object.
    #[error(
        "merge_hooks: settings.json is not a JSON object — manually inspect and repair the file"
    )]
    SettingsNotObject,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_ports::fs::FileSystem;
    use std::path::Path;

    // --- Display format tests ---

    #[test]
    fn read_file_display_contains_operation_name() {
        let err = MergeError::ReadFile {
            path: "hooks.json".to_string(),
            reason: "not found".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("merge_hooks"),
            "Display must contain operation name, got: {msg}"
        );
        assert!(
            msg.contains("readable"),
            "Display must contain remediation hint, got: {msg}"
        );
    }

    #[test]
    fn invalid_json_display_contains_operation_name() {
        let err = MergeError::InvalidJson {
            path: "settings.json".to_string(),
            reason: "unexpected eof".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("merge_hooks"),
            "Display must contain operation name, got: {msg}"
        );
        assert!(
            msg.contains("valid JSON"),
            "Display must contain remediation hint, got: {msg}"
        );
    }

    #[test]
    fn prompt_cancelled_display_contains_operation_name() {
        let err = MergeError::PromptCancelled;
        let msg = err.to_string();
        assert!(
            msg.contains("prompt_file_review"),
            "Display must contain operation name, got: {msg}"
        );
        assert!(
            msg.contains("re-run"),
            "Display must contain remediation hint, got: {msg}"
        );
    }

    #[test]
    fn create_dir_display_contains_operation_name() {
        let err = MergeError::CreateDir {
            path: "/dest".to_string(),
            reason: "permission denied".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("copy_dir"),
            "Display must contain operation name, got: {msg}"
        );
        assert!(
            msg.contains("permissions"),
            "Display must contain remediation hint, got: {msg}"
        );
    }

    #[test]
    fn settings_not_object_display_contains_operation_name() {
        let err = MergeError::SettingsNotObject;
        let msg = err.to_string();
        assert!(
            msg.contains("merge_hooks"),
            "Display must contain operation name, got: {msg}"
        );
        assert!(
            msg.contains("repair"),
            "Display must contain remediation hint, got: {msg}"
        );
    }

    // --- Compile-time signature test: merge_hooks must return Result<_, MergeError> ---
    // This test fails in RED (merge_hooks still returns Result<_, String>) and passes in GREEN.
    #[test]
    fn merge_hooks_returns_merge_error() {
        type MergeHooksFn =
            fn(&dyn FileSystem, &Path, &Path, bool) -> Result<(usize, usize, usize), MergeError>;
        fn assert_return_type(_: MergeHooksFn) {}
        assert_return_type(crate::merge::merge_hooks);
    }
}
