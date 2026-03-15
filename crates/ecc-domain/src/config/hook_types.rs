//! Typed domain model for hooks.json structure.
//!
//! These types replace raw `serde_json::Value` in domain function signatures,
//! enabling compile-time type safety and hash-based deduplication.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A single hook command within a hook entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HookCommand {
    #[serde(rename = "type")]
    pub hook_type: Option<String>,
    pub command: Option<HookCommandValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#async: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<serde_json::Number>,
}

/// The command field can be a single string or an array of strings.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HookCommandValue {
    Single(String),
    Array(Vec<String>),
}

impl HookCommandValue {
    /// Extract the command string for pattern matching.
    /// For Single, returns the string; for Array, returns the first element.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Single(s) => Some(s),
            Self::Array(arr) => arr.first().map(String::as_str),
        }
    }

    /// Check if the command value is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Single(s) => s.trim().is_empty(),
            Self::Array(arr) => arr.is_empty(),
        }
    }

    /// Check if all entries in an array command are non-empty strings.
    pub fn all_entries_valid(&self) -> bool {
        match self {
            Self::Single(s) => !s.trim().is_empty(),
            Self::Array(arr) => !arr.is_empty() && arr.iter().all(|s| !s.is_empty()),
        }
    }
}

/// A hook entry with an optional matcher and a list of hook commands.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HookEntry {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matcher: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hooks: Option<Vec<HookCommand>>,
}

/// Event name → list of hook entries (e.g., "PreToolUse" → [...]).
pub type HooksMap = BTreeMap<String, Vec<HookEntry>>;

#[cfg(test)]
mod tests {
    use super::*;

    // --- HookCommandValue ---

    #[test]
    fn single_command_as_str() {
        let cmd = HookCommandValue::Single("echo hello".to_string());
        assert_eq!(cmd.as_str(), Some("echo hello"));
    }

    #[test]
    fn array_command_as_str_returns_first() {
        let cmd = HookCommandValue::Array(vec!["echo".to_string(), "hello".to_string()]);
        assert_eq!(cmd.as_str(), Some("echo"));
    }

    #[test]
    fn empty_array_as_str_returns_none() {
        let cmd = HookCommandValue::Array(vec![]);
        assert_eq!(cmd.as_str(), None);
    }

    #[test]
    fn single_is_empty() {
        assert!(HookCommandValue::Single("".to_string()).is_empty());
        assert!(HookCommandValue::Single("  ".to_string()).is_empty());
        assert!(!HookCommandValue::Single("echo".to_string()).is_empty());
    }

    #[test]
    fn array_is_empty() {
        assert!(HookCommandValue::Array(vec![]).is_empty());
        assert!(!HookCommandValue::Array(vec!["echo".to_string()]).is_empty());
    }

    #[test]
    fn all_entries_valid_single() {
        assert!(HookCommandValue::Single("echo".to_string()).all_entries_valid());
        assert!(!HookCommandValue::Single("".to_string()).all_entries_valid());
    }

    #[test]
    fn all_entries_valid_array() {
        assert!(
            HookCommandValue::Array(vec!["echo".to_string(), "hello".to_string()])
                .all_entries_valid()
        );
        assert!(
            !HookCommandValue::Array(vec!["echo".to_string(), "".to_string()])
                .all_entries_valid()
        );
        assert!(!HookCommandValue::Array(vec![]).all_entries_valid());
    }

    // --- HookCommand serde ---

    #[test]
    fn hook_command_deserialize_single() {
        let json = r#"{"type": "command", "command": "echo hello"}"#;
        let cmd: HookCommand = serde_json::from_str(json).unwrap();
        assert_eq!(cmd.hook_type.as_deref(), Some("command"));
        assert_eq!(cmd.command.as_ref().unwrap().as_str(), Some("echo hello"));
    }

    #[test]
    fn hook_command_deserialize_array() {
        let json = r#"{"type": "command", "command": ["echo", "hello"]}"#;
        let cmd: HookCommand = serde_json::from_str(json).unwrap();
        match &cmd.command {
            Some(HookCommandValue::Array(arr)) => assert_eq!(arr.len(), 2),
            _ => panic!("expected array command"),
        }
    }

    #[test]
    fn hook_command_with_async_and_timeout() {
        let json = r#"{"type": "command", "command": "echo", "async": true, "timeout": 5.0}"#;
        let cmd: HookCommand = serde_json::from_str(json).unwrap();
        assert_eq!(cmd.r#async, Some(true));
        assert!(cmd.timeout.is_some());
    }

    // --- HookEntry serde ---

    #[test]
    fn hook_entry_with_matcher() {
        let json = r#"{"matcher": "Bash", "hooks": [{"type": "command", "command": "echo"}]}"#;
        let entry: HookEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.matcher.as_deref(), Some("Bash"));
        assert_eq!(entry.hooks.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn hook_entry_without_matcher() {
        let json = r#"{"hooks": [{"type": "command", "command": "echo"}]}"#;
        let entry: HookEntry = serde_json::from_str(json).unwrap();
        assert!(entry.matcher.is_none());
    }

    // --- HookEntry equality for dedup ---

    #[test]
    fn hook_entry_equality_enables_dedup() {
        let entry1: HookEntry =
            serde_json::from_str(r#"{"hooks": [{"type": "command", "command": "ecc-hook format"}]}"#).unwrap();
        let entry2: HookEntry =
            serde_json::from_str(r#"{"hooks": [{"type": "command", "command": "ecc-hook format"}]}"#).unwrap();
        assert_eq!(entry1, entry2);
    }

    #[test]
    fn hook_entry_inequality_different_commands() {
        let entry1: HookEntry =
            serde_json::from_str(r#"{"hooks": [{"type": "command", "command": "ecc-hook format"}]}"#).unwrap();
        let entry2: HookEntry =
            serde_json::from_str(r#"{"hooks": [{"type": "command", "command": "ecc-hook lint"}]}"#).unwrap();
        assert_ne!(entry1, entry2);
    }

    // --- HooksMap serde roundtrip ---

    #[test]
    fn hooks_map_roundtrip() {
        let json = r#"{"PreToolUse": [{"hooks": [{"type": "command", "command": "ecc-hook format"}]}]}"#;
        let map: HooksMap = serde_json::from_str(json).unwrap();
        assert!(map.contains_key("PreToolUse"));

        let serialized = serde_json::to_string(&map).unwrap();
        let map2: HooksMap = serde_json::from_str(&serialized).unwrap();
        assert_eq!(map, map2);
    }

    #[test]
    fn hooks_map_empty() {
        let json = "{}";
        let map: HooksMap = serde_json::from_str(json).unwrap();
        assert!(map.is_empty());
    }
}
