# Solution: TDD Fix-Loop Budget Cap

## Spec Reference
Concern: dev, Feature: tdd-fix-loop-budget

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `commands/implement.md` | modify | Add fix-round budget counter to Phase 3, budget enforcement to Phase 4, diagnostic report format, AskUserQuestion on budget exceeded | AC-001.1..16 |
| 2 | `skills/wave-dispatch/SKILL.md` | modify | Replace ONLY the subagent-returns-failure STOP (line 29 single-PC, line 66 wave) with budget-aware AskUserQuestion flow. Merge conflicts (line 45), worktree failures (line 46), and regression failures (line 58) remain immediate STOPs. | AC-001.3, AC-001.4..7, AC-001.13, AC-001.14 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | lint | implement.md contains fix_round_count budget language | AC-001.1, AC-001.11 | `grep -c 'fix_round_count' commands/implement.md` | >= 1 |
| PC-002 | lint | implement.md contains AskUserQuestion options for budget exceeded | AC-001.3, AC-001.4..6 | `grep -c 'Keep trying' commands/implement.md` | >= 1 |
| PC-003 | lint | implement.md contains diagnostic report format subsections | AC-001.10 | `grep -c '### Test Name' commands/implement.md` | >= 1 |
| PC-004 | lint | implement.md Phase 4 has budget enforcement | AC-001.9 | `grep -c 'fix_round_count' commands/implement.md` | >= 2 |
| PC-005 | lint | wave-dispatch contains budget-exceeded AskUserQuestion flow | AC-001.3, AC-001.13 | `grep -c 'AskUserQuestion' skills/wave-dispatch/SKILL.md` | >= 1 |
| PC-006 | lint | tdd-executor.md is unchanged (no budget references) | AC-001.11, AC-001.12 | `grep -c 'budget\|fix_round' agents/tdd-executor.md` | 0 |
| PC-007 | build | ecc validate commands passes | build quality | `ecc validate commands 2>/dev/null; echo $?` | 0 |
| PC-008 | lint | implement.md contains User Guidance context brief section | AC-001.7 | `grep -c '## User Guidance' commands/implement.md` | >= 1 |
| PC-009 | lint | implement.md contains hard cap at 8 rounds language | AC-001.14 | `grep -cE 'at most 3\|maximum.*(8\|eight)' commands/implement.md` | >= 1 |
| PC-010 | lint | implement.md scopes fix rounds to GREEN phase failures only | AC-001.16 | `grep -c 'GREEN.*fail' commands/implement.md` | >= 1 |
| PC-011 | lint | implement.md contains "fixed in" success annotation | AC-001.8 | `grep -c 'fixed in' commands/implement.md` | >= 1 |

### Coverage Check

| AC | Covering PCs |
|----|-------------|
| AC-001.1 | PC-001 |
| AC-001.2 | PC-001, PC-003 |
| AC-001.3 | PC-002, PC-005 |
| AC-001.4 | PC-002 |
| AC-001.5 | PC-002 |
| AC-001.6 | PC-002 |
| AC-001.7 | PC-002, **PC-008** |
| AC-001.8 | PC-001, **PC-011** |
| AC-001.9 | PC-004 |
| AC-001.10 | PC-003 |
| AC-001.11 | PC-001, PC-006 |
| AC-001.12 | PC-006 |
| AC-001.13 | PC-005 |
| AC-001.14 | PC-002, **PC-009** |
| AC-001.15 | implicit (Phase 5 not touched) |
| AC-001.16 | PC-001, **PC-010** |

All 16 ACs covered.

### E2E Test Plan

None -- markdown behavioral changes only.

### E2E Activation Rules

None.

## Test Strategy

TDD order:
1. PC-001: Add budget counter language to implement.md Phase 3
2. PC-002: Add AskUserQuestion options
3. PC-003: Add diagnostic report format
4. PC-004: Add budget to Phase 4
5. PC-005: Update wave-dispatch failure handling
6. PC-008: Add User Guidance context brief section
7. PC-009: Add hard cap at 8 rounds language
8. PC-010: Add GREEN-only scoping language
9. PC-011: Add "fixed in" success annotation
10. PC-006: Verify tdd-executor unchanged
11. PC-007: Validate commands

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CLAUDE.md | project | modify | Add fix-round budget to Gotchas section | AC-001.1 |
| 2 | CHANGELOG.md | project | modify | Add BL-080 entry | all |

## SOLID Assessment

N/A -- markdown behavioral instructions, no code architecture.

## Robert's Oath Check

CLEAN -- This change improves developer experience by preventing token waste. Small, focused change. No mess introduced.

## Security Notes

CLEAR -- No attack surface changes. No code, no I/O, no data handling.

## Rollback Plan

1. Revert `skills/wave-dispatch/SKILL.md`
2. Revert `commands/implement.md`
3. No data, config, or code rollback needed
