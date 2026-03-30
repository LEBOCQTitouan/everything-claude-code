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
    /// Return the host CPU architecture.
    fn architecture(&self) -> Architecture;
}

/// CPU architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    /// x86_64 / AMD64.
    Amd64,
    /// ARM64 / AArch64.
    Arm64,
    /// Any other or undetected architecture.
    Unknown,
}

impl Architecture {
    /// Detect the current architecture at compile time.
    pub fn current() -> Self {
        if cfg!(target_arch = "x86_64") {
            Self::Amd64
        } else if cfg!(target_arch = "aarch64") {
            Self::Arm64
        } else {
            Self::Unknown
        }
    }

    /// Return the architecture label used in release artifact names.
    pub fn as_label(&self) -> &str {
        match self {
            Self::Amd64 => "x86_64",
            Self::Arm64 => "aarch64",
            Self::Unknown => "unknown",
        }
    }
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
