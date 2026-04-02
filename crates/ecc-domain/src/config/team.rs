//! Team manifest parsing and validation — pure domain, no I/O.
//!
//! Parses team manifests from Markdown with YAML frontmatter.
//! Team manifests define agent team compositions for coordinated execution.

use serde::Deserialize;
use std::collections::HashSet;

/// Valid coordination strategies for team manifests.
pub const VALID_COORDINATION_STRATEGIES: &[&str] = &["sequential", "parallel", "wave-dispatch"];

/// A parsed team manifest from YAML frontmatter.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TeamManifest {
    pub name: String,
    pub description: String,
    pub coordination: String,
    pub agents: Vec<TeamAgent>,
    #[serde(default)]
    pub max_concurrent: Option<u32>,
}

/// A single agent entry within a team manifest.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TeamAgent {
    pub name: String,
    pub role: String,
    #[serde(default)]
    pub allowed_tools: Option<Vec<String>>,
}

/// Errors from team manifest parsing/validation (pure domain, no I/O).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TeamValidationError {
    /// YAML frontmatter could not be parsed.
    ParseError(String),
    /// Agents list is empty.
    EmptyAgentsList,
    /// Unknown coordination strategy.
    UnknownStrategy(String),
    /// Duplicate agent name.
    DuplicateAgent(String),
    /// max-concurrent is < 1.
    InvalidMaxConcurrent(u32),
}

impl std::fmt::Display for TeamValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(msg) => write!(f, "Parse error: {msg}"),
            Self::EmptyAgentsList => write!(f, "Team manifest must define at least one agent"),
            Self::UnknownStrategy(s) => write!(f, "Unknown coordination strategy '{s}'"),
            Self::DuplicateAgent(name) => write!(f, "Duplicate agent '{name}' in team manifest"),
            Self::InvalidMaxConcurrent(v) => write!(f, "max-concurrent must be >= 1, got {v}"),
        }
    }
}

impl std::error::Error for TeamValidationError {}

/// Extract the raw YAML frontmatter string from Markdown content.
///
/// Returns the text between the first `---` and the next `---`.
pub fn extract_frontmatter_raw(content: &str) -> Option<&str> {
    let clean = content.strip_prefix('\u{FEFF}').unwrap_or(content);
    let rest = clean.strip_prefix("---")?;
    let end = rest.find("\n---")?;
    Some(&rest[..end])
}

/// Parse a team manifest from Markdown content.
///
/// Extracts YAML frontmatter and deserializes it into `TeamManifest`.
pub fn parse_team_manifest(content: &str) -> Result<TeamManifest, TeamValidationError> {
    let raw = extract_frontmatter_raw(content).ok_or_else(|| {
        TeamValidationError::ParseError(
            "Missing or malformed YAML frontmatter (no --- delimiters)".to_string(),
        )
    })?;
    serde_saphyr::from_str(raw).map_err(|e| TeamValidationError::ParseError(e.to_string()))
}

/// Validate a parsed team manifest (pure domain rules, no I/O).
///
/// Returns a list of validation errors. Empty = valid.
pub fn validate_team_manifest(manifest: &TeamManifest) -> Vec<TeamValidationError> {
    let mut errors = Vec::new();

    if manifest.agents.is_empty() {
        errors.push(TeamValidationError::EmptyAgentsList);
    }

    if !VALID_COORDINATION_STRATEGIES.contains(&manifest.coordination.as_str()) {
        errors.push(TeamValidationError::UnknownStrategy(
            manifest.coordination.clone(),
        ));
    }

    let mut seen = HashSet::new();
    for agent in &manifest.agents {
        if !seen.insert(&agent.name) {
            errors.push(TeamValidationError::DuplicateAgent(agent.name.clone()));
        }
    }

    if let Some(mc) = manifest.max_concurrent
        && mc < 1
    {
        errors.push(TeamValidationError::InvalidMaxConcurrent(mc));
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_MANIFEST: &str = r#"---
name: test-team
description: A test team
coordination: wave-dispatch
max-concurrent: 2
agents:
  - name: tdd-executor
    role: implementer
    allowed-tools: ["Read", "Write", "Edit"]
  - name: code-reviewer
    role: reviewer
---

# Test Team
Body content here.
"#;

    #[test]
    fn parses_valid_manifest() {
        let manifest = parse_team_manifest(VALID_MANIFEST).unwrap();
        assert_eq!(manifest.name, "test-team");
        assert_eq!(manifest.description, "A test team");
        assert_eq!(manifest.coordination, "wave-dispatch");
        assert_eq!(manifest.agents.len(), 2);
        assert_eq!(manifest.agents[0].name, "tdd-executor");
        assert_eq!(manifest.agents[0].role, "implementer");
        assert_eq!(manifest.max_concurrent, Some(2));
    }

    #[test]
    fn rejects_missing_frontmatter() {
        let content = "# No frontmatter\nJust a markdown file.";
        let result = parse_team_manifest(content);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, TeamValidationError::ParseError(_)));
    }

    #[test]
    fn rejects_unclosed_frontmatter() {
        let content = "---\nname: test\n# Missing closing ---";
        let result = parse_team_manifest(content);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_empty_agents() {
        let content = r#"---
name: empty-team
description: No agents
coordination: parallel
agents: []
---
"#;
        let manifest = parse_team_manifest(content).unwrap();
        let errors = validate_team_manifest(&manifest);
        assert!(errors.contains(&TeamValidationError::EmptyAgentsList));
    }

    #[test]
    fn rejects_unknown_strategy() {
        let content = r#"---
name: bad-team
description: Unknown strategy
coordination: round-robin
agents:
  - name: test-agent
    role: tester
---
"#;
        let manifest = parse_team_manifest(content).unwrap();
        let errors = validate_team_manifest(&manifest);
        assert!(
            errors.iter().any(
                |e| matches!(e, TeamValidationError::UnknownStrategy(s) if s == "round-robin")
            )
        );
    }

    #[test]
    fn rejects_duplicate_agent() {
        let content = r#"---
name: dup-team
description: Duplicate agents
coordination: parallel
agents:
  - name: same-agent
    role: first
  - name: same-agent
    role: second
---
"#;
        let manifest = parse_team_manifest(content).unwrap();
        let errors = validate_team_manifest(&manifest);
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, TeamValidationError::DuplicateAgent(n) if n == "same-agent"))
        );
    }

    #[test]
    fn rejects_zero_max_concurrent() {
        let content = r#"---
name: zero-team
description: Zero concurrent
coordination: parallel
max-concurrent: 0
agents:
  - name: test-agent
    role: tester
---
"#;
        let manifest = parse_team_manifest(content).unwrap();
        let errors = validate_team_manifest(&manifest);
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, TeamValidationError::InvalidMaxConcurrent(0)))
        );
    }

    #[test]
    fn valid_manifest_passes() {
        let manifest = parse_team_manifest(VALID_MANIFEST).unwrap();
        let errors = validate_team_manifest(&manifest);
        assert!(
            errors.is_empty(),
            "valid manifest should have no errors: {errors:?}"
        );
    }

    #[test]
    fn allowed_tools_defaults_none() {
        let content = r#"---
name: minimal-team
description: Minimal
coordination: sequential
agents:
  - name: test-agent
    role: tester
---
"#;
        let manifest = parse_team_manifest(content).unwrap();
        assert_eq!(
            manifest.agents[0].allowed_tools, None,
            "allowed_tools should default to None when omitted"
        );
    }
}
