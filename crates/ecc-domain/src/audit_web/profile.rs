//! Audit-web profile domain types with YAML serde.

use serde::{Deserialize, Serialize};

use super::dimension::AuditDimension;

const CURRENT_PROFILE_VERSION: u32 = 1;

/// Top-level profile persisted to `docs/audits/audit-web-profile.yaml`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditWebProfile {
    pub version: u32,
    pub dimensions: Vec<AuditDimension>,
    pub thresholds: DimensionThreshold,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub improvement_history: Vec<ImprovementSuggestion>,
}

/// Thresholds that control which ring a technology is placed in during
/// Phase 3 synthesis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DimensionThreshold {
    /// Minimum strategic-fit score to place in Adopt ring (default: 4).
    pub adopt_min_fit: u8,
    /// Minimum maturity score to place in Adopt ring (default: 4).
    pub adopt_min_maturity: u8,
    /// Maximum effort score still acceptable for Adopt ring (default: 2).
    pub adopt_max_effort: u8,
}

/// A single improvement suggestion persisted to the profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImprovementSuggestion {
    pub text: String,
    pub accepted: bool,
    pub date: String,
}

/// Errors from profile parse/load operations.
#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    #[error("malformed YAML: {0}")]
    MalformedYaml(String),
    #[error(
        "unsupported profile version {version}. Current version is {current}. \
         Please upgrade your ECC installation."
    )]
    UnsupportedVersion { version: u32, current: u32 },
}

/// Parse a YAML string into an `AuditWebProfile`.
///
/// Validates the schema version after deserialization.
pub fn parse_profile(yaml: &str) -> Result<AuditWebProfile, ProfileError> {
    let profile: AuditWebProfile =
        serde_saphyr::from_str(yaml).map_err(|e| ProfileError::MalformedYaml(e.to_string()))?;
    if profile.version != CURRENT_PROFILE_VERSION {
        return Err(ProfileError::UnsupportedVersion {
            version: profile.version,
            current: CURRENT_PROFILE_VERSION,
        });
    }
    Ok(profile)
}

/// Serialize an `AuditWebProfile` to a YAML string.
pub fn serialize_profile(profile: &AuditWebProfile) -> String {
    serde_saphyr::to_string(profile).expect("AuditWebProfile is always serializable")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit_web::dimension::standard_dimensions;

    fn make_profile() -> AuditWebProfile {
        AuditWebProfile {
            version: 1,
            dimensions: standard_dimensions(),
            thresholds: DimensionThreshold {
                adopt_min_fit: 4,
                adopt_min_maturity: 4,
                adopt_max_effort: 2,
            },
            improvement_history: vec![ImprovementSuggestion {
                text: "Add Kubernetes dimension".to_owned(),
                accepted: true,
                date: "2026-03-30".to_owned(),
            }],
        }
    }

    #[test]
    fn profile_construction() {
        let profile = make_profile();
        assert_eq!(profile.version, 1);
        assert_eq!(profile.dimensions.len(), 8);
        assert_eq!(profile.thresholds.adopt_min_fit, 4);
        assert_eq!(profile.thresholds.adopt_min_maturity, 4);
        assert_eq!(profile.thresholds.adopt_max_effort, 2);
        assert_eq!(profile.improvement_history.len(), 1);
        assert!(profile.improvement_history[0].accepted);
    }

    #[test]
    fn yaml_round_trip() {
        let original = make_profile();
        let yaml = serialize_profile(&original);
        let parsed = parse_profile(&yaml).expect("round-trip parse should succeed");
        assert_eq!(original, parsed);
    }

    #[test]
    fn corrupted_yaml_error() {
        let bad_yaml = "version: 1\ndimensions: [\nunterminiertes: {broken:";
        let result = parse_profile(bad_yaml);
        assert!(
            matches!(result, Err(ProfileError::MalformedYaml(_))),
            "expected MalformedYaml, got {result:?}"
        );
    }

    #[test]
    fn unknown_version_error() {
        let yaml = "version: 99\ndimensions: []\nthresholds:\n  adopt_min_fit: 4\n  adopt_min_maturity: 4\n  adopt_max_effort: 2\nimprovement_history: []\n";
        let result = parse_profile(yaml);
        match result {
            Err(ProfileError::UnsupportedVersion { version, current }) => {
                assert_eq!(version, 99);
                assert_eq!(current, CURRENT_PROFILE_VERSION);
                let msg = format!("{}", ProfileError::UnsupportedVersion { version, current });
                assert!(
                    msg.contains("upgrade"),
                    "error message should mention upgrade: {msg}"
                );
            }
            other => panic!("expected UnsupportedVersion error, got {other:?}"),
        }
    }
}
