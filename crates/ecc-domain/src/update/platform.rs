use std::fmt;

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
}

impl fmt::Display for Architecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Amd64 => write!(f, "Amd64"),
            Self::Arm64 => write!(f, "Arm64"),
            Self::Unknown => write!(f, "Unknown"),
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

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MacOS => write!(f, "macOS"),
            Self::Linux => write!(f, "Linux"),
            Self::Windows => write!(f, "Windows"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_current_returns_a_variant() {
        // Just verify it returns a non-panicking value.
        let p = Platform::current();
        // On any supported CI host this should be MacOS, Linux, or Windows.
        assert!(matches!(
            p,
            Platform::MacOS | Platform::Linux | Platform::Windows | Platform::Unknown
        ));
    }

    #[test]
    fn architecture_current_returns_a_variant() {
        let a = Architecture::current();
        assert!(matches!(
            a,
            Architecture::Amd64 | Architecture::Arm64 | Architecture::Unknown
        ));
    }

    #[test]
    fn platform_display() {
        assert_eq!(Platform::MacOS.to_string(), "macOS");
        assert_eq!(Platform::Linux.to_string(), "Linux");
        assert_eq!(Platform::Windows.to_string(), "Windows");
        assert_eq!(Platform::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn architecture_display() {
        assert_eq!(Architecture::Amd64.to_string(), "Amd64");
        assert_eq!(Architecture::Arm64.to_string(), "Arm64");
        assert_eq!(Architecture::Unknown.to_string(), "Unknown");
    }
}
