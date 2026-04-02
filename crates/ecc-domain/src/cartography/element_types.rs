//! Element type definitions for the cartography registry.
//!
//! Zero I/O — pure domain types only.

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
        assert_eq!(infer_element_type_from_crate("ecc-domain"), ElementType::DomainEntity);
        assert_eq!(infer_element_type_from_crate("ecc-ports"), ElementType::Port);
        assert_eq!(infer_element_type_from_crate("ecc-infra"), ElementType::Adapter);
        assert_eq!(infer_element_type_from_crate("ecc-app"), ElementType::Module);
        assert_eq!(infer_element_type_from_crate("ecc-cli"), ElementType::Module);
        assert_eq!(infer_element_type_from_crate("ecc-workflow"), ElementType::Module);
        assert_eq!(infer_element_type_from_crate("ecc-test-support"), ElementType::Module);
        assert_eq!(infer_element_type_from_crate("ecc-integration-tests"), ElementType::Module);
        assert_eq!(infer_element_type_from_crate("ecc-flock"), ElementType::Module);
    }

    #[test]
    fn path_type_inference() {
        assert_eq!(infer_element_type_from_path("agents/my-agent.md"), ElementType::Agent);
        assert_eq!(infer_element_type_from_path("commands/my-command.md"), ElementType::Command);
        assert_eq!(infer_element_type_from_path("skills/my-skill.md"), ElementType::Skill);
        assert_eq!(infer_element_type_from_path("hooks/my-hook.md"), ElementType::Hook);
        assert_eq!(infer_element_type_from_path("rules/my-rule.md"), ElementType::Rule);
        assert_eq!(infer_element_type_from_path("src/some-file.rs"), ElementType::Unknown);
        assert_eq!(infer_element_type_from_path("unknown/path"), ElementType::Unknown);
    }
}
