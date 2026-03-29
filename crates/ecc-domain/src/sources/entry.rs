//! Source entry domain types — pure types and validation logic.

use std::fmt;
use std::str::FromStr;

/// Type of knowledge source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceType {
    Repo,
    Doc,
    Blog,
    Package,
    Talk,
    Paper,
}

/// Technology Radar quadrant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Quadrant {
    Adopt,
    Trial,
    Assess,
    Hold,
}

/// A curated knowledge source entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceEntry {
    pub url: String,
    pub title: String,
    pub source_type: SourceType,
    pub quadrant: Quadrant,
    pub subject: String,
    pub added_by: String,
    pub added_date: String,
    pub last_checked: Option<String>,
    pub deprecation_reason: Option<String>,
    pub stale: bool,
}

/// Errors from sources domain operations.
#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("URL must be a valid URL format: {0}")]
    InvalidUrl(String),
    #[error("title must not be empty")]
    EmptyTitle,
    #[error("parse error at line {line}: {message}")]
    ParseError { line: usize, message: String },
    #[error("duplicate URL: {0}")]
    DuplicateUrl(String),
    #[error("unknown source type: {0}")]
    UnknownSourceType(String),
    #[error("unknown quadrant: {0}")]
    UnknownQuadrant(String),
}

impl fmt::Display for SourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Repo => "repo",
            Self::Doc => "doc",
            Self::Blog => "blog",
            Self::Package => "package",
            Self::Talk => "talk",
            Self::Paper => "paper",
        };
        write!(f, "{s}")
    }
}

impl FromStr for SourceType {
    type Err = SourceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "repo" => Ok(Self::Repo),
            "doc" => Ok(Self::Doc),
            "blog" => Ok(Self::Blog),
            "package" => Ok(Self::Package),
            "talk" => Ok(Self::Talk),
            "paper" => Ok(Self::Paper),
            _ => Err(SourceError::UnknownSourceType(s.to_owned())),
        }
    }
}

impl fmt::Display for Quadrant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Adopt => "adopt",
            Self::Trial => "trial",
            Self::Assess => "assess",
            Self::Hold => "hold",
        };
        write!(f, "{s}")
    }
}

impl FromStr for Quadrant {
    type Err = SourceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "adopt" => Ok(Self::Adopt),
            "trial" => Ok(Self::Trial),
            "assess" => Ok(Self::Assess),
            "hold" => Ok(Self::Hold),
            _ => Err(SourceError::UnknownQuadrant(s.to_owned())),
        }
    }
}

/// Validate URL structure (must start with http:// or https:// and contain a dot).
///
/// This is a domain rule — no I/O, no network calls.
pub fn validate_url(url: &str) -> Result<(), SourceError> {
    let has_valid_scheme = url.starts_with("http://") || url.starts_with("https://");
    let has_dot = url.contains('.');
    if has_valid_scheme && has_dot {
        Ok(())
    } else {
        Err(SourceError::InvalidUrl(url.to_owned()))
    }
}

/// Validate title is non-empty after trimming whitespace.
pub fn validate_title(title: &str) -> Result<(), SourceError> {
    if title.trim().is_empty() {
        Err(SourceError::EmptyTitle)
    } else {
        Ok(())
    }
}

impl SourceEntry {
    /// Returns true when the entry has been deprecated.
    pub fn is_deprecated(&self) -> bool {
        self.deprecation_reason.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_construction() {
        let entry = SourceEntry {
            url: "https://example.com".to_owned(),
            title: "Example Source".to_owned(),
            source_type: SourceType::Doc,
            quadrant: Quadrant::Adopt,
            subject: "testing".to_owned(),
            added_by: "human".to_owned(),
            added_date: "2026-03-29".to_owned(),
            last_checked: Some("2026-03-29".to_owned()),
            deprecation_reason: None,
            stale: false,
        };
        assert_eq!(entry.url, "https://example.com");
        assert_eq!(entry.title, "Example Source");
        assert_eq!(entry.source_type, SourceType::Doc);
        assert_eq!(entry.quadrant, Quadrant::Adopt);
        assert_eq!(entry.subject, "testing");
        assert_eq!(entry.added_by, "human");
        assert_eq!(entry.added_date, "2026-03-29");
        assert_eq!(entry.last_checked, Some("2026-03-29".to_owned()));
        assert!(entry.deprecation_reason.is_none());
        assert!(!entry.stale);
    }

    #[test]
    fn source_type_all_variants_from_str() {
        assert_eq!(SourceType::from_str("repo").unwrap(), SourceType::Repo);
        assert_eq!(SourceType::from_str("doc").unwrap(), SourceType::Doc);
        assert_eq!(SourceType::from_str("blog").unwrap(), SourceType::Blog);
        assert_eq!(
            SourceType::from_str("package").unwrap(),
            SourceType::Package
        );
        assert_eq!(SourceType::from_str("talk").unwrap(), SourceType::Talk);
        assert_eq!(SourceType::from_str("paper").unwrap(), SourceType::Paper);
        assert!(SourceType::from_str("unknown").is_err());
    }

    #[test]
    fn source_type_display() {
        assert_eq!(SourceType::Repo.to_string(), "repo");
        assert_eq!(SourceType::Doc.to_string(), "doc");
        assert_eq!(SourceType::Blog.to_string(), "blog");
        assert_eq!(SourceType::Package.to_string(), "package");
        assert_eq!(SourceType::Talk.to_string(), "talk");
        assert_eq!(SourceType::Paper.to_string(), "paper");
    }

    #[test]
    fn quadrant_all_variants_from_str() {
        assert_eq!(Quadrant::from_str("adopt").unwrap(), Quadrant::Adopt);
        assert_eq!(Quadrant::from_str("trial").unwrap(), Quadrant::Trial);
        assert_eq!(Quadrant::from_str("assess").unwrap(), Quadrant::Assess);
        assert_eq!(Quadrant::from_str("hold").unwrap(), Quadrant::Hold);
        assert!(Quadrant::from_str("unknown").is_err());
    }

    #[test]
    fn quadrant_display() {
        assert_eq!(Quadrant::Adopt.to_string(), "adopt");
        assert_eq!(Quadrant::Trial.to_string(), "trial");
        assert_eq!(Quadrant::Assess.to_string(), "assess");
        assert_eq!(Quadrant::Hold.to_string(), "hold");
    }

    #[test]
    fn validate_url_accepts_http() {
        assert!(validate_url("http://example.com").is_ok());
    }

    #[test]
    fn validate_url_accepts_https() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("https://docs.rust-lang.org/std/").is_ok());
    }

    #[test]
    fn validate_url_rejects_invalid() {
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("not-a-url").is_err());
        assert!(validate_url("").is_err());
        assert!(validate_url("https://nodot").is_err());
    }

    #[test]
    fn validate_title_accepts_non_empty() {
        assert!(validate_title("My Source").is_ok());
        assert!(validate_title("  Some Title  ").is_ok());
    }

    #[test]
    fn validate_title_rejects_empty() {
        assert!(validate_title("").is_err());
        assert!(validate_title("   ").is_err());
        assert!(validate_title("\t\n").is_err());
    }

    #[test]
    fn deprecated_lifecycle() {
        let active = SourceEntry {
            url: "https://example.com".to_owned(),
            title: "Active".to_owned(),
            source_type: SourceType::Repo,
            quadrant: Quadrant::Adopt,
            subject: "testing".to_owned(),
            added_by: "human".to_owned(),
            added_date: "2026-01-01".to_owned(),
            last_checked: None,
            deprecation_reason: None,
            stale: false,
        };
        assert!(!active.is_deprecated());

        let deprecated = SourceEntry {
            deprecation_reason: Some("superseded by newer library".to_owned()),
            ..active
        };
        assert!(deprecated.is_deprecated());
    }

    #[test]
    fn error_variants() {
        let errors: Vec<SourceError> = vec![
            SourceError::InvalidUrl("bad-url".to_owned()),
            SourceError::EmptyTitle,
            SourceError::ParseError {
                line: 1,
                message: "oops".to_owned(),
            },
            SourceError::DuplicateUrl("https://example.com".to_owned()),
            SourceError::UnknownSourceType("pdf".to_owned()),
            SourceError::UnknownQuadrant("maybe".to_owned()),
        ];
        assert_eq!(errors.len(), 6);
        for err in &errors {
            assert!(!format!("{err}").is_empty());
        }
    }
}
