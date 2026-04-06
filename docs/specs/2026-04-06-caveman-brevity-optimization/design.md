# Solution: Caveman-Style Brevity Token Optimization

## Spec Reference
Concern: refactor, Feature: caveman-brevity-token-optimization

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `rules/common/brevity.md` | create | Global brevity rule all agents inherit | AC-001.1, AC-001.2 |
| 2 | `commands/*.md` (30+ files) | modify | Compress verbose prose to tables/bullets | AC-002.1..3 |
| 3 | `agents/*.md` (57 files) | modify | Remove redundant examples and boilerplate | AC-003.1..3 |
| 4 | `skills/*/SKILL.md` (100+ files) | modify | Collapse multi-paragraph explanations | AC-004.1..3 |
| 5 | `CLAUDE.md` | modify | Reference brevity rule | AC-001.3 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | lint | brevity.md exists | AC-001.1 | `test -f rules/common/brevity.md && echo "exists"` | exists |
| PC-002 | lint | Command lines <= 4115 (30% of 5878) | AC-002.2 | `wc -l commands/*.md \| tail -1 \| awk '{print ($1 <= 4115) ? "PASS" : "FAIL"}'` | PASS |
| PC-003 | lint | Agent lines <= 5550 (30% of 7928) | AC-003.2 | `wc -l agents/*.md \| tail -1 \| awk '{print ($1 <= 5550) ? "PASS" : "FAIL"}'` | PASS |
| PC-004 | lint | Skill lines <= 17093 (30% of 24418) | AC-004.2 | `wc -l skills/*/SKILL.md \| tail -1 \| awk '{print ($1 <= 17093) ? "PASS" : "FAIL"}'` | PASS |
| PC-005 | lint | CLAUDE.md references brevity | AC-001.3 | `grep -c 'brevity' CLAUDE.md` | >= 1 |
| PC-006 | build | cargo build passes | AC-002.3, AC-003.3, AC-004.3 | `cargo build 2>&1 \| tail -1` | Finished |
| PC-007 | build | ecc validate passes | AC-002.3, AC-003.3 | `ecc validate commands 2>/dev/null && ecc validate agents 2>/dev/null; echo $?` | 0 |

### Coverage Check

| AC | Covering PCs |
|----|-------------|
| AC-001.1 | PC-001 |
| AC-001.2 | PC-001 (rule content) |
| AC-001.3 | PC-005 |
| AC-002.1 | PC-002 (outcome of compression) |
| AC-002.2 | PC-002 |
| AC-002.3 | PC-006, PC-007 |
| AC-003.1 | PC-003 |
| AC-003.2 | PC-003 |
| AC-003.3 | PC-006, PC-007 |
| AC-004.1 | PC-004 |
| AC-004.2 | PC-004 |
| AC-004.3 | PC-006 |

All 12 ACs covered.

### E2E Test Plan
None -- instruction text changes only.

### E2E Activation Rules
None.

## Test Strategy

TDD order:
1. PC-001: Create brevity rule
2. PC-002: Compress commands (highest impact -- loaded every session)
3. PC-003: Compress agents
4. PC-004: Compress skills
5. PC-005: Update CLAUDE.md
6. PC-006: Verify build
7. PC-007: Validate commands + agents

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CLAUDE.md | project | modify | Add brevity rule reference | AC-001.3 |
| 2 | CHANGELOG.md | project | modify | Add BL-123 entry | all |

## SOLID Assessment
N/A -- markdown instruction text, no code architecture.

## Robert's Oath Check
CLEAN -- reduces waste without adding complexity.

## Security Notes
CLEAR -- no attack surface changes.

## Rollback Plan
1. Revert CLAUDE.md
2. Revert all skill files
3. Revert all agent files
4. Revert all command files
5. Delete rules/common/brevity.md

## Bounded Contexts Affected
No bounded contexts affected -- no domain files modified.
