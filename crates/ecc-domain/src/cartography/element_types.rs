//! Element type definitions for the cartography registry.
//!
//! Zero I/O — pure domain types only.

use serde::{Deserialize, Serialize};

/// Two-tier element type taxonomy.
///
/// Universal variants apply to any software project; ECC overlay variants
/// are specific to the ECC domain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ElementType {
    /// A software module or package.
    Module,
    /// An interface or protocol definition.
    Interface,
    /// A configuration file or setting.
    Config,
    /// An unknown or unclassified element type.
    Unknown,
    /// An ECC slash command.
    Command,
    /// An ECC agent.
    Agent,
    /// An ECC skill.
    Skill,
    /// An ECC hook.
    Hook,
    /// An ECC rule file.
    Rule,
    /// A Rust crate.
    Crate,
    /// A hexagonal architecture port (trait interface).
    Port,
    /// A hexagonal architecture adapter (implementation).
    Adapter,
    /// A domain entity in the bounded context.
    DomainEntity,
}

/// A registry entry for a single codebase element.
///
/// All fields are serialisable; the struct is the canonical in-memory
/// representation used by generators and validators.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ElementEntry {
    /// Unique identifier for the element (e.g., "spec-dev", "ecc-domain").
    pub slug: String,
    /// The type of element (agent, skill, command, etc.).
    pub element_type: ElementType,
    /// One-line description of what this element does.
    pub purpose: String,
    /// List of elements this one depends on.
    pub uses: Vec<String>,
    /// List of elements that depend on this one.
    pub used_by: Vec<String>,
    /// Workflow flows this element participates in.
    pub participating_flows: Vec<String>,
    /// User journeys this element participates in.
    pub participating_journeys: Vec<String>,
    /// Source file paths that define this element.
    pub sources: Vec<String>,
    /// ISO 8601 date of the last update to this entry.
    pub last_updated: String,
}

/// Infer the element type for a Rust crate by name.
///
/// Returns [`ElementType::Module`] for unrecognised crate names.
pub fn infer_element_type_from_crate(crate_name: &str) -> ElementType {
    match crate_name {
        "ecc-domain" => ElementType::DomainEntity,
        "ecc-ports" => ElementType::Port,
        "ecc-infra" => ElementType::Adapter,
        "ecc-app"
        | "ecc-cli"
        | "ecc-workflow"
        | "ecc-test-support"
        | "ecc-integration-tests"
        | "ecc-flock" => ElementType::Module,
        _ => ElementType::Unknown,
    }
}

/// Infer the element type from a source file path using directory prefix matching.
///
/// Returns [`ElementType::Unknown`] when no prefix matches.
pub fn infer_element_type_from_path(source_path: &str) -> ElementType {
    if source_path.starts_with("agents/") {
        ElementType::Agent
    } else if source_path.starts_with("commands/") {
        ElementType::Command
    } else if source_path.starts_with("skills/") {
        ElementType::Skill
    } else if source_path.starts_with("hooks/") {
        ElementType::Hook
    } else if source_path.starts_with("rules/") {
        ElementType::Rule
    } else {
        ElementType::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn element_type_all_variants() {
        // Universal variants
        let _module = ElementType::Module;
        let _interface = ElementType::Interface;
        let _config = ElementType::Config;
        let _unknown = ElementType::Unknown;
        // ECC-specific variants
        let _command = ElementType::Command;
        let _agent = ElementType::Agent;
        let _skill = ElementType::Skill;
        let _hook = ElementType::Hook;
        let _rule = ElementType::Rule;
        let _crate_type = ElementType::Crate;
        let _port = ElementType::Port;
        let _adapter = ElementType::Adapter;
        let _domain_entity = ElementType::DomainEntity;
    }

    #[test]
    fn element_type_unknown_serde() {
        let serialized = serde_json::to_string(&ElementType::Unknown).unwrap();
        assert_eq!(serialized, "\"unknown\"");
        let deserialized: ElementType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, ElementType::Unknown);
    }

    #[test]
    fn element_entry_json_roundtrip() {
        let entry = ElementEntry {
            slug: "my-slug".to_string(),
            element_type: ElementType::Agent,
            purpose: "Does things".to_string(),
            uses: vec!["dep-a".to_string()],
            used_by: vec!["dep-b".to_string()],
            participating_flows: vec!["flow-1".to_string()],
            participating_journeys: vec!["journey-1".to_string()],
            sources: vec!["agents/my-agent.md".to_string()],
            last_updated: "2026-04-02".to_string(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let roundtripped: ElementEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtripped, entry);
    }

    #[test]
    fn crate_type_inference() {
        assert_eq!(
            infer_element_type_from_crate("ecc-domain"),
            ElementType::DomainEntity
        );
        assert_eq!(
            infer_element_type_from_crate("ecc-ports"),
            ElementType::Port
        );
        assert_eq!(
            infer_element_type_from_crate("ecc-infra"),
            ElementType::Adapter
        );
        assert_eq!(
            infer_element_type_from_crate("ecc-app"),
            ElementType::Module
        );
        assert_eq!(
            infer_element_type_from_crate("ecc-cli"),
            ElementType::Module
        );
        assert_eq!(
            infer_element_type_from_crate("ecc-workflow"),
            ElementType::Module
        );
        assert_eq!(
            infer_element_type_from_crate("ecc-test-support"),
            ElementType::Module
        );
        assert_eq!(
            infer_element_type_from_crate("ecc-integration-tests"),
            ElementType::Module
        );
        assert_eq!(
            infer_element_type_from_crate("ecc-flock"),
            ElementType::Module
        );
    }

    #[test]
    fn path_type_inference() {
        assert_eq!(
            infer_element_type_from_path("agents/my-agent.md"),
            ElementType::Agent
        );
        assert_eq!(
            infer_element_type_from_path("commands/my-command.md"),
            ElementType::Command
        );
        assert_eq!(
            infer_element_type_from_path("skills/my-skill.md"),
            ElementType::Skill
        );
        assert_eq!(
            infer_element_type_from_path("hooks/my-hook.md"),
            ElementType::Hook
        );
        assert_eq!(
            infer_element_type_from_path("rules/my-rule.md"),
            ElementType::Rule
        );
        assert_eq!(
            infer_element_type_from_path("src/some-file.rs"),
            ElementType::Unknown
        );
        assert_eq!(
            infer_element_type_from_path("unknown/path"),
            ElementType::Unknown
        );
    }
}
