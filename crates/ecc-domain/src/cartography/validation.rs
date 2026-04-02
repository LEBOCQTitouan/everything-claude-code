//! Schema validation for cartography journey and flow files.
//!
//! Pure string validation -- checks for required markdown headers.
//! Zero I/O.

/// Validates a journey file's required sections.
///
/// Required sections: `## Overview`, `## Mermaid Diagram`, `## Steps`, `## Related Flows`
///
/// Returns `Ok(())` if all required sections are present, or `Err(missing)` where
/// `missing` is a list of section names that are absent.
pub fn validate_journey(content: &str) -> Result<(), Vec<String>> {
    todo!("implement journey validation")
}

/// Validates a flow file's required sections.
///
/// Required sections: `## Overview`, `## Mermaid Diagram`, `## Source-Destination`,
/// `## Transformation Steps`, `## Error Paths`
///
/// Returns `Ok(())` if all required sections are present, or `Err(missing)` where
/// `missing` is a list of section names that are absent.
pub fn validate_flow(content: &str) -> Result<(), Vec<String>> {
    todo!("implement flow validation")
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // validate_journey
    // -----------------------------------------------------------------------

    #[test]
    fn journey_passes_when_all_required_sections_present() {
        let content = "# My Journey

## Overview
Some actor does something.

## Mermaid Diagram

## Steps
1. First step
2. Second step

## Related Flows
- related-flow
";
        assert!(validate_journey(content).is_ok());
    }

    #[test]
    fn journey_reports_missing_section_name_when_overview_absent() {
        let content = "# My Journey

## Mermaid Diagram

## Steps
1. First step

## Related Flows
- related-flow
";
        let err = validate_journey(content).unwrap_err();
        assert!(
            err.iter().any(|s| s.contains("Overview")),
            "expected 'Overview' in errors, got: {:?}",
            err
        );
    }

    #[test]
    fn journey_reports_missing_section_name_when_mermaid_diagram_absent() {
        let content = "# My Journey

## Overview
Some actor does something.

## Steps
1. First step

## Related Flows
- related-flow
";
        let err = validate_journey(content).unwrap_err();
        assert!(
            err.iter().any(|s| s.contains("Mermaid Diagram")),
            "expected 'Mermaid Diagram' in errors, got: {:?}",
            err
        );
    }

    #[test]
    fn journey_reports_missing_section_name_when_steps_absent() {
        let content = "# My Journey

## Overview
Some actor does something.

## Mermaid Diagram

## Related Flows
- related-flow
";
        let err = validate_journey(content).unwrap_err();
        assert!(
            err.iter().any(|s| s.contains("Steps")),
            "expected 'Steps' in errors, got: {:?}",
            err
        );
    }

    #[test]
    fn journey_reports_missing_section_name_when_related_flows_absent() {
        let content = "# My Journey

## Overview
Some actor does something.

## Mermaid Diagram

## Steps
1. First step
";
        let err = validate_journey(content).unwrap_err();
        assert!(
            err.iter().any(|s| s.contains("Related Flows")),
            "expected 'Related Flows' in errors, got: {:?}",
            err
        );
    }

    #[test]
    fn journey_reports_all_missing_sections_when_all_absent() {
        let content = "# My Journey

No required sections here.
";
        let err = validate_journey(content).unwrap_err();
        assert_eq!(err.len(), 4, "expected 4 missing sections, got: {:?}", err);
    }

    // -----------------------------------------------------------------------
    // validate_flow
    // -----------------------------------------------------------------------

    #[test]
    fn flow_passes_when_all_required_sections_present() {
        let content = "# My Flow

## Overview
Describes the data flow.

## Mermaid Diagram

## Source-Destination
Source: Service A
Destination: Service B

## Transformation Steps
1. Transform input

## Error Paths
- On failure: retry
";
        assert!(validate_flow(content).is_ok());
    }

    #[test]
    fn flow_reports_missing_section_name_when_overview_absent() {
        let content = "# My Flow

## Mermaid Diagram

## Source-Destination
Source: Service A

## Transformation Steps
1. Transform input

## Error Paths
- On failure: retry
";
        let err = validate_flow(content).unwrap_err();
        assert!(
            err.iter().any(|s| s.contains("Overview")),
            "expected 'Overview' in errors, got: {:?}",
            err
        );
    }

    #[test]
    fn flow_reports_missing_section_name_when_source_destination_absent() {
        let content = "# My Flow

## Overview
Describes the data flow.

## Mermaid Diagram

## Transformation Steps
1. Transform input

## Error Paths
- On failure: retry
";
        let err = validate_flow(content).unwrap_err();
        assert!(
            err.iter().any(|s| s.contains("Source-Destination")),
            "expected 'Source-Destination' in errors, got: {:?}",
            err
        );
    }

    #[test]
    fn flow_reports_missing_section_name_when_transformation_steps_absent() {
        let content = "# My Flow

## Overview
Describes the data flow.

## Mermaid Diagram

## Source-Destination
Source: Service A

## Error Paths
- On failure: retry
";
        let err = validate_flow(content).unwrap_err();
        assert!(
            err.iter().any(|s| s.contains("Transformation Steps")),
            "expected 'Transformation Steps' in errors, got: {:?}",
            err
        );
    }

    #[test]
    fn flow_reports_missing_section_name_when_error_paths_absent() {
        let content = "# My Flow

## Overview
Describes the data flow.

## Mermaid Diagram

## Source-Destination
Source: Service A

## Transformation Steps
1. Transform input
";
        let err = validate_flow(content).unwrap_err();
        assert!(
            err.iter().any(|s| s.contains("Error Paths")),
            "expected 'Error Paths' in errors, got: {:?}",
            err
        );
    }

    #[test]
    fn flow_reports_all_missing_sections_when_all_absent() {
        let content = "# My Flow

No required sections here.
";
        let err = validate_flow(content).unwrap_err();
        assert_eq!(err.len(), 5, "expected 5 missing sections, got: {:?}", err);
    }
}
