//! Typed error enum for the Claw REPL module.

/// Errors that can occur in Claw REPL operations.
#[derive(Debug, thiserror::Error)]
pub enum ClawError {
    /// No home directory could be determined from the environment.
    #[error("run_repl: cannot determine home directory — ensure $HOME is set")]
    NoHomeDir,

    /// A session storage operation failed.
    #[error("save_session: failed to save session '{name}' — check filesystem permissions: {reason}")]
    SaveSession { name: String, reason: String },

    /// An invalid session name was provided.
    #[error(
        "save_session: invalid session name '{name}' — use alphanumeric characters and hyphens only"
    )]
    InvalidSessionName { name: String },

    /// The session directory could not be created.
    #[error(
        "save_session: failed to create sessions directory — check filesystem permissions: {reason}"
    )]
    CreateSessionDir { reason: String },

    /// Branch operation failed.
    #[error(
        "branch_session: failed to branch from '{source_name}' to '{target_name}' — check filesystem permissions: {reason}"
    )]
    BranchSession {
        source_name: String,
        target_name: String,
        reason: String,
    },

    /// Clear session file removal failed.
    #[error(
        "clear_session: failed to clear session '{name}' — check filesystem permissions: {reason}"
    )]
    ClearSession { name: String, reason: String },

    /// Skill not found in skills directory.
    #[error("load_skill: skill '{name}' not found — run `ecc install` to install skills, or create ~/.claude/skills/{name}/SKILL.md")]
    SkillNotFound { name: String },

    /// Claude subprocess failed to start or returned non-zero.
    #[error("run_claude: claude subprocess failed — ensure `claude` is on PATH and you are authenticated: {message}")]
    ClaudeSubprocess { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claw::{ClawConfig, ClawPorts};

    // --- Display format tests ---

    #[test]
    fn no_home_dir_display_contains_operation_name() {
        let err = ClawError::NoHomeDir;
        let msg = err.to_string();
        assert!(
            msg.contains("run_repl"),
            "Display must contain operation name, got: {msg}"
        );
        assert!(
            msg.contains("$HOME"),
            "Display must contain remediation hint, got: {msg}"
        );
    }

    #[test]
    fn save_session_display_contains_operation_name() {
        let err = ClawError::SaveSession {
            name: "default".to_string(),
            reason: "permission denied".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("save_session"),
            "Display must contain operation name, got: {msg}"
        );
        assert!(
            msg.contains("permissions"),
            "Display must contain remediation hint, got: {msg}"
        );
    }

    #[test]
    fn invalid_session_name_display_contains_hint() {
        let err = ClawError::InvalidSessionName {
            name: "bad/name".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("save_session"),
            "Display must contain operation name, got: {msg}"
        );
        assert!(
            msg.contains("alphanumeric"),
            "Display must contain remediation hint, got: {msg}"
        );
    }

    #[test]
    fn skill_not_found_display_contains_operation_name() {
        let err = ClawError::SkillNotFound {
            name: "tdd".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("load_skill"),
            "Display must contain operation name, got: {msg}"
        );
        assert!(
            msg.contains("ecc install"),
            "Display must contain remediation hint, got: {msg}"
        );
    }

    #[test]
    fn claude_subprocess_display_contains_operation_name() {
        let err = ClawError::ClaudeSubprocess {
            message: "rate limited".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("run_claude"),
            "Display must contain operation name, got: {msg}"
        );
        assert!(
            msg.contains("PATH"),
            "Display must contain remediation hint, got: {msg}"
        );
    }

    // --- Compile-time signature test: run_repl must return Result<(), ClawError> ---
    // This test fails in RED (run_repl still returns anyhow::Result) and passes in GREEN.
    #[test]
    fn run_repl_returns_claw_error() {
        fn assert_return_type(_: fn(&ClawConfig, &ClawPorts<'_>) -> Result<(), ClawError>) {}
        assert_return_type(crate::claw::run_repl);
    }
}
