//! Shared helpers for tier 2 tool hooks.

use crate::hook::HookPorts;
use std::path::Path;

/// Error from file path validation.
#[derive(Debug, thiserror::Error)]
pub enum PathValidationError {
    /// File path starts with a dash, which could be interpreted as a flag.
    #[error("validate_file_path: rejected path starting with dash: {path}. Remove the leading dash.")]
    DashPrefix { path: String },
}

/// Reject file paths that start with `-` to prevent flag injection.
pub(super) fn validate_file_path(path: &str) -> Result<(), PathValidationError> {
    if path.starts_with('-') {
        return Err(PathValidationError::DashPrefix {
            path: path.to_owned(),
        });
    }
    Ok(())
}

/// Extract `tool_input.file_path` from stdin JSON.
pub(super) fn extract_file_path(stdin: &str) -> String {
    serde_json::from_str::<serde_json::Value>(stdin)
        .ok()
        .and_then(|v| {
            v.get("tool_input")
                .and_then(|ti| ti.get("file_path"))
                .and_then(|c| c.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_default()
}

/// Extract `tool_input.command` from stdin JSON.
pub(super) fn extract_command(stdin: &str) -> String {
    serde_json::from_str::<serde_json::Value>(stdin)
        .ok()
        .and_then(|v| {
            v.get("tool_input")
                .and_then(|ti| ti.get("command"))
                .and_then(|c| c.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_default()
}

/// Split a shell command into segments by ; && || &
pub(super) fn split_shell_segments(command: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let chars: Vec<char> = command.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        if let Some(q) = quote {
            if ch == q {
                quote = None;
            }
            current.push(ch);
            i += 1;
            continue;
        }

        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            current.push(ch);
            i += 1;
            continue;
        }

        let next = chars.get(i + 1).copied().unwrap_or('\0');

        if ch == ';' || ch == '&' || (ch == '|' && next == '|') {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                segments.push(trimmed);
            }
            current.clear();
            if (ch == '&' && next == '&') || (ch == '|' && next == '|') {
                i += 1;
            }
            i += 1;
            continue;
        }

        current.push(ch);
        i += 1;
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        segments.push(trimmed);
    }
    segments
}

/// Find project root by walking up from a file path looking for package.json.
pub(super) fn find_project_root(file_path: &Path, ports: &HookPorts<'_>) -> std::path::PathBuf {
    let mut dir = file_path.parent().unwrap_or(file_path).to_path_buf();
    for _ in 0..20 {
        if ports.fs.exists(&dir.join("package.json")) {
            return dir;
        }
        match dir.parent() {
            Some(p) if p != dir => dir = p.to_path_buf(),
            _ => break,
        }
    }
    file_path.parent().unwrap_or(file_path).to_path_buf()
}

/// Find an ancestor directory containing a specific file.
pub(super) fn find_ancestor_with(
    file_path: &Path,
    target: &str,
    ports: &HookPorts<'_>,
) -> Option<std::path::PathBuf> {
    let mut dir = file_path.parent()?.to_path_buf();
    for _ in 0..20 {
        if ports.fs.exists(&dir.join(target)) {
            return Some(dir);
        }
        match dir.parent() {
            Some(p) if p != dir => dir = p.to_path_buf(),
            _ => break,
        }
    }
    None
}

/// Detect the configured formatter (biome or prettier) in a project.
pub(super) fn detect_formatter(project_root: &Path, ports: &HookPorts<'_>) -> Option<String> {
    let biome_configs = ["biome.json", "biome.jsonc"];
    for cfg in &biome_configs {
        if ports.fs.exists(&project_root.join(cfg)) {
            return Some("biome".to_string());
        }
    }

    let prettier_configs = [
        ".prettierrc",
        ".prettierrc.json",
        ".prettierrc.js",
        ".prettierrc.cjs",
        ".prettierrc.mjs",
        ".prettierrc.yml",
        ".prettierrc.yaml",
        ".prettierrc.toml",
        "prettier.config.js",
        "prettier.config.cjs",
        "prettier.config.mjs",
    ];
    for cfg in &prettier_configs {
        if ports.fs.exists(&project_root.join(cfg)) {
            return Some("prettier".to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_simple_command() {
        let segs = split_shell_segments("echo hello");
        assert_eq!(segs, vec!["echo hello"]);
    }

    #[test]
    fn split_and_chain() {
        let segs = split_shell_segments("npm run build && npm test");
        assert_eq!(segs, vec!["npm run build", "npm test"]);
    }

    #[test]
    fn split_semicolon() {
        let segs = split_shell_segments("cd dir; npm run dev");
        assert_eq!(segs, vec!["cd dir", "npm run dev"]);
    }

    #[test]
    fn split_preserves_quotes() {
        let segs = split_shell_segments(r#"echo "hello && world""#);
        assert_eq!(segs, vec![r#"echo "hello && world""#]);
    }
}
