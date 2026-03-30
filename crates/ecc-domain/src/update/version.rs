use crate::update::error::UpdateError;
use std::fmt;

/// A validated semantic version string (e.g., "4.3.0" or "4.3.0-rc.1").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version(String);

impl Version {
    /// Parse and validate a semver string.
    ///
    /// Accepts `major.minor.patch` and `major.minor.patch-prerelease` formats.
    ///
    /// # Errors
    ///
    /// Returns [`UpdateError::InvalidVersion`] if the string is not valid semver.
    pub fn parse(s: &str) -> Result<Self, UpdateError> {
        todo!("implement Version::parse")
    }

    /// Returns true if this version is newer than `other`.
    pub fn is_newer_than(&self, other: &Version) -> bool {
        todo!("implement Version::is_newer_than")
    }

    /// Returns true if this is a pre-release version (contains `-`).
    pub fn is_prerelease(&self) -> bool {
        todo!("implement Version::is_prerelease")
    }

    /// Return the raw version string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_semver() {
        assert!(Version::parse("4.3.0").is_ok());
        assert!(Version::parse("1.0.0").is_ok());
        assert!(Version::parse("4.3.0-rc.1").is_ok());
        assert!(Version::parse("not-a-version").is_err());
        assert!(Version::parse("1.2").is_err());
        assert!(Version::parse("").is_err());
    }

    #[test]
    fn is_newer_than() {
        let v1 = Version::parse("4.3.1").unwrap();
        let v2 = Version::parse("4.3.0").unwrap();
        assert!(v1.is_newer_than(&v2));
        assert!(!v2.is_newer_than(&v1));
        assert!(!v1.is_newer_than(&v1));
    }

    #[test]
    fn detects_prerelease() {
        let v = Version::parse("4.3.0-rc.1").unwrap();
        assert!(v.is_prerelease());
        let v2 = Version::parse("4.3.0-alpha").unwrap();
        assert!(v2.is_prerelease());
    }

    #[test]
    fn stable_is_not_prerelease() {
        let v = Version::parse("4.3.0").unwrap();
        assert!(!v.is_prerelease());
    }
}
