# Spec: BL-068 — Deterministic Workflow State Machine (Remaining Gaps)

## Problem Statement

The ECC workflow state machine (implemented via BL-052) is missing an Idle phase for cycle closure, uses loose JSON parsing in the phase-gate hook (silently defaulting corrupt state to "done"), has no E2E lifecycle test (flagged by audit CORR-001), silently swallows memory write errors during transitions, and doesn't archive state on reset (asymmetric with init). These gaps reduce reliability and debuggability of the spec-driven pipeline.

## Research Summary

- Enum-based state machines are the idiomatic Rust approach; exhaustive `match` ensures every state is handled
- Side-effect failures (like memory writes) should warn, not block — consistent with DDD event handler patterns
- `assert_cmd` + `predicates` crate is recommended for CLI integration testing, but direct function calls are faster for lifecycle tests
- Separate pure state machine logic from I/O — the transition function should be pure, persistence in a wrapper layer
- Always deserialize into typed enums and fail loudly on unknown variants — never default to untyped strings

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Add Idle phase variant to Phase enum | Enables proper cycle closure (Done → Idle) and distinguishes "never initialized" from "completed" | Yes |
| 2 | Implementation order: US-001 → US-002 → US-005 → US-004 → US-003 | Idle phase is foundational; typed parsing depends on it; E2E test validates everything last | No |
| 3 | Add `Phase::is_gated()` domain method | Centralizes gating logic in domain layer; eliminates string comparisons in adapter | No |
| 4 | Memory write errors warn, don't block | Phase transition is the consistency boundary; memory writes are eventual side-effects | No |
| 5 | Extract `archive_stale_state` to shared `io.rs` | Both init and reset need it; eliminates duplication | No |
| 6 | 100% test coverage on all gaps | Per user request; addresses audit CORR-001 finding | No |
| 7 | Reset writes Idle state (not delete) after archiving | With Idle phase, reset transitions to Idle instead of deleting state.json. "No state.json" means "never initialized"; "phase: idle" means "completed, ready for next" | No |
| 8 | Reset must acquire state lock | Current reset.rs does raw fs::remove_file without lock — TOCTOU race with concurrent init. Archive-before-clear requires lock | No |
| 9 | Phase-gate: from_json failure → pass with WorkflowOutput::warn | Typed deserialization may fail on corrupt state. Phase-gate must not block on deserialization failure — returns WorkflowOutput::warn (exit 0, stderr) | No |
| 10 | `FromStr` accepts "idle" | CLI `ecc-workflow transition idle` must work. `resolve_transition_by_name` uses FromStr | No |

## User Stories

### US-001: Add Idle Phase to Workflow State Machine

**As a** developer using the ECC pipeline, **I want** an explicit Idle phase, **so that** I can distinguish "never initialized" from "completed and ready for next task."

#### Acceptance Criteria

- AC-001.1: Given the Phase enum, then it contains Idle, Plan, Solution, Implement, Done. Idle serializes/deserializes as "idle". `Phase::from_str("idle")` returns `Ok(Phase::Idle)`. `Phase::Idle.to_string()` returns `"idle"`.
- AC-001.2: Given phase is Done, when `resolve_transition(Done, Idle)` is called, then it returns Ok(Idle).
- AC-001.3: Given phase is Idle, when `resolve_transition(Idle, Plan)` is called, then it returns Ok(Plan).
- AC-001.4: Given phase is Idle, when `resolve_transition(Idle, Solution|Implement|Done)` is called, then it returns Err(IllegalTransition).
- AC-001.5: Given phase is Idle, when `resolve_transition(Idle, Idle)` is called, then it returns Ok(Idle) (idempotent re-entry).
- AC-001.6: Given `Phase::is_gated()`, then Plan and Solution return true; Idle, Implement, and Done return false.
- AC-001.7: Given state.json has phase "idle", when phase-gate runs, then it passes (no gating).

#### Dependencies

- Depends on: none

### US-002: Replace Phase-Gate Manual JSON Parsing with Typed Deserialization

**As a** maintainer, **I want** phase-gate to use `WorkflowState::from_json()`, **so that** corrupt state.json is detected instead of silently defaulting to "done."

#### Acceptance Criteria

- AC-002.1: Given a valid state.json, when phase-gate reads the phase, then it uses `read_state()` or `WorkflowState::from_json()`.
- AC-002.2: Given state.json with `{"phase": 123}` (invalid type), when phase-gate runs, then `WorkflowState::from_json` returns Err, and phase-gate returns `WorkflowOutput::warn(...)` (exit 0, stderr) with message indicating corrupt state. It does NOT block.
- AC-002.3: Given state.json missing the "phase" key, when phase-gate runs, then `WorkflowState::from_json` returns Err, and phase-gate returns `WorkflowOutput::warn(...)`.
- AC-002.4: Given valid state.json with phase "plan", when phase-gate runs for Write to src/main.rs, then it blocks (existing behavior preserved). Phase-gate uses `state.phase.is_gated()` instead of string comparison.
- AC-002.5: Given state.json with `{"phase": "banana"}` (unknown variant), when phase-gate runs, then `WorkflowState::from_json` returns Err, and phase-gate returns `WorkflowOutput::warn(...)`.

#### Dependencies

- Depends on: US-001

### US-003: Add E2E Workflow Lifecycle Test

**As a** developer modifying the state machine, **I want** an integration test covering init → transitions → done → reset, **so that** regressions are caught.

#### Acceptance Criteria

- AC-003.1: Given a fresh temp directory, when init → transition(solution) → transition(implement) → transition(done), then each succeeds and final phase is "done".
- AC-003.2: Given phase is Done, when reset runs, then state is archived and state.json has phase "idle".
- AC-003.3: Given workflow was reset, when init runs again, then it succeeds with phase "plan".
- AC-003.4: Given phase is Plan, when transition("implement") is called, then it returns "Illegal transition".
- AC-003.5: Given transition from Plan to Solution with --artifact plan, then artifacts.plan is a non-null ISO 8601 timestamp.

#### Dependencies

- Depends on: US-001, US-005

### US-004: Surface Memory Write Errors in Transition Command

**As a** developer, **I want** to know when memory writes fail during transitions, **so that** I can diagnose missing entries.

#### Acceptance Criteria

- AC-004.1: Given a successful transition where memory writes fail, then output includes warnings and exit code is still 0.
- AC-004.2: Given transition calls write_action, write_work_item, write_daily, write_memory_index, then each error is individually captured (no `let _ =`).
- AC-004.3: Given all memory writes succeed, then output has no warnings.
- AC-004.4: Given state.json was written but memory writes fail, then state.json retains the new phase (no rollback).

#### Dependencies

- Depends on: none

### US-005: Archive State on Reset Instead of Deleting

**As a** developer, **I want** reset to archive state.json before clearing, **so that** I can recover previous state.

#### Acceptance Criteria

- AC-005.1: Given state.json with phase "implement", when reset --force runs, then state is archived to .claude/workflow/archive/state-YYYYMMDDHHMMSS.json, and state.json is replaced with a minimal Idle-phase state (`{"phase":"idle","concern":"","feature":"","started_at":"...","toolchain":{...},"artifacts":{...},"completed":[]}`).
- AC-005.2: Given state.json with phase "done", when reset runs, then it is also archived (unlike init which skips done states).
- AC-005.3: Given archive directory doesn't exist, when reset runs, then it creates the directory.
- AC-005.4: Given no state.json, when reset runs, then it returns pass "No active workflow to reset."
- AC-005.5: Given archive directory can't be created, when reset runs, then it blocks (fail-safe, state.json NOT modified).
- AC-005.6: Given reset is called, then it acquires the state lock before reading/archiving/writing state.json (prevents TOCTOU race with concurrent init).

#### Dependencies

- Depends on: US-001 (optional)

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `crates/ecc-domain/src/workflow/phase.rs` | Domain | Add Idle variant, is_gated() method |
| `crates/ecc-domain/src/workflow/transition.rs` | Domain | Add Idle transition rules |
| `crates/ecc-workflow/src/commands/phase_gate.rs` | Adapter | Use typed deserialization, Phase::is_gated() |
| `crates/ecc-workflow/src/commands/transition.rs` | Adapter | Surface memory write errors |
| `crates/ecc-workflow/src/commands/reset.rs` | Adapter | Archive before clear |
| `crates/ecc-workflow/src/io.rs` | Adapter | Extract shared archive logic |
| `crates/ecc-integration-tests/` | Test | E2E lifecycle test |

## Constraints

- ecc-domain must remain pure (zero I/O imports)
- Existing state.json files (no "idle" phase) must continue to parse
- All state reads/writes must use lock semantics (flock)
- Phase-gate must not block on corrupt state (warn instead)

## Non-Requirements

- Event sourcing or persistent workflow history database
- Automatic Idle-to-Plan promotion on session start
- Memory write retry logic (observability only)
- Backward migration for existing state.json files

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| Phase-gate hook (ecc-workflow) | Modified parsing | Integration test for typed deserialization |
| Reset command (ecc-workflow) | New archive behavior | Integration test for archive-before-clear |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Architecture | Major | ADR | Create ADR for Idle phase lifecycle |
| Domain | Minor | Glossary | Add "Idle phase", "gated phase" |
| Bug fix | Minor | CHANGELOG.md | Add BL-068 entry |

## Open Questions

None — all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Scope boundaries / dependency order? | Keep original: US-001→US-002→US-005→US-004→US-003. Out of scope: event sourcing, history DB, retry, auto-promotion | User (order), Recommended (scope) |
| 2 | Edge case: string comparisons for gating? | Add Phase::is_gated() domain method | Recommended |
| 3 | Test coverage targets? | 100% everywhere | User |
| 4 | Performance constraints? | No constraints (<1ms is fine) | Recommended |
| 5 | Security implications? | No concerns (local file ops only) | Recommended |
| 6 | Breaking changes? | Acceptable — no existing state.json has "idle" | Recommended |
| 7 | Domain glossary additions? | Add "Idle phase" and "gated phase" | Recommended |
| 8 | ADR decisions? | ADR for Idle phase lifecycle | User |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Add Idle Phase | 7 | none |
| US-002 | Typed Phase-Gate Parsing | 5 | US-001 |
| US-003 | E2E Lifecycle Test | 5 | US-001, US-005 |
| US-004 | Memory Write Error Surfacing | 4 | none |
| US-005 | Archive State on Reset | 6 | US-001 (optional) |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | Idle variant with serde + FromStr | US-001 |
| AC-001.2 | Done→Idle transition legal | US-001 |
| AC-001.3 | Idle→Plan transition legal | US-001 |
| AC-001.4 | Idle→non-Plan transitions illegal | US-001 |
| AC-001.5 | Idle re-entry idempotent | US-001 |
| AC-001.6 | Phase::is_gated() | US-001 |
| AC-001.7 | Phase-gate passes for Idle | US-001 |
| AC-002.1 | Typed deserialization in phase-gate | US-002 |
| AC-002.2 | Corrupt state → warn (invalid type) | US-002 |
| AC-002.3 | Missing phase → warn | US-002 |
| AC-002.4 | Valid plan phase → blocks writes | US-002 |
| AC-002.5 | Unknown variant → warn | US-002 |
| AC-003.1 | Full forward lifecycle | US-003 |
| AC-003.2 | Reset after done → Idle | US-003 |
| AC-003.3 | Re-init after reset | US-003 |
| AC-003.4 | Illegal transition rejected | US-003 |
| AC-003.5 | Artifact timestamps recorded | US-003 |
| AC-004.1 | Memory failure → warn, exit 0 | US-004 |
| AC-004.2 | Each error captured individually | US-004 |
| AC-004.3 | Success → no warnings | US-004 |
| AC-004.4 | No rollback on memory failure | US-004 |
| AC-005.1 | Archive + Idle state on reset | US-005 |
| AC-005.2 | Done states also archived | US-005 |
| AC-005.3 | Archive dir auto-created | US-005 |
| AC-005.4 | No state → pass | US-005 |
| AC-005.5 | Archive failure → block (fail-safe) | US-005 |
| AC-005.6 | Reset acquires state lock | US-005 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Completeness | 90 | PASS | All gaps covered, 10 decisions, 27 ACs |
| Correctness | 88 | PASS | Codebase-verified, existing APIs reused |
| Consistency | 92 | PASS | Decisions align with ACs, order respects deps |
| Testability | 95 | PASS | All ACs deterministically testable |
| Feasibility | 93 | PASS | Incremental mods, no new deps |
| Clarity | 91 | PASS | Well-structured, constraints prevent creep |
| Safety | 89 | PASS | Lock + fail-safe + corrupt-state tolerance |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-28-deterministic-workflow-state/spec.md | Full spec + phase summary |
