use crate::update::{ArtifactName, Version};

/// Describes a planned update operation.
#[derive(Debug, Clone)]
pub struct UpdatePlan {
    pub current_version: Version,
    pub target_version: Version,
    pub artifact_name: ArtifactName,
    pub is_downgrade: bool,
    pub is_already_current: bool,
}

impl UpdatePlan {
    /// Create a new update plan, computing `is_downgrade` and `is_already_current`
    /// from the version comparison.
    pub fn new(current: &Version, target: &Version, artifact: &ArtifactName) -> Self {
        let is_already_current = current == target;
        let is_downgrade = !is_already_current && current.is_newer_than(target);
        Self {
            current_version: current.clone(),
            target_version: target.clone(),
            artifact_name: artifact.clone(),
            is_downgrade,
            is_already_current,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::update::{ArtifactName, Version};

    fn artifact() -> ArtifactName {
        ArtifactName::resolve("macos", "arm64").unwrap()
    }

    #[test]
    fn detects_downgrade() {
        let current = Version::parse("4.3.0").unwrap();
        let target = Version::parse("4.2.0").unwrap();
        let plan = UpdatePlan::new(&current, &target, &artifact());
        assert!(plan.is_downgrade);
        assert!(!plan.is_already_current);
    }

    #[test]
    fn detects_already_current() {
        let current = Version::parse("4.3.0").unwrap();
        let target = Version::parse("4.3.0").unwrap();
        let plan = UpdatePlan::new(&current, &target, &artifact());
        assert!(plan.is_already_current);
        assert!(!plan.is_downgrade);
    }

    #[test]
    fn identifies_upgrade() {
        let current = Version::parse("4.2.0").unwrap();
        let target = Version::parse("4.3.0").unwrap();
        let plan = UpdatePlan::new(&current, &target, &artifact());
        assert!(!plan.is_downgrade);
        assert!(!plan.is_already_current);
    }
}
