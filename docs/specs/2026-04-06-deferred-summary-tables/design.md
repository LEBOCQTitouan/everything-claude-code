# Solution: Deferred Pipeline Summary Tables

## Spec Reference
Concern: dev, Feature: deferred-summary-tables

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `agents/tdd-executor.md` | modify | Add test_names field to structured result output | AC-003.1 |
| 2 | `commands/implement.md` | modify | Add "Post-TDD Coverage Measurement" subsection between Phase 3 and Phase 4, test_names in TDD Log, Coverage Delta in implement-done.md schema | AC-001.1..4, AC-003.2..3 |
| 3 | `commands/design.md` | modify | Add bounded context enumeration in Phase 9 output schema | AC-002.1..3 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | lint | implement.md contains cargo llvm-cov coverage step | AC-001.1 | `grep -c 'cargo llvm-cov' commands/implement.md` | >= 1 |
| PC-002 | lint | implement.md contains Coverage Delta table format | AC-001.2 | `grep -c 'Coverage Delta' commands/implement.md` | >= 1 |
| PC-003 | lint | implement.md contains graceful skip for missing tool | AC-001.3 | `grep -c 'Coverage data unavailable' commands/implement.md` | >= 1 |
| PC-004 | lint | implement-done schema has Coverage Delta section | AC-001.4 | `grep -c '## Coverage Delta' commands/implement.md` | >= 1 |
| PC-005 | lint | design.md contains Bounded Contexts Affected table | AC-002.1 | `grep -c 'Bounded Contexts Affected' commands/design.md` | >= 1 |
| PC-006 | lint | design.md references bounded-contexts.md | AC-002.2 | `grep -c 'bounded-contexts.md' commands/design.md` | >= 1 |
| PC-007 | lint | design.md has "No bounded contexts affected" fallback | AC-002.3 | `grep -c 'No bounded contexts affected' commands/design.md` | >= 1 |
| PC-008 | lint | tdd-executor.md has test_names in output | AC-003.1 | `grep -c 'test_names' agents/tdd-executor.md` | >= 1 |
| PC-009 | lint | implement.md TDD Log has Test Names column | AC-003.2 | `grep -c 'Test Names' commands/implement.md` | >= 1 |
| PC-010 | lint | implement.md TDD Log shows "--" for missing test_names | AC-003.3 | `grep -c '"--"' commands/implement.md` | >= 1 |
| PC-011 | lint | implement.md documents test_names backward compat | AC-003.4 | `grep -c 'test_names' commands/implement.md` | >= 2 |
| PC-012 | lint | implement.md has before-snapshot fallback chain | AC-001.1 | `grep -c 'No before-snapshot' commands/implement.md` | >= 1 |
| PC-013 | build | ecc validate commands passes | AC-003.5 | `ecc validate commands 2>/dev/null; echo $?` | 0 |
| PC-014 | build | ecc validate agents passes | AC-003.5 | `ecc validate agents 2>/dev/null; echo $?` | 0 |

### Coverage Check

| AC | Covering PCs |
|----|-------------|
| AC-001.1 | PC-001, PC-012 |
| AC-001.2 | PC-002 |
| AC-001.3 | PC-003 |
| AC-001.4 | PC-004 |
| AC-002.1 | PC-005 |
| AC-002.2 | PC-006 |
| AC-002.3 | PC-007 |
| AC-003.1 | PC-008 |
| AC-003.2 | PC-009 |
| AC-003.3 | PC-010 |
| AC-003.4 | PC-011 |
| AC-003.5 | PC-013, PC-014 |

All 14 ACs covered.

### E2E Test Plan

None -- markdown behavioral changes only.

### E2E Activation Rules

None.

## Test Strategy

TDD order:
1. PC-008: tdd-executor test_names field (foundation -- schema change)
2. PC-001..004: Coverage delta in implement.md
3. PC-009..011: Test names in TDD Log + backward compat
4. PC-012: Before-snapshot fallback chain
5. PC-005..007: Bounded context enumeration in design.md
6. PC-013..014: Validate commands and agents

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CLAUDE.md | project | modify | Add test_names migration note to Gotchas; add glossary terms | AC-003.4 |
| 2 | docs/domain/glossary.md | project | modify | Add coverage delta, bounded context enumeration, per-test-name inventory | all |
| 3 | CHANGELOG.md | project | modify | Add BL-050 entry | all |

## SOLID Assessment

N/A -- markdown behavioral instructions only.

## Robert's Oath Check

CLEAN -- improves pipeline visibility without adding complexity.

## Security Notes

CLEAR -- no attack surface changes.

## Rollback Plan

1. Revert `commands/design.md`
2. Revert `commands/implement.md`
3. Revert `agents/tdd-executor.md`
