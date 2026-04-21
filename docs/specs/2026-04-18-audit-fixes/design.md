# Solution: Fix All Actionable HIGH+MEDIUM Audit Findings (2026-04-18)

## Spec Reference
Concern: `fix` | Feature: Fix all actionable HIGH+MEDIUM audit findings from full-2026-04-18

## Validated Scope

| US | Finding | Actual Scope | Files |
|----|---------|-------------|-------|
| US-001 | Oversized files | 2 files (backlog.rs 1,297, phase_gate.rs 867) | ~6 after split |
| US-002 | Swallowed errors | 18 `let _ =` in delta_helpers.rs | 1 file |
| US-003 | Clock port (BL-133) | 11 SystemTime::now() in ecc-app | ~15 files |
| US-004 | deny_unknown_fields | 28 types across 16 files (10 excluded) | 16 files |
| US-005 | xtask env test | 4 tests in 2 files | 4 files |
| US-006 | missing_docs ecc-domain | 717 pub items across 145 files | 145 files |
| US-007 | health command | New --health flag on ecc status | 3 files |

## TDD Order

1. US-005 (xtask serial_test) — smallest, infra fix
2. US-002 (swallowed errors) — mechanical, 1 file
3. US-004 (deny_unknown_fields) — mechanical, 16 files
4. US-001 (decompose oversized) — structural, pub use re-exports
5. US-003 (clock port) — largest cascade
6. US-007 (health command) — greenfield
7. US-006 (missing_docs) — highest volume

## SOLID Assessment
PASS — decomposition improves SRP, clock port injection improves DIP.

## Robert's Oath Check
CLEAN — fixes audit debt, proof via 8 PCs.

## Security Notes
CLEAR — deny_unknown_fields is defensive hardening. No new attack surface.

## Rollback Plan
Each US is independently revertable. Reverse order: US-006 → US-007 → US-003 → US-001 → US-004 → US-002 → US-005.

## Bounded Contexts Affected

| Context | Role | Files |
|---------|------|-------|
| Hook Runtime | Service + Ports | dispatch.rs, mod.rs, handlers/* |
| Metrics | Service | metrics_mgmt.rs, metrics_session.rs |
| Workflow | State management | phase_gate.rs |
| Backlog | Entity + operations | backlog.rs → backlog/ |
| Configuration | Value Objects | deny_rules.rs, manifest.rs, team.rs, hook_types.rs |
