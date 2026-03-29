# Design: Deterministic Convention Linting (BL-069)

## Overview

Add `ecc validate conventions` subcommand that performs deterministic naming, value, and placement checks across agents, commands, and skills. Pure domain functions in `ecc-domain`, orchestration in `ecc-app`, thin CLI wiring in `ecc-cli`.

## PC Table

| PC | Description | AC Coverage | Layer | Files Changed |
|----|------------|-------------|-------|---------------|
| PC-001 | Add `VALID_TOOLS` constant and `is_kebab_case()` to domain | AC-001.2, AC-002.2, AC-002.3 | Entity | `crates/ecc-domain/src/config/validate.rs` |
| PC-002 | Add `parse_tool_list()` pure function to domain | AC-002.1, AC-002.5 | Entity | `crates/ecc-domain/src/config/validate.rs` |
| PC-003 | Add `check_naming_consistency()` and `check_tool_values()` pure functions to domain | AC-001.1, AC-001.3, AC-002.2, AC-002.3, AC-002.4 | Entity | `crates/ecc-domain/src/config/validate.rs` |
| PC-004 | Add `Conventions` variant to `ValidateTarget` and `conventions.rs` orchestrator in ecc-app | AC-001.4, AC-002.6, AC-003.1, AC-003.2 | UseCase | `crates/ecc-app/src/validate/mod.rs`, `crates/ecc-app/src/validate/conventions.rs` (new) |
| PC-005 | Add `Conventions` CLI subcommand wiring | AC-002.6 | Adapter | `crates/ecc-cli/src/commands/validate.rs` |
| PC-006 | Fix existing violations + expand `VALID_TOOLS` for command-only tools | AC-004.2 | Entity | `crates/ecc-domain/src/config/validate.rs`, content files as needed |
| PC-007 | Add meta-test (integration test running `ecc validate conventions` on ECC repo) | AC-004.1 | Test | `crates/ecc-integration-tests/tests/validate_flow.rs` |
| PC-008 | Lint + build gate | All | Infra | N/A (commands only) |

## Dependency Order

```
PC-001 → PC-002 → PC-003 → PC-004 → PC-005 → PC-006 → PC-007 → PC-008
```

PC-001 through PC-003 are pure domain (Entity layer). PC-004 is UseCase. PC-005 is Adapter. PC-006 fixes content. PC-007 validates everything end-to-end.

## Architecture

```
ecc-cli (PC-005)          ecc-app (PC-004)           ecc-domain (PC-001..003)
  Conventions variant  -->  conventions.rs       -->  is_kebab_case()
  in CliValidateTarget      validate_conventions()    parse_tool_list()
  map_target()              walks agents/,commands/,   check_naming_consistency()
                            skills/ via FileSystem     check_tool_values()
                            port, calls domain fns,    VALID_TOOLS constant
                            collects errors/warns,
                            prints via TerminalIO
```

## Detailed Design

### PC-001: VALID_TOOLS constant and is_kebab_case

**File**: `crates/ecc-domain/src/config/validate.rs`

**Changes**:

```rust
/// Valid tool identifiers for agent/command frontmatter.
pub const VALID_TOOLS: &[&str] = &[
    "Read", "Write", "Edit", "MultiEdit", "Bash", "Glob", "Grep",
    "Agent", "Task", "WebSearch", "TodoWrite", "TodoRead",
    "AskUserQuestion",
    // Command-only tools (used in allowed-tools but not agent tools)
    "LS", "Skill", "EnterPlanMode", "ExitPlanMode",
    "TaskCreate", "TaskUpdate", "TaskGet", "TaskList",
];

/// Check if a string matches kebab-case: `^[a-z][a-z0-9]*(-[a-z0-9]+)*$`
pub fn is_kebab_case(s: &str) -> bool {
    // Hand-rolled for zero-dep, <50 lines
    if s.is_empty() { return false; }
    let bytes = s.as_bytes();
    if !bytes[0].is_ascii_lowercase() { return false; }
    let mut prev_hyphen = false;
    for &b in &bytes[1..] {
        match b {
            b'-' => {
                if prev_hyphen { return false; }
                prev_hyphen = true;
            }
            b'a'..=b'z' | b'0'..=b'9' => { prev_hyphen = false; }
            _ => return false,
        }
    }
    !prev_hyphen
}
```

**Layers**: Entity

**Pass condition**: `cargo test -p ecc-domain -- is_kebab_case` passes + `cargo test -p ecc-domain -- valid_tools` passes

### PC-002: parse_tool_list

**File**: `crates/ecc-domain/src/config/validate.rs`

**Changes**:

```rust
/// Parse a bracket-delimited tool list from frontmatter value string.
///
/// Handles: `["Read", "Write"]`, `[Read, Write]`, `Read` (bare string),
/// `[]` (empty list). Returns parsed tool names with whitespace/quotes trimmed.
pub fn parse_tool_list(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() { return vec![]; }

    // Strip outer brackets if present
    let inner = if trimmed.starts_with('[') && trimmed.ends_with(']') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };

    if inner.trim().is_empty() { return vec![]; }

    inner.split(',')
        .map(|s| s.trim().trim_matches('"').trim_matches('\'').trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
```

**Layers**: Entity

**Pass condition**: `cargo test -p ecc-domain -- parse_tool_list` passes (tests for bracket, bare string, empty, quoted)

### PC-003: check_naming_consistency and check_tool_values

**File**: `crates/ecc-domain/src/config/validate.rs`

**Changes**:

Add two severity types and two pure check functions:

```rust
/// Severity for convention lint findings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LintSeverity {
    Error,
    Warn,
}

/// A single convention lint finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintFinding {
    pub severity: LintSeverity,
    pub file: String,
    pub message: String,
}

/// Check filename-vs-frontmatter naming consistency for a single file.
///
/// Returns findings (may be empty). `file_stem` is the filename without extension.
/// `frontmatter_name` is the `name` field value from frontmatter (None if missing).
/// `entity_kind` is "agent", "skill", etc. for error messages.
pub fn check_naming_consistency(
    file_stem: &str,
    frontmatter_name: Option<&str>,
    entity_kind: &str,
) -> Vec<LintFinding> {
    let mut findings = Vec::new();
    let label = format!("{entity_kind} '{file_stem}'");

    // Kebab-case check on filename
    if !is_kebab_case(file_stem) {
        findings.push(LintFinding {
            severity: LintSeverity::Error,
            file: file_stem.to_string(),
            message: format!("{label}: filename is not kebab-case (expected pattern: ^[a-z][a-z0-9]*(-[a-z0-9]+)*$)"),
        });
    }

    // Name mismatch check
    match frontmatter_name {
        Some(name) if name.trim().is_empty() => {
            findings.push(LintFinding {
                severity: LintSeverity::Warn,
                file: file_stem.to_string(),
                message: format!("{label}: frontmatter 'name' is empty, skipping name match"),
            });
        }
        Some(name) if name != file_stem => {
            findings.push(LintFinding {
                severity: LintSeverity::Error,
                file: file_stem.to_string(),
                message: format!("{label}: filename '{file_stem}' differs from frontmatter name '{name}'"),
            });
        }
        None => {
            findings.push(LintFinding {
                severity: LintSeverity::Warn,
                file: file_stem.to_string(),
                message: format!("{label}: missing frontmatter 'name' field, skipping name match"),
            });
        }
        _ => {} // name matches
    }

    findings
}

/// Validate tool names against VALID_TOOLS registry.
///
/// `raw_tools` is the raw frontmatter value for tools/allowed-tools.
/// Returns findings. Any invalid tool produces an ERROR for the whole file.
pub fn check_tool_values(
    file_stem: &str,
    raw_tools: &str,
    field_name: &str,
) -> Vec<LintFinding> {
    let tools = parse_tool_list(raw_tools);
    let invalid: Vec<_> = tools.iter()
        .filter(|t| !VALID_TOOLS.contains(&t.as_str()))
        .collect();

    if invalid.is_empty() {
        return vec![];
    }

    vec![LintFinding {
        severity: LintSeverity::Error,
        file: file_stem.to_string(),
        message: format!(
            "'{file_stem}': invalid {field_name} {:?} — valid tools: {}",
            invalid, VALID_TOOLS.join(", ")
        ),
    }]
}
```

**Layers**: Entity

**Pass condition**: `cargo test -p ecc-domain -- check_naming` passes, `cargo test -p ecc-domain -- check_tool_values` passes

### PC-004: Conventions orchestrator in ecc-app

**New file**: `crates/ecc-app/src/validate/conventions.rs`

**Changes to `mod.rs`**: Add `mod conventions;` and `Conventions` variant.

The orchestrator:
1. Walks `agents/` -- for each `.md` file: extract frontmatter, run `check_naming_consistency` (file_stem vs `name`), run `check_tool_values` on `tools` field
2. Walks `commands/` -- for each `.md` file: extract frontmatter, run `check_naming_consistency`, run `check_tool_values` on `allowed-tools` field
3. Walks `skills/` -- for each subdirectory: check `is_dir`, read `SKILL.md` if exists, run `check_naming_consistency` (dir name vs `name`), check for empty dirs (no .md files = WARN)
4. Collects all `LintFinding`s, prints ERRORs to stderr, WARNs to stdout
5. Returns `true` if zero ERRORs (WARNs allowed), `false` if any ERROR

```rust
use ecc_domain::config::validate::{
    check_naming_consistency, check_tool_values, extract_frontmatter,
    LintFinding, LintSeverity,
};
use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

pub(super) fn validate_conventions(
    root: &Path,
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
) -> bool {
    let mut findings: Vec<LintFinding> = Vec::new();
    let mut total_checked: usize = 0;

    // 1. Agents
    let agents_dir = root.join("agents");
    if fs.exists(&agents_dir) {
        if let Ok(files) = fs.read_dir(&agents_dir) {
            for file in files.iter().filter(|f| f.to_string_lossy().ends_with(".md")) {
                total_checked += 1;
                let content = match fs.read_to_string(file) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                let stem = file.file_stem().unwrap_or_default().to_string_lossy();
                let fm = extract_frontmatter(&content);
                let fm_name = fm.as_ref().and_then(|m| m.get("name")).map(|s| s.as_str());
                findings.extend(check_naming_consistency(&stem, fm_name, "agent"));
                if let Some(ref map) = fm {
                    if let Some(tools) = map.get("tools") {
                        findings.extend(check_tool_values(&stem, tools, "tools"));
                    }
                }
            }
        }
    }

    // 2. Commands
    let commands_dir = root.join("commands");
    if fs.exists(&commands_dir) {
        if let Ok(files) = fs.read_dir(&commands_dir) {
            for file in files.iter().filter(|f| f.to_string_lossy().ends_with(".md")) {
                total_checked += 1;
                let content = match fs.read_to_string(file) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                let stem = file.file_stem().unwrap_or_default().to_string_lossy();
                let fm = extract_frontmatter(&content);
                let fm_name = fm.as_ref().and_then(|m| m.get("name")).map(|s| s.as_str());
                findings.extend(check_naming_consistency(&stem, fm_name, "command"));
                if let Some(ref map) = fm {
                    if let Some(tools) = map.get("allowed-tools") {
                        findings.extend(check_tool_values(&stem, tools, "allowed-tools"));
                    }
                }
            }
        }
    }

    // 3. Skills — check directories
    let skills_dir = root.join("skills");
    if fs.exists(&skills_dir) {
        if let Ok(entries) = fs.read_dir(&skills_dir) {
            for entry in entries.iter().filter(|e| fs.is_dir(e)) {
                total_checked += 1;
                let dir_name = entry.file_name().unwrap_or_default().to_string_lossy().to_string();

                // Check for empty directory (no .md files)
                if let Ok(children) = fs.read_dir(entry) {
                    let has_md = children.iter().any(|c| c.to_string_lossy().ends_with(".md"));
                    if !has_md {
                        findings.push(LintFinding {
                            severity: LintSeverity::Warn,
                            file: dir_name.clone(),
                            message: format!("skill directory '{dir_name}/' contains no .md files"),
                        });
                        continue;
                    }
                }

                // Read SKILL.md for naming check
                let skill_md = entry.join("SKILL.md");
                if fs.exists(&skill_md) {
                    if let Ok(content) = fs.read_to_string(&skill_md) {
                        let fm = extract_frontmatter(&content);
                        let fm_name = fm.as_ref().and_then(|m| m.get("name")).map(|s| s.as_str());
                        findings.extend(check_naming_consistency(&dir_name, fm_name, "skill"));
                    }
                }
            }
        }
    }

    // 4. Report findings
    let error_count = findings.iter().filter(|f| f.severity == LintSeverity::Error).count();
    let warn_count = findings.iter().filter(|f| f.severity == LintSeverity::Warn).count();

    for f in &findings {
        match f.severity {
            LintSeverity::Error => terminal.stderr_write(&format!("ERROR: {}\n", f.message)),
            LintSeverity::Warn => terminal.stdout_write(&format!("WARN: {}\n", f.message)),
        }
    }

    if error_count == 0 {
        terminal.stdout_write(&format!(
            "Convention check OK: {total_checked} files checked, {warn_count} warnings\n"
        ));
        true
    } else {
        terminal.stderr_write(&format!(
            "Convention check FAILED: {error_count} errors, {warn_count} warnings in {total_checked} files\n"
        ));
        false
    }
}
```

**Changes to `mod.rs`**:
- Add `mod conventions;` to module list
- Add `Conventions` to `ValidateTarget` enum
- Add match arm: `ValidateTarget::Conventions => conventions::validate_conventions(root, fs, terminal)`

**Layers**: UseCase

**Pass condition**: `cargo test -p ecc-app -- conventions` passes (unit tests using InMemoryFileSystem)

### PC-005: CLI wiring

**File**: `crates/ecc-cli/src/commands/validate.rs`

**Changes**:
- Add `/// Validate convention consistency (naming, tools, placement)` variant `Conventions` to `CliValidateTarget`
- Add `CliValidateTarget::Conventions => ecc_app::validate::ValidateTarget::Conventions` to `map_target()`

**Layers**: Adapter

**Pass condition**: `cargo build` succeeds, `cargo run -- validate conventions --ecc-root . --help` works

### PC-006: Fix existing violations

**Finding from exploration**: Commands use tools `LS`, `Skill`, `EnterPlanMode`, `ExitPlanMode`, `TaskCreate`, `TaskUpdate`, `TaskGet`, `TaskList` which must be in `VALID_TOOLS`. These are already included in the PC-001 constant definition above.

**Action**: After PC-005, run `ecc validate conventions --ecc-root .` against the repo. If any naming violations exist (filename != frontmatter name, non-kebab-case filenames), fix the content files. Based on exploration, current agents and skills appear compliant -- all agent filenames match their frontmatter `name` and use kebab-case. Commands may lack `name` frontmatter (WARN, not ERROR).

**Layers**: Entity (constant expansion is already in PC-001)

**Pass condition**: `cargo run -- validate conventions --ecc-root .` exits 0

### PC-007: Meta-test

**File**: `crates/ecc-integration-tests/tests/validate_flow.rs`

**Changes**: Add one test function following the existing `validate_target()` helper pattern:

```rust
#[test]
fn validate_conventions_passes() {
    validate_target("conventions");
}
```

**Layers**: Test

**Pass condition**: `cargo test -p ecc-integration-tests -- validate_conventions` passes

### PC-008: Lint + build gate

**Commands**:
```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo build --release
cargo test
```

**Layers**: Infra (CI only)

**Pass condition**: All four commands exit 0

## Test Strategy

### Unit Tests (PC-001 through PC-003, in `ecc-domain`)

| Test | PC | AC |
|------|----|----|
| `is_kebab_case` accepts `my-agent`, `a`, `a1`, `abc-def-ghi` | PC-001 | AC-001.2 |
| `is_kebab_case` rejects `MyAgent`, `my_agent`, `-bad`, `bad-`, `BAD`, `` | PC-001 | AC-001.2 |
| `parse_tool_list` parses `["Read", "Write"]` | PC-002 | AC-002.1 |
| `parse_tool_list` parses bare `Read` as single element | PC-002 | AC-002.1 |
| `parse_tool_list` parses `[]` as empty vec | PC-002 | AC-002.5 |
| `parse_tool_list` handles whitespace: `[ Read , Write ]` | PC-002 | AC-002.1 |
| `check_naming_consistency` returns ERROR when stem != name | PC-003 | AC-001.1 |
| `check_naming_consistency` returns WARN when name is None | PC-003 | AC-002.4 |
| `check_naming_consistency` returns empty when stem == name | PC-003 | AC-001.1 |
| `check_naming_consistency` returns ERROR for non-kebab stem | PC-003 | AC-001.2 |
| `check_tool_values` returns ERROR for unknown tool | PC-003 | AC-002.2 |
| `check_tool_values` returns empty for all-valid tools | PC-003 | AC-002.2 |
| `check_tool_values` returns empty for empty list | PC-003 | AC-002.5 |

### Unit Tests (PC-004, in `ecc-app`)

| Test | AC |
|------|----|
| No agents/commands/skills dirs -> OK with 0 checked | AC-001.4 |
| Agent with mismatched name -> ERROR, returns false | AC-001.1 |
| Agent with invalid tool -> ERROR, returns false | AC-002.2 |
| Command with invalid allowed-tool -> ERROR, returns false | AC-002.3 |
| All valid -> returns true, prints OK count | AC-001.4 |
| WARN-only findings -> returns true (exit 0) | AC-002.6 |
| Mixed ERROR + WARN -> returns false (exit 1) | AC-002.6 |
| Skill dir with no .md files -> WARN | AC-003.1 |
| Skill dir with SKILL.md -> naming check runs | AC-001.3 |
| Skill dir with SKILL.md, all clean -> no warnings | AC-003.2 |

### Integration Test (PC-007)

| Test | AC |
|------|----|
| `ecc validate conventions --ecc-root <workspace>` exits 0 | AC-004.1 |

## Risks & Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| `VALID_TOOLS` list incomplete (new Claude Code tools added) | Medium | Meta-test will catch immediately; constant is easy to update |
| `extract_frontmatter` multiline values break tool parsing | Low | Existing function only extracts single-line key:value pairs; tool lists are always single-line |
| Skill directories with nested subdirectories confuse `read_dir` | Low | Only check immediate children of `skills/`; `read_dir` returns direct children only |
| Commands may not have `name` frontmatter field | Low | Missing `name` produces WARN (not ERROR) per AC-002.4; no false positives |

## Rollback Plan

All changes are additive. Revert the full commit range to undo. Content fixes from PC-006 are cosmetic filename/frontmatter corrections — reverting them restores the pre-existing (non-conforming) state harmlessly.

Reverse order:
1. Revert integration test (PC-007)
2. Revert content fixes (PC-006)
3. Revert CLI wiring (PC-005)
4. Revert app orchestrator (PC-004)
5. Revert domain functions (PC-001-003)

## SOLID Assessment

PASS — Pure domain functions with zero I/O (SRP, DIP). App orchestrates via FileSystem port (DIP). CLI thin dispatch (SRP). No new interfaces needed (ISP). Additive changes only (OCP).

## Robert's Oath Check

CLEAN — Proof via 23+ unit tests + meta-test. Small atomic PCs. No mess introduced.

## Security Notes

CLEAR — Reads local markdown files only. No user input, no injection surfaces, no secrets.

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CLAUDE.md | Project | Add CLI command | `ecc validate conventions` | US-001 |
| 2 | CHANGELOG.md | Project | Add entry | Convention linting subcommand | US-001 |

## Boy Scout Candidates

During REFACTOR steps, scan for:
- `validate.rs`: the existing `extract_frontmatter` tests could use `assert_eq!` with better messages
- `agents.rs`: `required_fields` array could be a domain constant
- Any `unwrap_or_default()` on file_name that should propagate errors
