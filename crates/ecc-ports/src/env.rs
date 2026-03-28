use std::path::PathBuf;

/// Port for environment access (env vars, home dir, platform info).
pub trait Environment: Send + Sync {
    /// Return the value of an environment variable, or `None` if unset.
    fn var(&self, name: &str) -> Option<String>;
    /// Return the current user's home directory, or `None` if unavailable.
    fn home_dir(&self) -> Option<PathBuf>;
    /// Return the current working directory, or `None` if unavailable.
    fn current_dir(&self) -> Option<PathBuf>;
    /// Return the system's temporary directory.
    fn temp_dir(&self) -> PathBuf;
    /// Return the host operating system platform.
    fn platform(&self) -> Platform;
}

/// Host operating system platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// Apple macOS.
    MacOS,
    /// Linux (any distribution).
    Linux,
    /// Microsoft Windows.
    Windows,
    /// Any other or undetected platform.
    Unknown,
}

impl Platform {
    /// Detect the current platform at compile time.
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::MacOS
        } else if cfg!(target_os = "linux") {
            Self::Linux
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Unknown
        }
    }
}
