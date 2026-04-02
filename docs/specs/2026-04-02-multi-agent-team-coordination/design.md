# Design: Multi-Agent Team Coordination (BL-104)

## Overview

Add declarative team manifests (Markdown + YAML frontmatter) to ECC, with Rust-side parsing and validation (`ecc validate teams`), install-time deployment, two new skills, and command integration via Markdown edits. The design follows ECC's content-layer-primary approach: most changes are Markdown files; Rust changes are limited to parsing, validation, and install wiring.

## Architecture Decisions

| # | Decision | Rationale |
|---|----------|-----------|
| 1 | Parse team frontmatter with `serde_saphyr` (already a dependency) | Frontmatter contains nested `agents:` list; the existing `extract_frontmatter` line-by-line parser cannot handle sequences. `serde_saphyr` is already used in `ecc-domain` for backlog and audit-web. |
| 2 | Domain types in `ecc-domain::config::team` (pure, no I/O) | Follows hexagonal architecture: domain crate owns types + validation, app crate does I/O |
| 3 | Validation cross-references agent names by listing `agents/*.md` via FileSystem port | No new port trait needed; reuses existing `FileSystem::read_dir` pattern from agents/skills validators |
| 4 | Install merges `teams/` using existing `merge_directory` helper | Same pattern as agents/commands install -- no new merge logic needed |

## File Changes Table (dependency order)

| # | File | Action | Layer | Spec Ref | Depends On |
|---|------|--------|-------|----------|------------|
| 1 | `crates/ecc-domain/src/config/team.rs` | **New** | Entity | AC-001.1, AC-001.2, AC-001.3, AC-001.5, AC-001.6, AC-002.2, AC-002.5, AC-002.6 | -- |
| 2 | `crates/ecc-domain/src/config/mod.rs` | **Modify**: add `pub mod team;` | Entity | -- | 1 |
| 3 | `crates/ecc-app/src/validate/teams.rs` | **New** | UseCase | AC-002.1, AC-002.3, AC-002.4 | 1 |
| 4 | `crates/ecc-app/src/validate/mod.rs` | **Modify**: add `Teams` variant + wiring | Adapter | -- | 3 |
| 5 | `crates/ecc-cli/src/commands/validate.rs` | **Modify**: add `Teams` subcommand + mapping | Framework | -- | 4 |
| 6 | `crates/ecc-app/src/install/global/steps.rs` | **Modify**: add teams merge step | UseCase | AC-001.4 | -- |
| 7 | `teams/implement-team.md` | **New** | Content | AC-006.1, AC-006.2 | -- |
| 8 | `teams/audit-team.md` | **New** | Content | AC-006.1, AC-006.3 | -- |
| 9 | `teams/review-team.md` | **New** | Content | AC-006.1, AC-006.4 | -- |
| 10 | `skills/shared-state-protocol/SKILL.md` | **New** | Content | AC-003.1, AC-003.2, AC-003.3 | -- |
| 11 | `skills/task-handoff/SKILL.md` | **New** | Content | AC-004.1, AC-004.2, AC-004.3 | -- |
| 12 | `commands/implement.md` | **Modify**: add team manifest reading + legacy fallback | Content | AC-005.1, AC-005.3, AC-005.5 | 7 |
| 13 | `commands/audit-full.md` | **Modify**: add team manifest reading + legacy fallback | Content | AC-005.2, AC-005.3, AC-005.5 | 8 |

## Domain Types (File 1: `crates/ecc-domain/src/config/team.rs`)

```rust
use serde::Deserialize;

/// Valid coordination strategies for team manifests.
pub const VALID_COORDINATION_STRATEGIES: &[&str] = &["sequential", "parallel", "wave-dispatch"];

/// A parsed team manifest from YAML frontmatter.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TeamManifest {
    pub name: String,
    pub description: String,
    pub coordination: String,
    pub agents: Vec<TeamAgent>,
    #[serde(default)]
    pub max_concurrent: Option<u32>,
}

/// A single agent entry within a team manifest.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TeamAgent {
    pub name: String,
    pub role: String,
    #[serde(default)]
    pub allowed_tools: Option<Vec<String>>,
}

/// Errors from team manifest parsing/validation (pure domain, no I/O).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TeamValidationError {
    /// YAML frontmatter could not be parsed
    ParseError(String),
    /// agents list is empty
    EmptyAgentsList,
    /// Unknown coordination strategy
    UnknownStrategy(String),
    /// Duplicate agent name
    DuplicateAgent(String),
    /// max-concurrent is < 1
    InvalidMaxConcurrent(u32),
}

/// Extract the raw YAML frontmatter string from Markdown content.
///
/// Returns the text between the first `---` and the next `---`.
pub fn extract_frontmatter_raw(content: &str) -> Option<&str> {
    let clean = content.strip_prefix('\u{FEFF}').unwrap_or(content);
    let rest = clean.strip_prefix("---")?;
    let end = rest.find("\n---")?;
    Some(&rest[..end])
}

/// Parse a team manifest from Markdown content.
///
/// Extracts YAML frontmatter and deserializes it into `TeamManifest`.
pub fn parse_team_manifest(content: &str) -> Result<TeamManifest, TeamValidationError> {
    let raw = extract_frontmatter_raw(content)
        .ok_or_else(|| TeamValidationError::ParseError(
            "Missing or malformed YAML frontmatter (no --- delimiters)".to_string()
        ))?;
    serde_saphyr::from_str(raw)
        .map_err(|e| TeamValidationError::ParseError(e.to_string()))
}

/// Validate a parsed team manifest (pure domain rules, no I/O).
///
/// Returns a list of validation errors. Empty = valid.
pub fn validate_team_manifest(manifest: &TeamManifest) -> Vec<TeamValidationError> {
    let mut errors = Vec::new();

    if manifest.agents.is_empty() {
        errors.push(TeamValidationError::EmptyAgentsList);
    }

    if !VALID_COORDINATION_STRATEGIES.contains(&manifest.coordination.as_str()) {
        errors.push(TeamValidationError::UnknownStrategy(
            manifest.coordination.clone(),
        ));
    }

    // Duplicate agent names
    let mut seen = std::collections::HashSet::new();
    for agent in &manifest.agents {
        if !seen.insert(&agent.name) {
            errors.push(TeamValidationError::DuplicateAgent(agent.name.clone()));
        }
    }

    // max-concurrent validation
    if let Some(mc) = manifest.max_concurrent {
        if mc < 1 {
            errors.push(TeamValidationError::InvalidMaxConcurrent(mc));
        }
    }

    errors
}

impl std::fmt::Display for TeamValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(msg) => write!(f, "Parse error: {msg}"),
            Self::EmptyAgentsList => {
                write!(f, "Team manifest must define at least one agent")
            }
            Self::UnknownStrategy(s) => {
                write!(f, "Unknown coordination strategy '{s}'")
            }
            Self::DuplicateAgent(name) => {
                write!(f, "Duplicate agent '{name}' in team manifest")
            }
            Self::InvalidMaxConcurrent(v) => {
                write!(f, "max-concurrent must be >= 1, got {v}")
            }
        }
    }
}
```

## App-Layer Validation (File 3: `crates/ecc-app/src/validate/teams.rs`)

```rust
pub(super) fn validate_teams(
    root: &Path,
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
) -> bool {
    let teams_dir = root.join("teams");
    if !fs.exists(&teams_dir) {
        terminal.stdout_write("No teams directory found, skipping validation\n");
        return true; // AC-002.4
    }

    // Collect known agent names from agents/ directory
    let known_agents: HashSet<String> = collect_agent_names(root, fs);

    let files = match fs.read_dir(&teams_dir) { /* ... */ };
    let md_files: Vec<_> = files.iter().filter(|f| ends_with_md(f)).collect();

    let mut has_errors = false;
    for file in &md_files {
        if !validate_team_file(file, fs, terminal, &known_agents) {
            has_errors = true;
        }
    }

    if has_errors { return false; }
    terminal.stdout_write(&format!("Validated {} team manifests\n", md_files.len()));
    true
}

fn validate_team_file(
    file: &Path,
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
    known_agents: &HashSet<String>,
) -> bool {
    // 1. Read file content
    // 2. parse_team_manifest() -- report parse errors (AC-001.5)
    // 3. validate_team_manifest() -- report domain errors
    // 4. Cross-reference: for each agent.name, check known_agents (AC-002.1)
    // 5. Tool escalation: for each agent with allowed_tools, read the
    //    referenced agent file's tools and warn on extras (AC-002.3)
    //    Warnings go to stdout (exit 0), errors go to stderr (exit 1)
}

fn collect_agent_names(root: &Path, fs: &dyn FileSystem) -> HashSet<String> {
    // Read agents/ directory, extract stem from each .md file
}
```

## Team Manifest Format (Files 7-9)

Example (`teams/implement-team.md`):

```markdown
---
name: implement-team
description: TDD implementation team for /implement Phase 3
coordination: wave-dispatch
max-concurrent: 2
agents:
  - name: tdd-executor
    role: Implements one Pass Condition per invocation using RED-GREEN-REFACTOR
    allowed-tools: ["Read", "Write", "Edit", "MultiEdit", "Bash", "Grep", "Glob"]
  - name: code-reviewer
    role: Reviews implementation quality after each TDD wave
    allowed-tools: ["Read", "Grep", "Glob"]
  - name: module-summary-updater
    role: Updates MODULE-SUMMARIES.md after code changes
    allowed-tools: ["Read", "Write", "Edit", "Grep", "Glob"]
  - name: diagram-updater
    role: Regenerates architecture diagrams after structural changes
    allowed-tools: ["Read", "Write", "Bash", "Grep", "Glob"]
---

# Implement Team

Team manifest for `/implement` Phase 3 (TDD Loop). The orchestrator dispatches
agents in waves controlled by `max-concurrent`.

## Usage

This manifest is read by `/implement` during Phase 3. Each agent entry maps to
a subagent spawn. The `allowed-tools` field restricts the subagent's tool access
below or equal to the agent's own tool list.
```

## Pass Conditions Table

| PC | Description | Spec Ref | Bash Command | Phase |
|----|-------------|----------|--------------|-------|
| PC-001 | `parse_team_manifest` returns `TeamManifest` for valid frontmatter with name, description, agents, coordination | AC-001.1, AC-001.2, AC-001.3 | `cargo test -p ecc-domain config::team::tests::parses_valid_manifest` | 1 |
| PC-002 | `parse_team_manifest` returns `ParseError` for missing `---` delimiters | AC-001.5 | `cargo test -p ecc-domain config::team::tests::rejects_missing_frontmatter` | 1 |
| PC-003 | `parse_team_manifest` returns `ParseError` for malformed YAML (missing closing `---`) | AC-001.5 | `cargo test -p ecc-domain config::team::tests::rejects_unclosed_frontmatter` | 1 |
| PC-004 | `validate_team_manifest` returns `EmptyAgentsList` when `agents: []` | AC-001.6 | `cargo test -p ecc-domain config::team::tests::rejects_empty_agents` | 1 |
| PC-005 | `validate_team_manifest` returns `UnknownStrategy` for `coordination: unknown` | AC-002.2 | `cargo test -p ecc-domain config::team::tests::rejects_unknown_strategy` | 1 |
| PC-006 | `validate_team_manifest` returns `DuplicateAgent` when same name appears twice | AC-002.5 | `cargo test -p ecc-domain config::team::tests::rejects_duplicate_agent` | 1 |
| PC-007 | `validate_team_manifest` returns `InvalidMaxConcurrent` for `max_concurrent: Some(0)` | AC-002.6 | `cargo test -p ecc-domain config::team::tests::rejects_zero_max_concurrent` | 1 |
| PC-008 | `validate_team_manifest` returns empty vec for valid manifest | AC-001.1 | `cargo test -p ecc-domain config::team::tests::valid_manifest_passes` | 1 |
| PC-009 | `TeamAgent.allowed_tools` defaults to `None` when omitted | AC-001.2 | `cargo test -p ecc-domain config::team::tests::allowed_tools_defaults_none` | 1 |
| PC-010 | `validate_teams` returns true and prints "No teams directory found" when dir missing | AC-002.4 | `cargo test -p ecc-app validate::teams::tests::no_dir_succeeds` | 2 |
| PC-011 | `validate_teams` returns false with "Agent 'X' not found in agents/" when agent missing | AC-002.1 | `cargo test -p ecc-app validate::teams::tests::rejects_unknown_agent` | 2 |
| PC-012 | `validate_teams` emits warning (not error) for tool privilege escalation | AC-002.3 | `cargo test -p ecc-app validate::teams::tests::warns_on_tool_escalation` | 2 |
| PC-013 | `validate_teams` returns true for valid manifest with all agents present | AC-001.1 | `cargo test -p ecc-app validate::teams::tests::valid_manifest_passes` | 2 |
| PC-014 | `validate_teams` reports parse error including file path | AC-001.5 | `cargo test -p ecc-app validate::teams::tests::reports_parse_error_with_path` | 2 |
| PC-015 | `ecc validate teams` CLI subcommand is wired and delegates to app layer | -- | `cargo test -p ecc-cli commands::validate::tests::teams_target_maps` | 3 |
| PC-016 | Install step merges `teams/` directory to `~/.claude/teams/` | AC-001.4 | `cargo test -p ecc-app install::global::tests::installs_teams_directory` | 4 |
| PC-017 | `teams/implement-team.md` exists with required frontmatter fields | AC-006.1, AC-006.2 | `cargo test -p ecc-app validate::teams -- --ignored installs_valid_templates` OR `ecc validate teams` | 5 |
| PC-018 | `teams/audit-team.md` exists with coordination: parallel | AC-006.1, AC-006.3 | `ecc validate teams` | 5 |
| PC-019 | `teams/review-team.md` exists with coordination: sequential | AC-006.1, AC-006.4 | `ecc validate teams` | 5 |
| PC-020 | `skills/shared-state-protocol/SKILL.md` exists with required sections | AC-003.1, AC-003.2, AC-003.3 | `ecc validate skills` | 6 |
| PC-021 | `skills/task-handoff/SKILL.md` exists with required sections | AC-004.1, AC-004.2, AC-004.3 | `ecc validate skills` | 6 |
| PC-022 | `/implement` reads `teams/implement-team.md` in Phase 3 | AC-005.1 | [manual review] | 7 |
| PC-023 | `/audit-full` reads `teams/audit-team.md` in parallel dispatch | AC-005.2 | [manual review] | 7 |
| PC-024 | Commands fail with error when no manifest and no `ECC_LEGACY_DISPATCH` | AC-005.3 | [manual review] | 7 |
| PC-025 | Commands emit deprecation warning with `ECC_LEGACY_DISPATCH=1` | AC-005.5 | [manual review] | 7 |
| PC-026 | `max-concurrent` value passed as concurrency cap | AC-005.4 | [manual review] | 7 |

## TDD Phases and Order

### Phase 1: Domain Types and Pure Validation
**Layers**: Entity
**Files**: `crates/ecc-domain/src/config/team.rs`, `crates/ecc-domain/src/config/mod.rs`
**PCs**: PC-001 through PC-009
**Rationale**: Pure domain types with zero dependencies. Foundation for everything else. All tests are unit tests using serde deserialization -- no I/O, no ports.

**RED**: Write 9 tests for parsing + validation in `config::team::tests`
**GREEN**: Implement `TeamManifest`, `TeamAgent`, `TeamValidationError`, `parse_team_manifest`, `validate_team_manifest`
**REFACTOR**: Extract `extract_frontmatter_raw` to reuse in other parsers if applicable

**Boy Scout Delta**: `crates/ecc-domain/src/config/validate.rs` -- the `extract_frontmatter` function duplicates the `---` delimiter logic now factored into `extract_frontmatter_raw`. Consider making the old function delegate to the new one (separate commit).

---

### Phase 2: App-Layer Team Validation (I/O)
**Layers**: UseCase
**Files**: `crates/ecc-app/src/validate/teams.rs`, `crates/ecc-app/src/validate/mod.rs`
**PCs**: PC-010 through PC-014
**Rationale**: Needs Phase 1 domain types. Uses `InMemoryFileSystem` + `BufferedTerminal` test doubles (existing pattern from agents/skills validators).

**RED**: Write 5 tests for the validate_teams function
**GREEN**: Implement `validate_teams`, `validate_team_file`, `collect_agent_names`; add `Teams` variant to `ValidateTarget`; wire in `run_validate` match arm
**REFACTOR**: Extract shared patterns (e.g., "read dir, filter .md, validate each") if agents.rs and teams.rs share boilerplate

**Boy Scout Delta**: `crates/ecc-app/src/validate/agents.rs` -- the `validate_agents` function has an inline `files.iter().filter(|f| f.to_string_lossy().ends_with(".md"))` that could be a small helper shared with teams.

---

### Phase 3: CLI Wiring
**Layers**: Framework
**Files**: `crates/ecc-cli/src/commands/validate.rs`
**PCs**: PC-015
**Rationale**: Thin wiring only. Depends on Phase 2 `ValidateTarget::Teams` variant.

**RED**: Write 1 test verifying the `Teams` variant maps correctly
**GREEN**: Add `Teams` to `CliValidateTarget` enum and `map_target` match arm
**REFACTOR**: None expected (trivial change)

---

### Phase 4: Install Wiring
**Layers**: UseCase
**Files**: `crates/ecc-app/src/install/global/steps.rs`
**PCs**: PC-016
**Rationale**: Uses existing `merge_directory` helper. One-line addition to `step_merge_artifacts`.

**RED**: Write 1 test verifying teams directory is included in merge
**GREEN**: Add `merge::merge_directory` call for `teams/` in `step_merge_artifacts`
**REFACTOR**: None expected

---

### Phase 5: Team Manifest Templates (Content)
**Layers**: Content (no Rust)
**Files**: `teams/implement-team.md`, `teams/audit-team.md`, `teams/review-team.md`
**PCs**: PC-017, PC-018, PC-019
**Rationale**: Content files only. Verified by `ecc validate teams` (from Phase 2).

**RED**: Run `ecc validate teams` -- fails (no teams/ directory)
**GREEN**: Create 3 team manifest files with correct frontmatter
**REFACTOR**: Proofread agent names against `agents/` directory

---

### Phase 6: Skills (Content)
**Layers**: Content (no Rust)
**Files**: `skills/shared-state-protocol/SKILL.md`, `skills/task-handoff/SKILL.md`
**PCs**: PC-020, PC-021
**Rationale**: Content files only. Verified by `ecc validate skills`.

**RED**: Run `ecc validate skills` -- fails (new skill dirs missing)
**GREEN**: Create 2 skill directories with SKILL.md files containing required frontmatter + sections
**REFACTOR**: None

---

### Phase 7: Command Integration (Content)
**Layers**: Content (no Rust)
**Files**: `commands/implement.md`, `commands/audit-full.md`
**PCs**: PC-022 through PC-026
**Rationale**: Markdown-only changes to command files. All PCs are [manual review]. Depends on Phase 5 (team templates must exist for the commands to reference them).

**RED**: [manual] Review current command files -- no team manifest integration
**GREEN**: Add team manifest reading instructions + `ECC_LEGACY_DISPATCH` fallback to both commands
**REFACTOR**: Ensure consistent wording between the two commands

---

## E2E Assessment

- **Touches user-facing flows?** Yes -- `ecc validate teams` CLI command, `ecc install` teams deployment
- **Crosses 3+ modules end-to-end?** Yes -- domain (parsing) -> app (validation) -> CLI (wiring) -> install (deployment)
- **New E2E tests needed?** No -- the existing CI pipeline runs `ecc validate agents`, `ecc validate skills`, etc. Adding `ecc validate teams` to the CI `validate` step is sufficient. Integration tests in Phase 2 already cover the full validation path with in-memory doubles.
- **Existing E2E suite**: Run as gate after all phases: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`

## Risks and Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| `serde_saphyr` deserialize fails on nested YAML sequences in frontmatter | Medium | Phase 1 tests explicitly cover the `agents:` list format; if saphyr fails, we parse manually |
| `extract_frontmatter_raw` duplicates logic from `extract_frontmatter` | Low | Boy Scout Delta in Phase 1 unifies them |
| Command Markdown changes break existing dispatch | Medium | `ECC_LEGACY_DISPATCH=1` fallback (spec decision 7); rollback path documented |
| Team manifests reference agents that get renamed | Low | `ecc validate teams` catches this at CI time (AC-002.1) |

## Commit Plan

| # | Type | Message | Phase |
|---|------|---------|-------|
| 1 | test | `test: add team manifest domain parsing tests (PC-001–009)` | 1 |
| 2 | feat | `feat: implement team manifest domain types and validation` | 1 |
| 3 | refactor | `refactor: extract frontmatter raw helper` | 1 |
| 4 | chore | `chore(scout): unify extract_frontmatter via extract_frontmatter_raw` | 1 |
| 5 | test | `test: add team manifest app-layer validation tests (PC-010–014)` | 2 |
| 6 | feat | `feat: implement ecc validate teams app-layer validation` | 2 |
| 7 | test | `test: add teams CLI wiring test (PC-015)` | 3 |
| 8 | feat | `feat: wire ecc validate teams CLI subcommand` | 3 |
| 9 | test | `test: add teams install merge test (PC-016)` | 4 |
| 10 | feat | `feat: add teams/ to install merge artifacts` | 4 |
| 11 | feat | `feat: add team manifest templates (implement, audit, review)` | 5 |
| 12 | feat | `feat: add shared-state-protocol and task-handoff skills` | 6 |
| 13 | feat | `feat: integrate team manifests into implement and audit-full commands` | 7 |

## Additional Pass Conditions (Gate Checks)

| PC | Description | Spec Ref | Bash Command | Phase |
|----|-------------|----------|--------------|-------|
| PC-027 | Clippy passes with zero warnings | all | `cargo clippy --workspace -- -D warnings` | gate |
| PC-028 | Full workspace builds | all | `cargo build --workspace` | gate |
| PC-029 | All existing tests pass (regression) | all | `cargo test --workspace --exclude xtask` | gate |
| PC-030 | Format check passes | all | `cargo fmt --all -- --check` | gate |

## SOLID Assessment

**Verdict: CLEAN** (uncle-bob). Dependencies point inward at every layer. Domain types are pure with zero I/O. App layer uses port traits only. CLI is thin wiring. SRP, OCP, DIP all satisfied. Design mirrors existing `validate_agents` pattern exactly.

## Robert's Oath Check

**Verdict: CLEAN** (2 minor observations, non-blocking). Tests-first approach. Boy Scout Deltas scoped as separate commits. 7 phases, each independently shippable. ECC_LEGACY_DISPATCH=1 provides rollback. Minor: `max_concurrent: u32` prevents negatives by type system (AC-002.6 "negative" case is already handled).

## Security Notes

**Verdict: CLEAR**. `serde_saphyr` deserialization into typed structs is safe (no eval, no tags). Agent name cross-reference is set-membership check (no path construction from manifest content). Tool escalation is read-only comparison.

## Rollback Plan

Reverse dependency order:

| Order | File | Rollback Action |
|-------|------|-----------------|
| 1 | `commands/implement.md`, `commands/audit-full.md` | Revert team manifest reading + ECC_LEGACY_DISPATCH |
| 2 | `skills/shared-state-protocol/`, `skills/task-handoff/` | Delete directories |
| 3 | `teams/implement-team.md`, `audit-team.md`, `review-team.md` | Delete files |
| 4 | `crates/ecc-app/src/install/global/steps.rs` | Remove teams merge step |
| 5 | `crates/ecc-cli/src/commands/validate.rs` | Remove Teams target |
| 6 | `crates/ecc-app/src/validate/teams.rs` | Delete file, remove from mod.rs |
| 7 | `crates/ecc-domain/src/config/team.rs` | Delete file, remove from mod.rs |

All changes are additive — reverting any phase leaves the system in a valid state.

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CHANGELOG.md | Top | Add entry | "feat: add multi-agent team coordination" | All US |
| 2 | docs/adr/0040-content-layer-team-coordination.md | ADR | Create | Content-layer over Rust execution engine | Decision #1 |
| 3 | CLAUDE.md | Top | Modify | Add teams/ directory, `ecc validate teams`, team manifest gotcha | US-001 |
| 4 | docs/domain/bounded-contexts.md | Glossary | Modify | Add Team Manifest, Coordination Strategy, Role, Handoff Protocol | US-001 |
