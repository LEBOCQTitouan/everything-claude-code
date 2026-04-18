//! Source entry domain types — pure types and validation logic.

use std::fmt;
use std::str::FromStr;

/// Type of knowledge source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceType {
    /// A code repository.
    Repo,
    /// Documentation or guide.
    Doc,
    /// Blog post or article.
    Blog,
    /// Software package or library.
    Package,
    /// Recorded talk or presentation.
    Talk,
    /// Academic or technical paper.
    Paper,
}

/// Technology Radar quadrant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Quadrant {
    /// Ready for production use.
    Adopt,
    /// Recommended for experimental use on low-risk projects.
    Trial,
    /// Worth evaluating but not ready for mainstream use.
    Assess,
    /// No longer recommended; being phased out.
    Hold,
}

/// A validated URL value object for knowledge sources.
///
/// Enforces that the URL has a valid HTTP/HTTPS scheme and contains a dot.
/// Invalid URLs cannot be represented — construction fails at parse time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceUrl(
    /// The inner URL string.
    String,
);

impl SourceUrl {
    /// Parse and validate a URL string.
    ///
    /// Succeeds for `http://` and `https://` URLs containing a dot.
    /// Rejects empty strings, missing schemes, non-HTTP schemes, and host-only URLs without a dot.
    ///
    /// # Arguments
    ///
    /// * `raw` — The URL string to validate.
    ///
    /// # Returns
    ///
    /// `Ok(SourceUrl)` if valid, or `Err(SourceError::InvalidUrl)`.
    pub fn parse(raw: &str) -> Result<Self, SourceError> {
        let has_valid_scheme = raw.starts_with("http://") || raw.starts_with("https://");
        let has_dot = raw.contains('.');
        if has_valid_scheme && has_dot {
            Ok(Self(raw.to_owned()))
        } else {
            Err(SourceError::InvalidUrl(raw.to_owned()))
        }
    }

    /// Return the inner URL string.
    ///
    /// # Returns
    ///
    /// The URL as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SourceUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A curated knowledge source entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceEntry {
    /// The validated URL of the source.
    pub url: SourceUrl,
    /// The title of the source.
    pub title: String,
    /// The type of source.
    pub source_type: SourceType,
    /// The Technology Radar quadrant.
    pub quadrant: Quadrant,
    /// Subject or domain category.
    pub subject: String,
    /// The user who added this source.
    pub added_by: String,
    /// ISO 8601 date when this source was added.
    pub added_date: String,
    /// ISO 8601 date when this source was last verified to be accessible.
    pub last_checked: Option<String>,
    /// Reason for deprecation, if applicable.
    pub deprecation_reason: Option<String>,
    /// True if the source is no longer actively maintained.
    pub stale: bool,
}

/// Errors from sources domain operations.
#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    /// The URL is invalid or does not match the required format.
    #[error("URL must be a valid URL format: {0}")]
    InvalidUrl(String),
    /// The title is empty after trimming whitespace.
    #[error("title must not be empty")]
    EmptyTitle,
    /// An error occurred while parsing source data.
    #[error("parse error at line {line}: {message}")]
    ParseError {
        /// The line number where the error occurred.
        line: usize,
        /// Description of the parse error.
        message: String,
    },
    /// The URL is already present in the sources collection.
    #[error("duplicate URL: {0}")]
    DuplicateUrl(String),
    /// The source type string is not recognized.
    #[error("unknown source type: {0}")]
    UnknownSourceType(String),
    /// The quadrant string is not recognized.
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

/// Validate title is non-empty after trimming whitespace.
///
/// # Arguments
///
/// * `title` — The title to validate.
///
/// # Returns
///
/// `Ok(())` if the title is non-empty, or `Err(SourceError::EmptyTitle)`.
pub fn validate_title(title: &str) -> Result<(), SourceError> {
    if title.trim().is_empty() {
        Err(SourceError::EmptyTitle)
    } else {
        Ok(())
    }
}

impl SourceEntry {
    /// Returns true when the entry has been deprecated.
    ///
    /// # Returns
    ///
    /// True if `deprecation_reason` is set.
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
            url: SourceUrl::parse("https://example.com").unwrap(),
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
        assert_eq!(entry.url.as_str(), "https://example.com");
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

    // --- PC-001: SourceUrl::parse valid HTTPS ---
    #[test]
    fn source_url_parse_valid_https() {
        let url = SourceUrl::parse("https://example.com").unwrap();
        assert_eq!(url.as_str(), "https://example.com");
    }

    // --- PC-002: SourceUrl::parse valid HTTP ---
    #[test]
    fn source_url_parse_valid_http() {
        let url = SourceUrl::parse("http://example.com").unwrap();
        assert_eq!(url.as_str(), "http://example.com");
    }

    // --- PC-003: SourceUrl::parse rejects no-scheme ---
    #[test]
    fn source_url_parse_rejects_no_scheme() {
        assert!(SourceUrl::parse("not-a-url").is_err());
        assert!(SourceUrl::parse("example.com").is_err());
    }

    // --- PC-004: SourceUrl::parse rejects empty ---
    #[test]
    fn source_url_parse_rejects_empty() {
        assert!(SourceUrl::parse("").is_err());
    }

    // --- PC-005: SourceUrl::as_str returns inner ---
    #[test]
    fn source_url_as_str() {
        let url = SourceUrl::parse("https://docs.rust-lang.org/std/").unwrap();
        assert_eq!(url.as_str(), "https://docs.rust-lang.org/std/");
    }

    // --- PC-006: SourceUrl::parse rejects ftp:// ---
    #[test]
    fn source_url_parse_rejects_ftp() {
        assert!(SourceUrl::parse("ftp://example.com").is_err());
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
            url: SourceUrl::parse("https://example.com").unwrap(),
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
