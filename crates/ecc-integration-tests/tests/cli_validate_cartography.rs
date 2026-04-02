//! CLI integration tests for `ecc validate cartography`.
//!
//! Tests: PC-022 (exit codes), PC-023 (--coverage flag).
//! These tests call `run_validate_cartography` directly via app crate
//! (same pattern as other app-layer integration tests).

use ecc_app::validate_cartography::run_validate_cartography;
use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockExecutor};
use std::path::Path;

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

## Related Flows
- none
"
}

fn valid_flow() -> &'static str {
    "\
# My Flow

## Overview
Data moves.

## Mermaid Diagram
```mermaid
graph LR
A --> B
```

## Source-Destination
Source: A
Destination: B

## Transformation Steps
1. Transform

## Error Paths
- On error: fail
"
}

fn invalid_flow_missing_sections() -> &'static str {
    // Missing Source-Destination and Error Paths
    "\
# Broken Flow

## Overview
Data moves.

## Mermaid Diagram
```mermaid
graph LR
A --> B
```

## Transformation Steps
1. Transform
"
}

// ── PC-022: exits 0 for valid files, exits 1 with section name for invalid ────

/// PC-022: CLI ecc validate cartography on valid files exits 0;
/// on invalid exits 1 with section name.
#[test]
fn exits_zero_and_one_for_valid_and_invalid() {
    // --- valid scenario: all files pass schema ---
    let fs_valid = InMemoryFileSystem::new()
        .with_file(
            "/project/docs/cartography/journeys/ok-journey.md",
            valid_journey(),
        )
        .with_file(
            "/project/docs/cartography/flows/ok-flow.md",
            valid_flow(),
        );
    let shell = MockExecutor::new();
    let terminal_valid = BufferedTerminal::new();

    let result_valid = run_validate_cartography(
        &fs_valid,
        &shell,
        &terminal_valid,
        Path::new("/project"),
        false,
    );

    assert!(
        result_valid,
        "expected true (exit 0) when all cartography files are valid"
    );

    // --- invalid scenario: flow file missing required sections ---
    let fs_invalid = InMemoryFileSystem::new()
        .with_file(
            "/project/docs/cartography/journeys/ok-journey.md",
            valid_journey(),
        )
        .with_file(
            "/project/docs/cartography/flows/broken-flow.md",
            invalid_flow_missing_sections(),
        );
    let terminal_invalid = BufferedTerminal::new();

    let result_invalid = run_validate_cartography(
        &fs_invalid,
        &shell,
        &terminal_invalid,
        Path::new("/project"),
        false,
    );

    assert!(
        !result_invalid,
        "expected false (exit 1) when a cartography file has missing sections"
    );

    // Output must include at least one of the missing section names
    let stdout = terminal_invalid.stdout_output().join("");
    assert!(
        stdout.contains("Source-Destination") || stdout.contains("Error Paths"),
        "expected missing section name in output, got: {stdout}"
    );
}

// ── PC-023: coverage flag prints percentage ────────────────────────────────────

/// PC-023: CLI ecc validate cartography --coverage prints percentage.
#[test]
fn coverage_flag_prints_percentage() {
    let fs = InMemoryFileSystem::new()
        .with_file(
            "/project/docs/cartography/journeys/my-journey.md",
            valid_journey(),
        )
        .with_file("/project/src/main.rs", "fn main() {}");

    let shell = MockExecutor::new();
    let terminal = BufferedTerminal::new();

    let _result = run_validate_cartography(
        &fs,
        &shell,
        &terminal,
        Path::new("/project"),
        true, // coverage flag
    );

    let stdout = terminal.stdout_output().join("");
    assert!(
        stdout.contains('%') || stdout.contains("coverage") || stdout.contains("Coverage"),
        "expected coverage percentage in output when --coverage is passed, got: {stdout}"
    );
}
