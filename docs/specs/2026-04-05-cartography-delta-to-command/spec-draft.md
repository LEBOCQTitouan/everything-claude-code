# Spec: Create /doc-suite Command with Cartography Delta Processing

## Problem Statement

The cartography system's two-phase design (Stop hook writes deltas, SessionStart processes them) has a broken second phase: `start_cartography` exits early because `CLAUDE_PROJECT_DIR` is unavailable at hook execution time, and the 30-second async timeout is insufficient for spawning a `claude --agent` subprocess. This results in delta files accumulating indefinitely with no user visibility. Additionally, the `doc-orchestrator` agent exists but has no slash command to invoke it — documentation generation requires manually dispatching the agent. The cartography handler is 2,728 lines (3.4x the 800-line max), mixes domain logic with infrastructure, uses a TOCTOU file lock, has slug derivation split across 3 locations, and couples to the agent system via unvalidated string protocols.

## Research Summary

- Hooks should be lightweight validators/notifiers, not processing engines — heavy work belongs in explicit commands
- Moving from implicit to explicit invocation improves debuggability (silent failure → visible error)
- Doc generation pipelines are the natural home for cartography processing — it produces documentation artifacts
- File-level locking should use OS-level primitives (POSIX flock) not check-then-act patterns
- Domain logic extraction (classify_file, detect_project_type) should precede the structural move to keep each step behavior-preserving
- Slug derivation consolidation prevents latent correctness bugs from divergent implementations
- The principle of least surprise: cartography docs should be generated with all other docs

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Create `/doc-suite` command that invokes the doc-orchestrator agent | Agent exists but has no command; users need a slash command to run the doc pipeline | No |
| 2 | Add cartography phase to doc-orchestrator (Phase 1.5) | Processing requires Claude session for agent dispatch; doc pipeline is the natural home | Yes |
| 3 | Keep `stop:cartography` hook unchanged | It works correctly, is fast, and correctly placed | No |
| 4 | Reduce `start:cartography` to thin delta-check + reminder | Session start should be lightweight (<100ms); remind user to run `/doc-suite` | No |
| 5 | Delegate cartography `detect_project_type` to existing `ecc-app::detection::framework::detect_project_type` | Cartography handler has a simplified duplicate; the detection framework is richer and already tested | No |
| 6 | Extract `classify_file` to `ecc-domain::cartography::classification` as a pure function | Pure path-based classification with no I/O; belongs in domain | No |
| 7 | Wire slug derivation through `derive_slug` domain function | CCP violation — 3 divergent implementations | No |
| 8 | Use shell `flock(1)` for concurrency in doc-orchestrator cartography phase | `ecc-flock` is a Rust crate; doc-orchestrator is a markdown agent — shell flock bridges the gap | No |
| 9 | Remove git commit from cartographer agent; doc-orchestrator commits once after all deltas | Eliminates duplicate commit authority; single transaction for all cartography changes | No |
| 10 | Add trait abstractions to `ecc-domain::cartography` | SAP violation — D=1.00 Zone of Pain | No |
| 11 | Define JSON envelope for agent output protocol | Replace content-sniffing with structured format | No |
| 12 | Consolidate import paths through public re-exports | CRP violation — bypasses module boundary | No |
| 13 | Wire `derive_slug` in app layer (remove unused export or connect it) | CRP violation — exported but unused | No |
| 14 | Use `Handler` trait for hook dispatch | Reduce dispatch coupling (D=0.50); trait chosen over registry for compile-time safety | No |
| 15 | Add agent name validation test | Runtime string coupling with no compile-time check | No |
| 16 | Improve module-level doc comments with `#![warn(missing_docs)]` | Bus factor mitigation; enforced by lint | No |
| 17 | Keep current hook naming (`start:cartography`, `stop:cartography`) | Arch-reviewer LOW finding; current naming is internally consistent within cartography; no rename needed | No |

## User Stories

### US-001: /doc-suite Command

**As a** developer, **I want** a `/doc-suite` slash command that invokes the doc-orchestrator agent, **so that** I can run the full documentation pipeline from a Claude session.

#### Acceptance Criteria

- AC-001.1: A `commands/doc-suite.md` file exists with correct frontmatter and invokes the `doc-orchestrator` agent
- AC-001.2: The command passes through all arguments (`--scope`, `--phase`, `--dry-run`, etc.) to the doc-orchestrator agent
- AC-001.3: The command is listed in `docs/commands-reference.md`

#### Dependencies

- Depends on: none

### US-002: Cartography Phase in Doc-Orchestrator

**As a** developer, **I want** `/doc-suite` to process pending cartography deltas as part of the documentation pipeline, **so that** cartography docs are generated alongside all other documentation.

#### Acceptance Criteria

- AC-002.1: Given pending delta files in `.claude/cartography/`, when `/doc-suite` runs, then a cartography phase dispatches the cartographer agent for each delta
- AC-002.2: Given no pending deltas, when `/doc-suite` runs, then the cartography phase is skipped with a log message "No pending cartography deltas"
- AC-002.3: Given the cartography phase completes successfully, then processed deltas are archived to `.claude/cartography/processed/`
- AC-002.4: Given a delta processing failure, then the error is reported, the failed delta is NOT archived, and remaining deltas continue processing
- AC-002.5: The cartography phase runs after Phase 1 (Discovery) and before Phase 2 (Generation). The doc-orchestrator's `--phase` enum is extended to include `cartography` alongside existing values (`plan|analyze|cartography|generate|validate|coverage|diagrams|readme|claude-md|all`)
- AC-002.6: Given a malformed delta JSON file, then it is logged as error, skipped (not archived), and processing continues with remaining deltas
- AC-002.7: Given the `.claude/cartography/` directory does not exist, then the cartography phase is skipped (same behavior as no pending deltas)
- AC-002.8: Given concurrent `/doc-suite` invocations, then only one processes deltas at a time via shell `flock(1)` on `.claude/cartography/cartography-merge.lock`
- AC-002.9: All cartographer invocations write to the working tree; doc-orchestrator performs a single `git add docs/cartography/ && git commit -m "docs: process cartography deltas"` after all deltas succeed
- AC-002.10: The doc-orchestrator's TodoWrite checklist includes "Phase 1.5: Cartography" between Phase 1 and Phase 2

#### Dependencies

- Depends on: US-001

### US-003: Thin SessionStart Hook

**As a** developer, **I want** the session start to be fast and informative, **so that** I know when deltas are pending without blocking initialization.

#### Acceptance Criteria

- AC-003.1: Given pending deltas at session start, when the hook runs, then it prints "N pending cartography deltas — run `/doc-suite` to process"
- AC-003.2: Given no pending deltas, when the hook runs, then it exits silently (passthrough)
- AC-003.3: The hook completes in <100ms — it uses stat-walk to find `.claude/cartography/` and counts `pending-delta-*.json` files without subprocess spawning (no `git rev-parse`, no `claude`)
- AC-003.4: The hook uses CWD as project root (no `CLAUDE_PROJECT_DIR` dependency)

#### Dependencies

- Depends on: US-002 (doc-orchestrator must have the phase before the hook can reference `/doc-suite`)

### US-004: Consolidate detect_project_type

**As a** maintainer, **I want** a single project type detection implementation, **so that** there is no duplication between cartography and the detection framework.

#### Acceptance Criteria

- AC-004.1: The cartography handler's `detect_project_type` is removed
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
- AC-006.2: The cartographer agent definition references the domain function by name: "Use the `derive_slug` algorithm (lowercase, non-alnum→hyphen, collapse, 60-char truncate)"
- AC-006.3: No inline slug derivation remains in the app layer

#### Dependencies

- Depends on: US-005

### US-007: File Decomposition

**As a** maintainer, **I want** the cartography code split into focused modules under 800 lines each.

#### Acceptance Criteria

- AC-007.1: `stop_cartography` handler in its own module `delta_writer.rs`
- AC-007.2: Remaining `start_cartography` logic (thin hook) in `delta_reminder.rs`
- AC-007.3: Shared helpers (scaffold, delta collection) in `helpers.rs`
- AC-007.4: Each resulting file is under 800 lines
- AC-007.5: All 27 existing tests continue to pass
- AC-007.6: Delta processing logic (agent invocation, output validation, archiving) is removed from the hook handler entirely — it now lives only in the doc-orchestrator agent definition

#### Dependencies

- Depends on: US-003, US-004, US-005 (thin hook, detection consolidation, and domain extraction complete first)

### US-008: Remaining Smell Remediation

**As a** maintainer, **I want** remaining architectural smells addressed for health.

#### Acceptance Criteria

- AC-008.1: `ecc-domain::cartography` has at least one trait (e.g., `CartographyDocument` trait for journey/flow/element types) to improve SAP score
- AC-008.2: Cartographer agent output uses a JSON envelope: `{"status": "success"|"error", "type": "journey"|"flow"|"element", "file_path": "<relative-path>", "content": "<markdown>", "error": "<message>|null"}`
- AC-008.3: All cartography imports use public re-exports from `ecc_domain::cartography` — no `ecc_domain::cartography::validation::*` or `ecc_domain::cartography::element_types::*` direct paths
- AC-008.4: Hook dispatch uses a `Handler` trait: `fn handle(&self, stdin: &str, ports: &HookPorts) -> HookResult`
- AC-008.5: A test validates that every agent name string in hook handlers matches an existing file in `agents/`
- AC-008.6: All cartography modules (domain and app) have `//!` module-level doc comments; `ecc-domain/src/cartography/mod.rs` has `#![warn(missing_docs)]`

#### Dependencies

- Depends on: US-007 (file decomposition enables cleaner remediation)

### US-009: Test Coverage Improvement

**As a** maintainer, **I want** comprehensive test coverage before and after the refactoring.

#### Acceptance Criteria

- AC-009.1: Unit test for thin hook reminder output (prints count when deltas pending, passthrough when none)
- AC-009.2: Tests for `classify_file` in `ecc-domain::cartography::classification` (moved from hook handler tests)
- AC-009.3: Test verifying agent name `"cartographer"` matches an existing file in `agents/`
- AC-009.4: Tests for: (a) agent invocation returning non-zero exit code, (b) agent output not matching JSON envelope, (c) delta archive failure after successful processing
- AC-009.5: Safety-net tests for currently untested paths written BEFORE the refactoring move begins

#### Dependencies

- Depends on: none (AC-009.5 must be completed BEFORE US-004, US-005, US-006, US-007 begin)

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `commands/doc-suite.md` (new) | Command | Create slash command invoking doc-orchestrator |
| `agents/doc-orchestrator.md` | Agent | Add Phase 1.5: Cartography, extend `--phase` enum |
| `agents/cartographer.md` | Agent | Remove Step 5 (git ops), update slug reference, define JSON output envelope |
| `ecc-domain::cartography` | Domain | Add trait abstraction, `#![warn(missing_docs)]` |
| `ecc-domain::cartography::classification` (new) | Domain | `classify_file` extracted here |
| `ecc-app::hook::handlers::tier3_session::cartography` | App (hook) | Decompose into `delta_writer.rs`, `delta_reminder.rs`, `helpers.rs`; remove delta processing |
| `ecc-app::hook::mod` | App | Add `Handler` trait for dispatch |
| `hooks/hooks.json` | Config | Update `start:cartography` handler to thin version |
| `docs/commands-reference.md` | Docs | Add `/doc-suite` entry |

## Constraints

- All refactoring steps must be behavior-preserving (existing tests stay green after each step)
- Safety-net tests (US-009 AC-009.5) must be written BEFORE the refactoring move begins
- The `stop:cartography` hook is NOT touched (except wiring to `derive_slug` and delegation to detection framework)
- Each user story is independently shippable
- The doc-orchestrator and cartographer are markdown agent definitions — changes to them are markdown edits, not Rust code

## Non-Requirements

- No changes to the delta JSON schema (format written by `stop:cartography`)
- No changes to `cartography-journey-generator`, `cartography-flow-generator`, or `cartography-element-generator` agents
- No new `ecc` CLI subcommands (the integration is via slash command + agent, not Rust binary)
- No changes to `ecc-domain::cartography` types beyond adding `classification` submodule and one trait abstraction
- No changes to the cartography validation logic (`validate_cartography.rs`)
- No changes to existing doc-orchestrator phases 0-7 (cartography is additive)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| Agent dispatch (cartographer) | Moved from hook to doc-orchestrator | Now runs inside Claude session natively |
| `FileSystem` (delta read/archive) | Stays in hook handler (thin count) + doc-orchestrator (full processing) | Read in hook for count; full processing in doc-orchestrator |
| `TerminalIO` (hook reminder output) | New | Thin hook prints reminder to terminal |
| Doc-orchestrator pipeline | Extended | New cartography phase added |
| `/doc-suite` command | New | New user-facing slash command |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New command | commands/ | doc-suite.md | Create command definition |
| Command reference | docs/ | commands-reference.md | Add `/doc-suite` entry |
| Doc-orchestrator update | agents/ | doc-orchestrator.md | Add Phase 1.5 and `cartography` to `--phase` enum |
| Architecture change | docs/adr/ | New ADR | Hook-to-doc-orchestrator rationale |
| Agent update | agents/ | cartographer.md | Remove git ops, update slug, add JSON envelope |
| CLAUDE.md | CLAUDE.md | Slash Commands section | Add `/doc-suite` |

## Open Questions

None — all resolved during grill-me interview and adversarial review.
