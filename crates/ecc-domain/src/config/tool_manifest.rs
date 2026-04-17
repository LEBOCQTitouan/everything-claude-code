//! Tool manifest domain model — Value Object.
//!
//! `ToolManifest` is a **Value Object**: no identity, immutable once parsed,
//! structurally equal. It holds the canonical vocabulary of atomic tool names
//! and named preset bundles that group common tool combinations. All functions
//! here are pure (zero I/O).

#[cfg(test)]
mod tests {
    // ── helpers ──────────────────────────────────────────────────────────────

    fn minimal_yaml(tools: &[&str], presets: &[(&str, &[&str])]) -> String {
        let tools_list = tools
            .iter()
            .map(|t| format!("  - {t}"))
            .collect::<Vec<_>>()
            .join("\n");
        let presets_str = presets
            .iter()
            .map(|(name, ts)| {
                let items = ts
                    .iter()
                    .map(|t| format!("    - {t}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("  {name}:\n{items}")
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!("tools:\n{tools_list}\npresets:\n{presets_str}\n")
    }

    // ── PC-001: parses_valid_manifest_with_all_presets ────────────────────────

    #[test]
    fn parses_valid_manifest_with_all_presets() {
        use super::*;

        let yaml = r#"tools:
  - Read
  - Write
  - Edit
  - MultiEdit
  - Bash
  - Glob
  - Grep
  - Agent
  - Task
  - WebSearch
  - TodoWrite
  - TodoRead
  - AskUserQuestion
  - LS
  - Skill
  - EnterPlanMode
  - ExitPlanMode
  - TaskCreate
  - TaskUpdate
  - TaskGet
  - TaskList
presets:
  readonly-analyzer:
    - Read
    - Grep
    - Glob
  readonly-analyzer-shell:
    - Read
    - Grep
    - Glob
    - Bash
  tdd-executor:
    - Read
    - Write
    - Edit
    - Bash
  code-writer:
    - Read
    - Write
    - Edit
    - MultiEdit
    - Bash
  orchestrator:
    - Read
    - Write
    - Edit
    - Bash
    - Agent
    - Task
  audit-command:
    - Task
    - Read
    - Grep
    - Glob
    - LS
    - Bash
    - Write
    - TodoWrite
"#;

        let manifest = parse_tool_manifest(yaml).expect("should parse valid manifest");
        assert!(manifest.tools.contains(&"Read".to_string()));
        assert!(manifest.tools.contains(&"Write".to_string()));
        assert!(manifest.presets.contains_key("readonly-analyzer"));
        assert!(manifest.presets.contains_key("audit-command"));
        assert_eq!(manifest.presets.len(), 6);
    }

    // ── PC-003: rejects_unknown_tool_in_preset ────────────────────────────────

    #[test]
    fn rejects_unknown_tool_in_preset() {
        use super::*;

        let yaml = minimal_yaml(
            &["Read", "Write"],
            &[("my-preset", &["Read", "NotATool"])],
        );
        let manifest = parse_tool_manifest(&yaml).expect("parse should succeed");
        let errors = validate_tool_manifest(&manifest);
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, ToolManifestError::UnknownToolInPreset { tool, .. } if tool == "NotATool")),
            "expected UnknownToolInPreset for 'NotATool', got: {errors:?}"
        );
    }

    // ── PC-004: rejects_duplicate_preset_keys ────────────────────────────────

    #[test]
    fn rejects_duplicate_preset_keys() {
        use super::*;

        // serde_saphyr silently takes last for duplicate YAML keys; we pre-scan.
        let yaml = "tools:\n  - Read\npresets:\n  my-preset:\n    - Read\n  my-preset:\n    - Read\n";
        let result = parse_tool_manifest(yaml);
        assert!(
            matches!(result, Err(ToolManifestError::DuplicateTopLevelKey(_)))
                || result
                    .as_ref()
                    .err()
                    .map(|e| {
                        matches!(
                            e,
                            ToolManifestError::DuplicatePresetKey(_)
                        )
                    })
                    .unwrap_or(false),
            "expected DuplicateTopLevelKey or DuplicatePresetKey, got: {result:?}"
        );
    }

    // ── PC-005: rejects_duplicate_atomic_tools ────────────────────────────────

    #[test]
    fn rejects_duplicate_atomic_tools() {
        use super::*;

        let yaml = "tools:\n  - Read\n  - Read\npresets:\n  my-preset:\n    - Read\n";
        let result = parse_tool_manifest(yaml);
        // duplicate tools in the list — validate catches it
        let errors = match result {
            Ok(m) => validate_tool_manifest(&m),
            Err(e) => {
                // acceptable if parse itself rejects
                assert!(
                    matches!(e, ToolManifestError::DuplicateAtomicTool(_)),
                    "unexpected parse error: {e:?}"
                );
                return;
            }
        };
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, ToolManifestError::DuplicateAtomicTool(t) if t == "Read")),
            "expected DuplicateAtomicTool('Read'), got: {errors:?}"
        );
    }

    // ── PC-006: rejects_empty_preset ─────────────────────────────────────────

    #[test]
    fn rejects_empty_preset() {
        use super::*;

        let yaml = "tools:\n  - Read\npresets:\n  readonly: []\n";
        let manifest = parse_tool_manifest(yaml).expect("parse should succeed");
        let errors = validate_tool_manifest(&manifest);
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, ToolManifestError::EmptyPreset(n) if n == "readonly")),
            "expected EmptyPreset('readonly'), got: {errors:?}"
        );
    }

    // ── PC-007: rejects_invalid_preset_names ─────────────────────────────────

    #[test]
    fn rejects_invalid_preset_names() {
        use super::*;

        // These 7 fixture names must all be rejected with InvalidPresetName.
        // Note: empty-string key and keys starting with special chars are
        // YAML-level issues; we handle them in validate.
        let invalid_names: &[&str] = &[
            "-foo",
            "foo-",
            "Foo",
            "foo_bar",
            "foo.bar",
            "resto-fr-ça",
        ];

        for name in invalid_names {
            let yaml = format!("tools:\n  - Read\npresets:\n  \"{name}\":\n    - Read\n");
            let manifest = parse_tool_manifest(&yaml).expect("parse should succeed");
            let errors = validate_tool_manifest(&manifest);
            assert!(
                errors
                    .iter()
                    .any(|e| matches!(e, ToolManifestError::InvalidPresetName(n) if n == name)),
                "expected InvalidPresetName for '{name}', got: {errors:?}"
            );
        }

        // Empty string: YAML empty key "" is valid YAML but should be rejected
        let yaml_empty = "tools:\n  - Read\npresets:\n  \"\":\n    - Read\n";
        let manifest = parse_tool_manifest(yaml_empty).expect("parse should succeed for empty key");
        let errors = validate_tool_manifest(&manifest);
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, ToolManifestError::InvalidPresetName(n) if n.is_empty())),
            "expected InvalidPresetName for empty string, got: {errors:?}"
        );
    }

    // ── PC-012: bom_prefix_stripped_before_parse ─────────────────────────────

    #[test]
    fn bom_prefix_stripped_before_parse() {
        use super::*;

        let base = "tools:\n  - Read\npresets:\n  my-preset:\n    - Read\n";
        let with_bom = format!("\u{FEFF}{base}");

        let plain = parse_tool_manifest(base).expect("plain should parse");
        let bom = parse_tool_manifest(&with_bom).expect("BOM-prefixed should parse identically");
        assert_eq!(plain, bom, "BOM-prefixed manifest must parse identically");
    }

    // ── PC-013: rejects_duplicate_top_level_keys ─────────────────────────────

    #[test]
    fn rejects_duplicate_top_level_keys() {
        use super::*;

        // `presets:` appears twice at top level
        let yaml =
            "tools:\n  - Read\npresets:\n  my-preset:\n    - Read\npresets:\n  other:\n    - Read\n";
        let result = parse_tool_manifest(yaml);
        assert!(
            matches!(result, Err(ToolManifestError::DuplicateTopLevelKey(_))),
            "expected DuplicateTopLevelKey, got: {result:?}"
        );
    }

    // ── PC-014: rejects_oversized_manifest ───────────────────────────────────

    #[test]
    fn rejects_oversized_manifest() {
        use super::*;

        // Generate a string > 1 MB
        let oversized = "x".repeat(MAX_MANIFEST_BYTES + 1);
        let result = parse_tool_manifest(&oversized);
        assert!(
            matches!(result, Err(ToolManifestError::ManifestTooLarge)),
            "expected ManifestTooLarge, got: {result:?}"
        );
    }

    // ── PC-070: rejects_yaml_anchors_and_aliases ──────────────────────────────

    #[test]
    fn rejects_yaml_anchors_and_aliases() {
        use super::*;

        let yaml_anchor = "tools:\n  - &anchor Read\n  - Write\npresets:\n  p:\n    - Read\n";
        let result = parse_tool_manifest(yaml_anchor);
        assert!(
            matches!(result, Err(ToolManifestError::YamlAnchorsNotAllowed)),
            "expected YamlAnchorsNotAllowed for anchor, got: {result:?}"
        );

        let yaml_alias = "tools:\n  - Read\n  - *alias\npresets:\n  p:\n    - Read\n";
        let result = parse_tool_manifest(yaml_alias);
        assert!(
            matches!(result, Err(ToolManifestError::YamlAnchorsNotAllowed)),
            "expected YamlAnchorsNotAllowed for alias, got: {result:?}"
        );
    }

    // ── PC-002: module_doc_declares_value_object ──────────────────────────────

    #[test]
    fn module_doc_declares_value_object() {
        let source = include_str!("tool_manifest.rs");
        assert!(
            source.contains("Value Object"),
            "module doc-comment must declare this is a Value Object"
        );
    }

    // ── PC-010: manifest_tools_is_superset_of_legacy ─────────────────────────

    #[test]
    fn manifest_tools_is_superset_of_legacy() {
        use super::*;

        /// These are the legacy VALID_TOOLS values that must survive migration.
        const LEGACY_VALID_TOOLS: &[&str] = &[
            "Read",
            "Write",
            "Edit",
            "MultiEdit",
            "Bash",
            "Glob",
            "Grep",
            "Agent",
            "Task",
            "WebSearch",
            "TodoWrite",
            "TodoRead",
            "AskUserQuestion",
            "LS",
            "Skill",
            "EnterPlanMode",
            "ExitPlanMode",
            "TaskCreate",
            "TaskUpdate",
            "TaskGet",
            "TaskList",
        ];

        // Parse the canonical manifest embedded at compile time.
        let yaml = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../../manifest/tool-manifest.yaml"
        ));
        let manifest = parse_tool_manifest(yaml).expect("canonical manifest must parse");

        for tool in LEGACY_VALID_TOOLS {
            assert!(
                manifest.tools.contains(&tool.to_string()),
                "canonical manifest missing legacy tool: {tool}"
            );
        }
    }

    // ── PC-008: valid_tools_constant_removed ─────────────────────────────────

    #[test]
    fn valid_tools_constant_removed() {
        // Read the validate.rs source. The VALID_TOOLS constant must not appear
        // outside of test code (i.e., not in a const declaration).
        let source = include_str!("validate.rs");
        // Check that the constant declaration is gone.
        assert!(
            !source.contains("pub const VALID_TOOLS"),
            "VALID_TOOLS constant declaration must be removed from validate.rs"
        );
    }
}
