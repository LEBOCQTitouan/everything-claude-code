# Spec: Create /doc-suite Command with Cartography Delta Processing

## Problem Statement

Two problems converge: (1) the doc-orchestrator agent exists but has no slash command, so documentation generation requires manual agent dispatch, and (2) the cartography system's SessionStart hook fails silently because `CLAUDE_PROJECT_DIR` is unavailable and the 30s timeout is insufficient for AI agent invocation. The cartography handler is 2,728 lines (3.4x the 800-line max), duplicates `detect_project_type` with the detection framework, has `classify_file` domain logic in the adapter layer, splits slug derivation across 3 locations, uses a TOCTOU file lock, and routes agent output via content-sniffing.

## Research Summary

- Hooks should be lightweight validators/notifiers — heavy work belongs in explicit commands
- Slash commands are stateless orchestrators coordinating agents with focused scopes
- Doc generation follows extract → validate → format → publish pipeline with quality gates between stages
- Port/adapter mapping: move logic from hook adapter to command adapter via shared domain ports
- Moving implicit to explicit invocation improves debuggability (silent failure → visible error)
- Agent orchestration benefits from centralized coordination over distributed decision-making
- Domain logic extraction should precede structural moves to keep each step behavior-preserving

## Decisions Made

| # | Decision | Rationale | ADR? |
|---|----------|-----------|------|
| 1 | Create `/doc-suite` command invoking doc-orchestrator | Agent exists but no command; users need a slash command | No |
| 2 | Add cartography phase to doc-orchestrator as Phase 1.5 | Processing requires Claude session for native agent dispatch | Yes |
| 3 | Extract cartography phase instructions to a skill file | Keep doc-orchestrator under 400 lines (SELF-002) | No |
| 4 | Keep `stop:cartography` hook unchanged | Works correctly, fast, correctly placed | No |
| 5 | Reduce `start:cartography` to thin delta-check + reminder | Session start should be <100ms; point to `/doc-suite` | No |
| 6 | Delegate cartography `detect_project_type` to existing detection framework | Eliminates duplication; framework version is richer and tested | No |
| 7 | Extract `classify_file` to `ecc-domain::cartography::classification` | Pure function, no I/O — belongs in domain | No |
| 8 | Wire slug derivation through `derive_slug` domain function | CCP violation — 3 divergent implementations | No |
| 9 | Use shell `flock(1)` for concurrency in cartography phase | Agent is markdown, not Rust — shell flock bridges the gap | No |
| 10 | Remove git commit from cartographer agent; doc-orchestrator commits once | Single transaction for all cartography changes | No |
| 11 | Define JSON envelope for cartographer agent output | Replace content-sniffing with structured protocol | No |
| 12 | Use `Handler` trait for hook dispatch | Reduce coupling; compile-time safety over runtime registry | Yes |
| 13 | Add SAP trait to `ecc-domain::cartography` | D=1.00 Zone of Pain; needs at least one abstraction | No |
| 14 | Consolidate import paths through public re-exports | CRP violation — bypasses module boundary | No |
| 15 | Add agent name validation test | Runtime string coupling with no compile-time check | No |
| 16 | Add `#![warn(missing_docs)]` to cartography modules | Bus factor mitigation; enforced by lint | No |
| 17 | Keep current hook naming (`start:cartography`, `stop:cartography`) | Internally consistent; LOW severity, no rename needed | No |

## User Stories

### US-001: /doc-suite Command

**As a** developer, **I want** a `/doc-suite` slash command, **so that** I can invoke the documentation pipeline from a Claude session.

#### Acceptance Criteria

- AC-001.1: `commands/doc-suite.md` exists with correct frontmatter invoking doc-orchestrator agent
- AC-001.2: Command passes through all arguments (`--scope`, `--phase`, `--dry-run`, etc.) to the doc-orchestrator agent
- AC-001.3: Command is listed in `docs/commands-reference.md` and CLAUDE.md Slash Commands section

#### Dependencies

- Depends on: none

### US-002: Cartography Phase in Doc-Orchestrator

**As a** developer, **I want** `/doc-suite` to process pending cartography deltas, **so that** cartography docs are generated alongside all other documentation.

#### Acceptance Criteria

- AC-002.1: Given pending delta files in `.claude/cartography/`, when `/doc-suite` runs, then cartography phase dispatches cartographer agent for each delta
- AC-002.2: Given no pending deltas (or `.claude/cartography/` directory missing), the phase is skipped with log "No pending cartography deltas"
- AC-002.3: Given the cartography phase completes successfully, then processed deltas are archived to `.claude/cartography/processed/`
- AC-002.4: Given a delta processing failure, then the error is reported, the failed delta is NOT archived, and remaining deltas continue processing
- AC-002.5: Given a malformed delta JSON file, then it is logged as error, skipped (not archived), and processing continues
- AC-002.6: The doc-orchestrator's `--phase` enum is extended to include `cartography`: `plan|analyze|cartography|generate|validate|coverage|diagrams|readme|claude-md|all`
- AC-002.7: Given concurrent `/doc-suite` invocations, then only one processes deltas at a time via shell `flock(1)` on `.claude/cartography/cartography-merge.lock`
- AC-002.8: All cartographer invocations write to working tree; doc-orchestrator performs a single `git add docs/cartography/ && git commit -m "docs: process cartography deltas"` after all deltas succeed
- AC-002.9: Cartography phase instructions live in `skills/cartography-processing/SKILL.md`, referenced by doc-orchestrator (keeps doc-orchestrator under 400 lines)
- AC-002.10: Doc-orchestrator TodoWrite checklist includes "Phase 1.5: Cartography" between Phase 1 and Phase 2

#### Dependencies

- Depends on: US-001

### US-003: Thin SessionStart Hook

**As a** developer, **I want** the session start to be fast and informative, **so that** I know when deltas are pending without blocking initialization.

#### Acceptance Criteria

- AC-003.1: Given pending deltas at session start, when the hook runs, then it prints "N pending cartography deltas — run `/doc-suite --phase=cartography` to process"
- AC-003.2: Given no pending deltas, when the hook runs, then it exits silently (passthrough)
- AC-003.3: The hook completes in <100ms — stat-walk only, no subprocess spawning, no `CLAUDE_PROJECT_DIR` dependency
- AC-003.4: The hook uses CWD to find `.claude/cartography/` and counts `pending-delta-*.json` files

#### Dependencies

- Depends on: US-002 (doc-orchestrator must have the phase before hook can reference `/doc-suite`)

### US-004: Consolidate detect_project_type

**As a** maintainer, **I want** a single project type detection implementation, **so that** there is no duplication between cartography and the detection framework.

#### Acceptance Criteria

- AC-004.1: The cartography handler's `detect_project_type` function is removed
- AC-004.2: The `stop_cartography` handler delegates to `ecc-app::detection::framework::detect_project_type` and maps the rich `ProjectType` struct to the cartography `ProjectType` enum
- AC-004.3: All existing cartography tests pass after the consolidation
- AC-004.4: No second `detect_project_type` function exists in the cartography module

#### Dependencies

- Depends on: none

### US-005: Extract classify_file to Domain

**As a** maintainer, **I want** `classify_file` in the domain layer, **so that** it's reusable and testable without hook infrastructure.

#### Acceptance Criteria

- AC-005.1: `classify_file` is moved to `ecc-domain::cartography::classification` as a pure function taking `(path: &str, project_type: &ProjectType) -> String`
- AC-005.2: All existing tests pass after the move
- AC-005.3: Hook handler imports `classify_file` from domain instead of containing the logic

#### Dependencies

- Depends on: US-004 (project type consolidation first)

### US-006: Slug Derivation Consolidation

**As a** maintainer, **I want** a single slug derivation implementation, **so that** file paths are consistent across hook, doc-orchestrator, and agent.

#### Acceptance Criteria

- AC-006.1: The `stop_cartography` handler calls `derive_slug()` from `ecc-domain` instead of using raw classification strings
- AC-006.2: The cartographer agent definition references the domain function: "Use the `derive_slug` algorithm (lowercase, non-alnum→hyphen, collapse, 60-char truncate)"
- AC-006.3: No inline slug derivation remains in the app layer

#### Dependencies

- Depends on: US-005

### US-007: File Decomposition

**As a** maintainer, **I want** the cartography code split into focused modules under 800 lines each.

#### Acceptance Criteria

- AC-007.1: `stop_cartography` handler lives in its own module `delta_writer.rs`
- AC-007.2: Thin `start_cartography` (reminder) lives in `delta_reminder.rs`
- AC-007.3: Shared helpers (scaffold, delta collection, file counting) live in `helpers.rs`
- AC-007.4: Each resulting file is under 800 lines
- AC-007.5: All 27 existing tests continue to pass
- AC-007.6: Delta processing logic (agent invocation, output validation, archiving) is removed from the hook handler entirely — it lives only in the doc-orchestrator skill

#### Dependencies

- Depends on: US-003, US-004, US-005 (thin hook, detection consolidation, domain extraction complete first)

### US-008: Structured Agent Output Protocol

**As a** maintainer, **I want** cartographer output to use a structured JSON format, **so that** output routing is explicit rather than content-sniffing.

#### Acceptance Criteria

- AC-008.1: Cartographer agent outputs JSON envelope: `{"status": "success"|"error", "type": "journey"|"flow"|"element", "file_path": "<relative-path>", "content": "<markdown>", "error": "<message>|null"}`
- AC-008.2: Doc-orchestrator cartography phase parses JSON envelope to determine file type and write location
- AC-008.3: Invalid JSON from agent is treated as error (logged, delta not archived, processing continues)

#### Dependencies

- Depends on: US-002

### US-009: Handler Trait for Hook Dispatch

**As a** maintainer, **I want** hook dispatch to use a `Handler` trait, **so that** new handlers can be added without modifying the central match.

#### Acceptance Criteria

- AC-009.1: `Handler` trait defined with signature: `fn handle(&self, stdin: &str, ports: &HookPorts) -> HookResult`
- AC-009.2: Existing hook handlers implement the `Handler` trait
- AC-009.3: Dispatch function uses trait objects or a registry of trait implementors instead of a 70+ arm match
- AC-009.4: Adding a new handler requires implementing the trait and registering it, not modifying the central dispatch function

#### Dependencies

- Depends on: US-007 (file decomposition provides cleaner handler modules to wrap)

### US-010: Remaining Smell Remediation

**As a** maintainer, **I want** remaining architectural smells addressed for codebase health.

#### Acceptance Criteria

- AC-010.1: `ecc-domain::cartography` has at least one trait (e.g., `CartographyDocument` with `fn doc_type(&self) -> &str` and `fn validate(&self) -> Result<()>`) for SAP improvement
- AC-010.2: All cartography imports use public re-exports from `ecc_domain::cartography` — no `ecc_domain::cartography::validation::*` or `ecc_domain::cartography::element_types::*` direct paths
- AC-010.3: A test validates that every agent name string literal used in hook handlers matches an existing file in `agents/`
- AC-010.4: All cartography modules (domain and app) have `//!` module-level doc comments; `ecc-domain/src/cartography/mod.rs` has `#![warn(missing_docs)]`
- AC-010.5: TOCTOU file lock in cartography processing is replaced with shell `flock(1)` in the doc-orchestrator skill
- AC-010.6: `doc-orchestrator.md` stays under 400 lines after adding the cartography phase reference

#### Dependencies

- Depends on: US-007, US-008, US-009

### US-011: Test Coverage

**As a** maintainer, **I want** comprehensive test coverage before and after the refactoring.

#### Acceptance Criteria

- AC-011.1: Safety-net tests written BEFORE refactoring begins: (a) agent invocation returning non-zero exit code, (b) agent output not matching expected format, (c) delta archive failure after successful processing, (d) malformed delta JSON handling
- AC-011.2: Unit test for thin hook reminder output (prints count when deltas pending, passthrough when none)
- AC-011.3: Tests for `classify_file` in `ecc-domain::cartography::classification` (migrated from hook handler tests)
- AC-011.4: Test validating agent name `"cartographer"` matches an existing file in `agents/`
- AC-011.5: E2E test exercising the `/doc-suite --phase=cartography` flow end-to-end

#### Dependencies

- Depends on: none (AC-011.1 must be completed BEFORE US-004, US-005, US-006, US-007 begin)

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `commands/doc-suite.md` (new) | Command | Create slash command invoking doc-orchestrator |
| `agents/doc-orchestrator.md` | Agent | Add Phase 1.5 reference, extend `--phase` enum |
| `skills/cartography-processing/SKILL.md` (new) | Skill | Cartography phase instructions (extracted from doc-orchestrator) |
| `agents/cartographer.md` | Agent | Remove Step 5 (git ops), add JSON envelope output, update slug reference |
| `ecc-domain::cartography::classification` (new) | Domain | `classify_file` extracted here |
| `ecc-domain::cartography` | Domain | Add SAP trait, `#![warn(missing_docs)]` |
| `ecc-app::hook::handlers::tier3_session::cartography` | App (hook) | Decompose into `delta_writer.rs` + `delta_reminder.rs` + `helpers.rs` |
| `ecc-app::detection::framework` | App | No changes — new caller from cartography |
| `ecc-app::hook::mod` | App | Add `Handler` trait for dispatch |
| `hooks/hooks.json` | Config | Update `start:cartography` handler to thin version |
| `docs/commands-reference.md` | Docs | Add `/doc-suite` entry |
| `CLAUDE.md` | Docs | Add `/doc-suite` to Slash Commands section |

## Constraints

- Safety-net tests (US-011 AC-011.1) must be written BEFORE any refactoring begins
- All refactoring steps must be behavior-preserving (tests green after each step)
- The `stop:cartography` hook logic is NOT touched (except wiring `derive_slug` and delegation to detection framework)
- Each user story is independently shippable
- Doc-orchestrator and cartographer are markdown agent definitions — no Rust code changes for the integration itself
- `detect_project_type` I/O stays in adapter layer; only pure classification logic goes to domain

## Non-Requirements

- No changes to the delta JSON schema (format written by `stop:cartography`)
- No changes to `cartography-journey-generator`, `cartography-flow-generator`, or `cartography-element-generator` agents
- No new `ecc` CLI subcommands (integration is via slash command + agent, not Rust binary)
- No changes to `ecc-domain::cartography` value types beyond adding `classification` submodule and one trait
- No changes to `validate_cartography.rs`
- No changes to existing doc-orchestrator phases 0-7 (cartography is additive only)
- No restructuring of the hook tier system beyond adding the `Handler` trait

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| Agent dispatch (cartographer) | Moved from hook to doc-orchestrator | Now runs inside Claude session natively |
| `FileSystem` (delta read/archive) | Hook for count; doc-orchestrator for full processing | Split responsibility |
| `TerminalIO` (hook reminder) | New | Thin hook prints pending delta count |
| Doc-orchestrator pipeline | Extended | New cartography phase added |
| `/doc-suite` command | New | New user-facing slash command |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New command | commands/ | doc-suite.md | Create command definition |
| New skill | skills/ | cartography-processing/SKILL.md | Create skill file |
| Command reference | docs/ | commands-reference.md | Add `/doc-suite` entry |
| ADR | docs/adr/ | Hook-to-doc-orchestrator rationale | Create |
| ADR | docs/adr/ | Handler trait dispatch pattern | Create |
| Agent update | agents/ | cartographer.md | JSON envelope, remove git ops, slug ref |
| Agent update | agents/ | doc-orchestrator.md | Add Phase 1.5 reference, extend --phase |
| CLAUDE.md | CLAUDE.md | Slash Commands section | Add `/doc-suite` |

## Open Questions

None — all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Smell triage (17 smells) | Address all 17 now | User |
| 2 | Target architecture | Accept: /doc-suite + orchestrator phase + thin hook + domain extraction + decomposition + Handler trait | Recommended |
| 3 | Step independence | 10 independent steps, each shippable | Recommended |
| 4 | Downstream dependencies | cargo test + validate hooks after each step + integration test + E2E test | User (added E2E) |
| 5 | Rename vs behavioral change | 5 pure moves, 6 behavioral changes | Recommended |
| 6 | Performance budget | Cap doc-orchestrator at 400 lines by extracting carto to skill | User |
| 7 | ADR decisions | Two ADRs: hook-to-orchestrator + Handler trait | Recommended |
| 8 | Test safety net | Safety-net tests first, before any refactoring | Recommended |

**Smells addressed**: All 17 (#1-#17)
**Smells deferred**: None

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | /doc-suite Command | 3 | none |
| US-002 | Cartography Phase in Doc-Orchestrator | 10 | US-001 |
| US-003 | Thin SessionStart Hook | 4 | US-002 |
| US-004 | Consolidate detect_project_type | 4 | none |
| US-005 | Extract classify_file to Domain | 3 | US-004 |
| US-006 | Slug Derivation Consolidation | 3 | US-005 |
| US-007 | File Decomposition | 6 | US-003, US-004, US-005 |
| US-008 | Structured Agent Output Protocol | 3 | US-002 |
| US-009 | Handler Trait for Hook Dispatch | 4 | US-007 |
| US-010 | Remaining Smell Remediation | 6 | US-007, US-008, US-009 |
| US-011 | Test Coverage | 5 | none (AC-011.1 before others) |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | commands/doc-suite.md exists with frontmatter | US-001 |
| AC-001.2 | Passes through arguments | US-001 |
| AC-001.3 | Listed in docs + CLAUDE.md | US-001 |
| AC-002.1 | Dispatches cartographer for each delta | US-002 |
| AC-002.2 | Skips when no deltas/dir missing | US-002 |
| AC-002.3 | Archives processed deltas | US-002 |
| AC-002.4 | Failed deltas not archived, continues | US-002 |
| AC-002.5 | Malformed JSON skipped | US-002 |
| AC-002.6 | --phase enum extended with cartography | US-002 |
| AC-002.7 | Shell flock concurrency | US-002 |
| AC-002.8 | Single git commit after all deltas | US-002 |
| AC-002.9 | Instructions in skill file | US-002 |
| AC-002.10 | TodoWrite includes Phase 1.5 | US-002 |
| AC-003.1 | Prints pending count + /doc-suite hint | US-003 |
| AC-003.2 | Silent passthrough when no deltas | US-003 |
| AC-003.3 | <100ms, no subprocess | US-003 |
| AC-003.4 | Uses CWD, no env var | US-003 |
| AC-004.1-4 | Remove dup detect_project_type, delegate to framework | US-004 |
| AC-005.1-3 | classify_file to domain | US-005 |
| AC-006.1-3 | Wire derive_slug everywhere | US-006 |
| AC-007.1-6 | Decompose into delta_writer/delta_reminder/helpers | US-007 |
| AC-008.1-3 | JSON envelope for agent output | US-008 |
| AC-009.1-4 | Handler trait for dispatch | US-009 |
| AC-010.1-6 | SAP trait, import cleanup, agent validation, docs, flock, size cap | US-010 |
| AC-011.1-5 | Safety-net first, then per-step tests + E2E | US-011 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Ambiguity | 75 | PASS | JSON envelope schema explicit, Handler trait signature specified |
| Edge Cases | 70 | PASS | Malformed delta, missing dir, concurrent runs all covered |
| Scope Creep Risk | 82 | PASS | 7 Non-Requirements fence scope |
| Dependency Gaps | 80 | PASS | detect_project_type consolidation explicit, shell flock for agent |
| Testability | 72 | PASS | Safety-net before refactor, specific test targets enumerated |
| Decision Completeness | 78 | PASS | 17 decisions, 2 ADRs, consolidation addressed |
| Rollback & Failure | 85 | PASS | Step independence, delta failure handling, no data migrations |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-04-05-doc-suite-with-cartography/spec.md | Full spec + Phase Summary |
| docs/specs/2026-04-05-doc-suite-with-cartography/campaign.md | Grill-me decisions |
