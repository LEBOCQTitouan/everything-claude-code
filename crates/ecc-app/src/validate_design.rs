//! `validate_design` use case — reads a design file and validates PC structure via domain.

use ecc_domain::spec::{
    DesignValidationOutput, check_coverage, check_ordering, parse_acs, parse_file_changes,
    parse_pcs,
};
use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

/// Read a design file, run PC validation, optionally check coverage against a spec file.
///
/// Returns `Ok(true)` when valid, `Ok(false)` on errors.
pub fn run_validate_design(
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
    path: &str,
    spec_path: Option<&str>,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Step 1: Read design file
    let content = match fs.read_to_string(Path::new(path)) {
        Ok(c) => c,
        Err(e) => {
            let msg = e.to_string();
            let is_not_found = msg.contains("not found");
            let error_msg = if is_not_found {
                format!("file not found: {path}")
            } else {
                msg
            };
            let output = DesignValidationOutput {
                valid: false,
                pc_count: 0,
                pcs: Vec::new(),
                uncovered_acs: None,
                phantom_acs: None,
                ordering_violations: Vec::new(),
                errors: vec![error_msg],
                warnings: Vec::new(),
            };
            let json = serde_json::to_string(&output)?;
            terminal.stdout_write(&json);
            return Ok(false);
        }
    };

    // Step 2: Parse PC table
    let pc_report = match parse_pcs(&content) {
        Ok(r) => r,
        Err(e) => {
            let output = DesignValidationOutput {
                valid: false,
                pc_count: 0,
                pcs: Vec::new(),
                uncovered_acs: None,
                phantom_acs: None,
                ordering_violations: Vec::new(),
                errors: vec![e.to_string()],
                warnings: Vec::new(),
            };
            let json = serde_json::to_string(&output)?;
            terminal.stdout_write(&json);
            return Ok(false);
        }
    };

    let mut all_errors = pc_report.errors.clone();
    let mut all_warnings = pc_report.warnings.clone();

    // Step 3: Optionally run coverage check
    let (uncovered_acs, phantom_acs) = if let Some(sp) = spec_path {
        match fs.read_to_string(Path::new(sp)) {
            Ok(spec_content) => match parse_acs(&spec_content) {
                Ok(ac_report) => {
                    all_errors.extend(ac_report.errors.clone());
                    all_warnings.extend(ac_report.warnings.clone());
                    let coverage = check_coverage(&ac_report.acs, &pc_report.pcs);
                    // Phantom ACs are warnings only — do not affect validity
                    for phantom in &coverage.phantom_acs {
                        all_warnings.push(format!("phantom AC referenced in PCs: {phantom}"));
                    }
                    (Some(coverage.uncovered_acs), Some(coverage.phantom_acs))
                }
                Err(e) => {
                    all_errors.push(e.to_string());
                    (Some(Vec::new()), Some(Vec::new()))
                }
            },
            Err(e) => {
                let msg = e.to_string();
                let is_not_found = msg.contains("not found");
                let error_msg = if is_not_found {
                    format!("spec file not found: {sp}")
                } else {
                    msg
                };
                let output = DesignValidationOutput {
                    valid: false,
                    pc_count: 0,
                    pcs: Vec::new(),
                    uncovered_acs: None,
                    phantom_acs: None,
                    ordering_violations: Vec::new(),
                    errors: vec![error_msg],
                    warnings: Vec::new(),
                };
                let json = serde_json::to_string(&output)?;
                terminal.stdout_write(&json);
                return Ok(false);
            }
        }
    } else {
        (None, None)
    };

    // Step 4: Ordering check via File Changes table
    let (file_changes, ordering_warnings) = parse_file_changes(&content);
    all_warnings.extend(ordering_warnings);
    let ordering_result = check_ordering(&pc_report.pcs, &file_changes);
    all_warnings.extend(ordering_result.warnings);

    // Warnings go to stderr
    for warning in &all_warnings {
        terminal.stderr_write(&format!("WARNING: {warning}\n"));
    }

    // valid = no errors from PC parsing AND no uncovered ACs (if coverage was checked)
    let has_uncovered = uncovered_acs
        .as_ref()
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    let valid = all_errors.is_empty() && !has_uncovered;

    let output = DesignValidationOutput {
        valid,
        pc_count: pc_report.pcs.len(),
        pcs: pc_report.pcs,
        uncovered_acs,
        phantom_acs,
        ordering_violations: ordering_result.violations,
        errors: all_errors,
        warnings: all_warnings,
    };
    let json = serde_json::to_string(&output)?;
    terminal.stdout_write(&json);
    Ok(valid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem};

    const VALID_DESIGN: &str = "\
# Solution

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | First test | AC-001.1 | cargo test | PASS |
| PC-002 | unit | Second test | AC-001.2 | cargo test | PASS |
";

    const VALID_SPEC: &str = "\
## User Stories

### US-001

- AC-001.1: First criterion
- AC-001.2: Second criterion
";

    const MALFORMED_PC_DESIGN: &str = "\
# Solution

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | Only five cols | AC-001.1 | cargo test |
";

    #[test]
    fn valid_design_json_output() {
        let fs = InMemoryFileSystem::new().with_file("/design.md", VALID_DESIGN);
        let terminal = BufferedTerminal::new();

        let result = run_validate_design(&fs, &terminal, "/design.md", None).unwrap();

        assert!(result, "expected true for valid design");
        let stdout = terminal.stdout_output();
        let json_str = stdout.join("");
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
        assert_eq!(parsed["valid"], true);
        assert!(parsed["pc_count"].as_u64().unwrap() >= 1);
        assert!(parsed.get("pcs").is_some());
    }

    #[test]
    fn with_spec_runs_coverage() {
        let fs = InMemoryFileSystem::new()
            .with_file("/design.md", VALID_DESIGN)
            .with_file("/spec.md", VALID_SPEC);
        let terminal = BufferedTerminal::new();

        let result = run_validate_design(&fs, &terminal, "/design.md", Some("/spec.md")).unwrap();

        assert!(result, "expected true when all ACs are covered");
        let stdout = terminal.stdout_output();
        let json_str = stdout.join("");
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
        assert_eq!(parsed["valid"], true);
        // uncovered_acs should be present and empty
        let uncovered = parsed["uncovered_acs"]
            .as_array()
            .expect("uncovered_acs array");
        assert!(uncovered.is_empty(), "all ACs should be covered");
    }

    #[test]
    fn without_spec_skips_coverage() {
        let fs = InMemoryFileSystem::new().with_file("/design.md", VALID_DESIGN);
        let terminal = BufferedTerminal::new();

        let result = run_validate_design(&fs, &terminal, "/design.md", None).unwrap();

        assert!(result, "expected true");
        let stdout = terminal.stdout_output();
        let json_str = stdout.join("");
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
        // When no spec provided, uncovered_acs should be null (not present in JSON)
        assert!(
            parsed.get("uncovered_acs").is_none(),
            "uncovered_acs should be absent when no spec provided"
        );
    }

    #[test]
    fn nonexistent_spec_error() {
        let fs = InMemoryFileSystem::new().with_file("/design.md", VALID_DESIGN);
        let terminal = BufferedTerminal::new();

        let result =
            run_validate_design(&fs, &terminal, "/design.md", Some("/nonexistent_spec.md"))
                .unwrap();

        assert!(!result, "expected false for nonexistent spec");
        let stdout = terminal.stdout_output();
        let json_str = stdout.join("");
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
        assert_eq!(parsed["valid"], false);
        let errors = parsed["errors"].as_array().expect("errors array");
        assert!(!errors.is_empty());
    }

    #[test]
    fn phantom_acs_do_not_fail() {
        // Design references AC-099.9 which doesn't exist in spec → phantom warning, NOT error
        let design_with_phantom = "\
# Solution

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | First test | AC-001.1, AC-099.9 | cargo test | PASS |
";
        let spec_only_001 = "\
## User Stories

### US-001

- AC-001.1: Only AC
";
        let fs = InMemoryFileSystem::new()
            .with_file("/design.md", design_with_phantom)
            .with_file("/spec.md", spec_only_001);
        let terminal = BufferedTerminal::new();

        let result = run_validate_design(&fs, &terminal, "/design.md", Some("/spec.md")).unwrap();

        // Phantom ACs are warnings only — should not fail validation
        assert!(result, "phantom ACs should not cause validation failure");
        let stdout = terminal.stdout_output();
        let json_str = stdout.join("");
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
        assert_eq!(parsed["valid"], true);
        // phantom_acs should be present and non-empty
        let phantom = parsed["phantom_acs"].as_array().expect("phantom_acs array");
        assert!(!phantom.is_empty(), "phantom AC should be reported");
    }

    #[test]
    fn design_file_not_found() {
        let fs = InMemoryFileSystem::new();
        let terminal = BufferedTerminal::new();

        let result = run_validate_design(&fs, &terminal, "/nonexistent.md", None).unwrap();

        assert!(!result);
        let stdout = terminal.stdout_output();
        let json_str = stdout.join("");
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
        assert_eq!(parsed["valid"], false);
    }

    #[test]
    fn design_with_malformed_pc_returns_false() {
        let fs = InMemoryFileSystem::new().with_file("/design.md", MALFORMED_PC_DESIGN);
        let terminal = BufferedTerminal::new();

        let result = run_validate_design(&fs, &terminal, "/design.md", None).unwrap();

        assert!(!result, "malformed PC row should fail validation");
        let stdout = terminal.stdout_output();
        let json_str = stdout.join("");
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
        assert_eq!(parsed["valid"], false);
        let errors = parsed["errors"].as_array().expect("errors array");
        assert!(!errors.is_empty());
    }
}
