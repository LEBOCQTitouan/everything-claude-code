# Solution: BL-149 Agentic Self-Evaluation in /implement

## Spec Reference
Concern: dev, Feature: BL-149 agentic self-evaluation in implement TDD loop

## File Changes (dependency order)
| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | skills/pc-evaluation/SKILL.md | create | Evaluation rubric: 3 dimensions, PASS/WARN/FAIL criteria, trigger rules | US-001 |
| 2 | agents/pc-evaluator.md | create | Read-only agent executing the rubric | US-002 |
| 3 | commands/implement.md | modify | Phase 3: add post-PC evaluation dispatch with conditional triggers, escalation | US-003 |
| 4 | commands/implement.md | modify | Phase 7: add Self-Evaluation Log to implement-done schema | US-004 |
| 5 | teams/implement-team.md | modify | Add pc-evaluator entry | US-005 AC-005.3 |
| 6 | skills/progress-tracking/SKILL.md | modify | Add eval@timestamp status | US-005 AC-005.4 |
| 7 | skills/tasks-generation/SKILL.md | modify | Add eval status marker | US-005 AC-005.5 |
| 8 | docs/adr/0063-pc-self-evaluation.md | create | ADR for evaluation architecture | Decision #1 |
| 9 | CHANGELOG.md | modify | BL-149 entry | US-005 AC-005.1 |
| 10 | CLAUDE.md | modify | Glossary: self-evaluation | US-005 AC-005.2 |

## Pass Conditions
| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | grep | Skill defines 3 dimensions | AC-001.1 | `grep -cE 'AC satisfaction\|regression\|achievability' skills/pc-evaluation/SKILL.md` | >=3 |
| PC-002 | grep | Skill has PASS/WARN/FAIL criteria | AC-001.2 | `grep -cE 'PASS\|WARN\|FAIL' skills/pc-evaluation/SKILL.md` | >=9 |
| PC-003 | lint | ecc validate skills | AC-001.3 | `ecc validate skills` | exit 0 |
| PC-004 | grep | Skill documents trigger rules | AC-001.4 | `grep -cE 'fix_round_count\|integration\|wave boundary' skills/pc-evaluation/SKILL.md` | >=3 |
| PC-005 | grep | Agent tools read-only | AC-002.1 | `grep -c 'Read, Grep, Glob' agents/pc-evaluator.md` | >=1 |
| PC-006 | grep | Agent structured output | AC-002.3 | `grep -cE 'ac_satisfied\|regressions\|achievability' agents/pc-evaluator.md` | >=3 |
| PC-007 | lint | ecc validate agents | AC-002.4 | `ecc validate agents` | exit 0 |
| PC-008 | grep | implement.md dispatches evaluator | AC-003.1 | `grep -c 'pc-evaluator' commands/implement.md` | >=1 |
| PC-009 | grep | Conditional triggers | AC-003.1-3 | `grep -cE 'fix_round_count.*0\|integration.*e2e\|wave boundary' commands/implement.md` | >=2 |
| PC-010 | grep | WARN logging | AC-003.5 | `grep -cE 'WARN.*tasks.md\|logged.*WARN' commands/implement.md` | >=1 |
| PC-011 | grep | FAIL escalation | AC-003.6 | `grep -cE 'FAIL.*AskUserQuestion\|Re-dispatch.*Accept.*Abort' commands/implement.md` | >=1 |
| PC-012 | grep | 3-WARN auto-escalate | AC-003.7 | `grep -cE '3.*WARN\|consecutive.*WARN' commands/implement.md` | >=1 |
| PC-013 | grep | Pause and revise | AC-003.8 | `grep -cE 'Pause.*revise\|revise.*spec' commands/implement.md` | >=1 |
| PC-014 | grep | Self-Evaluation Log section | AC-004.1 | `grep -c 'Self-Evaluation Log' commands/implement.md` | >=1 |
| PC-015 | grep | Verdict columns | AC-004.2 | `grep -cE 'AC Verdict.*Regression.*Achievability' commands/implement.md` | >=1 |
| PC-016 | grep | Team manifest entry | AC-005.3 | `grep -c 'pc-evaluator' teams/implement-team.md` | >=1 |
| PC-017 | grep | progress-tracking eval status | AC-005.4 | `grep -c 'eval@' skills/progress-tracking/SKILL.md` | >=1 |
| PC-018 | grep | tasks-generation eval marker | AC-005.5 | `grep -cE 'eval' skills/tasks-generation/SKILL.md` | >=1 |
| PC-019 | grep | CHANGELOG | AC-005.1 | `grep -c 'BL-149' CHANGELOG.md` | >=1 |
| PC-020 | grep | CLAUDE.md glossary | AC-005.2 | `grep -c 'self-evaluation' CLAUDE.md` | >=1 |

### Coverage Check
23 ACs covered by 20 PCs. AC-002.2 (agent receives PC result, AC text, files) is verified by the agent's input spec in the file (PC-006 checks output structure, input is documented in the agent body). AC-003.4 (clean unit skip) is the negation of AC-003.1-3 triggers — verified by the conditional trigger PCs. AC-004.3 (skipped PCs show SKIPPED) is a schema detail verified by reading implement.md.

### E2E Test Plan
No E2E tests — pure command/skill/agent.

### E2E Activation Rules
None.

## Test Strategy
TDD order:
1. PC-001-004: Skill file (rubric foundation)
2. PC-005-007: Agent file (depends on skill)
3. PC-008-015: implement.md modifications (depends on agent)
4. PC-016-018: Team manifest + skill updates
5. PC-019-020: Doc updates (last)

## Doc Update Plan
| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | skills/pc-evaluation/SKILL.md | skill | Create | 3-dimension rubric with PASS/WARN/FAIL | US-001 |
| 2 | agents/pc-evaluator.md | agent | Create | Read-only evaluator | US-002 |
| 3 | docs/adr/0063-pc-self-evaluation.md | ADR | Create | Evaluation architecture decision | Decision #1 |
| 4 | CHANGELOG.md | project | Modify | BL-149 entry | US-005 |
| 5 | CLAUDE.md | project | Modify | Glossary: self-evaluation | US-005 |

## SOLID Assessment
PASS — pure markdown, no code dependencies.

## Robert's Oath Check
CLEAN — proof (20 PCs), small releases (5 phases), no mess.

## Security Notes
CLEAR — read-only agent, no user input handling, no APIs.

## Rollback Plan
Reverse: 10→9→8→7→6→5→4→3→2→1. Revert docs, revert skills, revert team, revert command, delete ADR, delete agent, delete skill.

## Bounded Contexts Affected
No bounded contexts affected — pure command/skill/agent.
