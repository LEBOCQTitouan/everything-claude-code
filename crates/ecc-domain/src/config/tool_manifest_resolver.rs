//! Pure resolver for effective tool lists — no I/O, no panics.
//!
//! `resolve_effective_tools` is the single entry point: given a
//! `FrontmatterToolSpec` and a `ToolManifest`, it returns either a
//! `ResolvedTools` (with optional warnings about outlier inline tools) or a
//! `ResolveError`.

use crate::config::tool_manifest::ToolManifest;

/// Preset name regex: must match `^[a-z][a-z0-9-]*[a-z0-9]$`.
const TOOL_SET_REGEX: &str = r"^[a-z][a-z0-9-]*[a-z0-9]$";

/// The tool specification extracted from frontmatter before resolution.
///
/// Either `tool_set`, `inline_tools`, or both may be present.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontmatterToolSpec {
    /// Named preset reference (e.g. `readonly-analyzer`).
    /// `None` means the frontmatter had no `tool-set:` key.
    /// `Some(s)` where `s` looks like a YAML array indicates
    /// `ResolveError::ArrayNotSupported`.
    pub tool_set: Option<String>,
    /// Inline tool list from the `tools:` frontmatter field.
    pub inline_tools: Option<Vec<String>>,
}

/// The result of a successful resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTools {
    /// Effective tool list (union of preset + inline, deduped).
    pub tools: Vec<String>,
    /// Zero or more WARN messages for outlier inline tools.
    pub warnings: Vec<String>,
}

/// Errors from `resolve_effective_tools`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    /// The `tool-set:` value matched no preset in the manifest.
    UnknownPreset(String),
    /// The `tool-set:` value appears to be a YAML array (`[a, b]`).
    ArrayNotSupported,
    /// Resolution would produce an empty tool list.
    EmptyResolution,
    /// Neither `tool-set:` nor `tools:` was provided.
    NeitherToolSetNorTools,
    /// The `tool-set:` value does not match the required kebab-case regex.
    InvalidToolSetReference(String),
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownPreset(p) => write!(f, "unknown preset '{p}'"),
            Self::ArrayNotSupported => write!(
                f,
                "tool-set: must be a single string, not an array"
            ),
            Self::EmptyResolution => write!(f, "effective tool list is empty after resolution"),
            Self::NeitherToolSetNorTools => {
                write!(f, "neither tool-set nor tools field provided")
            }
            Self::InvalidToolSetReference(v) => write!(
                f,
                "invalid tool-set value '{v}': must match {TOOL_SET_REGEX}"
            ),
        }
    }
}

impl std::error::Error for ResolveError {}

/// Resolve the effective tool list for a frontmatter spec against a manifest.
///
/// Rules:
/// - Neither `tool-set` nor `tools` → `ResolveError::NeitherToolSetNorTools`
/// - `tool-set: [a, b]` (array indicator) → `ResolveError::ArrayNotSupported`
/// - `tool-set:` value not matching kebab regex → `ResolveError::InvalidToolSetReference`
/// - `tool-set:` not found in manifest → `ResolveError::UnknownPreset`
/// - `tool-set:` only → preset tools
/// - `tools:` only → inline tools
/// - Both → union, deduped by exact string equality. Any inline tool not in
///   the preset emits a WARN (exit 0, not exit 1).
/// - Empty result → `ResolveError::EmptyResolution`
pub fn resolve_effective_tools(
    spec: &FrontmatterToolSpec,
    manifest: &ToolManifest,
) -> Result<ResolvedTools, ResolveError> {
    // Neither provided
    if spec.tool_set.is_none() && spec.inline_tools.is_none() {
        return Err(ResolveError::NeitherToolSetNorTools);
    }

    let mut preset_tools: Option<Vec<String>> = None;

    if let Some(ref ts_value) = spec.tool_set {
        // Array indicator check: starts with '[' or contains comma-like patterns
        let trimmed = ts_value.trim();
        if trimmed.starts_with('[') {
            return Err(ResolveError::ArrayNotSupported);
        }

        // Validate against kebab regex before lookup
        if !is_valid_tool_set_value(trimmed) {
            return Err(ResolveError::InvalidToolSetReference(ts_value.clone()));
        }

        // Look up in manifest
        match manifest.presets.get(trimmed) {
            Some(tools) => preset_tools = Some(tools.clone()),
            None => return Err(ResolveError::UnknownPreset(ts_value.clone())),
        }
    }

    let inline = spec.inline_tools.as_deref().unwrap_or(&[]);

    // Build merged list
    let merged: Vec<String> = match &preset_tools {
        Some(preset) => {
            // Union: start with preset, add inline tools not already present
            let mut result = preset.clone();
            let preset_set: std::collections::HashSet<&str> =
                preset.iter().map(String::as_str).collect();
            for tool in inline {
                if !preset_set.contains(tool.as_str()) {
                    result.push(tool.clone());
                }
            }
            result
        }
        None => {
            // Only inline tools
            inline.to_vec()
        }
    };

    if merged.is_empty() {
        return Err(ResolveError::EmptyResolution);
    }

    // Compute WARN for outlier inline tools (tools in inline but not in preset)
    let warnings = if let Some(ref preset) = preset_tools {
        let preset_set: std::collections::HashSet<&str> =
            preset.iter().map(String::as_str).collect();
        inline
            .iter()
            .filter(|t| !preset_set.contains(t.as_str()))
            .map(|t| {
                format!(
                    "inline tool '{t}' is not in preset '{}' (outlier)",
                    spec.tool_set.as_deref().unwrap_or("")
                )
            })
            .collect()
    } else {
        vec![]
    };

    Ok(ResolvedTools {
        tools: merged,
        warnings,
    })
}

/// Returns `true` if the tool-set value matches `^[a-z][a-z0-9-]*[a-z0-9]$`.
fn is_valid_tool_set_value(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    let bytes = value.as_bytes();
    if !value.is_ascii() {
        return false;
    }
    if bytes.len() < 2 {
        return false;
    }
    if !bytes[0].is_ascii_lowercase() {
        return false;
    }
    let last = *bytes.last().unwrap();
    if !last.is_ascii_lowercase() && !last.is_ascii_digit() {
        return false;
    }
    for &b in &bytes[1..bytes.len() - 1] {
        if !b.is_ascii_lowercase() && !b.is_ascii_digit() && b != b'-' {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::tool_manifest::{ToolManifest, parse_tool_manifest};

    fn make_manifest() -> ToolManifest {
        let yaml = r#"tools:
  - Read
  - Write
  - Edit
  - Bash
  - Grep
  - Glob
  - FakeTool
presets:
  readonly-analyzer:
    - Read
    - Grep
    - Glob
  code-writer:
    - Read
    - Write
    - Edit
    - Bash
"#;
        parse_tool_manifest(yaml).expect("fixture manifest must parse")
    }

    // ── PC-017: union_dedupes_and_warns_on_outliers ───────────────────────────

    #[test]
    fn union_dedupes_and_warns_on_outliers() {
        let manifest = make_manifest();
        let spec = FrontmatterToolSpec {
            tool_set: Some("readonly-analyzer".to_string()),
            inline_tools: Some(vec![
                "Read".to_string(),  // already in preset — no warn
                "Bash".to_string(),  // NOT in readonly-analyzer — outlier
            ]),
        };
        let result = resolve_effective_tools(&spec, &manifest).expect("should resolve");

        // Deduped: Read appears once (from preset)
        let read_count = result.tools.iter().filter(|t| t == &"Read").count();
        assert_eq!(read_count, 1, "Read should appear exactly once (deduped)");

        // Bash is included (outlier, but still merged)
        assert!(result.tools.contains(&"Bash".to_string()), "Bash should be in merged list");

        // Warning for Bash (outlier)
        assert_eq!(result.warnings.len(), 1, "should have exactly 1 warning (Bash is outlier)");
        assert!(
            result.warnings[0].contains("Bash"),
            "warning should name outlier tool 'Bash'"
        );
    }

    // ── PC-021: rejects_array_tool_set ───────────────────────────────────────

    #[test]
    fn rejects_array_tool_set() {
        let manifest = make_manifest();
        let spec = FrontmatterToolSpec {
            tool_set: Some("[a, b]".to_string()),
            inline_tools: None,
        };
        let result = resolve_effective_tools(&spec, &manifest);
        assert!(
            matches!(result, Err(ResolveError::ArrayNotSupported)),
            "expected ArrayNotSupported, got: {result:?}"
        );
    }

    // ── PC-071: rejects_invalid_tool_set_value ────────────────────────────────

    #[test]
    fn rejects_invalid_tool_set_value() {
        let manifest = make_manifest();

        let invalid_values = [
            "-foo",
            "foo-",
            "Foo",
            "FOO",
            "foo_bar",
            "foo bar",
            "",
        ];

        for v in &invalid_values {
            let spec = FrontmatterToolSpec {
                tool_set: Some(v.to_string()),
                inline_tools: None,
            };
            let result = resolve_effective_tools(&spec, &manifest);
            assert!(
                matches!(result, Err(ResolveError::InvalidToolSetReference(_))),
                "expected InvalidToolSetReference for '{v}', got: {result:?}"
            );
        }
    }

    // ── additional unit coverage ──────────────────────────────────────────────

    #[test]
    fn neither_tool_set_nor_tools_errors() {
        let manifest = make_manifest();
        let spec = FrontmatterToolSpec {
            tool_set: None,
            inline_tools: None,
        };
        assert!(matches!(
            resolve_effective_tools(&spec, &manifest),
            Err(ResolveError::NeitherToolSetNorTools)
        ));
    }

    #[test]
    fn unknown_preset_errors() {
        let manifest = make_manifest();
        let spec = FrontmatterToolSpec {
            tool_set: Some("nonexistent".to_string()),
            inline_tools: None,
        };
        assert!(matches!(
            resolve_effective_tools(&spec, &manifest),
            Err(ResolveError::UnknownPreset(_))
        ));
    }

    #[test]
    fn tool_set_only_returns_preset_tools() {
        let manifest = make_manifest();
        let spec = FrontmatterToolSpec {
            tool_set: Some("readonly-analyzer".to_string()),
            inline_tools: None,
        };
        let resolved = resolve_effective_tools(&spec, &manifest).expect("should resolve");
        assert!(resolved.tools.contains(&"Read".to_string()));
        assert!(resolved.tools.contains(&"Grep".to_string()));
        assert!(resolved.tools.contains(&"Glob".to_string()));
        assert!(resolved.warnings.is_empty());
    }

    #[test]
    fn inline_only_returns_inline_tools() {
        let manifest = make_manifest();
        let spec = FrontmatterToolSpec {
            tool_set: None,
            inline_tools: Some(vec!["Read".to_string(), "Write".to_string()]),
        };
        let resolved = resolve_effective_tools(&spec, &manifest).expect("should resolve");
        assert_eq!(resolved.tools, vec!["Read".to_string(), "Write".to_string()]);
        assert!(resolved.warnings.is_empty());
    }

    pub(crate) mod proptests {
        use super::*;
        use proptest::prelude::*;

        fn arb_manifest() -> ToolManifest {
            let yaml = r#"tools:
  - Read
  - Write
  - Edit
presets:
  my-preset:
    - Read
    - Write
"#;
            parse_tool_manifest(yaml).unwrap()
        }

        proptest! {
            // ── PC-020: never_panics ──────────────────────────────────────────

            #[test]
            fn never_panics(
                tool_set in proptest::option::of(proptest::arbitrary::any::<String>()),
                inline in proptest::option::of(proptest::collection::vec(proptest::arbitrary::any::<String>(), 0..10))
            ) {
                let manifest = arb_manifest();
                let spec = FrontmatterToolSpec { tool_set, inline_tools: inline };
                // Must not panic — result may be Ok or Err
                let _ = resolve_effective_tools(&spec, &manifest);
            }
        }
    }
}
