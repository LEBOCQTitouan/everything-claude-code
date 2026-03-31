use crate::update::error::UpdateError;
use crate::update::platform::{Architecture, Platform};
use std::fmt;

/// Identifies the release artifact name for the current platform.
///
/// Names follow the release.yml convention: `ecc-{os}-{arch}` where
/// `os` is `darwin`, `linux`, or `win32` and `arch` is `arm64` or `x64`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactName(String);

impl ArtifactName {
    /// Resolve the artifact name for the given platform and architecture.
    ///
    /// # Errors
    ///
    /// Returns [`UpdateError::UnsupportedPlatform`] if the combination is not supported.
    pub fn resolve(platform: Platform, arch: Architecture) -> Result<Self, UpdateError> {
        let name = match (platform, arch) {
            (Platform::MacOS, Architecture::Arm64) => "ecc-darwin-arm64",
            (Platform::MacOS, Architecture::Amd64) => "ecc-darwin-x64",
            (Platform::Linux, Architecture::Amd64) => "ecc-linux-x64",
            (Platform::Linux, Architecture::Arm64) => "ecc-linux-arm64",
            (Platform::Windows, Architecture::Amd64) => "ecc-win32-x64",
            _ => {
                return Err(UpdateError::UnsupportedPlatform { platform, arch });
            }
        };
        Ok(Self(name.to_string()))
    }

    /// Return the artifact name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ArtifactName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::update::platform::{Architecture, Platform};

    #[test]
    fn resolves_macos_arm64() {
        let name = ArtifactName::resolve(Platform::MacOS, Architecture::Arm64).expect("should resolve");
        assert_eq!(name.as_str(), "ecc-darwin-arm64");
    }

    #[test]
    fn resolves_macos_x64() {
        let name = ArtifactName::resolve(Platform::MacOS, Architecture::Amd64).expect("should resolve");
        assert_eq!(name.as_str(), "ecc-darwin-x64");
    }

    #[test]
    fn resolves_linux_x64() {
        let name = ArtifactName::resolve(Platform::Linux, Architecture::Amd64).expect("should resolve");
        assert_eq!(name.as_str(), "ecc-linux-x64");
    }

    #[test]
    fn resolves_linux_arm64() {
        let name = ArtifactName::resolve(Platform::Linux, Architecture::Arm64).expect("should resolve");
        assert_eq!(name.as_str(), "ecc-linux-arm64");
    }

    #[test]
    fn resolves_windows_x64() {
        let name = ArtifactName::resolve(Platform::Windows, Architecture::Amd64).expect("should resolve");
        assert_eq!(name.as_str(), "ecc-win32-x64");
    }

    #[test]
    fn rejects_unsupported_unknown_platform() {
        let result = ArtifactName::resolve(Platform::Unknown, Architecture::Amd64);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, UpdateError::UnsupportedPlatform { .. }));
    }

    #[test]
    fn rejects_unsupported_unknown_arch() {
        let result = ArtifactName::resolve(Platform::Linux, Architecture::Unknown);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UpdateError::UnsupportedPlatform { .. }));
    }

    #[test]
    fn artifact_all_five_targets_resolve() {
        // PC-003: all 5 release targets resolve correctly
        let targets = [
            (Platform::MacOS, Architecture::Arm64, "ecc-darwin-arm64"),
            (Platform::MacOS, Architecture::Amd64, "ecc-darwin-x64"),
            (Platform::Linux, Architecture::Amd64, "ecc-linux-x64"),
            (Platform::Linux, Architecture::Arm64, "ecc-linux-arm64"),
            (Platform::Windows, Architecture::Amd64, "ecc-win32-x64"),
        ];
        for (platform, arch, expected) in &targets {
            let name = ArtifactName::resolve(*platform, *arch)
                .unwrap_or_else(|_| panic!("should resolve {expected}"));
            assert_eq!(name.as_str(), *expected);
        }
    }

    #[test]
    fn rejects_unsupported_windows_arm64() {
        // Windows Arm64 not in release matrix
        let result = ArtifactName::resolve(Platform::Windows, Architecture::Arm64);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UpdateError::UnsupportedPlatform { .. }));
    }
}
