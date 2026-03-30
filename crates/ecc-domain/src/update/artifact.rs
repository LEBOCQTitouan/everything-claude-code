use crate::update::error::UpdateError;
use std::fmt;

/// Identifies the release artifact name for the current platform.
///
/// Names follow the release.yml convention: `ecc-{os}-{arch}` where
/// `os` is `darwin` or `linux` and `arch` is `arm64` or `x64`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactName(String);

impl ArtifactName {
    /// Resolve the artifact name for the given platform and architecture.
    ///
    /// # Errors
    ///
    /// Returns [`UpdateError::UnsupportedPlatform`] if the combination is not supported.
    pub fn resolve(platform: &str, arch: &str) -> Result<Self, UpdateError> {
        let name = match (platform, arch) {
            ("macos", "arm64") => "ecc-darwin-arm64",
            ("macos", "x86_64") => "ecc-darwin-x64",
            ("linux", "x86_64") => "ecc-linux-x64",
            ("linux", "aarch64") => "ecc-linux-arm64",
            _ => {
                return Err(UpdateError::UnsupportedPlatform {
                    platform: platform.to_string(),
                    arch: arch.to_string(),
                })
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

    #[test]
    fn resolves_macos_arm64() {
        let name = ArtifactName::resolve("macos", "arm64").expect("should resolve");
        assert_eq!(name.as_str(), "ecc-darwin-arm64");
    }

    #[test]
    fn resolves_macos_x64() {
        let name = ArtifactName::resolve("macos", "x86_64").expect("should resolve");
        assert_eq!(name.as_str(), "ecc-darwin-x64");
    }

    #[test]
    fn resolves_linux_x64() {
        let name = ArtifactName::resolve("linux", "x86_64").expect("should resolve");
        assert_eq!(name.as_str(), "ecc-linux-x64");
    }

    #[test]
    fn resolves_linux_arm64() {
        let name = ArtifactName::resolve("linux", "aarch64").expect("should resolve");
        assert_eq!(name.as_str(), "ecc-linux-arm64");
    }

    #[test]
    fn rejects_unsupported_platform() {
        let result = ArtifactName::resolve("freebsd", "x86_64");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, UpdateError::UnsupportedPlatform { .. }));
    }

    #[test]
    fn matches_release_naming_convention() {
        // Verifies the ecc-{os}-{arch} naming convention from release.yml
        let darwin_arm = ArtifactName::resolve("macos", "arm64").unwrap();
        assert!(darwin_arm.as_str().starts_with("ecc-darwin-"));

        let linux_x64 = ArtifactName::resolve("linux", "x86_64").unwrap();
        assert!(linux_x64.as_str().starts_with("ecc-linux-"));

        // arch must be x64 or arm64 (not x86_64 or aarch64 verbatim)
        assert!(darwin_arm.as_str().ends_with("arm64") || darwin_arm.as_str().ends_with("x64"));
        assert!(linux_x64.as_str().ends_with("arm64") || linux_x64.as_str().ends_with("x64"));
    }

    #[test]
    fn rejects_windows_platform() {
        let result = ArtifactName::resolve("windows", "x86_64");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UpdateError::UnsupportedPlatform { .. }));
    }

    #[test]
    fn rejects_unknown_architecture() {
        let result = ArtifactName::resolve("linux", "mips");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UpdateError::UnsupportedPlatform { .. }));
    }
}
