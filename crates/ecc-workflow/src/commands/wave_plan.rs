//! `wave-plan` subcommand.
//!
//! Reads a design.md file, parses the PC table and File Changes table,
//! computes a deterministic wave plan, and outputs JSON to stdout.
//!
//! Exit codes:
//!   0 — pass (valid JSON with waves)
//!   0 — warn (no File Changes table; all PCs treated as independent)
//!   2 — block (nonexistent path, path traversal, or no PC table)

use std::path::Path;

use serde::Serialize;

use crate::output::WorkflowOutput;

/// JSON output schema for the wave plan.
#[derive(Serialize)]
struct WavePlanOutput {
    status: &'static str,
    waves: Vec<WaveOutput>,
    total_pcs: usize,
    max_per_wave: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    warnings: Option<Vec<String>>,
}

/// JSON representation of a single wave.
#[derive(Serialize)]
struct WaveOutput {
    id: u16,
    pcs: Vec<String>,
    files: Vec<String>,
}

/// Run the `wave-plan` subcommand.
pub fn run(design_path: &str, project_dir: &Path) -> WorkflowOutput {
    match run_inner(design_path, project_dir) {
        Ok(output) => output,
        Err(e) => WorkflowOutput::block(format!("wave-plan failed: {e}")),
    }
}

fn run_inner(design_path: &str, project_dir: &Path) -> Result<WorkflowOutput, anyhow::Error> {
    let path = Path::new(design_path);

    // AC-003.4: Check existence first, then canonicalize.
    if !path.exists() {
        anyhow::bail!("design file not found: {design_path}");
    }

    // AC-003.5: Resolve and check for path traversal.
    let resolved = std::fs::canonicalize(path)?;
    let project_root =
        std::fs::canonicalize(project_dir).unwrap_or_else(|_| project_dir.to_path_buf());
    if !resolved.starts_with(&project_root) {
        anyhow::bail!("path escapes project directory: {design_path}");
    }

    let content = std::fs::read_to_string(&resolved)?;

    // AC-003.3: Parse PC table — block if missing.
    let pc_report = ecc_domain::spec::pc::parse_pcs(&content)
        .map_err(|e| anyhow::anyhow!("no PC table: {e}"))?;

    // AC-003.6: Parse File Changes — warn if missing.
    let (file_changes, fc_warnings) = ecc_domain::spec::ordering::parse_file_changes(&content);
    let has_file_changes = !file_changes.is_empty();

    // Compute wave plan.
    let plan = ecc_domain::spec::wave::compute_wave_plan(&pc_report.pcs, &file_changes, 4);

    // Build wave outputs with PC IDs serialized as "PC-NNN" strings.
    let waves: Vec<WaveOutput> = plan
        .waves
        .iter()
        .map(|w| WaveOutput {
            id: w.id,
            pcs: w.pc_ids.iter().map(|id| id.to_string()).collect(),
            files: w.files.clone(),
        })
        .collect();

    // When no File Changes table exists, include warnings and use "warn" status.
    let warnings = if has_file_changes {
        None
    } else {
        let mut msgs: Vec<String> = fc_warnings.into_iter().filter(|w| !w.is_empty()).collect();
        msgs.push("no File Changes table found — all PCs treated as independent".to_owned());
        Some(msgs)
    };

    let output = WavePlanOutput {
        status: if has_file_changes { "pass" } else { "warn" },
        waves,
        total_pcs: plan.total_pcs,
        max_per_wave: plan.max_per_wave,
        warnings,
    };

    let json = serde_json::to_string(&output)?;

    if has_file_changes {
        Ok(WorkflowOutput::pass(json))
    } else {
        Ok(WorkflowOutput::warn(json))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use tempfile::TempDir;

    /// Fixture: a valid design.md with both PC and File Changes tables.
    fn valid_design_content() -> &'static str {
        r#"# Design: Test

## File Changes

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `src/a.rs` | CREATE | reason | AC-001.1 |
| 2 | `src/b.rs` | CREATE | reason | AC-002.1 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | Test A | AC-001.1 | `cargo test a` | PASS |
| PC-002 | unit | Test B | AC-002.1 | `cargo test b` | PASS |
"#
    }

    /// PC-017: wave-plan outputs valid JSON with waves array for valid design (AC-003.1, AC-003.2)
    #[test]
    fn valid_design_json_output() {
        let tmp = TempDir::new().unwrap();
        let design_path = tmp.path().join("design.md");
        std::fs::write(&design_path, valid_design_content()).unwrap();

        let output = run(design_path.to_str().unwrap(), tmp.path());

        // Must be pass status
        assert!(
            matches!(output.status, crate::output::Status::Pass),
            "expected Pass status, got {:?}: {}",
            output.status,
            output.message
        );

        // Message must be parseable JSON
        let json: Value =
            serde_json::from_str(&output.message).expect("output.message must be valid JSON");

        // Must have a "waves" array
        let waves = json.get("waves").expect("JSON must have 'waves' key");
        assert!(waves.is_array(), "'waves' must be an array");

        let waves_arr = waves.as_array().unwrap();
        assert!(!waves_arr.is_empty(), "'waves' array must not be empty");

        // Each wave must have id, pcs, and files
        for wave in waves_arr {
            assert!(wave.get("id").is_some(), "wave must have 'id'");
            assert!(wave.get("pcs").is_some(), "wave must have 'pcs'");
            assert!(wave.get("files").is_some(), "wave must have 'files'");

            let pcs = wave.get("pcs").unwrap();
            assert!(pcs.is_array(), "'pcs' must be an array");

            let files = wave.get("files").unwrap();
            assert!(files.is_array(), "'files' must be an array");
        }

        // Must have total_pcs and max_per_wave
        assert!(
            json.get("total_pcs").is_some(),
            "JSON must have 'total_pcs'"
        );
        assert!(
            json.get("max_per_wave").is_some(),
            "JSON must have 'max_per_wave'"
        );
        assert_eq!(json["total_pcs"], 2);
    }

    /// PC-019: wave-plan exits block for nonexistent path (AC-003.4)
    #[test]
    fn nonexistent_path_blocks() {
        let tmp = TempDir::new().unwrap();
        let nonexistent = tmp.path().join("does_not_exist.md");

        let output = run(nonexistent.to_str().unwrap(), tmp.path());

        assert!(
            matches!(output.status, crate::output::Status::Block),
            "expected Block status for nonexistent path, got {:?}: {}",
            output.status,
            output.message
        );
    }

    /// PC-020: wave-plan rejects path traversal (AC-003.5)
    #[test]
    fn path_traversal_rejected() {
        // Create a file OUTSIDE the project dir
        let outside_tmp = TempDir::new().unwrap();
        let outside_file = outside_tmp.path().join("design.md");
        std::fs::write(&outside_file, valid_design_content()).unwrap();

        // Project dir is a different tempdir
        let project_tmp = TempDir::new().unwrap();

        let output = run(outside_file.to_str().unwrap(), project_tmp.path());

        assert!(
            matches!(output.status, crate::output::Status::Block),
            "expected Block status for path traversal, got {:?}: {}",
            output.status,
            output.message
        );
    }

    /// PC-018: wave-plan exits block when no PC table (AC-003.3)
    #[test]
    fn no_pc_table_blocks() {
        let tmp = TempDir::new().unwrap();
        let design_path = tmp.path().join("design.md");
        // File Changes table present but no Pass Conditions table
        let content = r#"# Design: No PCs

## File Changes

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `src/a.rs` | CREATE | reason | AC-001.1 |

No pass conditions table here.
"#;
        std::fs::write(&design_path, content).unwrap();

        let output = run(design_path.to_str().unwrap(), tmp.path());

        assert!(
            matches!(output.status, crate::output::Status::Block),
            "expected Block status when no PC table, got {:?}: {}",
            output.status,
            output.message
        );
    }

    /// PC-021: wave-plan warns with default waves when no File Changes table (AC-003.6)
    #[test]
    fn no_file_changes_warns() {
        let tmp = TempDir::new().unwrap();
        let design_path = tmp.path().join("design.md");
        // Only PC table, no File Changes table
        let content = r#"# Design: No File Changes

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | Test A | AC-001.1 | `cargo test a` | PASS |
| PC-002 | unit | Test B | AC-002.1 | `cargo test b` | PASS |
"#;
        std::fs::write(&design_path, content).unwrap();

        let output = run(design_path.to_str().unwrap(), tmp.path());

        // Must be warn status (not block)
        assert!(
            matches!(output.status, crate::output::Status::Warn),
            "expected Warn status when no File Changes table, got {:?}: {}",
            output.status,
            output.message
        );

        // Message must be parseable JSON with waves
        let json: Value =
            serde_json::from_str(&output.message).expect("output.message must be valid JSON");

        let waves = json.get("waves").expect("JSON must have 'waves' key");
        assert!(waves.is_array(), "'waves' must be an array");
        assert!(
            !waves.as_array().unwrap().is_empty(),
            "waves must not be empty"
        );

        // Must include warnings
        let warnings = json.get("warnings").expect("JSON must have 'warnings' key");
        assert!(warnings.is_array(), "'warnings' must be an array");
        assert!(
            !warnings.as_array().unwrap().is_empty(),
            "warnings must not be empty"
        );
    }
}
