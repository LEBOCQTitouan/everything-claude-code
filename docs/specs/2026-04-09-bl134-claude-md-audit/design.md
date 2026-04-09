# Design: Audit CLAUDE.md for Inferable Content (BL-134)

## Spec Reference
\`docs/specs/2026-04-09-bl134-claude-md-audit/spec.md\`

## File Changes

| # | File | Change | Layer | Spec Ref |
|---|------|--------|-------|----------|
| 1 | \`CLAUDE.md\` | Remove inferable sections, add pointers | Root | AC-001.1–AC-001.12 |

## Pass Conditions

| PC | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | verify | Running Tests removed | AC-001.1 | \`! grep -q '## Running Tests' CLAUDE.md\` | exit 0 |
| PC-002 | verify | Architecture has pointer | AC-001.2 | \`grep 'docs/ARCHITECTURE.md' CLAUDE.md\` | exit 0 |
| PC-003 | verify | CLI Commands removed | AC-001.3 | \`! grep -q '## CLI Commands' CLAUDE.md\` | exit 0 |
| PC-004 | verify | Slash Commands removed | AC-001.4 | \`! grep -q '## Slash Commands' CLAUDE.md\` | exit 0 |
| PC-005 | verify | Doc Hierarchy removed | AC-001.5 | \`! grep -q '## Doc Hierarchy' CLAUDE.md\` | exit 0 |
| PC-006 | verify | Dev Notes removed | AC-001.6 | \`! grep -q '## Development Notes' CLAUDE.md\` | exit 0 |
| PC-007 | verify | Gotchas preserved | AC-001.7 | \`grep -q 'Brevity rule' CLAUDE.md\` | exit 0 |
| PC-008 | verify | Pipeline rules preserved | AC-001.8 | \`grep -q 'Spec-Driven Pipeline' CLAUDE.md\` | exit 0 |
| PC-009 | verify | Command workflows preserved | AC-001.9 | \`grep -q 'Command Workflows' CLAUDE.md\` | exit 0 |
| PC-010 | verify | Glossary preserved | AC-001.10 | \`grep -q 'Glossary:' CLAUDE.md\` | exit 0 |
| PC-011 | verify | Line count reduced >= 30 | AC-001.11 | \`test \$(wc -l < CLAUDE.md) -le 90\` | exit 0 |
| PC-012 | verify | ecc validate passes | AC-001.12 | \`ecc validate claude-md 2>&1; echo exit:\$?\` | exit:0 |

## Coverage Check

All 12 ACs covered by PC-001 through PC-012.

## E2E Test Plan

No E2E tests needed — doc-only change.

## E2E Activation Rules

No E2E tests activated.

## Test Strategy

Single wave: edit CLAUDE.md, then verify all 12 PCs.

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CHANGELOG.md | root | Add entry | BL-134 CLAUDE.md inferable content audit | US-001 |

## SOLID Assessment

**PASS.** Doc-only change. No code affected.

## Robert's Oath Check

**CLEAN.** Reducing documentation clutter follows the Boy Scout Rule.

## Security Notes

**CLEAR.** No code changes, no security surface.

## Rollback Plan

\`git revert\` the single commit touching CLAUDE.md.

## Bounded Contexts Affected

No bounded contexts affected — doc-only change.
