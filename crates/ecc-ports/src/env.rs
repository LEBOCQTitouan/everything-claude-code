use std::path::PathBuf;

/// Port for environment access (env vars, home dir, platform info).
pub trait Environment: Send + Sync {
    fn var(&self, name: &str) -> Option<String>;
    fn home_dir(&self) -> Option<PathBuf>;
    fn current_dir(&self) -> Option<PathBuf>;
    fn temp_dir(&self) -> PathBuf;
    fn platform(&self) -> Platform;
}

/// Host operating system platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    MacOS,
    Linux,
    Windows,
    Unknown,
}

impl Platform {
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
