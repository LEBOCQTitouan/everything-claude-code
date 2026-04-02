//! `validate_design` use case — reads a design file and validates PC structure via domain.

use ecc_domain::spec::{
    AcId, DesignValidationOutput, OrderingViolation, check_coverage, check_ordering, parse_acs,
    parse_file_changes, parse_pcs,
};

/// Result of spec coverage check: `(uncovered_acs, phantom_acs, errors, warnings)`.
type CoverageCheckResult = (Vec<AcId>, Vec<AcId>, Vec<String>, Vec<String>);
use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

/// Emit an error-only `DesignValidationOutput` to stdout and return `Ok(false)`.
fn emit_design_error(
    terminal: &dyn TerminalIO,
    error_msg: String,
) -> Result<bool, Box<dyn std::error::Error>> {
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
    Ok(false)
}

/// Normalize an FS error message, replacing "not found" with a user-friendly prefix.
fn normalize_fs_error(msg: String, prefix: &str) -> String {
    if msg.contains("not found") {
        format!("{prefix} not found")
    } else {
        msg
    }
}

/// Build and emit the final `DesignValidationOutput`, returning whether the design is valid.
fn emit_design_output(
    terminal: &dyn TerminalIO,
    pc_report: ecc_domain::spec::PcReport,
    uncovered_acs: Option<Vec<AcId>>,
    phantom_acs: Option<Vec<AcId>>,
    ordering_violations: Vec<OrderingViolation>,
    all_errors: Vec<String>,
    all_warnings: Vec<String>,
) -> Result<bool, Box<dyn std::error::Error>> {
    for warning in &all_warnings {
        terminal.stderr_write(&format!("WARNING: {warning}\n"));
    }
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
        ordering_violations,
        errors: all_errors,
        warnings: all_warnings,
    };
    let json = serde_json::to_string(&output)?;
    terminal.stdout_write(&json);
    Ok(valid)
}

/// Run coverage check against a spec file: parse ACs and check against parsed PCs.
///
/// Returns `(uncovered_acs, phantom_acs, extra_errors, extra_warnings)` on success,
/// or `Err(...)` to signal that an early-exit error output was already emitted.
fn check_spec_coverage(
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
    spec_path: &str,
    pcs: &[ecc_domain::spec::PassCondition],
) -> Result<CoverageCheckResult, Box<dyn std::error::Error>> {
    let spec_content = match fs.read_to_string(Path::new(spec_path)) {
        Ok(c) => c,
        Err(e) => {
            let error_msg = normalize_fs_error(e.to_string(), &format!("spec file: {spec_path}"));
            emit_design_error(terminal, error_msg)?;
            return Err("early_exit".into());
        }
    };

    match parse_acs(&spec_content) {
        Ok(ac_report) => {
            let extra_errors = ac_report.errors.clone();
            let mut extra_warnings = ac_report.warnings.clone();
            let coverage = check_coverage(&ac_report.acs, pcs);
            for phantom in &coverage.phantom_acs {
                extra_warnings.push(format!("phantom AC referenced in PCs: {phantom}"));
            }
            Ok((
                coverage.uncovered_acs,
                coverage.phantom_acs,
                extra_errors,
                extra_warnings,
            ))
        }
        Err(e) => Ok((Vec::new(), Vec::new(), vec![e.to_string()], Vec::new())),
    }
}

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
            let error_msg = normalize_fs_error(e.to_string(), &format!("file: {path}"));
            return emit_design_error(terminal, error_msg);
        }
    };

    // Step 2: Parse PC table
    let pc_report = match parse_pcs(&content) {
        Ok(r) => r,
        Err(e) => return emit_design_error(terminal, e.to_string()),
    };

    let mut all_errors = pc_report.errors.clone();
    let mut all_warnings = pc_report.warnings.clone();

    // Step 3: Optionally run coverage check
    let (uncovered_acs, phantom_acs) = if let Some(sp) = spec_path {
        match check_spec_coverage(fs, terminal, sp, &pc_report.pcs) {
            Ok((uncovered, phantom, extra_errors, extra_warnings)) => {
                all_errors.extend(extra_errors);
                all_warnings.extend(extra_warnings);
                (Some(uncovered), Some(phantom))
            }
            Err(_) => return Ok(false), // early exit already emitted
        }
    } else {
        (None, None)
    };

    // Step 4: Ordering check via File Changes table
    let (file_changes, ordering_warnings) = parse_file_changes(&content);
    all_warnings.extend(ordering_warnings);
    let ordering_result = check_ordering(&pc_report.pcs, &file_changes);
    all_warnings.extend(ordering_result.warnings);

    emit_design_output(
        terminal,
        pc_report,
        uncovered_acs,
        phantom_acs,
        ordering_result.violations,
        all_errors,
        all_warnings,
    )
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
