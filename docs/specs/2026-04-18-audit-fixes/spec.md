# Spec: Fix All Actionable HIGH + MEDIUM Audit Findings (2026-04-18)

Source: docs/audits/full-2026-04-18.md | Scope: HIGH

## Problem Statement

The 2026-04-18 full audit (grade B) identified 2 HIGH and 21 MEDIUM findings. Excluding bus factor (organizational) and non-actionable items, 16 findings require code changes across 8 domains.

## Research Summary

- Prior audit had 5 HIGH → now 2 HIGH (3 resolved since last fix)
- `backlog.rs` at 1,467 lines (2x limit), `phase_gate.rs` at 959
- Clock port bypass grew from 1→11 callsites (BL-133 debt)
- 23 `let _ =` swallowed errors in delta_helpers.rs
- 37 Deserialize types without `deny_unknown_fields`

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Structural fixes | Proper decomposition, not patches | No |
| 2 | Domain-grouped, independently shippable | Each group lands without others | No |
| 3 | Exclude bus factor + LOW | Organizational + deferred | No |

## User Stories

### US-001: Decompose 13 Oversized Files [CONV-001, ARCH-005]
All 13 files under 800 lines. Priority: backlog.rs, phase_gate.rs, transition.rs.

### US-002: Replace 23 Swallowed Errors [ERR, CORR-004]
`let _ =` → `if let Err(e) = { tracing::warn!() }`. Mechanical fix.

### US-003: Clock Port Injection [CORR-002, BL-133]
Add Clock to HookPorts/MetricsPorts. Replace 11 SystemTime::now() calls.

### US-004: Add deny_unknown_fields [SEC-003]
37 Deserialize types. Exclude forward-compat types.

### US-005: Fix xtask Environment Test [TEST-002]
serial_test or env cleanup isolation.

### US-006: Enable missing_docs in ecc-domain [DOC-001]
`#![warn(missing_docs)]` + document all pub items.

### US-007: Add ecc health Command [OBS-003]
SQLite, git, writable dir, state file checks.

## Affected Modules
ecc-app (errors, clock), ecc-domain (docs), ecc-workflow (oversized), ecc-cli (health), ecc-infra (clock adapter).

## Constraints
No public API changes. All tests pass. File decomposition preserves imports via pub use.

## Non-Requirements
Bus factor. LOW findings. ecc-workflow port extraction. Component refactor.

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| Clock port | New consumers | Tests control time |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| CHANGELOG | project | CHANGELOG.md | fix entry |

## Open Questions
None.
