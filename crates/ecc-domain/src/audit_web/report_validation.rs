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

/// Validate a markdown report string for required sections, score ranges,
/// and citation counts.
pub fn validate_report(content: &str) -> ReportValidationResult {
    todo!("validate_report not implemented")
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
