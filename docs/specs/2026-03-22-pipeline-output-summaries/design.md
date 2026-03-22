# Solution: Pipeline output summaries + DRY cleanup (BL-048)

## Spec Reference
Concern: refactor, Feature: Comprehensive output summaries for spec → design → implement pipeline (BL-048)

## File Changes (dependency order)
| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | tests/test-pipeline-summaries.sh | create | Bash test suite | US-001–005 |
| 2 | skills/spec-pipeline-shared/SKILL.md | create | Extract shared sections | US-004 |
| 3 | commands/spec-dev.md | modify | DRY + summaries + accumulator | US-001, US-004 |
| 4 | commands/spec-fix.md | modify | DRY + summaries + accumulator | US-001, US-004 |
| 5 | commands/spec-refactor.md | modify | DRY + summaries + accumulator | US-001, US-004 |
| 6 | commands/design.md | modify | Summary tables + persistence | US-002 |
| 7 | commands/implement.md | modify | Summaries + commit accumulator | US-003 |
| 8 | docs/adr/0009-phase-summary-convention.md | create | ADR | AC-005.1 |
| 9 | CHANGELOG.md | modify | BL-048 entry | AC-005.2 |

## Pass Conditions
35 PCs covering 27 ACs. TDD order: shared skill → spec commands → design → implement → docs → build.

## Test Strategy
TDD order: shared skill (PC-001–005) → spec DRY + summaries (PC-006–016) → design summaries (PC-017–021) → implement summaries (PC-022–027) → docs (PC-028–030) → cross-cutting + build (PC-031–035).

## Doc Update Plan
ADR 0009 (Phase Summary convention), CHANGELOG (BL-048 entry). No additional ADRs.

## SOLID Assessment
PASS

## Robert's Oath Check
CLEAN

## Security Notes
CLEAR

## Rollback Plan
Revert CHANGELOG, delete ADR 0009, revert implement.md, revert design.md, revert spec-refactor.md, revert spec-fix.md, revert spec-dev.md, delete spec-pipeline-shared skill, delete test file.
