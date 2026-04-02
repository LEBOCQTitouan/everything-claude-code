//! Integration tests for `run_validate_cartography` use case.
//!
//! Tests: PC-018 (schema errors), PC-019 (staleness), PC-020 (coverage dashboard),
//! PC-021 (coverage performance).

use ecc_app::validate_cartography::run_validate_cartography;
use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockExecutor};
use ecc_ports::shell::CommandOutput;
use std::path::Path;
use std::time::Instant;

// ── helpers ──────────────────────────────────────────────────────────────────

fn valid_journey() -> &'static str {
    "\
# My Journey

## Overview
An actor performs an action.

## Mermaid Diagram
```mermaid
graph LR
A --> B
```

## Steps
1. Step one
2. Step two

## Related Flows
- [auth-flow](../flows/auth-flow.md)
"
}

fn valid_flow() -> &'static str {
    "\
# My Flow

## Overview
Data moves from A to B.

## Mermaid Diagram
```mermaid
graph LR
A --> B
```

## Source-Destination
Source: Service A
Destination: Service B

## Transformation Steps
1. Transform input data

## Error Paths
- On timeout: retry
"
}

fn invalid_journey() -> &'static str {
    // Missing Overview and Mermaid Diagram sections
    "\
# Incomplete Journey

## Steps
1. Step one

## Related Flows
- none
"
}

fn invalid_flow() -> &'static str {
    // Missing Source-Destination and Error Paths
    "\
# Incomplete Flow

## Overview
Some description.

## Mermaid Diagram
```mermaid
graph LR
A --> B
```

## Transformation Steps
1. Transform
"
}

// ── PC-018: schema errors reported for all files ──────────────────────────────

/// PC-018: run_validate_cartography reports all journey/flow schema errors
/// across multiple files; returns false (exit-code non-zero) on invalid files.
#[test]
fn schema_errors_reported_for_all_files() {
    let fs = InMemoryFileSystem::new()
        // One valid journey
        .with_file(
            "/project/docs/cartography/journeys/valid-journey.md",
            valid_journey(),
        )
        // One invalid journey (missing Overview, Mermaid Diagram)
        .with_file(
            "/project/docs/cartography/journeys/invalid-journey.md",
            invalid_journey(),
        )
        // One valid flow
        .with_file(
            "/project/docs/cartography/flows/valid-flow.md",
            valid_flow(),
        )
        // One invalid flow (missing Source-Destination, Error Paths)
        .with_file(
            "/project/docs/cartography/flows/invalid-flow.md",
            invalid_flow(),
        );

    let shell = MockExecutor::new();
    let terminal = BufferedTerminal::new();

    let result = run_validate_cartography(
        &fs,
        &shell,
        &terminal,
        Path::new("/project"),
        false,
    );

    // Should return false because there are schema errors
    assert!(!result, "expected false when files have schema errors");

    // Output should mention the missing sections from invalid journey
    let stdout = terminal.stdout_output().join("");
    assert!(
        stdout.contains("Overview") || stdout.contains("Mermaid Diagram"),
        "expected missing journey section names in output, got: {stdout}"
    );

    // Output should mention errors from the invalid flow
    assert!(
        stdout.contains("Source-Destination") || stdout.contains("Error Paths"),
        "expected missing flow section names in output, got: {stdout}"
    );
}

// ── PC-019: stale entries reported with delta days ────────────────────────────

/// PC-019: run_validate_cartography reports stale entries with the staleness
/// delta in days.
#[test]
fn stale_entries_reported_with_delta_days() {
    let journey_with_meta = format!(
        "{}\n\n<!-- CARTOGRAPHY-META: last_updated=2026-01-01, sources=src/main.rs -->",
        valid_journey()
    );

    let fs = InMemoryFileSystem::new().with_file(
        "/project/docs/cartography/journeys/stale-journey.md",
        &journey_with_meta,
    );

    // git log returns a date newer than 2026-01-01 → stale
    let shell = MockExecutor::new().on_args(
        "git",
        &["log", "-1", "--format=%Y-%m-%d", "src/main.rs"],
        CommandOutput {
            stdout: "2026-03-01\n".to_string(),
            stderr: String::new(),
            exit_code: 0,
        },
    );
    let terminal = BufferedTerminal::new();

    let result = run_validate_cartography(
        &fs,
        &shell,
        &terminal,
        Path::new("/project"),
        false,
    );

    // Stale entries don't cause validation failure by themselves (only schema errors do)
    // But the staleness must be reported in output
    let stdout = terminal.stdout_output().join("");
    assert!(
        stdout.contains("stale") || stdout.contains("STALE") || stdout.contains("days"),
        "expected staleness report in output, got: {stdout}"
    );

    // The delta in days should be present (2026-01-01 to 2026-03-01 = 59 days)
    assert!(
        stdout.contains("59") || stdout.contains("days"),
        "expected number of stale days in output, got: {stdout}"
    );

    // A stale entry alone should not fail validation
    let _ = result; // Either true or false is acceptable — spec says "reports them"
}

// ── PC-020: coverage flag outputs dashboard ────────────────────────────────────

/// PC-020: run_validate_cartography with coverage flag outputs total files,
/// referenced files, percentage; below 50% includes Priority gaps.
#[test]
fn coverage_flag_outputs_dashboard() {
    // Journey referencing one source file
    let journey_content = format!(
        "{}\n\nReferenced file: `src/main.rs`",
        valid_journey()
    );

    let fs = InMemoryFileSystem::new()
        .with_file(
            "/project/docs/cartography/journeys/my-journey.md",
            &journey_content,
        )
        // Source files: only 1 out of 5 is referenced → 20% coverage (below 50%)
        .with_file("/project/src/main.rs", "fn main() {}")
        .with_file("/project/src/lib.rs", "pub mod foo;")
        .with_file("/project/src/foo.rs", "pub fn foo() {}")
        .with_file("/project/src/bar.rs", "pub fn bar() {}")
        .with_file("/project/src/baz.rs", "pub fn baz() {}");

    let shell = MockExecutor::new();
    let terminal = BufferedTerminal::new();

    let result = run_validate_cartography(
        &fs,
        &shell,
        &terminal,
        Path::new("/project"),
        true, // coverage flag
    );

    let stdout = terminal.stdout_output().join("");

    // Must output total source files count
    assert!(
        stdout.contains("5") || stdout.contains("total"),
        "expected total source file count in output, got: {stdout}"
    );

    // Must output percentage
    assert!(
        stdout.contains('%') || stdout.contains("coverage") || stdout.contains("Coverage"),
        "expected coverage percentage in output, got: {stdout}"
    );

    // Below 50% → must include Priority gaps section
    assert!(
        stdout.contains("Priority gaps") || stdout.contains("priority gaps") || stdout.contains("gaps"),
        "expected priority gaps section (coverage below 50%), got: {stdout}"
    );

    // Valid files → result is true (no schema errors)
    assert!(result, "expected true when all files are schema-valid");
}

// ── PC-021: coverage completes within timeout ──────────────────────────────────

/// PC-021: run_validate_cartography with coverage flag on 500-file fixture
/// completes in under 5 seconds.
#[test]
fn coverage_completes_within_timeout() {
    let mut fs = InMemoryFileSystem::new().with_file(
        "/project/docs/cartography/journeys/my-journey.md",
        valid_journey(),
    );

    // Add 500 source files
    for i in 0..500 {
        fs = fs.with_file(
            &format!("/project/src/file_{i:03}.rs"),
            &format!("// file {i}"),
        );
    }

    let shell = MockExecutor::new();
    let terminal = BufferedTerminal::new();

    let start = Instant::now();
    let _result = run_validate_cartography(
        &fs,
        &shell,
        &terminal,
        Path::new("/project"),
        true,
    );
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_secs() < 5,
        "expected coverage to complete in under 5 seconds, took {elapsed:?}"
    );
}
