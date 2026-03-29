# Solution: Audit adversarial challenge phase (BL-083)

## Spec Reference
Concern: dev, Feature: Adversarial challenge phase for all audit commands (BL-083)

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `agents/audit-challenger.md` | create | New adversary agent (Sonnet, read-only + WebSearch) | AC-001.1-5 |
| 2 | `commands/audit-archi.md` | modify | Add adversary challenge after analysis | AC-002.1 |
| 3 | `commands/audit-code.md` | modify | Add adversary challenge after analysis | AC-002.1 |
| 4 | `commands/audit-convention.md` | modify | Add adversary challenge after analysis | AC-002.1 |
| 5 | `commands/audit-doc.md` | modify | Add adversary challenge after analysis | AC-002.1 |
| 6 | `commands/audit-errors.md` | modify | Add adversary challenge after analysis | AC-002.1 |
| 7 | `commands/audit-evolution.md` | modify | Add adversary challenge after analysis | AC-002.1 |
| 8 | `commands/audit-observability.md` | modify | Add adversary challenge after analysis | AC-002.1 |
| 9 | `commands/audit-security.md` | modify | Add adversary challenge after analysis | AC-002.1 |
| 10 | `commands/audit-test.md` | modify | Add adversary challenge after analysis | AC-002.1 |
| 11 | `commands/audit-web.md` | modify | Add adversary challenge after analysis | AC-002.1 |
| 12 | `commands/audit-full.md` | modify | Reference adversary pattern | AC-002.2 |
| 13 | `agents/audit-orchestrator.md` | modify | Add adversary challenge after each domain agent | AC-002.2 |
| 14 | `docs/commands-reference.md` | modify | Note adversary phase in audit descriptions | US-002 |
| 15 | `CHANGELOG.md` | modify | Entry | US-001 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | audit-challenger frontmatter correct (model, skills, memory, NO Write/Edit) | AC-001.1 | `test -f agents/audit-challenger.md && grep -q '^model: sonnet' agents/audit-challenger.md && grep -q 'clean-craft' agents/audit-challenger.md && grep -q 'memory: project' agents/audit-challenger.md && ! grep -q '"Write"\|"Edit"' agents/audit-challenger.md` | exit 0 |
| PC-002 | unit | audit-challenger clean bill of health section exists | AC-001.2 | `grep -q 'clean bill of health' agents/audit-challenger.md && grep -q 'no issues to challenge' agents/audit-challenger.md` | exit 0 |
| PC-003 | unit | audit-challenger retry logic with structured output check | AC-001.3 | `grep -q 'retry.*stricter prompt' agents/audit-challenger.md && grep -q 'finding ID.*verdict.*rationale' agents/audit-challenger.md` | exit 0 |
| PC-004 | unit | audit-challenger disagreement handling with both perspectives | AC-001.4 | `grep -q 'both perspectives' agents/audit-challenger.md && grep -q 'user.*final.*decision' agents/audit-challenger.md` | exit 0 |
| PC-005 | unit | audit-challenger graceful degradation on spawn failure | AC-001.5 | `grep -q 'Adversary challenge skipped' agents/audit-challenger.md` | exit 0 |
| PC-006 | unit | All 10 domain audit commands reference audit-challenger | AC-002.1 | `for f in commands/audit-{archi,code,convention,doc,errors,evolution,observability,security,test,web}.md; do grep -q 'audit-challenger' "$f" \|\| exit 1; done` | exit 0 |
| PC-007 | unit | audit-orchestrator references audit-challenger | AC-002.2 | `grep -q 'audit-challenger' agents/audit-orchestrator.md` | exit 0 |
| PC-008 | integration | ecc validate agents passes | AC-002.4 | `ecc validate agents` | exit 0 |
| PC-009 | integration | ecc validate commands passes | AC-002.3 | `ecc validate commands` | exit 0 |
| PC-010 | lint | clippy clean | all | `cargo clippy -- -D warnings` | exit 0 |
| PC-011 | build | build succeeds | all | `cargo build` | exit 0 |
| PC-012 | integration | ecc validate conventions passes | AC-002.5 | `ecc validate conventions` | exit 0 |

### Coverage Check

All ACs covered:
- AC-001.1 → PC-001
- AC-001.2 → PC-002
- AC-001.3 → PC-003
- AC-001.4 → PC-004
- AC-001.5 → PC-005
- AC-002.1 → PC-006
- AC-002.2 → PC-007
- AC-002.3 → PC-009
- AC-002.4 → PC-008
- AC-002.5 → PC-012

### E2E Test Plan

No E2E tests needed — markdown config only.

### E2E Activation Rules

None.

## Test Strategy

TDD order:
1. PC-001-005: Create audit-challenger.md agent (single file, all content checks)
2. PC-006: Modify 10 domain audit commands
3. PC-007: Modify audit-orchestrator
4. PC-008-009: Validate agents + commands
5. PC-010-011: Clippy + build gate

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | docs/commands-reference.md | Docs | Update audit descriptions | Note adversary challenge phase | US-002 |
| 2 | CHANGELOG.md | Project | Add entry | Adversarial challenge for all audit commands | US-001 |

## SOLID Assessment

N/A — markdown-only changes. PASS by inspection.

## Robert's Oath Check

CLEAN — follows established adversary pattern, no mess, clean additions.

## Security Notes

CLEAR — no code, no injection surfaces, agent is read-only.

## Rollback Plan

Reverse order:
1. Revert CHANGELOG.md
2. Revert docs/commands-reference.md
3. Revert agents/audit-orchestrator.md
4. Revert commands/audit-full.md
5. Revert 10 domain audit commands
6. Delete agents/audit-challenger.md
