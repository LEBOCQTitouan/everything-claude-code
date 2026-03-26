# Solution: Auto-Commit Backlog Edits (BL-059)

## Spec Reference
Concern: dev, Feature: BL-059 auto-commit backlog edits

## File Changes (dependency order)
| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `commands/backlog.md` | modify | Add commit instruction blocks to add/promote/archive subcommands | US-001, AC-001.1–001.9 |
| 2 | `CHANGELOG.md` | modify | Add BL-059 entry | US-002, AC-002.1 |

## Pass Conditions
| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | add has MUST commit | AC-001.6, AC-001.9 | `grep -qi 'MUST commit immediately' commands/backlog.md` | exit 0 |
| PC-002 | unit | add commit message format | AC-001.1 | `grep -q 'docs(backlog): add BL-' commands/backlog.md` | exit 0 |
| PC-003 | unit | promote commit format | AC-001.2, AC-001.7 | `grep -q 'docs(backlog): promote BL-' commands/backlog.md` | exit 0 |
| PC-004 | unit | archive commit format | AC-001.3, AC-001.8 | `grep -q 'docs(backlog): archive BL-' commands/backlog.md` | exit 0 |
| PC-005 | unit | git add scoped | AC-001.1 | `grep -q 'git add docs/backlog/' commands/backlog.md` | exit 0 |
| PC-006 | unit | list has NO commit | AC-001.4 | `! grep -A20 '### .*list' commands/backlog.md \| grep -qi 'MUST commit'` | exit 0 |
| PC-007 | unit | match has NO commit | AC-001.5 | `! grep -A20 '### .*match' commands/backlog.md \| grep -qi 'MUST commit'` | exit 0 |
| PC-008 | unit | CHANGELOG BL-059 | AC-002.1 | `grep -q 'BL-059' CHANGELOG.md` | exit 0 |
| PC-009 | build | cargo test | AC-002.1 | `cargo test` | pass |
| PC-010 | lint | cargo clippy | AC-002.1 | `cargo clippy -- -D warnings` | exit 0 |

### Coverage Check
All 10 ACs covered by 10 PCs.

### E2E Test Plan
No E2E boundaries affected.

### E2E Activation Rules
No E2E tests to activate.

## Test Strategy
1. PC-001–007: Modify backlog.md
2. PC-008: Add CHANGELOG entry
3. PC-009–010: Quality gate

## Doc Update Plan
| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `CHANGELOG.md` | Project | Add entry | BL-059: auto-commit for add/promote/archive | AC-002.1 |

## SOLID Assessment
PASS. Pure markdown.

## Robert's Oath Check
CLEAN. Atomic commits for backlog mutations.

## Security Notes
CLEAR. No code, no input handling.

## Rollback Plan
1. Revert CHANGELOG.md
2. Revert commands/backlog.md
