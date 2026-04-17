//! Tool manifest domain model — Value Object.
//!
//! `ToolManifest` is a **Value Object**: no identity, immutable once parsed,
//! structurally equal. It holds the canonical vocabulary of atomic tool names
//! and named preset bundles that group common tool combinations. All functions
//! here are pure (zero I/O).

use std::collections::{HashMap, HashSet};

/// Maximum manifest size in bytes (1 MiB). Manifests larger than this are
/// rejected before parsing to prevent resource exhaustion.
pub const MAX_MANIFEST_BYTES: usize = 1_048_576; // 1 MiB

/// Preset name regex: `^[a-z][a-z0-9-]*[a-z0-9]$`
///
/// ASCII-only kebab-case, must start and end with a lowercase letter or digit,
/// no consecutive hyphens, no underscores, no dots, no Unicode.
pub const PRESET_NAME_REGEX: &str = r"^[a-z][a-z0-9-]*[a-z0-9]$";

/// A parsed tool manifest — the canonical vocabulary of atomic tool names and
/// named preset bundles. This is a **Value Object** (immutable, no identity).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolManifest {
    /// Ordered list of atomic tool identifiers (the full vocabulary).
    pub tools: Vec<String>,
    /// Named preset bundles: preset-name → list of atomic tool names.
    pub presets: HashMap<String, Vec<String>>,
}

/// Errors produced by `parse_tool_manifest` and `validate_tool_manifest`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolManifestError {
    /// The manifest exceeds `MAX_MANIFEST_BYTES`.
    ManifestTooLarge,
    /// A YAML anchor (`&`) or alias (`*`) was found — rejected for DoS safety.
    YamlAnchorsNotAllowed,
    /// A top-level YAML key appears more than once (e.g., `presets:` twice).
    DuplicateTopLevelKey(String),
    /// The YAML could not be deserialized.
    ParseError(String),
    /// A preset key appears more than once in the `presets:` mapping.
    DuplicatePresetKey(String),
    /// An atomic tool name appears more than once in the `tools:` list.
    DuplicateAtomicTool(String),
    /// A preset references a tool name not present in the `tools:` list.
    UnknownToolInPreset {
        /// The preset that contains the unknown reference.
        preset: String,
        /// The unknown tool name.
        tool: String,
    },
    /// A preset's tool list is empty.
    EmptyPreset(String),
    /// A preset name does not match the required regex pattern.
    InvalidPresetName(String),
}

impl std::fmt::Display for ToolManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ManifestTooLarge => write!(
                f,
                "manifest exceeds maximum size of {} bytes",
                MAX_MANIFEST_BYTES
            ),
            Self::YamlAnchorsNotAllowed => {
                write!(f, "YAML anchors (&) and aliases (*) are not allowed")
            }
            Self::DuplicateTopLevelKey(k) => {
                write!(f, "duplicate top-level YAML key: '{k}'")
            }
            Self::ParseError(msg) => write!(f, "parse error: {msg}"),
            Self::DuplicatePresetKey(k) => write!(f, "duplicate preset key: '{k}'"),
            Self::DuplicateAtomicTool(t) => write!(f, "duplicate atomic tool: '{t}'"),
            Self::UnknownToolInPreset { preset, tool } => {
                write!(f, "preset '{preset}' references unknown tool '{tool}'")
            }
            Self::EmptyPreset(name) => write!(f, "preset '{name}' is empty"),
            Self::InvalidPresetName(name) => write!(
                f,
                "invalid preset name '{name}': must match {PRESET_NAME_REGEX}"
            ),
        }
    }
}

impl std::error::Error for ToolManifestError {}

/// Internal raw deserialization target for the YAML manifest.
#[derive(Debug, serde::Deserialize)]
struct RawManifest {
    tools: Vec<String>,
    presets: HashMap<String, Vec<String>>,
}

/// Parse a tool manifest from a YAML string.
///
/// This function:
/// 1. Rejects manifests larger than `MAX_MANIFEST_BYTES`.
/// 2. Strips a leading U+FEFF BOM.
/// 3. Rejects YAML anchors (`&`) and aliases (`*`).
/// 4. Detects duplicate top-level keys before handing to serde.
/// 5. Deserializes via `serde_saphyr`.
///
/// Returns a `ToolManifest` on success, or the first `ToolManifestError`
/// encountered during parsing.
pub fn parse_tool_manifest(input: &str) -> Result<ToolManifest, ToolManifestError> {
    // 1. Size check
    if input.len() > MAX_MANIFEST_BYTES {
        return Err(ToolManifestError::ManifestTooLarge);
    }

    // 2. BOM strip
    let yaml = input.strip_prefix('\u{FEFF}').unwrap_or(input);

    // 3. Anchor/alias pre-scan
    if contains_yaml_anchors_or_aliases(yaml) {
        return Err(ToolManifestError::YamlAnchorsNotAllowed);
    }

    // 4. Duplicate top-level key detection
    if let Some(dup) = detect_duplicate_top_level_key(yaml) {
        return Err(ToolManifestError::DuplicateTopLevelKey(dup));
    }

    // 4b. Duplicate preset key detection (keys nested under `presets:`)
    if let Some(dup) = detect_duplicate_preset_key(yaml) {
        return Err(ToolManifestError::DuplicatePresetKey(dup));
    }

    // 5. Deserialize
    let raw: RawManifest =
        serde_saphyr::from_str(yaml).map_err(|e| ToolManifestError::ParseError(e.to_string()))?;

    Ok(ToolManifest {
        tools: raw.tools,
        presets: raw.presets,
    })
}

/// Validate a parsed `ToolManifest` (pure domain rules, no I/O).
///
/// Returns a (possibly empty) list of validation errors. An empty list means
/// the manifest is valid.
pub fn validate_tool_manifest(manifest: &ToolManifest) -> Vec<ToolManifestError> {
    let mut errors = Vec::new();

    // Duplicate atomic tools
    let mut seen_tools: HashSet<&str> = HashSet::new();
    for tool in &manifest.tools {
        if !seen_tools.insert(tool.as_str()) {
            errors.push(ToolManifestError::DuplicateAtomicTool(tool.clone()));
        }
    }

    let tool_set: HashSet<&str> = manifest.tools.iter().map(String::as_str).collect();

    for (name, tools) in &manifest.presets {
        // Invalid preset name
        if !is_valid_kebab_identifier(name) {
            errors.push(ToolManifestError::InvalidPresetName(name.clone()));
        }

        // Empty preset
        if tools.is_empty() {
            errors.push(ToolManifestError::EmptyPreset(name.clone()));
        }

        // Unknown tools in preset
        for tool in tools {
            if !tool_set.contains(tool.as_str()) {
                errors.push(ToolManifestError::UnknownToolInPreset {
                    preset: name.clone(),
                    tool: tool.clone(),
                });
            }
        }
    }

    errors
}

// ── Private helpers ──────────────────────────────────────────────────────────

/// Returns `true` if the YAML text contains any `&` anchor or `*` alias
/// indicators that are not inside quoted strings or comments.
///
/// This is a conservative pre-parse check to defend against billion-laughs
/// style DoS attacks. We scan line by line; if a line (outside a comment)
/// contains a bare `&` or `*` that is not inside a quoted string literal,
/// we reject.
fn contains_yaml_anchors_or_aliases(yaml: &str) -> bool {
    for line in yaml.lines() {
        let trimmed = line.trim();
        // Skip comment lines
        if trimmed.starts_with('#') {
            continue;
        }
        // Check for anchor/alias indicators outside quoted regions
        if has_bare_anchor_or_alias(trimmed) {
            return true;
        }
    }
    false
}

/// Returns `true` if a YAML line (already trimmed, non-comment) contains
/// a bare `&` or `*` outside of a quoted string.
fn has_bare_anchor_or_alias(line: &str) -> bool {
    let mut in_single = false;
    let mut in_double = false;
    for c in line.chars() {
        match c {
            '\'' if !in_double => {
                in_single = !in_single;
            }
            '"' if !in_single => {
                in_double = !in_double;
            }
            '&' | '*' if !in_single && !in_double => {
                return true;
            }
            _ => {}
        }
    }
    false
}

/// Scan for duplicate top-level YAML keys by parsing the raw line structure.
///
/// Returns the first duplicated key, or `None`.
fn detect_duplicate_top_level_key(yaml: &str) -> Option<String> {
    let mut seen: HashSet<String> = HashSet::new();
    for line in yaml.lines() {
        // Top-level keys: lines that start without indentation and contain `:`
        if line.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
            && let Some(colon) = line.find(':')
        {
            let key = line[..colon].trim().to_string();
            if !key.is_empty() && !seen.insert(key.clone()) {
                return Some(key);
            }
        }
    }
    None
}

/// Scan for duplicate keys nested one level under the `presets:` top-level key.
///
/// Returns the first duplicated preset name, or `None`.
fn detect_duplicate_preset_key(yaml: &str) -> Option<String> {
    let mut in_presets = false;
    let mut seen: HashSet<String> = HashSet::new();
    for line in yaml.lines() {
        // Detect entry into `presets:` section
        if line.trim_end() == "presets:" {
            in_presets = true;
            seen.clear();
            continue;
        }
        if in_presets {
            // Exit presets section when we see another non-indented key
            if !line.starts_with(' ') && !line.trim().is_empty() {
                in_presets = false;
                continue;
            }
            // Preset sub-keys are indented exactly 2 spaces (not 4+)
            if line.starts_with("  ")
                && !line.starts_with("    ")
                && let Some(colon) = line.find(':')
            {
                let raw_key = line[..colon].trim().trim_matches('"').trim_matches('\'');
                let key = raw_key.to_string();
                if !key.is_empty() && !seen.insert(key.clone()) {
                    return Some(key);
                }
            }
        }
    }
    None
}

/// Returns `true` if the string matches `^[a-z][a-z0-9-]*[a-z0-9]$`.
///
/// Rules:
/// - ASCII-only
/// - First char: lowercase letter `a-z`
/// - Last char: lowercase letter `a-z` or digit `0-9`
/// - Middle chars: lowercase letters, digits, or hyphens
/// - Minimum length: 2 characters
///
/// Used for both preset names and tool-set references.
pub fn is_valid_kebab_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let bytes = name.as_bytes();
    // Must be pure ASCII
    if !name.is_ascii() {
        return false;
    }
    if bytes.len() < 2 {
        // Single character: must be `[a-z]` — the pattern requires last char
        // to be `[a-z0-9]` AND first to be `[a-z]`. Single char satisfies
        // both only if it's `[a-z]` — but the pattern `^[a-z][a-z0-9-]*[a-z0-9]$`
        // requires at least 2 chars (the two anchored character classes).
        return false;
    }
    // First char: a-z
    if !bytes[0].is_ascii_lowercase() {
        return false;
    }
    // Last char: a-z or 0-9
    let last = *bytes.last().unwrap();
    if !last.is_ascii_lowercase() && !last.is_ascii_digit() {
        return false;
    }
    // Middle chars: a-z, 0-9, or -
    for &b in &bytes[1..bytes.len() - 1] {
        if !b.is_ascii_lowercase() && !b.is_ascii_digit() && b != b'-' {
            return false;
        }
    }
    true
}

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

        let yaml = minimal_yaml(&["Read", "Write"], &[("my-preset", &["Read", "NotATool"])]);
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
        let yaml =
            "tools:\n  - Read\npresets:\n  my-preset:\n    - Read\n  my-preset:\n    - Read\n";
        let result = parse_tool_manifest(yaml);
        assert!(
            matches!(result, Err(ToolManifestError::DuplicateTopLevelKey(_)))
                || result
                    .as_ref()
                    .err()
                    .map(|e| { matches!(e, ToolManifestError::DuplicatePresetKey(_)) })
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
        let invalid_names: &[&str] = &["-foo", "foo-", "Foo", "foo_bar", "foo.bar", "resto-fr-ça"];

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
        let yaml = "tools:\n  - Read\npresets:\n  my-preset:\n    - Read\npresets:\n  other:\n    - Read\n";
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
            "/../../manifest/tool-manifest.yaml"
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
