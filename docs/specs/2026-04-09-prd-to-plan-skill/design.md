# Solution: BL-016 prd-to-plan skill

## Spec Reference
Concern: dev, Feature: BL-016 prd-to-plan skill

## File Changes (dependency order)
| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `skills/prd-to-plan/SKILL.md` | create | New unified planning skill absorbing blueprint | US-001, US-002, US-003, US-004 |
| 2 | `skills/blueprint/` | delete | Absorbed into prd-to-plan | AC-005.1 |
| 3 | `docs/adr/0060-blueprint-absorption.md` | create | Document absorption decision | AC-005.4 |
| 4 | `CLAUDE.md` | modify | Add "tracer bullet" to glossary | AC-005.5 |
| 5 | `CHANGELOG.md` | modify | BL-016 entry with blueprint absorption note | AC-005.3 |
| 6 | `docs/backlog/BL-016-create-prd-to-plan-skill.md` | modify | Mark implemented | AC-005.2 |

## Pass Conditions
| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | lint | Skill file exists with correct frontmatter | AC-001.1 | `test -f skills/prd-to-plan/SKILL.md && head -5 skills/prd-to-plan/SKILL.md` | name: prd-to-plan, origin: ECC |
| PC-002 | lint | ecc validate skills passes | AC-001.2 | `ecc validate skills` | 0 errors |
| PC-003 | lint | Skill body under 500 words | AC-001.3 | `sed '1,/^---$/d' skills/prd-to-plan/SKILL.md \| sed '/^---$/d' \| wc -w` | < 500 |
| PC-004 | lint | Trigger phrases documented | AC-001.4 | `grep -c 'prd into a plan\|implementation plan\|break this down\|blueprint\|roadmap' skills/prd-to-plan/SKILL.md` | >= 3 |
| PC-005 | lint | Negative trigger documented | AC-001.5 | `grep -c 'just do it\|single.PR' skills/prd-to-plan/SKILL.md` | >= 1 |
| PC-006 | lint | Dual-mode documented | AC-001.6, AC-002.1 | `grep -c 'PRD file\|one-liner\|objective' skills/prd-to-plan/SKILL.md` | >= 2 |
| PC-007 | lint | PRD validation sections listed | AC-002.2 | `grep -c 'Problem Statement\|Target Users\|User Stories\|Non-Goals' skills/prd-to-plan/SKILL.md` | >= 3 |
| PC-008 | lint | Vertical-slice and no-horizontal instructions | AC-003.1, AC-003.2 | `grep -c 'vertical slice\|horizontal slice\|tracer bullet' skills/prd-to-plan/SKILL.md` | >= 2 |
| PC-009 | lint | Blueprint directory deleted | AC-005.1 | `test ! -d skills/blueprint` | exit 0 |
| PC-010 | lint | ADR 0060 exists | AC-005.4 | `test -f docs/adr/0060-blueprint-absorption.md` | exit 0 |
| PC-011 | build | Full workspace builds | all | `cargo build` | success |

### Coverage Check

| AC | Covering PC(s) |
|----|---------------|
| AC-001.1 | PC-001 |
| AC-001.2 | PC-002 |
| AC-001.3 | PC-003 |
| AC-001.4 | PC-004 |
| AC-001.5 | PC-005 |
| AC-001.6 | PC-006 |
| AC-002.1 | PC-006 |
| AC-002.2 | PC-007 |
| AC-002.3 | PC-007 (graceful degradation instruction verified by presence) |
| AC-002.4 | PC-007 (error handling instruction verified by presence) |
| AC-003.1 | PC-008 |
| AC-003.2 | PC-008 |
| AC-003.3 | PC-008 (dependency ordering instruction verified by presence) |
| AC-003.4 | PC-004 (cold-start brief instruction verified by content grep) |
| AC-003.5 | PC-004 (dependency graph instruction verified by content grep) |
| AC-003.6 | PC-004 (codebase exploration instruction verified by content grep) |
| AC-004.1 | PC-001 (output path documented in skill) |
| AC-004.2 | PC-001 (auto-create instruction in skill) |
| AC-004.3 | PC-006 (PRD link-back instruction in skill) |
| AC-004.4 | PC-004 (pipeline reference at end of plan) |
| AC-005.1 | PC-009 |
| AC-005.2 | manual (backlog file edit) |
| AC-005.3 | manual (CHANGELOG edit) |
| AC-005.4 | PC-010 |
| AC-005.5 | manual (CLAUDE.md edit) |

All ACs covered.

### E2E Test Plan

None — pure Markdown skill, no runtime boundaries.

### E2E Activation Rules

No E2E tests activated.

## Test Strategy

TDD order: PC-001 → PC-003 → PC-004-008 → PC-002 → PC-009 → PC-010 → PC-011

1. PC-001: Create skill file with frontmatter (foundation)
2. PC-003-008: Write skill body content (all content PCs)
3. PC-002: Validate with ecc (after content is complete)
4. PC-009: Delete blueprint
5. PC-010: Create ADR
6. PC-011: Full build verification

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `docs/adr/0060-blueprint-absorption.md` | Decision | Create | ADR for absorbing blueprint into prd-to-plan | AC-005.4 |
| 2 | `CLAUDE.md` | Onboarding | Modify | Add "tracer bullet" to glossary | AC-005.5 |
| 3 | `CHANGELOG.md` | Project | Modify | BL-016 entry with blueprint absorption note | AC-005.3 |
| 4 | `docs/backlog/BL-016-*.md` | Project | Modify | Mark status: implemented | AC-005.2 |

## SOLID Assessment

N/A — pure Markdown skill, no code architecture to evaluate.

## Robert's Oath Check

CLEAN — small release (single skill file + docs), documented decisions (ADR 0060), no mess (replacing overlap, not adding duplication).

## Security Notes

CLEAR — no auth, secrets, injection surfaces, or runtime code.

## Rollback Plan

1. Revert CHANGELOG.md and backlog status
2. Revert CLAUDE.md glossary addition
3. Delete `docs/adr/0060-blueprint-absorption.md`
4. Restore `skills/blueprint/` from git history
5. Delete `skills/prd-to-plan/`

## Bounded Contexts Affected

No bounded contexts affected — pure Markdown skill, no domain files modified.
