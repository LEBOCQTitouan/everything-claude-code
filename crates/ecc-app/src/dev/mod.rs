//! Dev mode use cases — toggle ECC config on/off in `~/.claude/`.
//!
//! Composes existing [`install`] and [`config::clean`] building blocks
//! to provide a quick profile switch between "ECC-enhanced" and "vanilla Claude".

mod format;
mod status;
mod switch;
mod toggle;

pub use format::{count_ecc_hooks_in_settings, format_status};
pub use status::{DevProfileStatus, DevStatus, detect_profile, dev_status};
pub use switch::dev_switch;
pub use toggle::{DevOffResult, dev_off, dev_on};

/// Errors returned by `dev_switch`.
#[derive(Debug, thiserror::Error)]
pub enum DevError {
    #[error("path must be absolute: {0}")]
    RelativePath(std::path::PathBuf),

    #[error("target directory does not exist: {0}")]
    TargetNotFound(std::path::PathBuf),

    #[error("target path escapes ECC root: {0}")]
    PathEscape(std::path::PathBuf),

    #[error("filesystem error: {0}")]
    Fs(#[from] ecc_ports::fs::FsError),
}
