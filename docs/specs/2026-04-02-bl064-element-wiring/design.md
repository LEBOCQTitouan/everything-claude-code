# Solution: BL-064 Element Wiring

## Spec Reference
Concern: dev, Feature: bl064-element-wiring

## File Changes (dependency order)
| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | cartography.rs | modify | Add elements/ scaffold, post-loop dispatch, INDEX regen | US-001, US-002, US-003 |
| 2 | validate_cartography.rs | modify | Add element scan, INDEX checks, staleness, coverage | US-004 |
| 3 | cartographer.md | modify | Add element dispatch step reference | US-002 |
| 4 | hooks/hooks.json | verify | Ensure cartography entries present | US-005 |
| 5 | commands/spec-dev.md | verify | Ensure actor registry integration | US-005 |
| 6 | BACKLOG.md + BL-064 file | modify | Status → implemented | US-006 |
| 7 | CLAUDE.md | modify | Update test count | US-006 |

## Pass Conditions
18 PCs covering all 19 ACs. See plan file for full table.

## Test Strategy
Phase 1 (handler) → Phase 2 (validate) → Phase 3 (config + status)

## SOLID Assessment
PASS — additive internal changes following existing patterns.

## Security Notes
CLEAR — no new attack surfaces.

## Rollback Plan
Revert in reverse: CLAUDE.md → backlog → spec-dev → hooks.json → cartographer.md → validate_cartography.rs → cartography.rs
