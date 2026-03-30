//! Secret detection for memory content validation.
//!
//! Detects common secret patterns before storing content in the memory system.
//! This is a defense-in-depth measure; it is not a complete secrets scanner.

use regex::Regex;

/// Patterns that indicate likely secrets in content.
///
/// Returns the kind of detected secret if found, or `None` if clean.
pub fn contains_likely_secret(content: &str) -> Option<String> {
    let patterns: &[(&str, &str)] = &[
        // OpenAI / Anthropic API keys
        (r"sk-[A-Za-z0-9]{20,}", "OpenAI-style API key (sk-)"),
        // GitHub personal access tokens
        (r"ghp_[A-Za-z0-9]{36,}", "GitHub PAT (ghp_)"),
        // AWS access key IDs
        (r"AKIA[0-9A-Z]{16}", "AWS Access Key ID"),
        // Generic bearer tokens (long base64-ish strings after 'Bearer ')
        (r"(?i)bearer\s+[A-Za-z0-9+/=_\-]{20,}", "Bearer token"),
        // Connection strings with passwords
        (r"(?i)://[^@\s]+:[^@\s]+@[^@\s]+", "Connection string with credentials"),
        // Slack tokens
        (r"xox[baprs]-[A-Za-z0-9\-]{10,}", "Slack token"),
    ];

    for (pattern, kind) in patterns {
        let re = match Regex::new(pattern) {
            Ok(r) => r,
            Err(_) => continue,
        };
        if re.is_match(content) {
            return Some(kind.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // PC-088: `contains_likely_secret` detects API keys, connection strings, bearer tokens
    #[test]
    fn test_detects_openai_api_key() {
        let content = "My API key is sk-abc123def456ghi789jkl000";
        let result = contains_likely_secret(content);
        assert!(result.is_some(), "should detect sk- key");
        assert!(result.unwrap().contains("sk-"));
    }

    #[test]
    fn test_detects_github_pat() {
        let content = "Token: ghp_abcdefghijklmnopqrstuvwxyz12345678901";
        let result = contains_likely_secret(content);
        assert!(result.is_some(), "should detect GitHub PAT");
    }

    #[test]
    fn test_detects_aws_access_key() {
        let content = "AWS key: AKIAIOSFODNN7EXAMPLE";
        let result = contains_likely_secret(content);
        assert!(result.is_some(), "should detect AWS key");
    }

    #[test]
    fn test_detects_bearer_token() {
        let content = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let result = contains_likely_secret(content);
        assert!(result.is_some(), "should detect Bearer token");
    }

    #[test]
    fn test_detects_connection_string() {
        let content = "DB: postgres://user:password@localhost:5432/mydb";
        let result = contains_likely_secret(content);
        assert!(result.is_some(), "should detect connection string with credentials");
    }

    #[test]
    fn test_clean_content_returns_none() {
        let content = "This is regular content about Rust programming patterns.";
        let result = contains_likely_secret(content);
        assert!(result.is_none(), "clean content should not be flagged");
    }

    #[test]
    fn test_short_sk_prefix_not_flagged() {
        // "sk-" alone or very short — not a real API key
        let content = "The sk-abc entry is short";
        let result = contains_likely_secret(content);
        // Short enough to not match the 20+ character requirement
        assert!(result.is_none(), "short sk- prefix should not be flagged");
    }

    #[test]
    fn test_detects_slack_token() {
        let content = "token=xoxb-1234567890-abcdefghij-klmnopqrst";
        let result = contains_likely_secret(content);
        assert!(result.is_some(), "should detect Slack token");
    }
}
