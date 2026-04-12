# Solution: BL-143 /project-foundation Command

## Spec Reference
Concern: dev, Feature: BL-143 project-foundation command

## File Changes (dependency order)
| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | skills/grill-me/SKILL.md | modify | Add foundation-mode section with PRD (Clarity+Assumptions) and arch (Clarity+Edge Cases) stages | US-002 |
| 2 | docs/adr/0061-grill-me-foundation-mode.md | create | ADR for Decision #4 — extending grill-me with foundation-mode | US-002, Decision #4 |
| 3 | commands/project-foundation.md | create | New command: project detection, codebase analysis, interview-me + grill-me, Plan Mode, adversarial review, doc generation | US-001,003-009 |
| 4 | docs/commands-reference.md | modify | Add /project-foundation entry | US-001 |
| 5 | CLAUDE.md | modify | Add command reference + glossary entries (foundation document, codebase-analysis phase) | US-001, US-007 |
| 6 | CHANGELOG.md | modify | Add BL-143 entry under Unreleased/Added | All |

## Pass Conditions
| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | lint | ecc validate skills passes | AC-002.3 | `ecc validate skills` | exit 0 |
| PC-002 | grep | foundation-mode section exists | AC-002.1,2 | `grep -ci 'foundation.mode' skills/grill-me/SKILL.md` | >=1 |
| PC-003 | grep | max 2 questions per stage | AC-002.1 | `grep -ciE 'max(imum)? 2 questions' skills/grill-me/SKILL.md` | >=1 |
| PC-004 | grep | existing modes intact | AC-002.4 | `grep -cE '### Standalone|### Spec-Mode|### Backlog-Mode' skills/grill-me/SKILL.md` | 3 |
| PC-005 | grep | ADR-0061 has 4 sections | AC-006.1 | `grep -c '## Status\|## Context\|## Decision\|## Consequences' docs/adr/0061-grill-me-foundation-mode.md` | 4 |
| PC-006 | grep | ADR references foundation-mode | Decision #4 | `grep -c 'foundation-mode' docs/adr/0061-grill-me-foundation-mode.md` | >=2 |
| PC-007 | lint | ecc validate commands passes | AC-001.1 | `ecc validate commands` | exit 0 |
| PC-008 | grep | frontmatter has description + allowed-tools | AC-001.2 | `head -10 commands/project-foundation.md \| grep -c 'description:\|allowed-tools:'` | 2 |
| PC-009 | grep | references narrative-conventions | AC-001.2 | `grep -c 'narrative-conventions' commands/project-foundation.md` | >=1 |
| PC-010 | grep | uses TodoWrite | AC-001.3 | `grep -c 'TodoWrite' commands/project-foundation.md` | >=1 |
| PC-011 | grep | uses ecc-workflow init + transition | AC-001.4 | `grep -c 'ecc-workflow' commands/project-foundation.md` | >=2 |
| PC-012 | grep | uses EnterPlanMode | AC-008.1 | `grep -c 'EnterPlanMode' commands/project-foundation.md` | >=1 |
| PC-013 | grep | commands-reference has entry | AC-001.1 | `grep -c 'project-foundation' docs/commands-reference.md` | >=1 |
| PC-014 | grep | CLAUDE.md has command ref | AC-007.3 | `grep -c 'project-foundation' CLAUDE.md` | >=1 |
| PC-015 | grep | CLAUDE.md has glossary entries | AC-007.3 | `grep -c 'foundation document' CLAUDE.md` | >=1 |
| PC-016 | grep | CHANGELOG has BL-143 | All | `grep -c 'BL-143' CHANGELOG.md` | >=1 |
| PC-017 | grep | command references docs/foundation/ output path | AC-004.5, AC-005.4 | `grep -c 'docs/foundation/' commands/project-foundation.md` | >=1 |
| PC-018 | grep | command references interview-me skill | AC-004.1 | `grep -c 'interview-me' commands/project-foundation.md` | >=1 |
| PC-019 | grep | command references spec-adversary for review | AC-009.1 | `grep -c 'spec-adversary' commands/project-foundation.md` | >=1 |
| PC-020 | grep | command has codebase detection logic (new vs existing) | AC-003.1, AC-003.2 | `grep -cE 'new repo|existing repo' commands/project-foundation.md` | >=2 |
| PC-021 | grep | command handles re-run with revision blocks | AC-009.5 | `grep -cE 'revision|already exists' commands/project-foundation.md` | >=1 |
| PC-022 | grep | command has re-entry support via workflow | AC-009.6 | `grep -c 're-entry' commands/project-foundation.md` | >=1 |
| PC-023 | grep | command references adversary retry on FAIL | AC-009.2 | `grep -cE 'FAIL.*re-enter\|retry\|loop' commands/project-foundation.md` | >=1 |
| PC-024 | grep | command has 3-FAIL limit with user escape | AC-009.4 | `grep -cE '3.*FAIL\|three.*fail\|max.*attempt' commands/project-foundation.md` | >=1 |
| PC-025 | grep | command handles ExitPlanMode / approval | AC-008.3 | `grep -c 'ExitPlanMode' commands/project-foundation.md` | >=1 |
| PC-026 | grep | command handles rejection / no-write | AC-008.4 | `grep -cE 'reject\|abort\|no files written' commands/project-foundation.md` | >=1 |
| PC-027 | grep | command references CLAUDE.md generation for new repos | AC-007.1 | `grep -cE 'generate.*CLAUDE\|create.*CLAUDE' commands/project-foundation.md` | >=1 |
| PC-028 | grep | command references PRD template sections | AC-004.3 | `grep -cE 'Problem.*Users.*Goals\|7 sections\|PRD.*template' commands/project-foundation.md` | >=1 |
| PC-029 | grep | command references architecture template | AC-005.2 | `grep -cE 'System Overview.*Bounded\|6 sections\|architecture.*template' commands/project-foundation.md` | >=1 |
| PC-030 | grep | command references ADR auto-numbering | AC-006.2, AC-006.3 | `grep -cE 'next.*ADR\|auto.*number\|ADR-0001' commands/project-foundation.md` | >=1 |

### Coverage Check
33 ACs covered by 30 PCs. 28 ACs have explicit PC coverage via the Verifies AC column. 5 ACs are runtime-only (no structural PC possible): AC-003.5 (contradiction handling — user precedence at runtime), AC-004.4 (pre-population from analysis — runtime behavior), AC-005.1 (grill-me fires specific stages — runtime), AC-005.3 (bounded contexts pre-populated — runtime), AC-006.4 (ADR contains specific rationale — runtime content quality). These 5 are verified by the adversarial review built into the command itself.

### E2E Test Plan
No E2E tests — pure command/skill, no hexagonal boundary changes.

### E2E Activation Rules
None activated.

## Test Strategy
TDD order:
1. PC-001-004: grill-me foundation-mode (skill file must exist first — command depends on it)
2. PC-005-006: ADR-0061 (documents the decision, independent)
3. PC-007-012, PC-017-030: command file + structural verification (the main deliverable, depends on #1)
4. PC-013-016: doc updates (reference the command, last)

## Doc Update Plan
| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CHANGELOG.md | project | Add entry | BL-143 /project-foundation + grill-me foundation-mode | All |
| 2 | CLAUDE.md | project | Add command ref + glossary | /project-foundation in pipeline section; glossary: foundation document, codebase-analysis phase | US-001, US-007 |
| 3 | docs/commands-reference.md | project | Add row | /project-foundation entry in pipeline table | US-001 |
| 4 | docs/adr/0061-grill-me-foundation-mode.md | project | Create ADR | Decision #4: extend grill-me with foundation-mode | Decision #4 |

## SOLID Assessment
PASS — pure markdown, no code dependencies, no architectural impact.

## Robert's Oath Check
CLEAN — small releases (4 phases), proof (30 PCs), no mess (additive changes only).

## Security Notes
CLEAR — no code execution, no user input handling, no secrets. Pure orchestration markdown.

## Rollback Plan
Reverse order: 6→5→4→3→2→1. Revert CHANGELOG (#6), revert CLAUDE.md (#5), revert commands-reference (#4), delete command file (#3), delete ADR (#2), revert grill-me skill (#1).

## Bounded Contexts Affected
No bounded contexts affected — pure command/skill work, no domain model changes.
