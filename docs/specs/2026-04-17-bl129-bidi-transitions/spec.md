# Spec: BL-129 Bidirectional Pipeline Transitions

## Problem Statement

The ECC workflow state machine is forward-only: transitions like implement→solution or solution→plan are rejected with `IllegalTransition`. When a design flaw is discovered during implementation, the only recovery is to reset the entire workflow and restart from idle — losing all accumulated artifacts and context. There is no transition audit trail, making it impossible to analyze pipeline friction or debug why phase changes were needed. The domain workflow module has D=1.00 (Zone of Pain) due to 11 concrete types with zero trait abstractions, despite 6 crates depending on it.

## Research Summary

- Enum-based state machines model transitions as (State, Event) match patterns; backward transitions require explicit reverse-direction handling
- Industry standard for audit trails is immutable transition logs with (from, to, event, timestamp, justification) — event-sourcing style
- Rust type-state pattern encodes state at compile-time but is not suited for arbitrary rollbacks; enum + policy approach is more flexible
- Forward-only + compensating actions is safer than true rollback for data integrity (Expand-Contract pattern)
- Partial rollback ambiguity is a key pitfall: backward transitions can leave intermediate artifacts stale; phase gates must validate invariants before allowing reversal
- No cap on transition history needed: each workflow session resets on completion, so history only accumulates within a single spec→implement cycle (~20 entries max)

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Replace `matches!` table with `TransitionPolicy` type | Open for extension, allows forward/backward pair lists as data | Yes |
| 2 | Mandatory justification string for backward transitions | Audit trail requires reason for rollback; forward transitions have `None` | No |
| 3 | Transition history as `Vec<TransitionRecord>` in `state.json` | No separate file needed; history is finite per workflow session | No |
| 4 | `TransitionPolicy::default()` returns current forward-only pairs | Backward compat: existing callers pass default, zero behavioral change | No |
| 5 | Artifact clearing on backward transition (nullify rolled-back phase timestamps) | Prevents stale artifact timestamps from misleading staleness checks | No |
| 6 | Extract `TransitionResolver` trait from concrete `resolve_transition` | Addresses COMP-001 (SAP: D=1.00); allows consumers to depend on behavior not concrete struct | Yes |
| 7 | No cap on history length | History is finite per workflow session (~20 entries max); reset on completion | No |
| 8 | Backfill existing transition tests before refactoring | User-requested upfront safety net | No |
| 9 | `actor` field in TransitionRecord is always "ecc-workflow" for CLI | Binary name is deterministic; future API callers may override | No |
| 10 | Artifact clearing includes both timestamps AND path fields | Stale paths (spec_path, design_path) are equally dangerous as stale timestamps | No |

## User Stories

### US-001: TransitionPolicy type + SAP trait extraction

**As a** developer, **I want** the transition table to be a data-driven policy type with a trait abstraction, **so that** backward transitions can be added without modifying existing logic and consumers can depend on behavior rather than concrete types.

#### Acceptance Criteria

- AC-001.1: Given the new `TransitionPolicy` type, when `TransitionPolicy::default()` is called, then it returns the current 5 forward pairs + Done→Idle
- AC-001.2: Given the `TransitionResolver` trait with method `fn resolve(&self, from: Phase, to: Phase, justification: Option<&str>) -> Result<TransitionResult, WorkflowError>`, when called via the trait with `TransitionPolicy::default()`, then existing forward transitions produce identical results to the current `resolve_transition` function
- AC-001.3: Given all existing transition tests, when run after the refactoring, then they all pass without modification (backward compat)
- AC-001.4: Given the `TransitionPolicy`, when backward pairs are added, then `resolve_transition` accepts them with a justification string

#### Dependencies

- Depends on: none

### US-002: Backward transitions with justification

**As a** developer, **I want** to transition backward (implement→solution, solution→plan, implement→plan) with a mandatory justification, **so that** design flaws discovered during implementation can be addressed without resetting the entire workflow.

#### Acceptance Criteria

- AC-002.1: Given the current phase is `implement`, when `ecc-workflow transition solution --justify "design flaw in X"` is run, then the phase changes to `solution`
- AC-002.2: Given a backward transition, when no justification is provided, then the transition is rejected with a clear error message
- AC-002.3: Given a backward transition from `implement` to `solution`, when the transition succeeds, then `artifacts.solution` and `artifacts.implement` timestamps are cleared (nullified)
- AC-002.4: Given a backward transition from `solution` to `plan`, when the transition succeeds, then `artifacts.plan` and `artifacts.solution` timestamps are cleared
- AC-002.5: Given forward re-entry after a backward transition (e.g., plan→solution after rollback), when the transition succeeds, then the artifact timestamp for that phase is re-stamped with the current time (not the original timestamp)
- AC-002.6: Given a backward transition from `implement` to `plan` (two-phase skip), when the transition succeeds, then `artifacts.plan`, `artifacts.solution`, and `artifacts.implement` timestamps are all cleared, and `spec_path`/`design_path`/`tasks_path` are also cleared
- AC-002.7: Given a backward transition with an empty or whitespace-only justification string, when the transition is attempted, then it is rejected with error "justification must be non-empty"
- AC-002.8: Given any backward transition, when artifact clearing occurs, then both the timestamp fields (e.g., `artifacts.solution`) AND the path fields (e.g., `design_path`) for the rolled-back phases are nullified

#### Dependencies

- Depends on: US-001

### US-003: Transition history

**As a** developer, **I want** every phase transition (forward and backward) to be logged to an append-only history, **so that** I can analyze pipeline friction and debug phase transition issues.

#### Acceptance Criteria

- AC-003.1: Given any successful transition (forward or backward), when the transition completes, then a `TransitionRecord` is appended to `state.json.history[]`
- AC-003.2: Given a `TransitionRecord`, it contains: `from` (Phase), `to` (Phase), `direction` (forward/backward), `justification` (Option<String>), `timestamp` (ISO 8601), `actor` (string — always "ecc-workflow" for CLI-initiated transitions; future API callers may set a different value)
- AC-003.3: Given `ecc workflow history` is run, then the transition history is displayed in chronological order with all fields
- AC-003.4: Given `ecc workflow history --json` is run, then the history is output as JSON array
- AC-003.5: Given a workflow reset (`ecc-workflow reset`), the history is preserved in the reset archive (not lost)

#### Dependencies

- Depends on: US-001

### US-004: Upfront test backfill

**As a** developer, **I want** integration-level tests for the existing workflow transition lifecycle, **so that** the refactoring has a safety net beyond unit tests.

#### Acceptance Criteria

- AC-004.1: Given the full forward lifecycle (idle→plan→solution→implement→done→idle), when run as an integration test, then all transitions succeed and state is correct at each step
- AC-004.2: Given the existing 25+ unit tests in transition.rs, when run after the refactoring, then all pass without modification

#### Dependencies

- Depends on: none (runs first)

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `crates/ecc-domain/src/workflow/transition.rs` | Domain | Replace `matches!` with `TransitionPolicy`, add `TransitionResolver` trait, `TransitionRecord`, `Direction` enum |
| `crates/ecc-domain/src/workflow/state.rs` | Domain | Add `history: Vec<TransitionRecord>` to `WorkflowState`, add artifact clearing logic |
| `crates/ecc-domain/src/workflow/mod.rs` | Domain | Re-export new types |
| `crates/ecc-domain/src/workflow/phase_verify.rs` | Domain | Add backward-direction hints to `phase_hint()` |
| `crates/ecc-workflow/src/commands/transition.rs` | Binary | Pass justification from CLI args, append TransitionRecord, clear artifacts on backward |
| `crates/ecc-workflow/src/commands/history.rs` | Binary | New command: display transition history (text + JSON) |
| `crates/ecc-workflow/src/main.rs` | Binary | Wire `history` subcommand |
| `crates/ecc-cli/src/commands/workflow.rs` | CLI | Pass `--justify` flag, add `history` subcommand delegation |

## Constraints

- All existing 25+ transition unit tests must pass without modification after refactoring
- `TransitionPolicy::default()` must produce identical behavior to current `matches!` table
- `ecc-domain` must remain pure (no I/O) — new types are `Serialize`/`Deserialize` only
- Forward transitions must NOT require justification (backward compat)
- `state.json` must remain forward-compatible: older readers ignore unknown fields
- Backward transitions: implement→solution, solution→plan, implement→plan. Done→Idle remains a legal forward transition (existing behavior). No new backward transitions FROM Done (Done→plan, Done→solution, etc. are illegal — use reset + re-init instead)

## Non-Requirements

- Not addressing COMP-003 (phase_gate decomposition)
- Not addressing COMP-004 (binary wiring dedup)
- Not addressing COMP-002 (state_resolver traits)
- Not adding a transition validation graph beyond forward/backward pairs
- Not auto-triggering commands on backward transition (user must manually run `/design`)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| ecc-workflow binary | New subcommand + modified transition | Integration test for backward transition + history |
| ecc-cli workflow | New --justify flag + history delegation | CLI integration test |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New domain types | Domain | docs/domain/bounded-contexts.md | Add TransitionPolicy, TransitionRecord to workflow context |
| ADR | Docs | docs/adr/0064-bidirectional-transitions.md | Bidi transitions architecture |
| ADR | Docs | docs/adr/0065-workflow-sap-trait-extraction.md | SAP improvement |
| CLI commands | Project | CLAUDE.md | Add `ecc workflow history`, document backward transitions |
| Changelog | Docs | CHANGELOG.md | Add BL-129 entry |

## Open Questions

None — all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Smell triage (10 smells) | Address #1,2,3,6,8; defer #4,5,7,9,10 | User (broader: also #3 SAP) |
| 2 | Target architecture | TransitionPolicy + justification + TransitionResolver trait | Recommended |
| 3 | Step independence | 3 steps, 1-2 coupled, 3 independent | Recommended |
| 4 | Downstream dependencies | Default policy const for backward compat | Recommended |
| 5 | Rename vs behavioral | All additive except resolve_transition sig | Recommended |
| 6 | Performance budget | No cap on history — finite per session | User |
| 7 | ADR decisions | Two ADRs: bidi transitions + SAP trait | User |
| 8 | Test safety net | Upfront backfill before refactoring | User |
| Smells addressed | #1 (transition table), #2 (artifact rollback), #3 (SAP), #6 (phase_hint), #8 (history) | — | — |
| Smells deferred | #4 (phase_gate), #5 (wiring), #7 (state_resolver), #9 (bus factor), #10 (allowlist) | — | — |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | TransitionPolicy + SAP trait | 4 | none |
| US-002 | Backward transitions + justification | 8 | US-001 |
| US-003 | Transition history | 5 | US-001 |
| US-004 | Upfront test backfill | 2 | none |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | TransitionPolicy::default() returns current pairs | US-001 |
| AC-001.2 | TransitionResolver trait with method signature | US-001 |
| AC-001.3 | Existing tests pass without modification | US-001 |
| AC-001.4 | Policy accepts backward pairs with justification | US-001 |
| AC-002.1 | implement→solution with --justify succeeds | US-002 |
| AC-002.2 | Missing justification rejected | US-002 |
| AC-002.3 | implement→solution clears artifacts | US-002 |
| AC-002.4 | solution→plan clears artifacts | US-002 |
| AC-002.5 | Forward re-entry re-stamps timestamps | US-002 |
| AC-002.6 | implement→plan clears all three phases | US-002 |
| AC-002.7 | Empty/whitespace justification rejected | US-002 |
| AC-002.8 | Path fields cleared alongside timestamps | US-002 |
| AC-003.1 | TransitionRecord appended on every transition | US-003 |
| AC-003.2 | Record contains from/to/direction/justification/timestamp/actor | US-003 |
| AC-003.3 | ecc workflow history displays chronologically | US-003 |
| AC-003.4 | ecc workflow history --json outputs JSON | US-003 |
| AC-003.5 | Reset preserves history in archive | US-003 |
| AC-004.1 | Full lifecycle integration test | US-004 |
| AC-004.2 | Existing unit tests pass unchanged | US-004 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Ambiguity | 75 | PASS | Actor defined, trait shape specified, paths explicit |
| Edge Cases | 68 | PASS | Two-phase skip, empty justification, Done→Idle fixed |
| Scope | 85 | PASS | Non-requirements explicit, SAP justified |
| Dependencies | 80 | PASS | US ordering correct, default policy backward compat |
| Testability | 82 | PASS | All ACs deterministically assertable |
| Decisions | 72 | PASS | 10 decisions, 2 ADRs |
| Rollback | 75 | PASS | Artifact clearing semantics defined for all paths |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-04-17-bl129-bidi-transitions/spec.md | Full spec |
| docs/specs/2026-04-17-bl129-bidi-transitions/campaign.md | Campaign manifest |
