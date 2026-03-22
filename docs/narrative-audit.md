# Narrative Audit Summary (BL-051)

Date: 2026-03-22

## Overview

Added explanatory narrative instructions to all 22 ECC command files following the "narrate before acting" convention defined in `skills/narrative-conventions/SKILL.md`. All changes are pure instruction additions — no structural logic was modified.

## Commands Touched

### Pipeline Commands (5)

| Command | Narrative Points Added |
|---------|----------------------|
| spec-dev.md | Agent delegation (requirements-analyst, architect), web research status, adversarial verdict translation with remediation |
| spec-fix.md | Agent delegation (code-reviewer, architect), web research status, adversarial verdict translation with remediation |
| spec-refactor.md | Parallel agent delegation (evolution-analyst, arch-reviewer, component-auditor), web research status, adversarial verdict translation with remediation |
| design.md | State validation gate remediation, SOLID/Robert/Security agent narration, AC coverage result communication, adversarial verdict translation |
| implement.md | State validation gate remediation, per-PC dispatch narration, regression re-verification reporting, code review findings communication |

### Audit Commands (10)

| Command | Narrative Points Added |
|---------|----------------------|
| audit-full.md | Domain agent dispatch narration, per-domain completion status reporting |
| audit-archi.md | Agent delegation narration, how to reference report in /spec |
| audit-code.md | Agent delegation narration |
| audit-security.md | Agent delegation narration |
| audit-test.md | Agent delegation narration |
| audit-convention.md | Agent delegation narration |
| audit-errors.md | Agent delegation narration |
| audit-observability.md | Agent delegation narration |
| audit-doc.md | Agent delegation narration |
| audit-evolution.md | Agent delegation narration |

### Utility Commands (7)

| Command | Narrative Points Added |
|---------|----------------------|
| verify.md | Reviewer agent narration, explanation of why both code-reviewer and arch-reviewer are needed |
| build-fix.md | Error classification explanation (Structural/Contractual/Incidental) |
| review.md | Programmer's Oath evaluation explanation |
| catchup.md | Consequences of stale workflow reset |
| backlog.md | Skill reference (minimal — already well-narrated) |
| spec.md | Skill reference (minimal — router already explains classification) |
| ecc-test-mode.md | Skill reference (minimal — already explanatory) |

## Shared Skill

`skills/narrative-conventions/SKILL.md` — defines 4 narration patterns:
1. **Agent Delegation**: which agent, what it analyzes, what to expect
2. **Gate Failure**: what blocked, why it matters, remediation steps
3. **Progress**: what phase begins, what it accomplishes, what comes next
4. **Result**: summarize findings conversationally before structured output

## ADR

`docs/adr/0011-command-narrative-convention.md` — documents the convention for future command authors.

## Verification

- 113 grep-based assertions across 20 test functions in `tests/test-narrative-audit.sh`
- All 22 command files reference `narrative-conventions` skill
- All files remain under 800 lines
- Manual spot-check: spec-dev, implement, verify confirmed clear and consistent
