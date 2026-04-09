# Design: ASCII Diagram Doc-Comments Convention

## Spec Reference
`docs/specs/2026-04-07-ascii-diagram-doc-comments/spec.md`

## File Changes

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | `skills/ascii-doc-diagrams/SKILL.md` | create | US-001 |
| 2 | `agents/code-reviewer.md` | modify (add skill to skills list) | US-002 |
| 3 | `commands/audit-code.md` | modify (add diagram convention check) | US-003 |
| 4 | `docs/backlog/BL-NNN-ascii-diagram-full-sweep.md` | create | US-004 |
| 5 | `CHANGELOG.md` | modify (add entry) | Doc Impact |

## Pass Conditions

| PC | Type | Command | Expected | Verifies |
|----|------|---------|----------|----------|
| PC-001 | validate | `ecc validate skills 2>&1 \| tail -3` | exit 0 | AC-001.1, AC-001.7 |
| PC-002 | content | `wc -w < skills/ascii-doc-diagrams/SKILL.md` | <550 | AC-001.6 |
| PC-003 | content | `grep -cE 'state transition\|flow.*decision\|composition.*box' skills/ascii-doc-diagrams/SKILL.md` | >=3 | AC-001.2 |
| PC-004 | content | `grep -c '# Pattern' skills/ascii-doc-diagrams/SKILL.md` | >=1 | AC-001.3 |
| PC-005 | content | `grep -cE '3\+.*branch\|3\+.*state\|3\+.*domain\|5\+.*caller\|ARCHITECTURE' skills/ascii-doc-diagrams/SKILL.md` | >=3 | AC-001.4 |
| PC-006 | content | `grep -cE '80.*col\|fenced\|text' skills/ascii-doc-diagrams/SKILL.md` | >=2 | AC-001.5 |
| PC-007 | content | `grep -c 'ascii-doc-diagrams' agents/code-reviewer.md` | >=1 | AC-002.1 |
| PC-008 | content | `grep -ci 'diagram.*HIGH\|HIGH.*diagram\|pattern.*HIGH' agents/code-reviewer.md` | >=1 | AC-002.2, AC-002.3 |
| PC-009 | content | `grep -c 'ascii-doc-diagrams' commands/audit-code.md` | >=1 | AC-003.1 |
| PC-010 | content | `grep -ci 'MEDIUM.*diagram\|diagram.*MEDIUM' commands/audit-code.md` | >=1 | AC-003.2 |
| PC-011 | content | `grep -c 'git diff\|changed files' commands/audit-code.md` | >=1 | AC-003.3 |
| PC-012 | content | `grep -c 'AC-001.4\|eligibility' commands/audit-code.md` | >=1 | AC-003.4 |
| PC-013 | content | `ls docs/backlog/BL-*ascii-diagram*.md 2>/dev/null \| wc -l` | >=1 | AC-004.1 |
| PC-014 | content | `grep -c '9 crate' docs/backlog/BL-*ascii-diagram*.md` | >=1 | AC-004.2 |
| PC-015 | validate | `ecc validate agents 2>&1 \| tail -3` | exit 0 | AC-002.1 |
| PC-016 | validate | `ecc validate commands 2>&1 \| tail -3` | exit 0 | AC-003.1 |
| PC-017 | content | `grep -cE 'GoF\|DDD\|Hexagonal\|Rust Idiom' skills/ascii-doc-diagrams/SKILL.md` | >=2 | AC-001.3 |
| PC-018 | content | `grep -c 'status.*open' docs/backlog/BL-*ascii-diagram*.md` | >=1 | AC-004.1 |
| PC-019 | content | `grep -ci 'ascii.*diagram\|doc-comment.*convention' CHANGELOG.md` | >=1 | Doc Impact |

## Coverage Check

All 15 ACs covered. AC-001.1-7, AC-002.1-3, AC-003.1-4, AC-004.1-2 each have at least 1 covering PC.

## E2E Test Plan

No E2E tests — Markdown-only changes, no port/adapter boundary crossings.

## Test Strategy

Wave 1 (skill file): PC-001 through PC-006, PC-017
Wave 2 (agent + command): PC-007 through PC-012, PC-015, PC-016
Wave 3 (backlog + changelog): PC-013, PC-014, PC-018, PC-019

## Doc Update Plan

| Doc | Action | Spec Ref |
|-----|--------|----------|
| CHANGELOG.md | Add convention entry | Doc Impact |

## SOLID Assessment
N/A — Markdown files only.

## Robert's Oath Check
CLEAN — minimal scope, single concern per file.

## Security Notes
CLEAR — no executable code, no user input processing.

## Rollback Plan
1. Delete `skills/ascii-doc-diagrams/SKILL.md`
2. Revert `agents/code-reviewer.md` skills list
3. Revert `commands/audit-code.md` diagram section
4. Delete `docs/backlog/BL-NNN-ascii-diagram-full-sweep.md`
5. Revert `CHANGELOG.md` entry

## Bounded Contexts Affected
No bounded contexts affected — all changes are Markdown skill/agent/command files outside the domain model.
