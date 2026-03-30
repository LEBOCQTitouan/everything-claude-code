//! Deterministic report structure validation.

/// Error variants from report validation (blocking).
#[derive(Debug, PartialEq)]
pub enum ReportError {
    MissingSections(Vec<String>),
    ScoreOutOfRange { section: String, score: i32 },
}

/// Warning variants from report validation (non-blocking).
#[derive(Debug, PartialEq)]
pub enum ReportWarning {
    LowCitations { section: String, count: usize },
}

/// The result of validating a report.
#[derive(Debug)]
pub struct ReportValidationResult {
    pub errors: Vec<ReportError>,
    pub warnings: Vec<ReportWarning>,
}

impl ReportValidationResult {
    /// Returns true when there are no blocking errors.
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Required section headings every radar report must contain.
const REQUIRED_SECTIONS: &[&str] = &[
    "Techniques",
    "Tools",
    "Platforms",
    "Languages & Frameworks",
    "Feature Opportunities",
];

/// Validate a markdown report string for required sections, score ranges,
/// and citation counts.
pub fn validate_report(content: &str) -> ReportValidationResult {
    use regex::Regex;

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Check for required sections (## Section Name)
    let missing: Vec<String> = REQUIRED_SECTIONS
        .iter()
        .filter(|&&section| {
            !content
                .lines()
                .any(|line| line.trim_start_matches('#').trim() == section)
        })
        .map(|s| (*s).to_owned())
        .collect();

    if !missing.is_empty() {
        errors.push(ReportError::MissingSections(missing));
    }

    // Parse scores: **Strategic Fit**: N/5
    static SCORE_RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let score_re =
        SCORE_RE.get_or_init(|| Regex::new(r"\*\*Strategic Fit\*\*:\s*(-?\d+)/5").expect("valid regex"));

    // Parse section headers to associate scores with sections
    static SECTION_RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let section_re =
        SECTION_RE.get_or_init(|| Regex::new(r"^#{1,6}\s+(.+)$").expect("valid regex"));

    // Parse citation links: [text](url)
    static LINK_RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let link_re = LINK_RE.get_or_init(|| Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").expect("valid regex"));

    let mut current_section = String::new();
    let mut section_score: Option<i32> = None;
    let mut section_citations: usize = 0;

    let flush_section =
        |section: &str, score: Option<i32>, citations: usize, errors: &mut Vec<ReportError>, warnings: &mut Vec<ReportWarning>| {
            if let Some(s) = score
                && !(0..=5_i32).contains(&s)
            {
                errors.push(ReportError::ScoreOutOfRange {
                    section: section.to_owned(),
                    score: s,
                });
            }
            let is_required = REQUIRED_SECTIONS.contains(&section);
            if is_required && citations < 3 {
                warnings.push(ReportWarning::LowCitations {
                    section: section.to_owned(),
                    count: citations,
                });
            }
        };

    for line in content.lines() {
        if let Some(caps) = section_re.captures(line) {
            // Flush previous section
            flush_section(&current_section, section_score, section_citations, &mut errors, &mut warnings);
            current_section = caps[1].trim().to_owned();
            section_score = None;
            section_citations = 0;
        } else if let Some(caps) = score_re.captures(line) {
            let score: i32 = caps[1].parse().unwrap_or(-999);
            section_score = Some(score);
        } else {
            section_citations += link_re.find_iter(line).count();
        }
    }
    // Flush final section
    flush_section(&current_section, section_score, section_citations, &mut errors, &mut warnings);

    ReportValidationResult { errors, warnings }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_report() -> String {
        r#"# Audit Web Report

## Techniques

**Strategic Fit**: 4/5
[source one](https://example.com/1)
[source two](https://example.com/2)
[source three](https://example.com/3)

## Tools

**Strategic Fit**: 3/5
[tool source 1](https://example.com/4)
[tool source 2](https://example.com/5)
[tool source 3](https://example.com/6)

## Platforms

**Strategic Fit**: 5/5
[platform ref 1](https://example.com/7)
[platform ref 2](https://example.com/8)
[platform ref 3](https://example.com/9)

## Languages & Frameworks

**Strategic Fit**: 4/5
[lang ref 1](https://example.com/10)
[lang ref 2](https://example.com/11)
[lang ref 3](https://example.com/12)

## Feature Opportunities

**Strategic Fit**: 2/5
[feature ref 1](https://example.com/13)
[feature ref 2](https://example.com/14)
[feature ref 3](https://example.com/15)
"#
        .to_owned()
    }

    #[test]
    fn valid_report_passes() {
        let result = validate_report(&valid_report());
        assert!(
            result.is_valid(),
            "expected valid report to pass, errors: {:?}",
            result.errors
        );
        assert!(
            result.warnings.is_empty(),
            "expected no warnings, got: {:?}",
            result.warnings
        );
    }

    #[test]
    fn missing_sections_error() {
        let report_without_feature_opportunities = r#"# Audit Web Report

## Techniques

**Strategic Fit**: 4/5
[source one](https://example.com/1)
[source two](https://example.com/2)
[source three](https://example.com/3)

## Tools

**Strategic Fit**: 3/5
[tool source 1](https://example.com/4)
[tool source 2](https://example.com/5)
[tool source 3](https://example.com/6)

## Platforms

**Strategic Fit**: 5/5
[platform ref 1](https://example.com/7)
[platform ref 2](https://example.com/8)
[platform ref 3](https://example.com/9)

## Languages & Frameworks

**Strategic Fit**: 4/5
[lang ref 1](https://example.com/10)
[lang ref 2](https://example.com/11)
[lang ref 3](https://example.com/12)
"#;
        let result = validate_report(report_without_feature_opportunities);
        assert!(
            !result.is_valid(),
            "expected validation to fail for missing section"
        );
        let has_missing_section_error = result.errors.iter().any(|e| {
            matches!(e, ReportError::MissingSections(sections) if sections.iter().any(|s| s.contains("Feature Opportunities")))
        });
        assert!(
            has_missing_section_error,
            "expected MissingSections error listing 'Feature Opportunities', got: {:?}",
            result.errors
        );
    }

    #[test]
    fn score_out_of_range() {
        let report_with_bad_score = r#"# Audit Web Report

## Techniques

**Strategic Fit**: 6/5
[source one](https://example.com/1)
[source two](https://example.com/2)
[source three](https://example.com/3)

## Tools

**Strategic Fit**: 3/5
[tool source 1](https://example.com/4)
[tool source 2](https://example.com/5)
[tool source 3](https://example.com/6)

## Platforms

**Strategic Fit**: 5/5
[platform ref 1](https://example.com/7)
[platform ref 2](https://example.com/8)
[platform ref 3](https://example.com/9)

## Languages & Frameworks

**Strategic Fit**: 4/5
[lang ref 1](https://example.com/10)
[lang ref 2](https://example.com/11)
[lang ref 3](https://example.com/12)

## Feature Opportunities

**Strategic Fit**: 2/5
[feature ref 1](https://example.com/13)
[feature ref 2](https://example.com/14)
[feature ref 3](https://example.com/15)
"#;
        let result = validate_report(report_with_bad_score);
        assert!(
            !result.is_valid(),
            "expected validation to fail for score out of range"
        );
        let has_score_error = result
            .errors
            .iter()
            .any(|e| matches!(e, ReportError::ScoreOutOfRange { score, .. } if *score == 6));
        assert!(
            has_score_error,
            "expected ScoreOutOfRange error with score=6, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn low_citation_warning() {
        let report_with_few_citations = r#"# Audit Web Report

## Techniques

**Strategic Fit**: 4/5
[only one source](https://example.com/1)
[only two sources](https://example.com/2)

## Tools

**Strategic Fit**: 3/5
[tool source 1](https://example.com/4)
[tool source 2](https://example.com/5)
[tool source 3](https://example.com/6)

## Platforms

**Strategic Fit**: 5/5
[platform ref 1](https://example.com/7)
[platform ref 2](https://example.com/8)
[platform ref 3](https://example.com/9)

## Languages & Frameworks

**Strategic Fit**: 4/5
[lang ref 1](https://example.com/10)
[lang ref 2](https://example.com/11)
[lang ref 3](https://example.com/12)

## Feature Opportunities

**Strategic Fit**: 2/5
[feature ref 1](https://example.com/13)
[feature ref 2](https://example.com/14)
[feature ref 3](https://example.com/15)
"#;
        let result = validate_report(report_with_few_citations);
        assert!(
            result.is_valid(),
            "low citations should be a warning, not an error: {:?}",
            result.errors
        );
        let has_warning = result.warnings.iter().any(|w| {
            matches!(w, ReportWarning::LowCitations { count, .. } if *count < 3)
        });
        assert!(
            has_warning,
            "expected LowCitations warning, got: {:?}",
            result.warnings
        );
    }
}
