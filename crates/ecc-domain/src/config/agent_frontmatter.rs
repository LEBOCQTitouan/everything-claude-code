//! Frontmatter validation types for agent and hook content files.
//!
//! These types implement the `Validatable` trait for agent and hook frontmatter
//! parsed from markdown files in the ECC configuration.

use crate::config::validate::VALID_MODELS;
use crate::traits::Validatable;

/// Parsed frontmatter fields for an agent file.
///
/// Implements `Validatable<String>` — errors are human-readable strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentFrontmatter {
    pub name: Option<String>,
    pub description: Option<String>,
    pub model: Option<String>,
    pub tools: Option<Vec<String>>,
}

/// Parsed frontmatter fields for a hook entry.
///
/// Implements `Validatable<String>` — errors are human-readable strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookFrontmatter {
    pub hook_type: Option<String>,
    pub command: Option<String>,
}

impl Validatable<String> for AgentFrontmatter {
    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.name.as_deref().map(str::is_empty).unwrap_or(true) {
            errors.push("agent 'name' field is missing or empty".to_string());
        }

        if self.description.as_deref().map(str::is_empty).unwrap_or(true) {
            errors.push("agent 'description' field is missing or empty".to_string());
        }

        match &self.model {
            Some(m) if VALID_MODELS.contains(&m.as_str()) => {}
            Some(m) => errors.push(format!(
                "agent 'model' value '{m}' is not valid; must be one of: {}",
                VALID_MODELS.join(", ")
            )),
            None => errors.push("agent 'model' field is missing".to_string()),
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

impl Validatable<String> for HookFrontmatter {
    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        match &self.hook_type {
            Some(t) if !t.is_empty() => {}
            _ => errors.push("hook 'type' field is missing or empty".to_string()),
        }

        match &self.command {
            Some(c) if !c.trim().is_empty() => {}
            _ => errors.push("hook 'command' field is missing or empty".to_string()),
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_agent_passes() {
        let a = AgentFrontmatter {
            name: Some("my-agent".to_string()),
            description: Some("Does stuff".to_string()),
            model: Some("opus".to_string()),
            tools: Some(vec!["Read".to_string()]),
        };
        assert!(a.validate().is_ok());
    }

    #[test]
    fn agent_missing_name_reports_error() {
        let a = AgentFrontmatter {
            name: None,
            description: Some("Desc".to_string()),
            model: Some("sonnet".to_string()),
            tools: None,
        };
        let errs = a.validate().unwrap_err();
        assert!(errs.iter().any(|e| e.contains("name")));
    }

    #[test]
    fn agent_invalid_model_reports_error() {
        let a = AgentFrontmatter {
            name: Some("agent".to_string()),
            description: Some("desc".to_string()),
            model: Some("gpt-4".to_string()),
            tools: None,
        };
        let errs = a.validate().unwrap_err();
        assert!(errs.iter().any(|e| e.contains("model")));
    }

    #[test]
    fn valid_hook_passes() {
        let h = HookFrontmatter {
            hook_type: Some("command".to_string()),
            command: Some("echo ok".to_string()),
        };
        assert!(h.validate().is_ok());
    }

    #[test]
    fn hook_missing_command_reports_error() {
        let h = HookFrontmatter {
            hook_type: Some("command".to_string()),
            command: None,
        };
        let errs = h.validate().unwrap_err();
        assert!(errs.iter().any(|e| e.contains("command")));
    }
}
