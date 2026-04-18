//! Cartography value types with serde derives.
//!
//! No I/O — pure value objects used by the cartography bounded context.

use serde::{Deserialize, Serialize};

/// Detected project type based on build file indicators.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
    Rust,
    Javascript,
    Typescript,
    Python,
    Go,
    Java,
    Unknown,
}

/// A single file changed in a session, with its classification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangedFile {
    /// Relative path from project root.
    pub path: String,
    /// Classification: crate name for Rust, package path for JS/TS, top-level directory otherwise.
    pub classification: String,
}

/// A session-scoped delta written by the stop:cartography hook.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionDelta {
    pub session_id: String,
    /// Unix timestamp (seconds since epoch).
    pub timestamp: u64,
    pub changed_files: Vec<ChangedFile>,
    pub project_type: ProjectType,
}

/// Metadata embedded in generated cartography documents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CartographyMeta {
    /// Date of last update in YYYY-MM-DD format.
    pub last_updated: String,
    /// File paths that contributed to this cartography entry.
    pub sources: Vec<String>,
    pub session_id: String,
}

impl From<&crate::detection::framework::ProjectType> for ProjectType {
    fn from(detected: &crate::detection::framework::ProjectType) -> Self {
        match detected.primary.as_str() {
            "rust" => Self::Rust,
            "javascript" => Self::Javascript,
            "typescript" => Self::Typescript,
            "python" => Self::Python,
            "go" => Self::Go,
            "java" => Self::Java,
            _ => Self::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_delta_json_round_trip() {
        let delta = SessionDelta {
            session_id: "sess-001".to_string(),
            timestamp: 1_700_000_000,
            changed_files: vec![
                ChangedFile {
                    path: "crates/ecc-domain/src/lib.rs".to_string(),
                    classification: "ecc-domain".to_string(),
                },
                ChangedFile {
                    path: "src/index.ts".to_string(),
                    classification: "src".to_string(),
                },
            ],
            project_type: ProjectType::Rust,
        };

        let json = serde_json::to_string(&delta).expect("serialize failed");
        let round_tripped: SessionDelta = serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(delta, round_tripped);
    }

    #[test]
    fn project_type_serializes_to_lowercase() {
        assert_eq!(
            serde_json::to_string(&ProjectType::Rust).unwrap(),
            "\"rust\""
        );
        assert_eq!(
            serde_json::to_string(&ProjectType::Javascript).unwrap(),
            "\"javascript\""
        );
        assert_eq!(
            serde_json::to_string(&ProjectType::Typescript).unwrap(),
            "\"typescript\""
        );
        assert_eq!(
            serde_json::to_string(&ProjectType::Unknown).unwrap(),
            "\"unknown\""
        );
    }

    #[test]
    fn project_type_deserializes_all_variants() {
        assert_eq!(
            serde_json::from_str::<ProjectType>("\"rust\"").unwrap(),
            ProjectType::Rust
        );
        assert_eq!(
            serde_json::from_str::<ProjectType>("\"javascript\"").unwrap(),
            ProjectType::Javascript
        );
        assert_eq!(
            serde_json::from_str::<ProjectType>("\"typescript\"").unwrap(),
            ProjectType::Typescript
        );
        assert_eq!(
            serde_json::from_str::<ProjectType>("\"python\"").unwrap(),
            ProjectType::Python
        );
        assert_eq!(
            serde_json::from_str::<ProjectType>("\"go\"").unwrap(),
            ProjectType::Go
        );
        assert_eq!(
            serde_json::from_str::<ProjectType>("\"java\"").unwrap(),
            ProjectType::Java
        );
        assert_eq!(
            serde_json::from_str::<ProjectType>("\"unknown\"").unwrap(),
            ProjectType::Unknown
        );
    }

    #[test]
    fn cartography_meta_json_round_trip() {
        let meta = CartographyMeta {
            last_updated: "2026-03-31".to_string(),
            sources: vec![
                "crates/ecc-domain/src/lib.rs".to_string(),
                "commands/spec-dev.md".to_string(),
            ],
            session_id: "sess-002".to_string(),
        };

        let json = serde_json::to_string(&meta).expect("serialize failed");
        let round_tripped: CartographyMeta =
            serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(meta, round_tripped);
    }
}
