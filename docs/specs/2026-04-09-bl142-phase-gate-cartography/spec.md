# Spec: Add docs/cartography/ to phase-gate allowlist

## Problem Statement

The phase-gate hook (`ecc-workflow phase-gate`) blocks Write/Edit operations to `docs/cartography/` during active workflow phases (plan, spec, design, implement). This was an oversight when the cartography system was introduced in BL-064. Three other `docs/` subdirectories (`docs/domain/`, `docs/guides/`, `docs/diagrams/`) are also missing from the allowlist. All four are written to by doc-orchestrator, diagram-updater, and cartographer agents during active phases.

## Research Summary

Web research skipped: trivial allowlist addition, no external patterns needed.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Add all four missing dirs | All four are written to by agents during workflow phases; prevents future blockers | No |
| 2 | Add tests for all four dirs | One test per dir verifies each was actually added to the allowlist; follows `phase_gate_allows_*_dir` pattern | No |
| 3 | Test in plan phase only | Existing tests only check plan phase; the phase-gate allowlist is phase-independent (same prefixes for all phases) | No |

## User Stories

### US-001: Phase-gate allows doc directory writes

**As a** doc-orchestrator agent, **I want** the phase-gate to allow writes to `docs/cartography/`, `docs/domain/`, `docs/guides/`, and `docs/diagrams/`, **so that** documentation processing succeeds during active workflow phases.

#### Acceptance Criteria

- AC-001.1: Given a workflow in plan phase, when a Write targets `docs/cartography/elements/foo.md`, then the phase-gate returns Status::Pass
- AC-001.2: Given a workflow in plan phase, when a Write targets `docs/domain/bounded-contexts.md`, then the phase-gate returns Status::Pass
- AC-001.3: Given a workflow in plan phase, when a Write targets `docs/guides/getting-started.md`, then the phase-gate returns Status::Pass
- AC-001.4: Given a workflow in plan phase, when a Write targets `docs/diagrams/flow.md`, then the phase-gate returns Status::Pass
- AC-001.5: `cargo test -p ecc-workflow -- phase_gate` passes with all new tests
- AC-001.6: Path traversal protection remains intact — `docs/cartography/../../../src/evil.rs` is blocked (existing `contains_encoded_traversal()` guard)

#### Dependencies

- None (BL-064 cartography system already merged)

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `crates/ecc-workflow/src/commands/phase_gate.rs` | Infra/CLI | Add 4 strings to allowlist + 4 tests |

## Constraints

- Must not remove any existing allowed prefixes
- Must follow existing test naming pattern (`phase_gate_allows_*_dir`)
- Path traversal guards must remain unaffected

## Non-Requirements

- Not refactoring the allowlist to a config file
- Not auditing all `docs/` subdirs beyond the four identified

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | — | No E2E boundaries crossed |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| CHANGELOG | Minor | CHANGELOG.md | Add fix entry |

## Open Questions

None — all resolved during grill-me interview and adversarial review.
