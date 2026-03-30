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
        if s.is_empty() {
            return Err(UpdateError::InvalidVersion { raw: s.to_string() });
        }
        // Strip optional leading 'v'
        let stripped = s.strip_prefix('v').unwrap_or(s);
        // Split on '-' to separate core from pre-release
        let core = stripped.split('-').next().unwrap_or("");
        let parts: Vec<&str> = core.split('.').collect();
        if parts.len() != 3 || parts.iter().any(|p| p.is_empty() || p.parse::<u64>().is_err()) {
            return Err(UpdateError::InvalidVersion { raw: s.to_string() });
        }
        Ok(Self(stripped.to_string()))
    }

    /// Returns true if this version is newer than `other`.
    pub fn is_newer_than(&self, other: &Version) -> bool {
        self.numeric_parts() > other.numeric_parts()
    }

    /// Returns true if this is a pre-release version (contains `-`).
    pub fn is_prerelease(&self) -> bool {
        self.0.contains('-')
    }

    /// Extract the numeric (major, minor, patch) tuple for comparison.
    fn numeric_parts(&self) -> (u64, u64, u64) {
        let core = self.0.split('-').next().unwrap_or(&self.0);
        let mut parts = core.split('.').map(|p| p.parse::<u64>().unwrap_or(0));
        let major = parts.next().unwrap_or(0);
        let minor = parts.next().unwrap_or(0);
        let patch = parts.next().unwrap_or(0);
        (major, minor, patch)
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
