# Spec: TDD Fix-Loop Budget Cap

## Problem Statement

ECC's /implement TDD loop has no cap on test-fix iterations. A failing test can trigger unbounded retry cycles, burning tokens with diminishing returns. Stripe's Minions research found that "at most two rounds of CI" balances speed and efficiency, noting "diminishing marginal returns for an LLM to run many rounds." The code review phase (Phase 5) already has a 2-round cap, but TDD (Phase 3) and E2E (Phase 4) do not.

## Research Summary

- Stripe's Minions blog: "at most two rounds of CI" as the optimal budget for agent CI loops
- Current state: Phase 5 code review already capped at "Max 2 fix rounds"
- Phase 3 TDD: no fix budget — tdd-executor has no retry counter
- Phase 4 E2E: "fix and re-run" with no cap mentioned
- Pattern generalizable: same budget logic applies to all fix loops for consistency

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Hardcoded 2-round budget | Stripe's published best practice; simple, no config needed | No |
| 2 | Per-PC counter, not session-wide | Each PC gets its own budget; one PC failing doesn't starve others | No |
| 3 | AskUserQuestion on budget exceeded | User stays in control; can grant more rounds, skip, abort, or provide guidance | No |
| 4 | Apply to all fix loops (Phase 3, 4, 5) | Consistent behavior; Phase 5 already has this, extend to 3 and 4 | No |

## User Stories

### US-001: TDD Fix-Loop Budget

**As a** developer running /implement, **I want** the TDD fix loop to stop after 2 failed fix attempts and ask me for help, **so that** tokens aren't wasted on diminishing-return retries.

#### Acceptance Criteria

- AC-001.1: Given a PC's GREEN phase fails, when the tdd-executor fixes and re-runs, then it tracks the fix attempt count starting at 1
- AC-001.2: Given fix attempt count reaches 2 and the test still fails, when the budget is exceeded, then the tdd-executor returns a structured failure with diagnostic report (test name, error output, attempted fixes)
- AC-001.3: Given a tdd-executor returns failure due to budget exceeded, when the parent orchestrator receives it, then it presents AskUserQuestion with options: "Keep trying (+2 rounds)", "Skip this PC", "Abort implementation"
- AC-001.4: Given the user selects "Keep trying", when the budget is extended, then 2 more fix rounds are granted and the tdd-executor is re-dispatched
- AC-001.5: Given the user selects "Skip this PC", when the PC is skipped, then it is marked as failed in tasks.md and the next PC proceeds
- AC-001.6: Given the user selects "Abort", when implementation stops, then the current state is preserved and the user is informed
- AC-001.7: Given the user provides free-text guidance via "Other", when re-dispatching, then the guidance is included in the tdd-executor's context brief as a "## User Guidance" section
- AC-001.8: Given a PC succeeds within the budget, when the fix counter is at 1 or 2, then the PC is marked as success with a note "fixed in N rounds"
- AC-001.9: Given E2E Phase 4 has a failing test, when fix attempts reach 2, then the same budget+ask behavior applies
- AC-001.10: Given the fix-round budget is in effect, when a diagnostic report is generated, then it includes these exact markdown subsections: `### Test Name`, `### Error Output` (last 50 lines), `### Files Modified` (bulleted list), `### Fix Attempts` (numbered list, one entry per round with what was tried)
- AC-001.11: Given the fix-round budget, when the counter is maintained, then it is owned by the parent orchestrator (implement.md), NOT by the tdd-executor. Each time a tdd-executor returns status: failure for a given PC, the parent increments that PC's counter. The tdd-executor has no knowledge of the budget.
- AC-001.12: Given the tdd-executor's structured result, when returning failure, then no new status values are added. Budget enforcement is parent-side only. The parent treats any failure return as a consumed fix round.
- AC-001.13: Given a tdd-executor crash or timeout (no structured result returned), when the parent receives no response, then it does NOT consume a fix round. The parent reports the crash to the user immediately without budget logic.
- AC-001.14: Given the user selects "Keep trying" repeatedly, when extensions are granted, then the user may select it at most 3 times per PC, for a maximum of 8 total fix rounds (2 initial + 3x2 extensions). After 8 rounds, the only options are "Skip this PC" or "Abort".
- AC-001.15: Given Phase 5 already has "Max 2 fix rounds" language, when this spec is applied, then Phase 5's existing language is unchanged. This spec adds budget enforcement with AskUserQuestion to Phase 3 and Phase 4 only.
- AC-001.16: Given RED phase compilation errors in the tdd-executor (line 69), when the agent fixes compilation and re-runs, then these compilation fixes do NOT consume fix rounds. Only GREEN phase test failures consume rounds.

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change Type |
|--------|-------|-------------|
| `agents/tdd-executor.md` | Agent | No changes needed -- budget is parent-side. Tdd-executor continues returning `failure` as before. |
| `commands/implement.md` | Command | Add fix-round counter + budget-exceeded handling with AskUserQuestion in Phase 3 and Phase 4. Add diagnostic report format. |
| `skills/wave-dispatch/SKILL.md` | Skill | Update failure handling to include budget-exceeded → AskUserQuestion flow before stopping the wave |

## Constraints

- Markdown-only changes -- no Rust code modified
- Must not break existing tdd-executor behavior for PCs that pass on first GREEN
- AskUserQuestion must be used (not silent abort) -- user stays in control
- Per-PC budget counter, not session-wide

## Non-Requirements

- Configurable budget via env var (hardcoded to 2 for v1)
- Automatic model escalation on failure
- Budget persistence across sessions
- Token cost tracking per fix round (separate concern, BL-096)

## E2E Boundaries Affected

None -- markdown behavioral changes only.

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Behavioral change | CLAUDE.md Gotchas | Add fix-round budget definition | AC-001.1 |
| Changelog | CHANGELOG.md | Add entry | all |

## Rollback Plan

1. Revert the 3 markdown file edits -- the fix-loop budget language is purely additive text
2. No data migration, no config changes, no code changes needed

## Open Questions

None -- all questions resolved during grill-me interview.
