# Implementation Complete: Fix phase-gate hook allowlist for spec/plan/design artifact paths (BL-046)

## Spec Reference
Concern: fix, Feature: Fix phase-gate hook allowlist for spec/plan/design artifact paths (BL-046)

## Changes Made
| # | File | Action | Solution Ref | Tests | Status |
|---|------|--------|--------------|-------|--------|
| 1 | .claude/hooks/phase-gate.sh | modify | PC-001–PC-018 | test-phase-gate.sh | done |
| 2 | tests/hooks/test-phase-gate.sh | create | PC-002–PC-018 | self | done |

## TDD Log
| PC ID | RED | GREEN | REFACTOR | Notes |
|-------|-----|-------|----------|-------|
| PC-001 | ✅ function absent | ✅ is_allowed_path exists | ✅ loop over array | — |
| PC-002–PC-017 | ✅ 9 tests fail (new paths blocked) | ✅ all 19 pass | ✅ true single source of truth | Review fix: array-driven loop |
| PC-018 | ✅ script exists | ✅ all 20 pass standalone | ⏭ no refactor needed | Added user-stories test |
| PC-019 | — | — | ⏭ shellcheck not installed | Non-blocking |
| PC-020 | — | ✅ cargo build passes | ⏭ no refactor needed | — |

## Pass Condition Results
| PC ID | Command | Expected | Actual | Status |
|-------|---------|----------|--------|--------|
| PC-001 | `grep -c 'is_allowed_path()' .claude/hooks/phase-gate.sh` | 1 | 1 | ✅ |
| PC-002 | `bash tests/hooks/test-phase-gate.sh \| grep 'PASS.*workflow_relative'` | match | match | ✅ |
| PC-003 | `bash tests/hooks/test-phase-gate.sh \| grep 'PASS.*specs_plan'` | match | match | ✅ |
| PC-004 | `bash tests/hooks/test-phase-gate.sh \| grep 'PASS.*plans_solution'` | match | match | ✅ |
| PC-005 | `bash tests/hooks/test-phase-gate.sh \| grep 'PASS.*docs_plans'` | match | match | ✅ |
| PC-006 | `bash tests/hooks/test-phase-gate.sh \| grep 'PASS.*docs_designs'` | match | match | ✅ |
| PC-007 | `bash tests/hooks/test-phase-gate.sh \| grep 'PASS.*docs_adr'` | match | match | ✅ |
| PC-008 | `bash tests/hooks/test-phase-gate.sh \| grep 'PASS.*blocked_src'` | match | match | ✅ |
| PC-009 | `bash tests/hooks/test-phase-gate.sh \| grep 'PASS.*empty_path'` | match | match | ✅ |
| PC-010 | `bash tests/hooks/test-phase-gate.sh \| grep 'PASS.*error_message'` | match | match | ✅ |
| PC-011 | `bash tests/hooks/test-phase-gate.sh \| grep 'PASS.*absolute_path'` | match | match | ✅ |
| PC-012 | `bash tests/hooks/test-phase-gate.sh \| grep 'PASS.*implement_ungated'` | match | match | ✅ |
| PC-013 | `bash tests/hooks/test-phase-gate.sh \| grep 'PASS.*done_ungated'` | match | match | ✅ |
| PC-014 | `bash tests/hooks/test-phase-gate.sh \| grep 'PASS.*bypass'` | match | match | ✅ |
| PC-015 | `bash tests/hooks/test-phase-gate.sh \| grep 'PASS.*no_state'` | match | match | ✅ |
| PC-016 | `bash tests/hooks/test-phase-gate.sh \| grep 'PASS.*malformed_json'` | match | match | ✅ |
| PC-017 | `bash tests/hooks/test-phase-gate.sh \| grep 'PASS.*integration_cycle'` | match | match | ✅ |
| PC-018 | `bash tests/hooks/test-phase-gate.sh` | exit 0, all PASS | 20/20 PASS, exit 0 | ✅ |
| PC-019 | `shellcheck .claude/hooks/phase-gate.sh` | no errors | shellcheck not installed | ⏭ |
| PC-020 | `cargo build` | Finished | Finished | ✅ |

All pass conditions: 19/20 ✅ (PC-019 skipped — shellcheck not installed)

## E2E Tests
No E2E tests required by solution

## Docs Updated
| # | Doc File | Level | What Changed |
|---|----------|-------|--------------|
| 1 | CHANGELOG.md | project | Added BL-046 fix entry |

## ADRs Created
None required

## Subagent Execution
Inline execution — subagent dispatch not used (shell script fix, all PCs share same 2 files)

## Code Review
PASS — 1 HIGH finding (array/function duplication) fixed by refactoring is_allowed_path to iterate over ALLOWED_PATHS. 1 MEDIUM finding (missing user-stories test) addressed. All 20 tests pass after fixes.

## Suggested Commit
fix(hooks): expand phase-gate allowlist for spec/plan/design artifact paths (BL-046)
