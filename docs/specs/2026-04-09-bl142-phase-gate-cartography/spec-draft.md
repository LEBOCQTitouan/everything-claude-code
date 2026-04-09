# Spec: Add docs/cartography/ to phase-gate allowlist

## Problem Statement

The phase-gate hook (`ecc-workflow phase-gate`) blocks Write/Edit operations to `docs/cartography/` during active workflow phases (plan, spec, design, implement). This was an oversight when the cartography system was introduced in BL-064. Three other `docs/` subdirectories (`docs/domain/`, `docs/guides/`, `docs/diagrams/`) are also missing from the allowlist.

## Research Summary

Web research skipped: trivial allowlist addition, no external patterns needed.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Add all four missing dirs, not just cartography | Prevents future instances of the same bug | No |
| 2 | Add one test for docs/cartography/ | Follows existing test pattern (phase_gate_allows_prds_dir) | No |

## User Stories

### US-001: Phase-gate allows cartography writes

**As a** doc-orchestrator agent, **I want** the phase-gate to allow writes to `docs/cartography/`, **so that** cartography processing succeeds during active workflow phases.

#### Acceptance Criteria

- AC-001.1: Given a workflow in plan/spec/design phase, when a Write targets `docs/cartography/elements/foo.md`, then the phase-gate allows it (exit 0)
- AC-001.2: Given a workflow in plan/spec/design phase, when a Write targets `docs/domain/bounded-contexts.md`, then the phase-gate allows it (exit 0)
- AC-001.3: Given a workflow in plan/spec/design phase, when a Write targets `docs/guides/getting-started.md`, then the phase-gate allows it (exit 0)
- AC-001.4: Given a workflow in plan/spec/design phase, when a Write targets `docs/diagrams/flow.md`, then the phase-gate allows it (exit 0)
- AC-001.5: `cargo test -p ecc-workflow -- phase_gate` passes with the new test

#### Dependencies

- None

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `crates/ecc-workflow/src/commands/phase_gate.rs` | Infra/CLI | Add 4 strings to allowlist + 1 test |

## Constraints

- Must not remove any existing allowed prefixes
- Must follow existing test naming pattern (`phase_gate_allows_*_dir`)

## Non-Requirements

- Not adding tests for all four new dirs (one is sufficient to verify the pattern)
- Not refactoring the allowlist to a config file

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | — | No E2E boundaries crossed |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| CHANGELOG | Minor | CHANGELOG.md | Add fix entry |

## Open Questions

None — all resolved during grill-me interview.
