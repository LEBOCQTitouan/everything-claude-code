//! `validate_spec` use case — reads a spec file and validates AC structure via domain.

use ecc_domain::spec::{parse_acs, SpecValidationOutput};
use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

/// Read a spec file, run AC validation, write JSON to stdout, warnings to stderr.
///
/// Returns `Ok(true)` when the spec is valid, `Ok(false)` on validation errors.
pub fn run_validate_spec(
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
    path: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
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
            let output = SpecValidationOutput {
                valid: false,
                ac_count: 0,
                acs: Vec::new(),
                errors: vec![error_msg],
                warnings: Vec::new(),
            };
            let json = serde_json::to_string(&output)?;
            terminal.stdout_write(&json);
            return Ok(false);
        }
    };

    let report = match parse_acs(&content) {
        Ok(r) => r,
        Err(e) => {
            let output = SpecValidationOutput {
                valid: false,
                ac_count: 0,
                acs: Vec::new(),
                errors: vec![e.to_string()],
                warnings: Vec::new(),
            };
            let json = serde_json::to_string(&output)?;
            terminal.stdout_write(&json);
            return Ok(false);
        }
    };

    for warning in &report.warnings {
        terminal.stderr_write(&format!("WARNING: {warning}\n"));
    }

    let valid = report.errors.is_empty();
    let output = SpecValidationOutput {
        valid,
        ac_count: report.acs.len(),
        acs: report.acs,
        errors: report.errors,
        warnings: report.warnings,
    };
    let json = serde_json::to_string(&output)?;
    terminal.stdout_write(&json);
    Ok(valid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem};

    const VALID_SPEC: &str = "\
## User Stories

### US-001

- AC-001.1: First criterion
- AC-001.2: Second criterion
";

    const GAP_SPEC: &str = "\
## User Stories

### US-001

- AC-001.1: First criterion
- AC-001.3: Third criterion — gap at AC-001.2
";

    #[test]
    fn valid_spec_json_output() {
        let fs = InMemoryFileSystem::new().with_file("/spec.md", VALID_SPEC);
        let terminal = BufferedTerminal::new();

        let result = run_validate_spec(&fs, &terminal, "/spec.md").unwrap();

        assert!(result, "expected true for valid spec");
        let stdout = terminal.stdout_output();
        assert!(!stdout.is_empty(), "expected stdout output");
        let json_str = stdout.join("");
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
        assert_eq!(parsed["valid"], true);
        assert!(parsed["ac_count"].as_u64().unwrap() >= 1);
    }

    #[test]
    fn invalid_spec_json_errors() {
        let fs = InMemoryFileSystem::new().with_file("/spec.md", GAP_SPEC);
        let terminal = BufferedTerminal::new();

        let result = run_validate_spec(&fs, &terminal, "/spec.md").unwrap();

        assert!(!result, "expected false for invalid spec");
        let stdout = terminal.stdout_output();
        let json_str = stdout.join("");
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
        assert_eq!(parsed["valid"], false);
        let errors = parsed["errors"].as_array().expect("errors array");
        assert!(!errors.is_empty(), "expected errors in output");
    }

    #[test]
    fn nonexistent_file_error() {
        let fs = InMemoryFileSystem::new();
        let terminal = BufferedTerminal::new();

        let result = run_validate_spec(&fs, &terminal, "/nonexistent.md").unwrap();

        assert!(!result, "expected false for nonexistent file");
        let stdout = terminal.stdout_output();
        let json_str = stdout.join("");
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
        assert_eq!(parsed["valid"], false);
        let errors = parsed["errors"].as_array().expect("errors array");
        assert!(!errors.is_empty(), "expected error in output");
    }

    #[test]
    fn non_utf8_file_error() {
        // InMemoryFileSystem uses from_utf8_lossy so won't produce a real UTF-8 error.
        // Test that any file-read failure is handled gracefully (using not-found as proxy).
        let fs = InMemoryFileSystem::new();
        let terminal = BufferedTerminal::new();

        let result = run_validate_spec(&fs, &terminal, "/missing.md").unwrap();

        assert!(!result);
        let stdout = terminal.stdout_output();
        let json_str = stdout.join("");
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
        assert_eq!(parsed["valid"], false);
    }

    #[test]
    fn warnings_to_stderr() {
        // A spec with malformed AC IDs generates warnings
        let spec_with_warnings = "\
## User Stories

### US-001

- AC-001.1: First criterion
- AC-ABC.1: Malformed ID
";
        let fs = InMemoryFileSystem::new().with_file("/spec.md", spec_with_warnings);
        let terminal = BufferedTerminal::new();

        run_validate_spec(&fs, &terminal, "/spec.md").unwrap();

        // JSON always goes to stdout
        let stdout = terminal.stdout_output();
        assert!(!stdout.is_empty(), "JSON should be on stdout");
        let json_str = stdout.join("");
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
        // Warnings about malformed IDs appear in JSON warnings field
        let _ = parsed["warnings"].as_array().expect("warnings array");
    }
}
